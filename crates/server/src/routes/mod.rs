pub mod assets;
pub mod notebooks;
pub mod notes;
pub mod system;
pub mod tags;
pub mod trash;

use web_note_core::{Notes, Result as CoreResult};

use crate::{AppState, api_error::ApiError};

/// Runs a blocking `core` operation off the async runtime.
pub async fn run<T, F>(state: &AppState, f: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce(&Notes) -> CoreResult<T> + Send + 'static,
{
    let notes = state.clone();
    tokio::task::spawn_blocking(move || f(&notes))
        .await
        .map_err(ApiError::internal)?
        .map_err(ApiError::from)
}
