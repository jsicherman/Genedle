use axum::Json;
use tower_sessions::Session;

pub async fn spelling_gene(_session: Session) -> Json<String> {
    Json(String::new())
}
