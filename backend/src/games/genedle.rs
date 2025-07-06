use axum::Json;
use chrono::Datelike;
use tower_sessions::Session;

const WORD_KEY: &str = "genedle.word";

async fn get_word(session: &Session) -> Option<u64> {
    session.get::<u64>(WORD_KEY).await.ok().flatten()
}

async fn init_word(session: &Session) -> Result<u64, anyhow::Error> {
    match get_word(session).await {
        None => {
            let word_of_the_day = chrono::Utc::now().num_days_from_ce() as u64;
            session.insert(WORD_KEY, word_of_the_day).await?;

            Ok(word_of_the_day)
        }
        Some(word_selection) => Ok(word_selection),
    }
}

pub async fn genedle(session: Session) -> Json<String> {
    match init_word(&session).await {
        Ok(word) => Json(word.to_string()),
        Err(err) => Json(format!("Error initializing word: {err}")),
    }
}
