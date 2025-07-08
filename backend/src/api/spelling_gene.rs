use crate::api::{GeneNamesDoc, GeneNamesResponse};
use axum::Json;
use axum::extract::Path;
use cached::proc_macro::cached;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct SpellingGeneGame {
    #[serde(flatten)]
    pub metadata: SpellingGeneMetadata,
    pub valid_symbols: BTreeSet<String>,
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct SpellingGeneMetadata {
    pub outer_letters: Vec<&'static str>,
    pub center_letter: &'static str,
}

pub async fn check_guess(
    Path((seed, min_length, min_words, num_letters, guess)): Path<(u64, usize, usize, u8, String)>,
) -> Json<bool> {
    match generate_game(min_length, min_words, num_letters, seed).await {
        Ok(game) => Json(game.valid_symbols.contains(&guess)),
        Err(_) => Json(false),
    }
}

pub async fn get_letters(
    Path((seed, min_length, min_words, num_letters)): Path<(u64, usize, usize, u8)>,
) -> Json<SpellingGeneMetadata> {
    generate_game(min_length, min_words, num_letters, seed)
        .await
        .map(|game| Json(game.metadata))
        .unwrap_or_else(|_| {
            Json(SpellingGeneMetadata {
                outer_letters: Vec::new(),
                center_letter: "",
            })
        })
}

async fn generate_game(
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
    const MAX_ITERS: usize = 10_000;
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

    let mut all_symbols: BTreeSet<String> = BTreeSet::new();

    let mut letters = VALID_LETTERS.to_vec();
    letters.shuffle(&mut rng);
    letters.truncate(num_letters as usize + 5);

    for letter in letters {
        if let Ok(symbols) = get_options(letter).await {
            all_symbols.extend(
                symbols
                    .into_iter()
                    .filter(|s| s.chars().count() >= min_length),
            );
        }
    }

    let mut iter = 0;
    while iter < MAX_ITERS {
        iter += 1;

        let mut letters = VALID_LETTERS.to_vec();
        letters.shuffle(&mut rng);
        letters.truncate(num_letters as usize);

        let mut letters_set: BTreeSet<char> = letters.iter().flat_map(|s| s.chars()).collect();
        let center_letter = letters.pop().unwrap();
        let center_char = center_letter.chars().next().unwrap();
        letters_set.insert(center_char);

        let filtered: BTreeSet<_> = all_symbols
            .iter()
            .filter(|symbol| {
                symbol.contains(center_char) && symbol.chars().all(|c| letters_set.contains(&c))
            })
            .collect();

        if filtered.len() >= min_words {
            return Ok(SpellingGeneGame {
                metadata: SpellingGeneMetadata {
                    outer_letters: letters,
                    center_letter,
                },
                valid_symbols: filtered.into_iter().cloned().collect(),
            });
        }
    }

    Err("Failed to generate a valid game".to_string())
}

#[cfg(test)]
mod tests {
    use crate::api::spelling_gene::generate_game;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_generate_game() {
        let game = generate_game(4, 10, 7, 20277).await.unwrap();

        println!("{game:#?}");

        assert!(game.metadata.outer_letters.len() == 6);
        assert!(game.valid_symbols.len() >= 10);
        assert!(game.valid_symbols.iter().all(|symbol| {
            symbol.chars().count() >= 4
                && symbol.chars().all(|c| {
                    c.to_string() == game.metadata.center_letter
                        || game
                            .metadata
                            .outer_letters
                            .contains(&c.to_string().as_str())
                })
        }));
    }
}
