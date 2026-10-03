use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("frontmatter is corrupt: {0}")]
    CorruptFrontmatter(String),
    #[error("note not found")]
    NoteNotFound,
    #[error("notebook not found")]
    NotebookNotFound,
    #[error("notebook already exists")]
    NotebookExists,
    #[error("notebook is not empty")]
    NotebookNotEmpty,
    #[error("the default notebook cannot be renamed or deleted")]
    ProtectedNotebook,
    #[error("invalid name")]
    InvalidName,
    #[error("the note was modified elsewhere")]
    Conflict,
    #[error("asset exceeds the size limit")]
    AssetTooLarge,
    #[error("unsupported asset type")]
    UnsupportedAsset,
    #[error("asset not found")]
    AssetNotFound,
    #[error("trash item not found")]
    TrashItemNotFound,
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error at {}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
