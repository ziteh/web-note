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
    dto::{Notebook, NotebookName},
};

const TAG: &str = "notebooks";

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_notebooks, create_notebook))
        .routes(routes!(rename_notebook, delete_notebook))
}

#[utoipa::path(get, path = "/api/notebooks", tag = TAG, responses((status = 200, body = Vec<Notebook>)))]
async fn list_notebooks(State(s): State<AppState>) -> Result<Json<Vec<Notebook>>, ApiError> {
    let notebooks = run(&s, |n| n.notebooks()).await?;
    Ok(Json(
        notebooks
            .into_iter()
            .map(|(name, count)| Notebook { name, count })
            .collect(),
    ))
}

#[utoipa::path(
    post, path = "/api/notebooks", tag = TAG, request_body = NotebookName,
    responses((status = 201), (status = 400, body = ErrorBody), (status = 409, body = ErrorBody))
)]
async fn create_notebook(
    State(s): State<AppState>,
    Json(req): Json<NotebookName>,
) -> Result<StatusCode, ApiError> {
    run(&s, move |n| n.create_notebook(&req.name)).await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    patch, path = "/api/notebooks/{name}", tag = TAG, params(("name" = String, Path)), request_body = NotebookName,
    responses((status = 204), (status = 404, body = ErrorBody), (status = 409, body = ErrorBody))
)]
async fn rename_notebook(
    State(s): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<NotebookName>,
) -> Result<StatusCode, ApiError> {
    run(&s, move |n| n.rename_notebook(&name, &req.name)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete, path = "/api/notebooks/{name}", tag = TAG, params(("name" = String, Path)),
    responses((status = 204), (status = 404, body = ErrorBody), (status = 409, body = ErrorBody))
)]
async fn delete_notebook(
    State(s): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    run(&s, move |n| n.delete_notebook(&name)).await?;
    Ok(StatusCode::NO_CONTENT)
}
