use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::{AppState, routes};

#[derive(OpenApi)]
#[openapi(info(title = "web-note", version = env!("CARGO_PKG_VERSION")))]
struct ApiDoc;

pub fn router(asset_body_limit: usize) -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(routes::notes::router())
        .merge(routes::notebooks::router())
        .merge(routes::tags::router())
        .merge(routes::assets::router(asset_body_limit))
        .merge(routes::trash::router())
        .merge(routes::system::router())
}

pub fn openapi_json() -> String {
    router(0)
        .into_openapi()
        .to_pretty_json()
        .expect("openapi spec serializes")
}
