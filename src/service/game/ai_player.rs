use tavla_core::{choose_move_sequence, DiceRoll, Move, Player};

use super::session::GameSession;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiAction {
    Wait,
    Roll,
    Move(Move),
    Confirm,
    Undo,
}

pub fn next_ai_action(game: &GameSession, ai_player: Player) -> AiAction {
    if !game.is_playing() || game.current_player != ai_player {
        return AiAction::Wait;
    }
    if game.remaining_dice.is_empty() && game.move_history.is_empty() {
        return AiAction::Roll;
    }

    let (turn_start_board, turn_start_dice) = match game.move_history.first() {
        Some((board, dice)) => (board, dice.as_slice()),
        None => (&game.board, game.remaining_dice.as_slice()),
    };
    let first_die = turn_start_dice.first().copied().unwrap_or(1);
    let second_die = turn_start_dice.get(1).copied().unwrap_or(first_die);
    let plan = choose_move_sequence(turn_start_board, ai_player, DiceRoll::new(first_die, second_die));

    let moves_played = game.move_history.len();
    match plan
        .get(moves_played)
        .copied()
        .filter(|planned| game.board.is_legal_move(ai_player, *planned))
    {
        Some(planned) => AiAction::Move(planned),
        None if moves_played >= game.required_moves => AiAction::Confirm,
        None if !game.move_history.is_empty() => AiAction::Undo,
        None => AiAction::Wait,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::game::session::{ActionResult, RoundOutcome};

    fn session_with(current: Player) -> GameSession {
        GameSession::new(1, 2, current)
    }

    fn apply(game: &mut GameSession, player: Player, action: AiAction) -> ActionResult {
        match action {
            AiAction::Roll => game.roll_dice(player),
            AiAction::Move(mv) => game.make_move(player, mv.origin, mv.die),
            AiAction::Confirm => game.confirm_turn(player),
            AiAction::Undo => game.undo(player),
            AiAction::Wait => panic!("bekleme aksiyonu uygulanamaz"),
        }
    }

    #[test]
    fn it_waits_when_it_is_not_its_turn() {
        assert_eq!(next_ai_action(&session_with(Player::White), Player::Black), AiAction::Wait);
    }

    #[test]
    fn a_fresh_turn_starts_with_a_roll() {
        assert_eq!(next_ai_action(&session_with(Player::Black), Player::Black), AiAction::Roll);
    }

    #[test]
    fn it_does_nothing_while_the_round_is_over() {
        let mut game = session_with(Player::Black);
        game.finish_round(Player::Black, 1);

        assert_eq!(next_ai_action(&game, Player::Black), AiAction::Wait);
    }

    #[test]
    fn it_plays_the_required_moves_and_then_confirms_the_turn() {
        for _ in 0..30 {
            let mut game = session_with(Player::Black);
            assert!(matches!(apply(&mut game, Player::Black, AiAction::Roll), ActionResult::Rolled { .. }));

            let mut played = 0;
            loop {
                match next_ai_action(&game, Player::Black) {
                    AiAction::Move(mv) => {
                        assert!(matches!(game.make_move(Player::Black, mv.origin, mv.die), ActionResult::MoveApplied { .. }));
                        played += 1;
                    }
                    AiAction::Confirm => {
                        assert!(matches!(game.confirm_turn(Player::Black), ActionResult::TurnPassed { closed_out: None }));
                        break;
                    }
                    other => panic!("beklenmeyen aksiyon: {other:?}"),
                }
            }

            assert!(played >= 2, "başlangıç konumunda en az iki hamle oynanır");
            assert_eq!(game.current_player, Player::White);
        }
    }

    #[test]
    fn an_interrupted_turn_resumes_with_the_same_plan() {
        let mut uninterrupted = session_with(Player::Black);
        apply(&mut uninterrupted, Player::Black, AiAction::Roll);
        let mut interrupted = uninterrupted.clone();

        let first = next_ai_action(&interrupted, Player::Black);
        apply(&mut interrupted, Player::Black, first);
        let mut restored: GameSession = serde_json::from_str(&serde_json::to_string(&interrupted).unwrap()).unwrap();

        for game in [&mut uninterrupted, &mut restored] {
            let mut moves = 0;
            while let AiAction::Move(mv) = next_ai_action(game, Player::Black) {
                game.make_move(Player::Black, mv.origin, mv.die);
                moves += 1;
                assert!(moves < 10);
            }
            assert_eq!(next_ai_action(game, Player::Black), AiAction::Confirm);
        }

        assert_eq!(uninterrupted.board, restored.board);
    }

    #[test]
    fn a_half_played_turn_that_does_not_match_the_plan_is_undone() {
        let mut game = session_with(Player::Black);
        apply(&mut game, Player::Black, AiAction::Roll);
        let dice = (game.remaining_dice[0], game.remaining_dice[1]);
        let plan = choose_move_sequence(&game.board, Player::Black, DiceRoll::new(dice.0, dice.1));
        let off_plan = tavla_core::legal_turn_sequences(&game.board, Player::Black, DiceRoll::new(dice.0, dice.1))
            .into_iter()
            .filter_map(|sequence| sequence.first().copied())
            .find(|first| Some(first) != plan.first());
        let Some(off_plan) = off_plan else { return };
        game.make_move(Player::Black, off_plan.origin, off_plan.die);

        let mut steps = 0;
        while let action @ (AiAction::Move(_) | AiAction::Undo) = next_ai_action(&game, Player::Black) {
            let result = apply(&mut game, Player::Black, action);
            assert!(!matches!(result, ActionResult::Err(_)), "{action:?} reddedildi");
            steps += 1;
            assert!(steps < 20, "yapay zeka takıldı");
        }

        assert_eq!(next_ai_action(&game, Player::Black), AiAction::Confirm);
    }

    #[test]
    fn two_computer_players_can_play_a_whole_round() {
        let mut game = session_with(Player::White);
        let mut winner = None;

        for _ in 0..20_000 {
            let player = game.current_player;
            let action = next_ai_action(&game, player);
            let result = match action {
                AiAction::Wait => panic!("sıradaki oyuncu bekliyor: {:?}", game.current_player),
                other => apply(&mut game, player, other),
            };
            match result {
                ActionResult::Err(reason) => panic!("{action:?} reddedildi: {reason}"),
                ActionResult::RoundWon { winner: round_winner, .. } => {
                    winner = Some(round_winner);
                    break;
                }
                _ => {}
            }
        }

        let winner = winner.expect("oyun bitmeli");
        assert_eq!(game.finish_round(winner, 0), RoundOutcome::MatchContinues);
    }
}
