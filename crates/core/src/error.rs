use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("frontmatter is corrupt: {0}")]
    CorruptFrontmatter(String),
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
