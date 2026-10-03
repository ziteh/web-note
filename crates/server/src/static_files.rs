use axum::{
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

const INDEX: &str = "index.html";
const HASHED_DIR: &str = "assets/";
const API_PREFIX: &str = "api/";
const BROTLI_EXT: &str = ".br";
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

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

    // Hashed files are only embedded brotli-compressed.
    if path.starts_with(HASHED_DIR)
        && let Some(file) = Frontend::get(&format!("{path}{BROTLI_EXT}"))
    {
        return (
            [
                (
                    header::CONTENT_TYPE,
                    mime_guess::from_path(path)
                        .first_or_octet_stream()
                        .to_string(),
                ),
                (header::CONTENT_ENCODING, "br".to_string()),
                (header::CACHE_CONTROL, IMMUTABLE.to_string()),
            ],
            file.data,
        )
            .into_response();
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
        IMMUTABLE
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
