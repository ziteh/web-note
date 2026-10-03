use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::extract::Query;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};
use web_note_core::index;

use super::run;
use crate::{
    AppState,
    api_error::{ApiError, ErrorBody},
    dto::{CreateNote, MoveNote, Note, NoteMeta, SaveNote},
};

const TAG: &str = "notes";

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_notes, create_note))
        .routes(routes!(get_note, save_note, delete_note))
        .routes(routes!(move_note))
        .routes(routes!(backlinks))
}

#[derive(Deserialize, IntoParams)]
struct ListParams {
    notebook: Option<String>,
    /// Repeat to require several tags.
    #[serde(default)]
    tag: Vec<String>,
    /// Full-text query over title and body.
    q: Option<String>,
}

#[utoipa::path(
    get, path = "/api/notes", tag = TAG, params(ListParams),
    responses((status = 200, body = Vec<NoteMeta>))
)]
async fn list_notes(
    State(s): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<Vec<NoteMeta>>, ApiError> {
    let query = index::Query {
        notebook: p.notebook,
        tags: p.tag,
        text: p.q,
    };
    let notes = run(&s, move |n| n.list(&query)).await?;
    Ok(Json(notes.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post, path = "/api/notes", tag = TAG, request_body = CreateNote,
    responses((status = 201, body = Note), (status = 404, body = ErrorBody))
)]
async fn create_note(
    State(s): State<AppState>,
    Json(req): Json<CreateNote>,
) -> Result<(StatusCode, Json<Note>), ApiError> {
    let note = run(&s, move |n| n.create(&req.notebook, &req.body, &req.tags)).await?;
    Ok((StatusCode::CREATED, Json(note.into())))
}

#[utoipa::path(
    get, path = "/api/notes/{id}", tag = TAG, params(("id" = String, Path)),
    responses((status = 200, body = Note), (status = 404, body = ErrorBody))
)]
async fn get_note(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Note>, ApiError> {
    Ok(Json(run(&s, move |n| n.get(&id)).await?.into()))
}

#[utoipa::path(
    put, path = "/api/notes/{id}", tag = TAG, params(("id" = String, Path)), request_body = SaveNote,
    responses((status = 200, body = Note), (status = 404, body = ErrorBody), (status = 409, body = ErrorBody))
)]
async fn save_note(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SaveNote>,
) -> Result<Json<Note>, ApiError> {
    let note = run(&s, move |n| {
        n.save(&id, &req.body, &req.tags, &req.base_version, req.force)
    })
    .await?;
    Ok(Json(note.into()))
}

#[utoipa::path(
    delete, path = "/api/notes/{id}", tag = TAG, params(("id" = String, Path)),
    responses((status = 204), (status = 404, body = ErrorBody))
)]
async fn delete_note(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    run(&s, move |n| n.delete(&id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post, path = "/api/notes/{id}/move", tag = TAG, params(("id" = String, Path)), request_body = MoveNote,
    responses((status = 200, body = Note), (status = 404, body = ErrorBody))
)]
async fn move_note(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<MoveNote>,
) -> Result<Json<Note>, ApiError> {
    Ok(Json(
        run(&s, move |n| n.move_note(&id, &req.notebook))
            .await?
            .into(),
    ))
}

#[utoipa::path(
    get, path = "/api/notes/{id}/backlinks", tag = TAG, params(("id" = String, Path)),
    responses((status = 200, body = Vec<NoteMeta>))
)]
async fn backlinks(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<NoteMeta>>, ApiError> {
    let notes = run(&s, move |n| n.backlinks(&id)).await?;
    Ok(Json(notes.into_iter().map(Into::into).collect()))
}
