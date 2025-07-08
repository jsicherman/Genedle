use crate::api::{GeneNamesDoc, GeneNamesResponse};
use axum::Json;
use axum::extract::Path;
use cached::proc_macro::cached;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use reqwest::Client;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    Normal,
    Hard,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub struct Guess {
    pub word: Vec<char>,
    pub session: u64,
    pub mode: GameMode,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GuessResult {
    Invalid(InvalidGuess),
    Valid(ValidGuess),
}

impl Serialize for GuessResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("GuessResult", 2)?;

        match self {
            GuessResult::Invalid(invalid) => {
                state.serialize_field("type", "invalid")?;
                state.serialize_field("data", invalid)
            }
            GuessResult::Valid(valid) => {
                state.serialize_field("type", "valid")?;
                state.serialize_field("data", valid)
            }
        }?;

        state.end()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InvalidGuess {
    InternalError(String),
    NotEnoughLetters,
    TooManyLetters,
    InvalidLetter,
    NotInCorpus,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ValidGuess {
    is_correct: bool,
    result: Vec<LetterFeedback>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LetterFeedback {
    Correct,
    Present,
    Absent,
}

pub async fn num_letters(Path(key): Path<u64>) -> Json<isize> {
    let count = get_word(key)
        .await
        .map_or(-1, |word| word.chars().count() as isize);

    Json(count)
}

#[cached]
async fn _num_letters(key: u64) -> isize {
    match get_word(key).await {
        Ok(word) => word.chars().count() as isize,
        Err(_) => -1,
    }
}

#[allow(unused)]
pub async fn valid_guess(Json(guess): Json<Guess>) -> Result<Option<InvalidGuess>, anyhow::Error> {
    match _valid_guess(guess.clone()).await {
        Ok(None) => Ok(None),
        Ok(Some(reason)) => Ok(Some(reason)),
        Err(err) => Err(anyhow::anyhow!(err)),
    }
}

#[cached]
async fn _valid_guess(guess: Guess) -> Result<Option<InvalidGuess>, String> {
    const API: &str = "https://rest.genenames.org/search/symbol/";
    const STATUS_SUCCESS: usize = 0;

    let len = _num_letters(guess.session).await;
    if len == -1 {
        return Ok(Some(InvalidGuess::InternalError(
            "Unable to fetch gene symbol".to_string(),
        )));
    }
    let len = len as usize;

    if guess.word.len() != len {
        return Ok(Some(if guess.word.len() < len {
            InvalidGuess::NotEnoughLetters
        } else {
            InvalidGuess::TooManyLetters
        }));
    }

    if guess.mode == GameMode::Normal {
        return Ok(None);
    }

    let guess = guess.word.iter().collect::<String>();

    let client = Client::new();
    let response = client
        .get(format!("{API}{guess}"))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if response.status().is_success() {
        let found = response
            .json::<GeneNamesResponse<GeneNamesDoc>>()
            .await
            .map(|response| {
                response.response_header.status == STATUS_SUCCESS
                    && response.response.num_found >= 1
                    && response.response.docs.iter().any(|doc| doc.symbol == guess)
            })
            .map_err(|err| err.to_string())?;

        if found {
            Ok(None)
        } else {
            Ok(Some(InvalidGuess::NotInCorpus))
        }
    } else {
        Err("Unable to query genenames.org".to_string())
    }
}

#[cached]
async fn get_word(key: u64) -> Result<String, String> {
    const API: &str = "https://rest.genenames.org/search/symbol/";
    const STATUS_SUCCESS: usize = 0;

    let mut rng: StdRng = SeedableRng::seed_from_u64(key);
    let first_letter = rng.random_range(b'A'..=b'Z') as char;

    let client = Client::new();
    let response = client
        .get(format!("{API}{first_letter}*"))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if response.status().is_success() {
        let fetched_symbol = response
            .json::<GeneNamesResponse<GeneNamesDoc>>()
            .await
            .map(|json| {
                if json.response_header.status == STATUS_SUCCESS {
                    let nth = rng.random_range(1..=json.response.num_found) - 1;
                    json.response.docs.into_iter().nth(nth)
                } else {
                    None
                }
            })
            .map_err(|err| err.to_string())?
            .map(|doc| doc.symbol);

        if let Some(symbol) = fetched_symbol {
            Ok(symbol)
        } else {
            Err("No gene symbol found".to_string())
        }
    } else {
        Err("Unable to query genenames.org".to_string())
    }
}

pub async fn guess(Json(guess): Json<Guess>) -> Json<GuessResult> {
    match _valid_guess(guess.clone()).await {
        Ok(None) => (),
        Ok(Some(reason)) => {
            return Json(GuessResult::Invalid(reason));
        }
        Err(err) => {
            return Json(GuessResult::Invalid(InvalidGuess::InternalError(
                err.to_string(),
            )));
        }
    };

    let word = match get_word(guess.session).await {
        Ok(word) => word,
        Err(err) => {
            return Json(GuessResult::Invalid(InvalidGuess::InternalError(
                err.to_string(),
            )));
        }
    }
    .chars()
    .collect::<Vec<_>>();

    let mut char_counts: HashMap<char, usize> = HashMap::new();
    for letter in &word {
        *char_counts.entry(*letter).or_default() += 1;
    }

    let mut result = vec![LetterFeedback::Absent; word.len()];

    for (i, (guessed, actual)) in guess.word.iter().zip(&word).enumerate() {
        if guessed == actual {
            result[i] = LetterFeedback::Correct;
            *char_counts.get_mut(guessed).unwrap() -= 1;
        }
    }

    for (i, guessed) in guess.word.iter().enumerate() {
        if result[i] == LetterFeedback::Absent {
            if let Some(count) = char_counts.get_mut(guessed) {
                if *count > 0 {
                    result[i] = LetterFeedback::Present;
                    *count -= 1;
                }
            }
        }
    }

    let is_correct = result
        .iter()
        .all(|&feedback| feedback == LetterFeedback::Correct);

    Json(GuessResult::Valid(ValidGuess { is_correct, result }))
}

#[cfg(test)]
mod tests {
    use crate::api::genedle::{
        GameMode, Guess, GuessResult, InvalidGuess, LetterFeedback, ValidGuess,
    };
    use axum::Json;

    #[tokio::test]
    async fn test_get_word() -> Result<(), String> {
        let result = super::get_word(1234567890).await?;
        assert_eq!(result, "MIB2".to_string());

        // two nearby seeds should return unpredictable results
        let result = super::get_word(1234567891).await?;
        assert_eq!(result, "TLX3".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn test_guess() -> Result<(), anyhow::Error> {
        let guess = Guess {
            word: "MIB".chars().collect(),
            session: 1234567890,
            mode: GameMode::Normal,
        };

        let response = super::guess(Json(guess)).await;
        assert_eq!(
            response.0,
            GuessResult::Invalid(InvalidGuess::NotEnoughLetters)
        );

        let guess = Guess {
            word: "MIB22".chars().collect(),
            session: 1234567890,
            mode: GameMode::Normal,
        };

        let response = super::guess(Json(guess)).await;
        assert_eq!(
            response.0,
            GuessResult::Invalid(InvalidGuess::TooManyLetters)
        );

        let guess = Guess {
            word: "MIB2".chars().collect(),
            session: 1234567890,
            mode: GameMode::Normal,
        };

        let response = super::guess(Json(guess)).await;
        assert_eq!(
            response.0,
            GuessResult::Valid(ValidGuess {
                is_correct: true,
                result: vec![LetterFeedback::Correct; 4],
            })
        );

        let guess = Guess {
            word: "AAAA".chars().collect(),
            session: 1234567890,
            mode: GameMode::Normal,
        };

        let response = super::guess(Json(guess)).await;
        assert_eq!(
            response.0,
            GuessResult::Valid(ValidGuess {
                is_correct: false,
                result: vec![LetterFeedback::Absent; 4],
            })
        );

        let guess = Guess {
            word: "MIB3".chars().collect(),
            session: 1234567890,
            mode: GameMode::Normal,
        };

        let response = super::guess(Json(guess)).await;
        assert_eq!(
            response.0,
            GuessResult::Valid(ValidGuess {
                is_correct: false,
                result: vec![
                    LetterFeedback::Correct,
                    LetterFeedback::Correct,
                    LetterFeedback::Correct,
                    LetterFeedback::Absent
                ],
            })
        );

        let guess = Guess {
            word: "2IBM".chars().collect(),
            session: 1234567890,
            mode: GameMode::Normal,
        };

        let response = super::guess(Json(guess)).await;
        assert_eq!(
            response.0,
            GuessResult::Valid(ValidGuess {
                is_correct: false,
                result: vec![
                    LetterFeedback::Present,
                    LetterFeedback::Correct,
                    LetterFeedback::Correct,
                    LetterFeedback::Present
                ],
            })
        );

        let guess = Guess {
            word: "M2B2".chars().collect(),
            session: 1234567890,
            mode: GameMode::Normal,
        };

        let response = super::guess(Json(guess)).await;
        assert_eq!(
            response.0,
            GuessResult::Valid(ValidGuess {
                is_correct: false,
                result: vec![
                    LetterFeedback::Correct,
                    LetterFeedback::Absent,
                    LetterFeedback::Correct,
                    LetterFeedback::Correct
                ],
            })
        );

        let guess = Guess {
            word: "2222".chars().collect(),
            session: 1234567890,
            mode: GameMode::Normal,
        };

        let response = super::guess(Json(guess)).await;
        assert_eq!(
            response.0,
            GuessResult::Valid(ValidGuess {
                is_correct: false,
                result: vec![
                    LetterFeedback::Absent,
                    LetterFeedback::Absent,
                    LetterFeedback::Absent,
                    LetterFeedback::Correct
                ],
            })
        );

        Ok(())
    }
}
