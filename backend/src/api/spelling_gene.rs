use crate::api::{GeneNamesDoc, GeneNamesResponse};
use axum::Json;
use axum::extract::Path;
use cached::proc_macro::cached;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SpellingGeneGame {
    pub outer_letters: Vec<&'static str>,
    pub center_letter: &'static str,
    pub valid_symbols: BTreeSet<String>,
}

pub async fn check_guess(
    Path((seed, min_length, min_words, num_letters, guess)): Path<(u64, usize, usize, u8, String)>,
) -> Json<bool> {
    match generate_game(min_length, min_words, num_letters, seed).await {
        Ok(game) => Json(game.valid_symbols.contains(&guess)),
        Err(_) => Json(false),
    }
}

pub async fn generate_game(
    min_length: usize,
    min_words: usize,
    num_letters: u8,
    seed: u64,
) -> Result<SpellingGeneGame, anyhow::Error> {
    _generate_game(min_length, min_words, num_letters, seed)
        .await
        .map_err(|err| anyhow::anyhow!(err))
}

#[cached]
async fn _generate_game(
    min_length: usize,
    min_words: usize,
    num_letters: u8,
    seed: u64,
) -> Result<SpellingGeneGame, String> {
    const API: &str = "https://rest.genenames.org/search/symbol/";
    const STATUS_SUCCESS: usize = 0;
    const MAX_ITERS: usize = 100;
    const VALID_LETTERS: [&str; 27] = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
        "S", "T", "U", "V", "W", "X", "Y", "Z", "-",
    ];

    let client = Client::new();
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let get_options = async |letter: &str| -> Result<BTreeSet<String>, anyhow::Error> {
        let starting_with = client
            .get(format!("{API}{letter}*"))
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?
            .json::<GeneNamesResponse<GeneNamesDoc>>()
            .await
            .map(|json| {
                if json.response_header.status == STATUS_SUCCESS {
                    Some(json.response.docs)
                } else {
                    None
                }
            })?;
        let containing = client
            .get(format!("{API}*{letter}"))
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?
            .json::<GeneNamesResponse<GeneNamesDoc>>()
            .await
            .map(|json| {
                if json.response_header.status == STATUS_SUCCESS {
                    Some(json.response.docs)
                } else {
                    None
                }
            })?;

        Ok(starting_with
            .into_iter()
            .chain(containing)
            .flat_map(|x| x.into_iter().map(|x| x.symbol))
            .collect())
    };

    let mut valid_words: BTreeSet<String> = BTreeSet::new();
    let mut letters = Vec::new();
    let mut iter = 0;

    // TODO: invert this logic to get words first then pick letters
    while valid_words.len() < min_words && iter < MAX_ITERS {
        iter += 1;
        letters = VALID_LETTERS.to_vec();
        letters.shuffle(&mut rng);
        letters.truncate(num_letters as usize);

        let chars: BTreeSet<_> = letters.iter().flat_map(|x| x.chars()).collect();

        valid_words = get_options(letters.last().unwrap())
            .await
            .map_err(|err| err.to_string())?
            .into_iter()
            .filter(|word| {
                word.chars().count() >= min_length && word.chars().all(|c| chars.contains(&c))
            })
            .collect();
    }

    let center_letter = letters.pop().unwrap();
    Ok(SpellingGeneGame {
        outer_letters: letters,
        center_letter,
        valid_symbols: valid_words,
    })
}

#[cfg(test)]
mod tests {
    use crate::api::spelling_gene::generate_game;

    #[tokio::test]
    async fn test_generate_game() {
        let game = generate_game(4, 10, 6, 0).await.unwrap();

        println!("{game:#?}");

        assert!(game.outer_letters.len() == 5);
        assert!(game.valid_symbols.len() >= 10);
        assert!(game.valid_symbols.iter().all(|symbol| {
            symbol.chars().count() >= 4
                && symbol.chars().all(|c| {
                    c.to_string() == game.center_letter
                        || game.outer_letters.contains(&c.to_string().as_str())
                })
        }));
    }
}
