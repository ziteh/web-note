use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::run;
use crate::{
    AppState,
    api_error::{ApiError, ErrorBody},
    dto::{RenameTag, RenameTagResult, Tag},
};

const TAG: &str = "tags";

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_tags))
        .routes(routes!(rename_tag))
}

/// Counts double as the preview of how many notes a rename or merge touches.
#[utoipa::path(get, path = "/api/tags", tag = TAG, responses((status = 200, body = Vec<Tag>)))]
async fn list_tags(State(s): State<AppState>) -> Result<Json<Vec<Tag>>, ApiError> {
    let tags = run(&s, |n| n.tags()).await?;
    Ok(Json(
        tags.into_iter()
            .map(|(tag, count)| Tag { tag, count })
            .collect(),
    ))
}

/// Rewrites every affected note. Merges when the target tag already exists.
#[utoipa::path(
    post, path = "/api/tags/rename", tag = TAG, request_body = RenameTag,
    responses((status = 200, body = RenameTagResult), (status = 400, body = ErrorBody))
)]
async fn rename_tag(
    State(s): State<AppState>,
    Json(req): Json<RenameTag>,
) -> Result<Json<RenameTagResult>, ApiError> {
    let affected = run(&s, move |n| n.rename_tag(&req.from, &req.to)).await?;
    Ok(Json(RenameTagResult { affected }))
}
