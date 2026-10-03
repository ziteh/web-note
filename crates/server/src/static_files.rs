use axum::{
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

const INDEX: &str = "index.html";
const HASHED_DIR: &str = "assets/";
const API_PREFIX: &str = "api/";

/// The built frontend. Run the web build first; a missing folder only means
/// nothing is served.
#[derive(RustEmbed)]
#[folder = "../../web/dist"]
#[allow_missing = true]
struct Frontend;

/// Serves embedded files, falling back to the SPA entry for client routes.
pub async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with(API_PREFIX) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let (file, name) = match Frontend::get(path) {
        Some(file) if !path.is_empty() => (file, path),
        _ => match Frontend::get(INDEX) {
            Some(file) => (file, INDEX),
            None => return StatusCode::NOT_FOUND.into_response(),
        },
    };

    // Files under the hashed directory change name when their content does.
    let cache = if name.starts_with(HASHED_DIR) {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    (
        [
            (header::CONTENT_TYPE, file.metadata.mimetype().to_string()),
            (header::CACHE_CONTROL, cache.to_string()),
        ],
        file.data,
    )
        .into_response()
}
