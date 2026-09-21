use chrono::{DateTime, NaiveDateTime};
use entity::{
    game_moves::GameMovePlayer,
    games::GameStatus,
    repository::game_batch_repo::{MatchEndRow, NewMoveRow, NewRoundRow, ScoreDelta},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tavla_core::Player;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutboxEvent {
    Move {
        game_id: u64,
        round_no: u16,
        sequence_no: u32,
        player: Player,
        origin_point: Option<u8>,
        die: u8,
        #[serde(default)]
        is_ai: bool,
        at_ms: i64,
    },
    RoundWon {
        game_id: u64,
        round_no: u16,
        winner_user_id: u64,
        white_score: u16,
        black_score: u16,
        at_ms: i64,
    },
    MatchFinished {
        game_id: u64,
        winner_user_id: u64,
        loser_user_id: u64,
        white_score: u16,
        black_score: u16,
        at_ms: i64,
        #[serde(default = "counts_towards_scores")]
        ranked: bool,
    },
    MatchAbandoned {
        game_id: u64,
        white_score: u16,
        black_score: u16,
        at_ms: i64,
    },
}

fn counts_towards_scores() -> bool {
    true
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct BatchPlan {
    pub moves: Vec<NewMoveRow>,
    pub rounds: Vec<NewRoundRow>,
    pub match_ends: Vec<PlannedMatchEnd>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PlannedMatchEnd {
    pub row: MatchEndRow,
    pub winner_and_loser: Option<(u64, u64)>,
}

pub fn score_deltas(match_ends: &[PlannedMatchEnd], applied: &[bool]) -> HashMap<u64, ScoreDelta> {
    let mut deltas: HashMap<u64, ScoreDelta> = HashMap::new();
    for (match_end, was_applied) in match_ends.iter().zip(applied) {
        let (true, Some((winner, loser))) = (*was_applied, match_end.winner_and_loser) else {
            continue;
        };
        deltas.entry(winner).or_default().wins += 1;
        deltas.entry(loser).or_default().losses += 1;
    }
    deltas
}

fn datetime_from_ms(at_ms: i64) -> NaiveDateTime {
    DateTime::from_timestamp_millis(at_ms)
        .map(|datetime| datetime.naive_utc())
        .unwrap_or_default()
}

fn white_or_black(player: Player) -> GameMovePlayer {
    match player {
        Player::White => GameMovePlayer::White,
        Player::Black => GameMovePlayer::Black,
    }
}

pub fn plan_batch(events: &[OutboxEvent]) -> BatchPlan {
    let mut plan = BatchPlan::default();

    for event in events {
        match event {
            OutboxEvent::Move { game_id, round_no, sequence_no, player, origin_point, die, is_ai, at_ms } => {
                plan.moves.push(NewMoveRow {
                    game_id: *game_id,
                    round_no: *round_no,
                    sequence_no: *sequence_no,
                    player: white_or_black(*player),
                    origin_point: *origin_point,
                    die: *die,
                    is_ai: *is_ai,
                    created_at: datetime_from_ms(*at_ms),
                });
            }
            OutboxEvent::RoundWon { game_id, round_no, winner_user_id, white_score, black_score, at_ms } => {
                plan.rounds.push(NewRoundRow {
                    game_id: *game_id,
                    round_no: *round_no,
                    winner_user_id: *winner_user_id,
                    white_score: *white_score,
                    black_score: *black_score,
                    created_at: datetime_from_ms(*at_ms),
                });
            }
            OutboxEvent::MatchFinished { game_id, winner_user_id, loser_user_id, white_score, black_score, at_ms, ranked } => {
                plan.match_ends.push(PlannedMatchEnd {
                    row: MatchEndRow {
                        game_id: *game_id,
                        status: GameStatus::Finished,
                        winner_user_id: Some(*winner_user_id),
                        white_score: *white_score,
                        black_score: *black_score,
                        finished_at: datetime_from_ms(*at_ms),
                    },
                    winner_and_loser: ranked.then_some((*winner_user_id, *loser_user_id)),
                });
            }
            OutboxEvent::MatchAbandoned { game_id, white_score, black_score, at_ms } => {
                plan.match_ends.push(PlannedMatchEnd {
                    row: MatchEndRow {
                        game_id: *game_id,
                        status: GameStatus::Abandoned,
                        winner_user_id: None,
                        white_score: *white_score,
                        black_score: *black_score,
                        finished_at: datetime_from_ms(*at_ms),
                    },
                    winner_and_loser: None,
                });
            }
        }
    }

    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT_MS: i64 = 1_790_000_000_000;

    fn move_event(sequence_no: u32, player: Player) -> OutboxEvent {
        OutboxEvent::Move { game_id: 7, round_no: 2, sequence_no, player, origin_point: Some(6), die: 3, is_ai: false, at_ms: AT_MS }
    }

    #[test]
    fn events_survive_a_json_round_trip() {
        let events = vec![
            move_event(0, Player::White),
            OutboxEvent::RoundWon { game_id: 7, round_no: 2, winner_user_id: 11, white_score: 1, black_score: 0, at_ms: AT_MS },
            OutboxEvent::MatchFinished { game_id: 7, winner_user_id: 11, loser_user_id: 12, white_score: 5, black_score: 3, at_ms: AT_MS, ranked: true },
            OutboxEvent::MatchAbandoned { game_id: 8, white_score: 1, black_score: 1, at_ms: AT_MS },
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            assert_eq!(serde_json::from_str::<OutboxEvent>(&json).unwrap(), event, "{json}");
        }
    }

    #[test]
    fn moves_are_grouped_in_order_with_the_white_black_mapping() {
        let plan = plan_batch(&[move_event(0, Player::White), move_event(1, Player::Black)]);

        assert_eq!(plan.moves.len(), 2);
        assert_eq!((plan.moves[0].sequence_no, &plan.moves[0].player), (0, &GameMovePlayer::White));
        assert_eq!((plan.moves[1].sequence_no, &plan.moves[1].player), (1, &GameMovePlayer::Black));
        assert_eq!(plan.moves[0].round_no, 2);
        assert_eq!(plan.moves[0].created_at.and_utc().timestamp_millis(), AT_MS);
        assert!(plan.rounds.is_empty() && plan.match_ends.is_empty());
    }

    #[test]
    fn a_finished_match_becomes_a_finished_row_with_a_winner_and_final_scores() {
        let plan = plan_batch(&[OutboxEvent::MatchFinished {
            game_id: 7,
            winner_user_id: 11,
            loser_user_id: 12,
            white_score: 5,
            black_score: 3,
            at_ms: AT_MS,
            ranked: true,
        }]);

        let end = &plan.match_ends[0];
        assert_eq!(end.row.status, GameStatus::Finished);
        assert_eq!(end.row.winner_user_id, Some(11));
        assert_eq!((end.row.white_score, end.row.black_score), (5, 3));
        assert_eq!(end.winner_and_loser, Some((11, 12)));
    }

    #[test]
    fn an_abandoned_match_has_no_winner_and_no_score_change() {
        let plan = plan_batch(&[OutboxEvent::MatchAbandoned { game_id: 8, white_score: 2, black_score: 1, at_ms: AT_MS }]);

        assert_eq!(plan.match_ends[0].row.status, GameStatus::Abandoned);
        assert_eq!(plan.match_ends[0].row.winner_user_id, None);
        assert!(score_deltas(&plan.match_ends, &[true]).is_empty());
    }

    #[test]
    fn score_deltas_add_up_per_user_and_skip_matches_that_were_already_closed() {
        let finished = |game_id, winner_user_id, loser_user_id| OutboxEvent::MatchFinished {
            game_id,
            winner_user_id,
            loser_user_id,
            white_score: 5,
            black_score: 0,
            at_ms: AT_MS,
            ranked: true,
        };
        let plan = plan_batch(&[finished(1, 10, 20), finished(2, 10, 30), finished(3, 40, 10)]);

        let deltas = score_deltas(&plan.match_ends, &[true, true, false]);

        assert_eq!(deltas.get(&10), Some(&ScoreDelta { wins: 2, losses: 0 }));
        assert_eq!(deltas.get(&20), Some(&ScoreDelta { wins: 0, losses: 1 }));
        assert_eq!(deltas.get(&30), Some(&ScoreDelta { wins: 0, losses: 1 }));
        assert_eq!(deltas.get(&40), None, "zaten kapatılmış maç puanı bir daha artırmamalı");
    }

    #[test]
    fn a_match_the_computer_played_is_closed_without_touching_wins_and_losses() {
        let plan = plan_batch(&[OutboxEvent::MatchFinished {
            game_id: 7,
            winner_user_id: 11,
            loser_user_id: 12,
            white_score: 5,
            black_score: 2,
            at_ms: AT_MS,
            ranked: false,
        }]);

        assert_eq!(plan.match_ends[0].row.status, GameStatus::Finished);
        assert_eq!(plan.match_ends[0].row.winner_user_id, Some(11));
        assert!(score_deltas(&plan.match_ends, &[true]).is_empty());
    }

    #[test]
    fn an_event_queued_before_the_ranked_flag_existed_still_counts() {
        let json = r#"{"kind":"match_finished","game_id":7,"winner_user_id":11,"loser_user_id":12,"white_score":5,"black_score":3,"at_ms":1}"#;

        let OutboxEvent::MatchFinished { ranked, .. } = serde_json::from_str(json).unwrap() else {
            panic!("MatchFinished bekleniyordu");
        };

        assert!(ranked);
    }

    #[test]
    fn the_computer_flag_of_a_move_reaches_the_row_and_defaults_to_human() {
        let ai_move = OutboxEvent::Move { game_id: 7, round_no: 1, sequence_no: 4, player: Player::Black, origin_point: None, die: 2, is_ai: true, at_ms: AT_MS };
        let old_event = r#"{"kind":"move","game_id":7,"round_no":1,"sequence_no":5,"player":"white","origin_point":6,"die":3,"at_ms":1}"#;

        let plan = plan_batch(&[ai_move, serde_json::from_str(old_event).unwrap()]);

        assert!(plan.moves[0].is_ai);
        assert!(!plan.moves[1].is_ai);
    }
}
