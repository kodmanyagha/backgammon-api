use rand::RngExt;
use serde::{Deserialize, Serialize};
use tavla_core::{legal_turn_sequences, Board, DiceRoll, Move, Origin, Player};

pub const MATCH_TARGET_SCORE: u16 = 5;

const ROUND_END_NOT_SCHEDULED: i64 = i64::MAX;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameSession {
    pub white_user_id: u64,
    pub black_user_id: u64,
    pub board: Board,
    pub current_player: Player,
    pub remaining_dice: Vec<u8>,
    #[serde(default)]
    pub last_roll: Option<[u8; 2]>,
    #[serde(default)]
    pub last_round_mars: bool,
    pub required_moves: usize,
    pub move_history: Vec<(Board, Vec<u8>)>,
    pub next_sequence_no: u32,
    #[serde(default = "first_round_no")]
    pub round_no: u16,
    #[serde(default)]
    pub white_score: u16,
    #[serde(default)]
    pub black_score: u16,
    #[serde(default)]
    pub last_round_winner: Option<Player>,
    #[serde(default)]
    pub phase: RoundPhase,
    #[serde(default)]
    pub ai_side: Option<Player>,
    #[serde(default)]
    pub ai_used: bool,
}

fn first_round_no() -> u16 {
    1
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum RoundPhase {
    #[default]
    Playing,
    RoundEnded { next_round_at_ms: i64 },
}

pub enum ActionResult {
    Ok,
    TurnPassed { closed_out: Option<Player> },
    Rolled { die1: u8, die2: u8, no_legal_moves: bool, closed_out: Option<Player> },
    MoveApplied { die: u8, origin_point: Option<u8>, sequence_no: u32, round_no: u16 },
    RoundWon { winner: Player, die: u8, origin_point: Option<u8>, sequence_no: u32, round_no: u16 },
    Err(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundOutcome {
    MatchContinues,
    MatchWon(Player),
}

pub fn random_first_player() -> Player {
    if rand::rng().random_bool(0.5) {
        Player::White
    } else {
        Player::Black
    }
}

impl GameSession {
    pub fn new(white_user_id: u64, black_user_id: u64, first_player: Player) -> Self {
        Self {
            white_user_id,
            black_user_id,
            board: Board::standard_starting_position(),
            current_player: first_player,
            remaining_dice: Vec::new(),
            last_roll: None,
            last_round_mars: false,
            required_moves: 0,
            move_history: Vec::new(),
            next_sequence_no: 0,
            round_no: first_round_no(),
            white_score: 0,
            black_score: 0,
            last_round_winner: None,
            phase: RoundPhase::Playing,
            ai_side: None,
            ai_used: false,
        }
    }

    pub fn hand_side_to_ai(&mut self, player: Player) {
        self.ai_side = Some(player);
        self.ai_used = true;
        if self.current_player == player {
            while let Some((board, remaining_dice)) = self.move_history.pop() {
                self.board = board;
                self.remaining_dice = remaining_dice;
            }
        }
    }

    pub fn score_for(&self, player: Player) -> u16 {
        match player {
            Player::White => self.white_score,
            Player::Black => self.black_score,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.phase == RoundPhase::Playing
    }

    pub fn next_round_at_ms(&self) -> Option<i64> {
        match self.phase {
            RoundPhase::RoundEnded { next_round_at_ms } if next_round_at_ms != ROUND_END_NOT_SCHEDULED => {
                Some(next_round_at_ms)
            }
            RoundPhase::RoundEnded { .. } | RoundPhase::Playing => None,
        }
    }

    pub fn finish_round(&mut self, winner: Player, next_round_at_ms: i64) -> RoundOutcome {
        let points = self.board.win_points(winner);
        self.last_round_mars = self.board.is_mars(winner);
        match winner {
            Player::White => self.white_score += points,
            Player::Black => self.black_score += points,
        }
        self.last_round_winner = Some(winner);
        self.phase = RoundPhase::RoundEnded { next_round_at_ms };

        if self.score_for(winner) >= MATCH_TARGET_SCORE {
            RoundOutcome::MatchWon(winner)
        } else {
            RoundOutcome::MatchContinues
        }
    }

    pub fn start_next_round(&mut self) {
        self.board = Board::standard_starting_position();
        self.current_player = self.last_round_winner.unwrap_or(self.current_player);
        self.remaining_dice.clear();
        self.last_roll = None;
        self.last_round_mars = false;
        self.move_history.clear();
        self.required_moves = 0;
        self.round_no += 1;
        self.phase = RoundPhase::Playing;
    }

    fn reject_when_round_is_over(&self) -> Option<ActionResult> {
        (!self.is_playing()).then_some(ActionResult::Err("round_not_active"))
    }

    pub fn player_for_user(&self, user_id: u64) -> Option<Player> {
        if user_id == self.white_user_id {
            Some(Player::White)
        } else if user_id == self.black_user_id {
            Some(Player::Black)
        } else {
            None
        }
    }

    pub fn user_id_for(&self, player: Player) -> u64 {
        match player {
            Player::White => self.white_user_id,
            Player::Black => self.black_user_id,
        }
    }

    pub fn roll_dice(&mut self, player: Player) -> ActionResult {
        if let Some(rejection) = self.reject_when_round_is_over() {
            return rejection;
        }
        if player != self.current_player {
            return ActionResult::Err("not_your_turn");
        }
        if !self.remaining_dice.is_empty() {
            return ActionResult::Err("dice_already_rolled");
        }

        let dice = DiceRoll::random();
        self.remaining_dice = dice.values();
        self.last_roll = Some([dice.die1, dice.die2]);
        self.move_history.clear();
        self.required_moves = legal_turn_sequences(&self.board, self.current_player, dice)
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0);

        let no_legal_moves = self.required_moves == 0;
        let closed_out = if no_legal_moves { self.pass_turn() } else { None };

        ActionResult::Rolled { die1: dice.die1, die2: dice.die2, no_legal_moves, closed_out }
    }

    fn pass_turn(&mut self) -> Option<Player> {
        self.move_history.clear();
        self.remaining_dice.clear();
        self.required_moves = 0;

        let next = self.current_player.opponent();
        if self.board.turn_is_skipped(next) {
            return Some(next);
        }
        self.current_player = next;
        None
    }

    pub fn make_move(&mut self, player: Player, origin: Origin, die: u8) -> ActionResult {
        if let Some(rejection) = self.reject_when_round_is_over() {
            return rejection;
        }
        if player != self.current_player {
            return ActionResult::Err("not_your_turn");
        }
        if self.remaining_dice.is_empty() {
            return ActionResult::Err("no_dice_rolled");
        }

        let mv = Move { origin, die };
        if !self.board.is_legal_move(player, mv) {
            return ActionResult::Err("illegal_move");
        }

        self.move_history
            .push((self.board.clone(), self.remaining_dice.clone()));
        self.board.apply_move(player, mv);
        remove_one(&mut self.remaining_dice, die);

        let origin_point = match origin {
            Origin::Bar => None,
            Origin::Point(point) => Some(point),
        };
        let sequence_no = self.next_sequence_no;
        self.next_sequence_no += 1;

        if let Some(winner) = self.board.winner() {
            self.phase = RoundPhase::RoundEnded { next_round_at_ms: ROUND_END_NOT_SCHEDULED };
            return ActionResult::RoundWon {
                winner,
                die,
                origin_point,
                sequence_no,
                round_no: self.round_no,
            };
        }

        ActionResult::MoveApplied {
            die,
            origin_point,
            sequence_no,
            round_no: self.round_no,
        }
    }

    pub fn undo(&mut self, player: Player) -> ActionResult {
        if let Some(rejection) = self.reject_when_round_is_over() {
            return rejection;
        }
        if player != self.current_player {
            return ActionResult::Err("not_your_turn");
        }
        let Some((board, remaining_dice)) = self.move_history.pop() else {
            return ActionResult::Err("nothing_to_undo");
        };
        self.board = board;
        self.remaining_dice = remaining_dice;

        ActionResult::Ok
    }

    pub fn confirm_turn(&mut self, player: Player) -> ActionResult {
        if let Some(rejection) = self.reject_when_round_is_over() {
            return rejection;
        }
        if player != self.current_player {
            return ActionResult::Err("not_your_turn");
        }
        if self.move_history.len() < self.required_moves {
            return ActionResult::Err("moves_remaining");
        }

        ActionResult::TurnPassed { closed_out: self.pass_turn() }
    }
}

fn remove_one(values: &mut Vec<u8>, value: u8) {
    if let Some(pos) = values.iter().position(|&v| v == value) {
        values.remove(pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocked_bar_session() -> GameSession {
        let mut points = [0i8; 24];
        for point in 19..=24usize {
            points[point - 1] = -2;
        }
        let board: Board = serde_json::from_value(serde_json::json!({
            "points": points,
            "bar_white": 1,
            "bar_black": 0,
            "off_white": 0,
            "off_black": 0,
        }))
        .expect("Board wire formatı");
        let mut session = GameSession::new(1, 2, Player::White);
        session.board = board;
        session
    }

    fn one_move_from_winning_session() -> GameSession {
        one_move_from_winning_session_with_black_off(0)
    }

    fn one_move_from_winning_session_with_black_off(black_off: u8) -> GameSession {
        let mut points = [0i8; 24];
        points[0] = 1;
        let board: Board = serde_json::from_value(serde_json::json!({
            "points": points,
            "bar_white": 0,
            "bar_black": 0,
            "off_white": 14,
            "off_black": black_off,
        }))
        .expect("Board wire formatı");
        let mut session = GameSession::new(1, 2, Player::White);
        session.board = board;
        session
    }

    fn win_the_round_as_white(session: &mut GameSession) -> ActionResult {
        let ActionResult::Rolled { die1, .. } = session.roll_dice(Player::White) else {
            panic!("zar atılmalıydı");
        };
        session.make_move(Player::White, Origin::Point(1), die1)
    }

    #[test]
    fn the_winning_move_ends_the_round_and_blocks_every_further_action() {
        let mut session = one_move_from_winning_session();

        let result = win_the_round_as_white(&mut session);

        assert!(matches!(result, ActionResult::RoundWon { winner: Player::White, .. }));
        assert!(!session.is_playing());
        assert!(matches!(session.roll_dice(Player::White), ActionResult::Err("round_not_active")));
        assert!(matches!(session.undo(Player::White), ActionResult::Err("round_not_active")));
        assert!(matches!(session.confirm_turn(Player::White), ActionResult::Err("round_not_active")));
        assert!(matches!(
            session.make_move(Player::White, Origin::Point(1), 1),
            ActionResult::Err("round_not_active")
        ));
    }

    #[test]
    fn finishing_a_round_gives_the_winner_a_point_and_schedules_the_next_round() {
        let mut session = one_move_from_winning_session_with_black_off(1);
        win_the_round_as_white(&mut session);

        let outcome = session.finish_round(Player::White, 12_345);

        assert_eq!(outcome, RoundOutcome::MatchContinues);
        assert_eq!((session.white_score, session.black_score), (1, 0));
        assert_eq!(session.last_round_winner, Some(Player::White));
        assert_eq!(session.next_round_at_ms(), Some(12_345));
    }

    #[test]
    fn the_next_round_starts_fresh_and_the_previous_winner_moves_first() {
        let mut session = one_move_from_winning_session();
        win_the_round_as_white(&mut session);
        let moves_played = session.next_sequence_no;
        session.finish_round(Player::Black, 1);

        session.start_next_round();

        assert!(session.is_playing());
        assert_eq!(session.board, Board::standard_starting_position());
        assert_eq!(session.current_player, Player::Black);
        assert_eq!(session.round_no, 2);
        assert!(session.remaining_dice.is_empty() && session.move_history.is_empty());
        assert_eq!(session.required_moves, 0);
        assert_eq!(session.next_sequence_no, moves_played, "hamle sırası maç boyunca devam etmeli");
        assert_eq!(session.next_round_at_ms(), None);
    }

    fn win_and_finish_the_round(session: &mut GameSession) -> RoundOutcome {
        let ActionResult::RoundWon { winner, .. } = win_the_round_as_white(session) else {
            panic!("tur kazanılmalıydı");
        };
        session.finish_round(winner, 0)
    }

    #[test]
    fn a_mars_is_worth_two_points() {
        let mut session = one_move_from_winning_session();

        win_and_finish_the_round(&mut session);

        assert_eq!((session.white_score, session.black_score), (2, 0));
    }

    #[test]
    fn the_round_remembers_whether_it_was_a_mars_until_the_next_round_starts() {
        let mut mars = one_move_from_winning_session();
        win_and_finish_the_round(&mut mars);
        assert!(mars.last_round_mars);
        mars.start_next_round();
        assert!(!mars.last_round_mars);

        let mut plain = one_move_from_winning_session_with_black_off(1);
        win_and_finish_the_round(&mut plain);
        assert!(!plain.last_round_mars);
    }

    #[test]
    fn a_win_after_the_opponent_bore_off_a_checker_is_worth_one_point() {
        let mut session = one_move_from_winning_session_with_black_off(1);

        win_and_finish_the_round(&mut session);

        assert_eq!((session.white_score, session.black_score), (1, 0));
    }

    #[test]
    fn a_mars_can_carry_the_score_past_the_target_and_still_ends_the_match() {
        let mut session = one_move_from_winning_session();
        session.white_score = MATCH_TARGET_SCORE - 1;

        let outcome = win_and_finish_the_round(&mut session);

        assert_eq!(outcome, RoundOutcome::MatchWon(Player::White));
        assert_eq!(session.white_score, MATCH_TARGET_SCORE + 1);
    }

    #[test]
    fn a_mars_from_two_points_leaves_the_match_going_and_from_three_points_ends_it() {
        let mut from_two = one_move_from_winning_session();
        from_two.white_score = 2;
        assert_eq!(win_and_finish_the_round(&mut from_two), RoundOutcome::MatchContinues);
        assert_eq!(from_two.white_score, 4);

        let mut from_three = one_move_from_winning_session();
        from_three.white_score = 3;
        assert_eq!(win_and_finish_the_round(&mut from_three), RoundOutcome::MatchWon(Player::White));
        assert_eq!(from_three.white_score, 5);
    }

    #[test]
    fn the_first_side_to_reach_the_target_wins_the_match() {
        let mut session = GameSession::new(1, 2, Player::White);

        for round in 1..MATCH_TARGET_SCORE {
            assert_eq!(session.finish_round(Player::White, 0), RoundOutcome::MatchContinues, "{round}. oyundan sonra maç sürmeli");
            session.start_next_round();
        }
        session.finish_round(Player::Black, 0);
        session.start_next_round();

        assert_eq!(session.finish_round(Player::White, 0), RoundOutcome::MatchWon(Player::White));
        assert_eq!((session.white_score, session.black_score), (MATCH_TARGET_SCORE, 1));
    }

    #[test]
    fn the_first_round_is_opened_by_either_side_at_random() {
        let mut seen = [false; 2];
        for _ in 0..200 {
            match random_first_player() {
                Player::White => seen[0] = true,
                Player::Black => seen[1] = true,
            }
        }

        assert!(seen.iter().all(|side| *side));
    }

    #[test]
    fn a_session_survives_a_json_round_trip_in_every_phase() {
        let mut session = one_move_from_winning_session();
        assert_eq!(serde_json::from_str::<GameSession>(&serde_json::to_string(&session).unwrap()).unwrap(), session);

        win_the_round_as_white(&mut session);
        session.finish_round(Player::White, 99);

        assert_eq!(serde_json::from_str::<GameSession>(&serde_json::to_string(&session).unwrap()).unwrap(), session);
    }

    #[test]
    fn handing_a_side_to_the_ai_rewinds_its_half_played_turn() {
        let mut session = GameSession::new(1, 2, Player::Black);
        let ActionResult::Rolled { .. } = session.roll_dice(Player::Black) else {
            panic!("zar atılmalıydı");
        };
        let after_roll = (session.board.clone(), session.remaining_dice.clone());
        let first_move = legal_turn_sequences(&session.board, Player::Black, DiceRoll::new(session.remaining_dice[0], session.remaining_dice[1]))
            .first()
            .and_then(|sequence| sequence.first().copied())
            .expect("başlangıçta hamle vardır");
        assert!(matches!(session.make_move(Player::Black, first_move.origin, first_move.die), ActionResult::MoveApplied { .. }));

        session.hand_side_to_ai(Player::Black);

        assert_eq!(session.ai_side, Some(Player::Black));
        assert!(session.ai_used);
        assert!(session.move_history.is_empty());
        assert_eq!((session.board.clone(), session.remaining_dice.clone()), after_roll);
    }

    #[test]
    fn handing_the_other_sides_seat_to_the_ai_leaves_the_current_turn_alone() {
        let mut session = GameSession::new(1, 2, Player::White);
        session.roll_dice(Player::White);
        let before = session.clone();

        session.hand_side_to_ai(Player::Black);

        assert_eq!(session.board, before.board);
        assert_eq!(session.remaining_dice, before.remaining_dice);
        assert_eq!(session.ai_side, Some(Player::Black));
    }

    #[test]
    fn a_snapshot_written_before_matches_existed_still_loads_as_round_one() {
        let mut value = serde_json::to_value(GameSession::new(1, 2, Player::White)).unwrap();
        let fields = value.as_object_mut().unwrap();
        for added_later in ["round_no", "white_score", "black_score", "last_round_winner", "phase", "ai_side", "ai_used"] {
            fields.remove(added_later);
        }

        let loaded: GameSession = serde_json::from_value(value).unwrap();

        assert_eq!((loaded.round_no, loaded.white_score, loaded.black_score), (1, 0, 0));
        assert!(loaded.is_playing());
    }

    #[test]
    fn roll_without_any_legal_move_reports_it_and_passes_the_turn() {
        let mut session = blocked_bar_session();

        let ActionResult::Rolled { no_legal_moves, closed_out, .. } = session.roll_dice(Player::White) else {
            panic!("zar atılmalıydı");
        };

        assert!(no_legal_moves);
        assert_eq!(closed_out, None);
        assert_eq!(session.required_moves, 0);
        assert_eq!(session.current_player, Player::Black);
        assert!(session.remaining_dice.is_empty());
    }

    #[test]
    fn a_normal_confirm_hands_the_turn_to_the_opponent() {
        let mut session = GameSession::new(1, 2, Player::White);

        let result = session.confirm_turn(Player::White);

        assert!(matches!(result, ActionResult::TurnPassed { closed_out: None }));
        assert_eq!(session.current_player, Player::Black);
    }

    #[test]
    fn a_closed_out_opponent_does_not_get_to_roll_and_the_turn_comes_straight_back() {
        let mut session = blocked_bar_session();
        session.current_player = Player::Black;

        let result = session.confirm_turn(Player::Black);

        assert!(matches!(result, ActionResult::TurnPassed { closed_out: Some(Player::White) }));
        assert_eq!(session.current_player, Player::Black);
        assert!(session.remaining_dice.is_empty() && session.move_history.is_empty());
        assert_eq!(session.required_moves, 0);
        assert!(matches!(session.roll_dice(Player::White), ActionResult::Err("not_your_turn")));
        assert!(matches!(session.roll_dice(Player::Black), ActionResult::Rolled { .. }));
    }

    #[test]
    fn when_both_sides_are_closed_out_the_turn_still_alternates() {
        let mut session = blocked_bar_session();
        let mut board = serde_json::to_value(&session.board).unwrap();
        for point in 1..=6usize {
            board["points"][point - 1] = serde_json::json!(2);
        }
        board["bar_black"] = serde_json::json!(1);
        session.board = serde_json::from_value(board).unwrap();
        session.current_player = Player::Black;

        let result = session.confirm_turn(Player::Black);

        assert!(matches!(result, ActionResult::TurnPassed { closed_out: None }));
        assert_eq!(session.current_player, Player::White);
    }

    #[test]
    fn the_last_roll_is_remembered_until_the_next_game_starts() {
        let mut session = GameSession::new(1, 2, Player::White);
        assert_eq!(session.last_roll, None);

        let ActionResult::Rolled { die1, die2, .. } = session.roll_dice(Player::White) else {
            panic!("zar atılmalıydı");
        };
        assert_eq!(session.last_roll, Some([die1, die2]));

        session.start_next_round();
        assert_eq!(session.last_roll, None);
    }

    #[test]
    fn roll_with_legal_moves_never_reports_no_legal_moves() {
        for _ in 0..200 {
            let mut session = GameSession::new(1, 2, Player::White);

            let ActionResult::Rolled { no_legal_moves, .. } = session.roll_dice(Player::White) else {
                panic!("zar atılmalıydı");
            };

            assert!(!no_legal_moves);
            assert!(session.required_moves > 0);
            assert_eq!(session.current_player, Player::White);
        }
    }
}
