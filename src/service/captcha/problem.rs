use rand::{seq::SliceRandom, Rng, RngExt};

pub const OPTION_COUNT: usize = 4;

const DISTRACTOR_SPREAD: u32 = 9;
const DISTRACTOR_COUNT: usize = OPTION_COUNT - 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
}

impl Operator {
    fn symbol(self) -> char {
        match self {
            Operator::Add => '+',
            Operator::Subtract => '−',
            Operator::Multiply => '×',
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArithmeticProblem {
    left: u32,
    operator: Operator,
    right: u32,
}

impl ArithmeticProblem {
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..3) {
            0 => Self {
                left: rng.random_range(3..=25),
                operator: Operator::Add,
                right: rng.random_range(3..=25),
            },
            1 => {
                let left = rng.random_range(8..=40);
                Self {
                    left,
                    operator: Operator::Subtract,
                    right: rng.random_range(2..left),
                }
            }
            _ => Self {
                left: rng.random_range(2..=9),
                operator: Operator::Multiply,
                right: rng.random_range(2..=9),
            },
        }
    }

    pub fn answer(&self) -> u32 {
        match self.operator {
            Operator::Add => self.left + self.right,
            Operator::Subtract => self.left - self.right,
            Operator::Multiply => self.left * self.right,
        }
    }

    pub fn question_text(&self) -> String {
        format!("{} {} {} = ?", self.left, self.operator.symbol(), self.right)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptchaQuestion {
    pub problem: ArithmeticProblem,
    pub option_values: Vec<u32>,
    pub correct_answer: u8,
}

impl CaptchaQuestion {
    pub fn random(rng: &mut impl Rng) -> Self {
        let problem = ArithmeticProblem::random(rng);
        let answer = problem.answer();

        let mut distractor_candidates: Vec<u32> = (answer.saturating_sub(DISTRACTOR_SPREAD)
            ..=answer + DISTRACTOR_SPREAD)
            .filter(|value| *value != answer)
            .collect();
        distractor_candidates.shuffle(rng);

        let mut option_values: Vec<u32> = distractor_candidates
            .into_iter()
            .take(DISTRACTOR_COUNT)
            .collect();
        let correct_index = rng.random_range(0..=option_values.len());
        option_values.insert(correct_index, answer);

        Self {
            problem,
            option_values,
            correct_answer: correct_index as u8 + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;

    fn seeded_rng(seed: u64) -> StdRng {
        StdRng::seed_from_u64(seed)
    }

    #[test]
    fn every_problem_has_a_positive_answer_that_fits_in_two_digits() {
        let mut rng = seeded_rng(1);

        for _ in 0..2000 {
            let answer = ArithmeticProblem::random(&mut rng).answer();
            assert!((1..100).contains(&answer), "answer {answer} out of range");
        }
    }

    #[test]
    fn answer_matches_the_operator() {
        let add = ArithmeticProblem { left: 12, operator: Operator::Add, right: 7 };
        let subtract = ArithmeticProblem { left: 12, operator: Operator::Subtract, right: 7 };
        let multiply = ArithmeticProblem { left: 12, operator: Operator::Multiply, right: 7 };

        assert_eq!(add.answer(), 19);
        assert_eq!(subtract.answer(), 5);
        assert_eq!(multiply.answer(), 84);
    }

    #[test]
    fn question_text_uses_typographic_operators() {
        let subtract = ArithmeticProblem { left: 12, operator: Operator::Subtract, right: 7 };
        let multiply = ArithmeticProblem { left: 3, operator: Operator::Multiply, right: 4 };

        assert_eq!(subtract.question_text(), "12 − 7 = ?");
        assert_eq!(multiply.question_text(), "3 × 4 = ?");
    }

    #[test]
    fn options_are_four_distinct_values_with_the_answer_at_the_reported_position() {
        let mut rng = seeded_rng(2);

        for _ in 0..2000 {
            let question = CaptchaQuestion::random(&mut rng);

            assert_eq!(question.option_values.len(), OPTION_COUNT);
            let mut sorted = question.option_values.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), OPTION_COUNT, "options must be distinct");

            assert!((1..=OPTION_COUNT as u8).contains(&question.correct_answer));
            let marked_option = question.option_values[question.correct_answer as usize - 1];
            assert_eq!(marked_option, question.problem.answer());
        }
    }

    #[test]
    fn the_correct_position_is_spread_over_all_four_slots() {
        let mut rng = seeded_rng(3);
        let mut seen = [false; OPTION_COUNT];

        for _ in 0..200 {
            seen[CaptchaQuestion::random(&mut rng).correct_answer as usize - 1] = true;
        }

        assert!(seen.iter().all(|slot| *slot));
    }
}
