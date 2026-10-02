use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use crate::{
    error::{Error, Result},
    frontmatter::Document,
};

pub fn read_document(path: &Path) -> Result<Document> {
    let text = fs::read_to_string(path).map_err(|e| io_err(path, e))?;
    Document::parse(&text)
}

pub fn write_document(path: &Path, doc: &Document) -> Result<()> {
    atomic_write(path, doc.render()?.as_bytes())
}

/// Writes to a temp file in the same directory, then renames over `path`.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    let write = || -> std::io::Result<()> {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        fs::rename(&tmp, path)
    };
    write().map_err(|e| {
        let _ = fs::remove_file(&tmp);
        io_err(path, e)
    })
}

fn io_err(path: &Path, source: std::io::Error) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_content_and_leaves_no_temp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.md");
        atomic_write(&path, b"one").unwrap();
        atomic_write(&path, b"two").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "two");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn document_round_trips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.md");
        let doc = Document::new(crate::id::generate(), "# Hi\n");
        write_document(&path, &doc).unwrap();
        let back = read_document(&path).unwrap();
        assert_eq!(back.id, doc.id);
        assert_eq!(back.body, doc.body);
    }

    #[test]
    fn missing_file_is_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let r = read_document(&dir.path().join("none.md"));
        assert!(matches!(r, Err(Error::Io { .. })));
    }
}
