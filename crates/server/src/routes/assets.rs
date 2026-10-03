use axum::{
    Json,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderName, StatusCode, header},
    response::IntoResponse,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::run;
use crate::{
    AppState,
    api_error::{ApiError, ErrorBody},
    dto::AssetUploaded,
};

const TAG: &str = "assets";
const NOSNIFF: (HeaderName, &str) = (header::X_CONTENT_TYPE_OPTIONS, "nosniff");

pub fn router(body_limit: usize) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(upload_asset))
        .layer(DefaultBodyLimit::max(body_limit.saturating_add(1)))
        .routes(routes!(get_asset))
}

/// Raw image bytes as the request body.
#[utoipa::path(
    post, path = "/api/assets", tag = TAG,
    request_body(content = Vec<u8>, content_type = "application/octet-stream"),
    responses(
        (status = 201, body = AssetUploaded),
        (status = 413, body = ErrorBody),
        (status = 415, body = ErrorBody),
    )
)]
async fn upload_asset(
    State(s): State<AppState>,
    body: Bytes,
) -> Result<(StatusCode, Json<AssetUploaded>), ApiError> {
    let name = run(&s, move |n| n.add_asset(&body)).await?;
    let path = format!("../.assets/{name}");
    Ok((StatusCode::CREATED, Json(AssetUploaded { name, path })))
}

#[utoipa::path(
    get, path = "/api/assets/{name}", tag = TAG, params(("name" = String, Path)),
    responses(
        (status = 200, body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 404, body = ErrorBody),
    )
)]
async fn get_asset(
    State(s): State<AppState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let (bytes, content_type) = run(&s, move |n| n.read_asset(&name)).await?;
    Ok(([(header::CONTENT_TYPE, content_type)], [NOSNIFF], bytes))
}
