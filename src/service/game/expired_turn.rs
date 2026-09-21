use tavla_core::Player;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpiredTurn {
    AutoPlay,
    AskToContinueWithAi { present: Player, absent: Player },
    NobodyPresent,
}

pub fn decide_expired_turn(
    current: Player,
    current_connected: bool,
    opponent_connected: bool,
    opponent_is_ai: bool,
) -> ExpiredTurn {
    let opponent = current.opponent();
    let opponent_in_game = opponent_connected || opponent_is_ai;

    match (current_connected, opponent_in_game, opponent_connected) {
        (true, true, _) => ExpiredTurn::AutoPlay,
        (true, false, _) => ExpiredTurn::AskToContinueWithAi { present: current, absent: opponent },
        (false, _, true) => ExpiredTurn::AskToContinueWithAi { present: opponent, absent: current },
        (false, _, false) => ExpiredTurn::NobodyPresent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT: Player = Player::White;

    #[test]
    fn an_idle_player_in_a_full_game_gets_their_turn_played_instead_of_losing() {
        assert_eq!(decide_expired_turn(CURRENT, true, true, false), ExpiredTurn::AutoPlay);
    }

    #[test]
    fn an_idle_player_facing_the_computer_gets_their_turn_played_too() {
        assert_eq!(decide_expired_turn(CURRENT, true, false, true), ExpiredTurn::AutoPlay);
    }

    #[test]
    fn an_idle_player_whose_opponent_dropped_is_asked_about_the_computer() {
        assert_eq!(
            decide_expired_turn(CURRENT, true, false, false),
            ExpiredTurn::AskToContinueWithAi { present: Player::White, absent: Player::Black }
        );
    }

    #[test]
    fn a_dropped_player_whose_time_ran_out_makes_the_connected_opponent_the_one_who_is_asked() {
        assert_eq!(
            decide_expired_turn(CURRENT, false, true, false),
            ExpiredTurn::AskToContinueWithAi { present: Player::Black, absent: Player::White }
        );
    }

    #[test]
    fn when_no_human_is_connected_nobody_is_asked() {
        assert_eq!(decide_expired_turn(CURRENT, false, false, false), ExpiredTurn::NobodyPresent);
        assert_eq!(decide_expired_turn(CURRENT, false, false, true), ExpiredTurn::NobodyPresent);
    }

    #[test]
    fn the_question_always_goes_to_the_connected_side() {
        for (current, current_connected, opponent_connected) in [
            (Player::White, true, false),
            (Player::Black, true, false),
            (Player::White, false, true),
            (Player::Black, false, true),
        ] {
            let ExpiredTurn::AskToContinueWithAi { present, absent } =
                decide_expired_turn(current, current_connected, opponent_connected, false)
            else {
                panic!("soru bekleniyordu");
            };
            assert_eq!(present, if current_connected { current } else { current.opponent() });
            assert_eq!(absent, present.opponent());
        }
    }
}
