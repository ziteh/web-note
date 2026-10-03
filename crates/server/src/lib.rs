mod api_error;
mod dto;
mod openapi;
mod routes;

use std::sync::Arc;

use web_note_core::Notes;

pub use openapi::openapi_json;

pub type AppState = Arc<Notes>;

pub fn app(notes: AppState) -> axum::Router {
    let limit = notes.max_asset_bytes();
    openapi::router(limit as usize)
        .with_state(notes)
        .split_for_parts()
        .0
}

#[cfg(feature = "swagger")]
pub fn app_with_swagger(notes: AppState) -> axum::Router {
    let limit = notes.max_asset_bytes();
    let (router, api) = openapi::router(limit as usize)
        .with_state(notes)
        .split_for_parts();
    router
        .merge(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
}
