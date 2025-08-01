mod api;
mod games;

use axum::Router;
use axum::routing::{get, post};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

#[tokio::main]
async fn main() {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let static_files = ServeDir::new("../frontend/build/client")
        .not_found_service(ServeFile::new("../frontend/build/client/index.html"));

    let app = Router::new()
        .fallback_service(static_files)
        .route("/games/genedle", get(games::genedle::genedle))
        .route("/games/genections", get(games::genections::genections))
        .route(
            "/games/spelling-gene",
            get(games::spelling_gene::spelling_gene),
        )
        .layer(session_layer)
        .route(
            "/api/v1/spelling-gene-guess/{seed}/{min_length}/{min_words}/{num_letters}/{guess}",
            get(api::spelling_gene::check_guess),
        )
        .route(
            "/api/v1/spelling-gene/{seed}/{min_length}/{min_words}/{num_letters}",
            get(api::spelling_gene::get_letters),
        )
        .route("/api/v1/genedle-guess", post(api::genedle::guess))
        .route(
            "/api/v1/genedle-letters/{id}",
            get(api::genedle::num_letters),
        )
        .layer(CorsLayer::permissive());

    let host = std::env::var("GENEDLE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("GENEDLE_PORT").unwrap_or_else(|_| "3000".to_string());

    let listener = TcpListener::bind(format!("{host}:{port}")).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
