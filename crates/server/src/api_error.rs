use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;
use web_note_core::Error;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: ErrorBody,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: String) -> Self {
        Self {
            status,
            body: ErrorBody { code, message },
        }
    }

    pub fn internal(err: impl std::fmt::Display) -> Self {
        eprintln!("internal error: {err}");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "internal error".into(),
        )
    }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        let (status, code) = match &err {
            Error::NoteNotFound => (StatusCode::NOT_FOUND, "note_not_found"),
            Error::NotebookNotFound => (StatusCode::NOT_FOUND, "notebook_not_found"),
            Error::AssetNotFound => (StatusCode::NOT_FOUND, "asset_not_found"),
            Error::TrashItemNotFound => (StatusCode::NOT_FOUND, "trash_item_not_found"),
            Error::NotebookExists => (StatusCode::CONFLICT, "notebook_exists"),
            Error::NotebookNotEmpty => (StatusCode::CONFLICT, "notebook_not_empty"),
            Error::Conflict => (StatusCode::CONFLICT, "conflict"),
            Error::ProtectedNotebook => (StatusCode::BAD_REQUEST, "protected_notebook"),
            Error::InvalidName => (StatusCode::BAD_REQUEST, "invalid_name"),
            Error::AssetTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, "asset_too_large"),
            Error::UnsupportedAsset => (StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_asset"),
            Error::CorruptFrontmatter(_) => (StatusCode::UNPROCESSABLE_ENTITY, "corrupt_note"),
            Error::Io { .. } | Error::Db(_) => return Self::internal(err),
        };
        Self::new(status, code, err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
