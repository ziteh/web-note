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
    dto::{AssetUploaded, OrphanAssetsTrashed},
};

const TAG: &str = "assets";
const NOSNIFF: (HeaderName, &str) = (header::X_CONTENT_TYPE_OPTIONS, "nosniff");

pub fn router(body_limit: usize) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(upload_asset))
        .routes(routes!(list_orphans))
        .routes(routes!(trash_orphans))
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

/// Assets that no note references, including notes in the trash.
#[utoipa::path(get, path = "/api/assets/orphans", tag = TAG, responses((status = 200, body = Vec<String>)))]
async fn list_orphans(State(s): State<AppState>) -> Result<Json<Vec<String>>, ApiError> {
    Ok(Json(run(&s, |n| n.orphan_assets()).await?))
}

/// Moves orphan assets to the trash rather than deleting them.
#[utoipa::path(post, path = "/api/assets/orphans/trash", tag = TAG, responses((status = 200, body = OrphanAssetsTrashed)))]
async fn trash_orphans(State(s): State<AppState>) -> Result<Json<OrphanAssetsTrashed>, ApiError> {
    let count = run(&s, |n| n.trash_orphan_assets()).await?;
    Ok(Json(OrphanAssetsTrashed { count }))
}
