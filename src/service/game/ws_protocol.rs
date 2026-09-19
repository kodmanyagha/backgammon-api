//! WebSocket tel protokolü: client<->server mesaj tipleri. Her değişiklikten
//! sonra sunucu TAM bir `State` anlık görüntüsü yayınlar (diff/patch değil —
//! neden: bkz. proje sohbeti, tavla için bant genişliği önemsiz, sağlamlık
//! ve basitlik daha değerli).

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
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OriginDto {
    Bar,
    Point { point: u8 },
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
        required_moves: usize,
        move_history_len: usize,
        your_color: Player,
        /// `current_player`'ın hamlesini bitirmesi (`ConfirmTurn`) için kalan
        /// süre — zar atılmadıysa `None` (bkz. `ManagedSession::turn_deadline`).
        turn_expires_in_ms: Option<u64>,
    },
    DiceRolled {
        die1: u8,
        die2: u8,
        no_legal_moves: bool,
    },
    OpponentConnected,
    OpponentDisconnected,
    Emoji {
        from: Player,
        emoji: String,
    },
    GameOver {
        winner: Player,
    },
    Error {
        message: String,
    },
}
