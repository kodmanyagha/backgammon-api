use rand::{Rng, RngExt};
use sha1::{Digest, Sha1};

use super::{
    problem::CaptchaQuestion,
    renderer::{render_text_png, ImageSize},
};

pub const HASH_LEN: usize = 40;

const QUESTION_IMAGE_SIZE: ImageSize = ImageSize { width: 360, height: 110 };
const OPTION_IMAGE_SIZE: ImageSize = ImageSize { width: 150, height: 110 };
const HASH_ENTROPY_BYTES: usize = 32;

#[derive(Clone, Debug)]
pub struct GeneratedChallenge {
    pub hash: String,
    pub question_png: Vec<u8>,
    pub option_pngs: Vec<Vec<u8>>,
    pub correct_answer: u8,
}

pub fn generate_challenge() -> anyhow::Result<GeneratedChallenge> {
    let mut rng = rand::rng();
    let question = CaptchaQuestion::random(&mut rng);

    let question_png = render_text_png(&question.problem.question_text(), QUESTION_IMAGE_SIZE, &mut rng)?;
    let option_pngs = question
        .option_values
        .iter()
        .map(|value| render_text_png(&value.to_string(), OPTION_IMAGE_SIZE, &mut rng))
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(GeneratedChallenge {
        hash: new_hash(&mut rng),
        question_png,
        option_pngs,
        correct_answer: question.correct_answer,
    })
}

fn new_hash(rng: &mut impl Rng) -> String {
    let mut entropy = [0u8; HASH_ENTROPY_BYTES];
    rng.fill(&mut entropy);

    Sha1::digest(entropy)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn is_valid_hash_format(hash: &str) -> bool {
    hash.len() == HASH_LEN
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use tiny_skia::Pixmap;

    use super::*;

    #[test]
    fn a_challenge_has_a_question_four_options_and_a_valid_hash() {
        let challenge = generate_challenge().expect("challenge is generated");

        assert!(is_valid_hash_format(&challenge.hash));
        assert_eq!(challenge.option_pngs.len(), 4);
        assert!((1..=4).contains(&challenge.correct_answer));

        let question = Pixmap::decode_png(&challenge.question_png).expect("question is a png");
        assert_eq!((question.width(), question.height()), (360, 110));
        for option in &challenge.option_pngs {
            let decoded = Pixmap::decode_png(option).expect("option is a png");
            assert_eq!((decoded.width(), decoded.height()), (150, 110));
        }
    }

    #[test]
    fn hashes_are_unique() {
        let mut hashes: Vec<String> = (0..50).map(|_| new_hash(&mut rand::rng())).collect();
        hashes.sort_unstable();
        hashes.dedup();

        assert_eq!(hashes.len(), 50);
    }

    #[test]
    fn hash_format_accepts_only_40_lowercase_hex_characters() {
        assert!(is_valid_hash_format("0123456789abcdef0123456789abcdef01234567"));
        assert!(!is_valid_hash_format(""));
        assert!(!is_valid_hash_format(&"a".repeat(39)));
        assert!(!is_valid_hash_format(&"a".repeat(41)));
        assert!(!is_valid_hash_format(&"A".repeat(40)));
        assert!(!is_valid_hash_format(&"g".repeat(40)));
        assert!(!is_valid_hash_format(&format!("{}\n", "a".repeat(39))));
    }
}
