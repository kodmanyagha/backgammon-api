# Tavla Api

Backend for the Tavla (backgammon) game. Built with Rust, Axum, MySQL, Redis.

Handles user authentication, guest sessions, role/permission management, matchmaking,
real-time gameplay over WebSocket, and persistence of game history and scores.

---

# How to run?

```bash
# Run backend service
watchexec -w src -r cargo run --bin tavla_api

# Run DB seeds
cargo run --bin seed
```

Swagger UI is served at `/swagger-ui` once the server is running.

---

# End-to-end tests

Python tests that start this server against a scratch database and play real games over HTTP and WebSocket live in
`../e2e` (`./run.sh`, `./run.sh fast`; see its README).

---

# Code layout of the game socket

`src/http/controllers/v1/games/ws/` is one folder, one job per file:

| File | Job |
| --- | --- |
| `mod.rs` | the HTTP upgrade (`handle_upgrade`), opening/rebuilding a session, the timing constants |
| `socket.rs` | one connection: message loop, ping/pong, presence |
| `watchdog.rs` | one task per match: turn clock, expired turns, AI steps, next-game start |
| `actions.rs` | what a client message does (roll, move, undo, confirm, resign), persistence, end of a game / match |
| `ai_offer.rs` | the "continue against the computer?" question and its answers |
| `broadcast.rs` | every message sent to one or both players |

---

# Live games and persistence

A match is a series of games; the first player to win 5 games wins the match. Between games the
server waits 5 seconds and then starts the next one by itself (the previous game's winner opens it;
the very first game is opened by a random side).

- **Redis is the source of truth of a live match.** After every action (roll, move, undo, confirm, end
  of a game) the whole match state is written to `game:{id}:state` (board, dice, undo history, score,
  round number, turn deadline as an absolute unix time). After an API restart a client that is still
  open reconnects to `/v1/games/{id}/ws` and the session is rebuilt from that key. The rebuilt turn
  clock gives the player at least 30 seconds. Redis must be persistent (AOF, `noeviction`; see
  `docker/redis/valkey.conf`).
- **The database is written in bulk.** Moves, finished games, closed matches and win/loss counts are
  queued in the Redis list `db:outbox`; `service/game/db_writer.rs` drains it every
  `FLUSH_INTERVAL` (500 ms) into a few multi-row statements inside one transaction. Replaying an event is
  harmless (unique keys, a match is only closed once). Only the `games` row is inserted right away, when
  two players are matched, because it gives the match its id.
- **Matchmaking lives in Redis too** (`mm:waiting`, `mm:matched:{user}`). A search ends with
  `no_opponent` after 60 seconds. Coming back to the app always starts a fresh search; an older match is
  never resumed through quick-match.
- **Nobody loses by being slow.** Every turn has a clock (`TURN_TIME_LIMIT`, 62 s = 60 s of play + 2 s of
  dice animation; it starts by itself when the turn starts and is renewed by the roll). When it runs out
  (`service/game/expired_turn.rs`):
  - both sides are in the game (or the opponent is the computer): the server plays the idle player's turn
    for them (roll if needed, then the best moves from the same decision code the app uses offline,
    `tavla_core::choose_move_sequence`) and both sides see every move;
  - one human has dropped (Android closes the network of an app in the background, so a player who is not
    connected is waited for one whole move time): the connected player gets an `ai_offer` ("continue against
    the computer?"), whether the idle player is the connected one or the dropped one. `accept_ai` makes the
    server play the missing side and, if it is the accepting player's turn, gives them a fresh move time;
    `decline_ai` (or no answer for 60 s) makes them win by forfeit. If the missing player comes back while the
    question is open it is withdrawn and they get a fresh move time; if they come back later they take their
    seat back from the computer;
  - nobody is connected: the match is abandoned after both have been gone for a move time.
  Matches nobody touched for 10 minutes are marked `abandoned` by the writer's sweep. Nothing is ever
  deleted from the database.
- **Computer moves are marked.** `game_moves.is_ai` is `1` for every move the server chose (the computer's seat and
  turns played for an idle player), and a match in which the computer took a seat never changes the wins/losses
  counts (`ranked: false`).
- **Every move is announced.** The `state` message that follows a move carries `last_move`
  (`{player, origin, die, is_ai}`, `null` for everything else: roll, confirm, undo, a new game, a reconnect), so the
  app can show the move as a flying checker instead of redrawing the board at once. The server plays turns quickly one
  step at a time (`AI_MOVE_DELAY`), each step its own `state`.
- **A mars is worth two points.** `game_ended` and `game_over` carry `mars` so both screens can say so. Winning a game while the opponent has not borne off a single checker adds 2 to the
  score instead of 1 (`Board::win_points` in `tavla_core`, used by the server and by the offline game). A match ends as
  soon as one side has 5 or more points, so a mars from 4 ends it at 6.
- **A match names the opponent.** The `matched` answer of `POST /v1/games/quick-match` carries `game_id` and
  `opponent_name` (the opponent's username), so the app can show who it was matched with before the game starts.
- **A rejected action is followed by the real state.** When a roll, move, undo or confirm is refused, the player gets the
  `error` and then a `state` addressed to them alone, so an app that applied the move locally first snaps back to the truth.
- **The last roll travels with every `state`** (`dice`, `null` before the first roll of a game). A player who joined after
  the opening roll, or missed `dice_rolled`, still sees the real dice (the app plays the roll effect for a fresh join).
- **The app is told whether the opponent is connected.** Every `state` carries `opponent_connected` (plus the live
  `opponent_connected` / `opponent_disconnected` messages). When it is the dropped opponent's turn and the server has
  said nothing for 5 s, the app shows "Cevap bekleniyor" under the opponent's icon with a 70 s countdown; any message
  (or the AI question) hides it. The question itself comes when the turn clock runs out.
- **Sides are `white` and `black`.** `Player::White` plays the light pieces ("Beyaz", bottom-right start
  offline) and is `games.white_user_id`; `Player::Black` plays the dark pieces and is `games.black_user_id`.
