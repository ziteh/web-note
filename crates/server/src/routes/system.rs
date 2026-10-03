use axum::{Json, extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::run;
use crate::{AppState, api_error::ApiError, dto::Status};

const TAG: &str = "system";

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(status))
        .routes(routes!(rescan))
        .routes(routes!(dismiss_duplicates))
}

#[utoipa::path(get, path = "/api/system/status", tag = TAG, responses((status = 200, body = Status)))]
async fn status(State(s): State<AppState>) -> Result<Json<Status>, ApiError> {
    Ok(Json(run(&s, |n| n.status()).await?.into()))
}

#[utoipa::path(post, path = "/api/system/rescan", tag = TAG, responses((status = 200, body = Status)))]
async fn rescan(State(s): State<AppState>) -> Result<Json<Status>, ApiError> {
    Ok(Json(run(&s, |n| n.rescan()).await?.into()))
}

#[utoipa::path(post, path = "/api/system/duplicates/dismiss", tag = TAG, responses((status = 204)))]
async fn dismiss_duplicates(State(s): State<AppState>) -> Result<StatusCode, ApiError> {
    run(&s, |n| {
        n.dismiss_duplicates();
        Ok(())
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
