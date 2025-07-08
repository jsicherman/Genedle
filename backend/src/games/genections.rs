use axum::Json;
use tower_sessions::Session;

pub async fn genections(_session: Session) -> Json<String> {
    Json(String::new())
}
