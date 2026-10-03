use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::run;
use crate::{
    AppState,
    api_error::{ApiError, ErrorBody},
    dto::{Note, TrashItem},
};

const TAG: &str = "trash";

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_trash, empty_trash))
        .routes(routes!(purge_item))
        .routes(routes!(restore_item))
}

#[utoipa::path(get, path = "/api/trash", tag = TAG, responses((status = 200, body = Vec<TrashItem>)))]
async fn list_trash(State(s): State<AppState>) -> Result<Json<Vec<TrashItem>>, ApiError> {
    let items = run(&s, |n| n.trash()).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

#[utoipa::path(delete, path = "/api/trash", tag = TAG, responses((status = 204)))]
async fn empty_trash(State(s): State<AppState>) -> Result<StatusCode, ApiError> {
    run(&s, |n| n.empty_trash()).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete, path = "/api/trash/{key}", tag = TAG, params(("key" = String, Path)),
    responses((status = 204), (status = 404, body = ErrorBody))
)]
async fn purge_item(
    State(s): State<AppState>,
    Path(key): Path<String>,
) -> Result<StatusCode, ApiError> {
    run(&s, move |n| n.purge(&key)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post, path = "/api/trash/{key}/restore", tag = TAG, params(("key" = String, Path)),
    responses((status = 200, body = Note), (status = 404, body = ErrorBody))
)]
async fn restore_item(
    State(s): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<Note>, ApiError> {
    Ok(Json(run(&s, move |n| n.restore(&key)).await?.into()))
}
