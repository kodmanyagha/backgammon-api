use serde::{Deserialize, Serialize};
use tavla_core::{Board, Origin, Player};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    RollDice,
    MakeMove { origin: OriginDto, die: u8 },
    Undo,
    ConfirmTurn,
    Resign,
    Emoji { emoji: String },
    AcceptAi,
    DeclineAi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OriginDto {
    Bar,
    Point { point: u8 },
}

impl From<Origin> for OriginDto {
    fn from(value: Origin) -> Self {
        match value {
            Origin::Bar => OriginDto::Bar,
            Origin::Point(point) => OriginDto::Point { point },
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct LastMoveDto {
    pub player: Player,
    pub origin: OriginDto,
    pub die: u8,
    pub is_ai: bool,
}

impl From<OriginDto> for Origin {
    fn from(value: OriginDto) -> Self {
        match value {
            OriginDto::Bar => Origin::Bar,
            OriginDto::Point { point } => Origin::Point(point),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    State {
        board: Board,
        current_player: Player,
        remaining_dice: Vec<u8>,
        dice: Option<[u8; 2]>,
        required_moves: usize,
        move_history_len: usize,
        your_color: Player,
        turn_expires_in_ms: Option<u64>,
        white_score: u16,
        black_score: u16,
        round_no: u16,
        last_move: Option<LastMoveDto>,
        opponent_connected: bool,
    },
    DiceRolled {
        die1: u8,
        die2: u8,
        no_legal_moves: bool,
    },
    OpponentConnected,
    OpponentDisconnected,
    AiOffer,
    Emoji {
        from: Player,
        emoji: String,
    },
    GameEnded {
        winner: Player,
        white_score: u16,
        black_score: u16,
        next_round_in_ms: u64,
        mars: bool,
    },
    GameOver {
        winner: Player,
        white_score: u16,
        black_score: u16,
        mars: bool,
    },
    Error {
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(last_move: Option<LastMoveDto>) -> ServerMessage {
        ServerMessage::State {
            board: Board::standard_starting_position(),
            current_player: Player::White,
            remaining_dice: vec![3],
            dice: Some([3, 5]),
            required_moves: 1,
            move_history_len: 1,
            your_color: Player::Black,
            turn_expires_in_ms: None,
            white_score: 0,
            black_score: 0,
            round_no: 1,
            last_move,
            opponent_connected: false,
        }
    }

    #[test]
    fn a_state_after_a_move_names_who_moved_from_where_with_which_die() {
        let last_move = LastMoveDto { player: Player::White, origin: OriginDto::Point { point: 6 }, die: 3, is_ai: true };

        let json = serde_json::to_value(state_with(Some(last_move))).unwrap();

        assert_eq!(
            json["last_move"],
            serde_json::json!({"player": "white", "origin": {"kind": "point", "point": 6}, "die": 3, "is_ai": true})
        );
    }

    #[test]
    fn the_end_of_a_game_and_of_a_match_say_whether_it_was_a_mars() {
        let ended = ServerMessage::GameEnded { winner: Player::White, white_score: 2, black_score: 0, next_round_in_ms: 5000, mars: true };
        let over = ServerMessage::GameOver { winner: Player::White, white_score: 6, black_score: 0, mars: false };

        assert_eq!(serde_json::to_value(ended).unwrap()["mars"], serde_json::json!(true));
        assert_eq!(serde_json::to_value(over).unwrap()["mars"], serde_json::json!(false));
    }

    #[test]
    fn a_state_carries_the_last_roll() {
        let json = serde_json::to_value(state_with(None)).unwrap();

        assert_eq!(json["dice"], serde_json::json!([3, 5]));
    }

    #[test]
    fn a_state_tells_whether_the_opponent_is_connected() {
        let json = serde_json::to_value(state_with(None)).unwrap();

        assert_eq!(json["opponent_connected"], serde_json::json!(false));
    }

    #[test]
    fn a_move_from_the_bar_uses_the_bar_origin() {
        let last_move = LastMoveDto { player: Player::Black, origin: OriginDto::Bar, die: 5, is_ai: false };

        let json = serde_json::to_value(state_with(Some(last_move))).unwrap();

        assert_eq!(json["last_move"]["origin"], serde_json::json!({"kind": "bar"}));
    }

    #[test]
    fn a_state_that_does_not_follow_a_move_has_no_last_move() {
        let json = serde_json::to_value(state_with(None)).unwrap();

        assert!(json["last_move"].is_null());
    }
}
