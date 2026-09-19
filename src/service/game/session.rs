//! Bir maçın SUNUCU-YETKİLİ (authoritative) durumu — saf, I/O'suz mantık.
//! `tavla_app`'in `Game`/`turn_flow`/`drag` modüllerindeki AYNI durum
//! makinesini (`move_history` ile geri alma, `required_moves` ile "Okey"
//! kısıtlaması) burada tekrar ediyoruz, ama artık `tavla_core` paylaşıldığı
//! için hamle doğrulama/zar atma AYNI kod — istemci ile sunucu asla
//! farklı kural yorumlayamaz.

use tavla_core::{legal_turn_sequences, Board, DiceRoll, Move, Origin, Player};

pub struct GameSession {
    pub gold_user_id: u64,
    pub purple_user_id: u64,
    pub board: Board,
    pub current_player: Player,
    pub remaining_dice: Vec<u8>,
    pub required_moves: usize,
    pub move_history: Vec<(Board, Vec<u8>)>,
    pub next_sequence_no: u32,
}

pub enum ActionResult {
    Ok,
    Rolled { die1: u8, die2: u8, no_legal_moves: bool },
    MoveApplied { die: u8, origin_point: Option<u8>, sequence_no: u32 },
    GameOver { winner: Player, die: u8, origin_point: Option<u8>, sequence_no: u32 },
    Err(&'static str),
}

impl GameSession {
    pub fn new(gold_user_id: u64, purple_user_id: u64) -> Self {
        Self {
            gold_user_id,
            purple_user_id,
            board: Board::standard_starting_position(),
            current_player: Player::Black,
            remaining_dice: Vec::new(),
            required_moves: 0,
            move_history: Vec::new(),
            next_sequence_no: 0,
        }
    }

    pub fn player_for_user(&self, user_id: u64) -> Option<Player> {
        if user_id == self.gold_user_id {
            Some(Player::Black)
        } else if user_id == self.purple_user_id {
            Some(Player::White)
        } else {
            None
        }
    }

    pub fn user_id_for(&self, player: Player) -> u64 {
        match player {
            Player::Black => self.gold_user_id,
            Player::White => self.purple_user_id,
        }
    }

    pub fn roll_dice(&mut self, player: Player) -> ActionResult {
        if player != self.current_player {
            return ActionResult::Err("not_your_turn");
        }
        if !self.remaining_dice.is_empty() {
            return ActionResult::Err("dice_already_rolled");
        }

        let dice = DiceRoll::random();
        self.remaining_dice = dice.values();
        self.move_history.clear();
        self.required_moves = legal_turn_sequences(&self.board, self.current_player, dice)
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0);

        let no_legal_moves = self.required_moves == 0;
        if no_legal_moves {
            self.current_player = self.current_player.opponent();
            self.remaining_dice.clear();
        }

        ActionResult::Rolled { die1: dice.die1, die2: dice.die2, no_legal_moves }
    }

    pub fn make_move(&mut self, player: Player, origin: Origin, die: u8) -> ActionResult {
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
            return ActionResult::GameOver {
                winner,
                die,
                origin_point,
                sequence_no,
            };
        }

        ActionResult::MoveApplied {
            die,
            origin_point,
            sequence_no,
        }
    }

    pub fn undo(&mut self, player: Player) -> ActionResult {
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
        if player != self.current_player {
            return ActionResult::Err("not_your_turn");
        }
        if self.move_history.len() < self.required_moves {
            return ActionResult::Err("moves_remaining");
        }

        self.move_history.clear();
        self.current_player = self.current_player.opponent();
        self.remaining_dice.clear();
        self.required_moves = 0;

        ActionResult::Ok
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

    /// Siyah'ın kırık taşı var ve Beyaz 19..=24'ün HEPSİNİ ikişer taşla
    /// tutuyor: Siyah hiçbir zarla giremez (bkz. "hareket edecek zar gelmedi").
    fn blocked_bar_session() -> GameSession {
        let mut points = [0i8; 24];
        for point in 19..=24usize {
            points[point - 1] = -2;
        }
        let board: Board = serde_json::from_value(serde_json::json!({
            "points": points,
            "bar_black": 1,
            "bar_white": 0,
            "off_black": 0,
            "off_white": 0,
        }))
        .expect("Board wire formatı");
        let mut session = GameSession::new(1, 2);
        session.board = board;
        session
    }

    /// Hiçbir hamle yapılamıyorsa sunucu bunu `DiceRolled`'a taşınacak
    /// `no_legal_moves` bayrağıyla AÇIKÇA bildirmeli (istemci artık ayrıca
    /// gelen `State`'e bakıp tahmin etmiyor) ve tur ANINDA rakibe geçmeli.
    #[test]
    fn roll_without_any_legal_move_reports_it_and_passes_the_turn() {
        let mut session = blocked_bar_session();

        let ActionResult::Rolled { no_legal_moves, .. } = session.roll_dice(Player::Black) else {
            panic!("zar atılmalıydı");
        };

        assert!(no_legal_moves);
        assert_eq!(session.required_moves, 0);
        assert_eq!(session.current_player, Player::White);
        assert!(session.remaining_dice.is_empty());
    }

    /// Standart başlangıçta her atışta oynanabilir hamle vardır: bayrak asla
    /// yanlışlıkla `true` gelmemeli, sıra da değişmemeli.
    #[test]
    fn roll_with_legal_moves_never_reports_no_legal_moves() {
        for _ in 0..200 {
            let mut session = GameSession::new(1, 2);

            let ActionResult::Rolled { no_legal_moves, .. } = session.roll_dice(Player::Black) else {
                panic!("zar atılmalıydı");
            };

            assert!(!no_legal_moves);
            assert!(session.required_moves > 0);
            assert_eq!(session.current_player, Player::Black);
        }
    }
}
