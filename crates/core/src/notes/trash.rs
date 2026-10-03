use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{Note, Notes, TRASH_DIR, io_err};
use crate::{
    error::{Error, Result},
    id, index, parse,
    store::read_document,
};

/// Trashed files live at `.trash/<notebook>/<stem>~<millis>.md`.
const STAMP_SEPARATOR: char = '~';

#[derive(Debug, Clone)]
pub struct TrashItem {
    /// File name; unique within the trash.
    pub key: String,
    pub notebook: String,
    pub id: Option<String>,
    pub title: Option<String>,
    pub trashed_at_millis: i64,
}

impl Notes {
    /// Moves a note to the trash.
    pub fn delete(&self, id: &str) -> Result<()> {
        let mut conn = self.conn();
        let rel = self.locate(&mut conn, id)?;
        let target = self.trash_target(&rel)?;
        fs::rename(self.root.join(&rel), &target).map_err(|e| io_err(&target, e))?;
        index::remove(&conn, id)
    }

    pub fn trash(&self) -> Result<Vec<TrashItem>> {
        let mut items = Vec::new();
        for (notebook, path) in self.trash_files()? {
            let key = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let doc = read_document(&path).ok();
            items.push(TrashItem {
                trashed_at_millis: stamp_of(&key),
                notebook,
                id: doc.as_ref().and_then(|d| d.id.clone()),
                title: doc.as_ref().and_then(|d| parse::title(&d.body)),
                key,
            });
        }
        items.sort_by_key(|i| std::cmp::Reverse(i.trashed_at_millis));
        Ok(items)
    }

    /// Puts a trashed note back into its original notebook. If its id or
    /// file name is already taken, it comes back with a fresh id.
    pub fn restore(&self, key: &str) -> Result<Note> {
        let (notebook, path) = self.find_trashed(key)?;
        let mut doc = read_document(&path)?;
        let dir = self.root.join(&notebook);
        fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;

        let mut conn = self.conn();
        let original = format!("{}.md", original_stem(key));
        let mut id = doc.id.clone().unwrap_or_else(id::generate);
        if index::path_of(&conn, &id)?.is_some() || dir.join(&original).exists() {
            id = id::generate();
        }
        let name = if doc.id.as_deref() == Some(&id) {
            original
        } else {
            format!("{id}.md")
        };
        doc.id = Some(id.clone());

        let rel = format!("{notebook}/{name}");
        self.write_and_index(&conn, &rel, &id, &doc)?;
        fs::remove_file(&path).map_err(|e| io_err(&path, e))?;
        self.load(&mut conn, &id)
    }

    pub fn purge(&self, key: &str) -> Result<()> {
        let (_, path) = self.find_trashed(key)?;
        fs::remove_file(&path).map_err(|e| io_err(&path, e))
    }

    pub fn empty_trash(&self) -> Result<()> {
        for (_, path) in self.trash_files()? {
            fs::remove_file(&path).map_err(|e| io_err(&path, e))?;
        }
        Ok(())
    }

    /// Keeps a copy of `text`, the content of the note at `rel`, in the trash.
    pub(super) fn trash_copy(&self, rel: &str, text: &str) -> Result<()> {
        let target = self.trash_target(rel)?;
        fs::write(&target, text).map_err(|e| io_err(&target, e))
    }

    fn trash_target(&self, rel: &str) -> Result<PathBuf> {
        let (notebook, name) = rel.split_once('/').unwrap_or_default();
        let stem = name.strip_suffix(".md").unwrap_or(name);
        let dir = self.root.join(TRASH_DIR).join(notebook);
        fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis());
        Ok(dir.join(format!("{stem}{STAMP_SEPARATOR}{millis}.md")))
    }

    /// (notebook, path) of every trashed note file.
    pub(super) fn trash_files(&self) -> Result<Vec<(String, PathBuf)>> {
        let trash = self.root.join(TRASH_DIR);
        let mut files = Vec::new();
        for nb in fs::read_dir(&trash).map_err(|e| io_err(&trash, e))? {
            let nb = nb.map_err(|e| io_err(&trash, e))?;
            if nb.file_name().to_string_lossy().starts_with('.') || !nb.path().is_dir() {
                continue;
            }
            let notebook = nb.file_name().to_string_lossy().into_owned();
            for f in fs::read_dir(nb.path()).map_err(|e| io_err(&nb.path(), e))? {
                let path = f.map_err(|e| io_err(&nb.path(), e))?.path();
                if path.extension().is_some_and(|x| x == "md") {
                    files.push((notebook.clone(), path));
                }
            }
        }
        Ok(files)
    }

    /// Looks the key up among existing files rather than joining it into a
    /// path.
    pub(super) fn find_trashed(&self, key: &str) -> Result<(String, PathBuf)> {
        self.trash_files()?
            .into_iter()
            .find(|(_, p)| p.file_name().is_some_and(|n| n.to_string_lossy() == key))
            .ok_or(Error::TrashItemNotFound)
    }
}

fn original_stem(key: &str) -> &str {
    let stem = key.strip_suffix(".md").unwrap_or(key);
    stem.rsplit_once(STAMP_SEPARATOR).map_or(stem, |(s, _)| s)
}

fn stamp_of(key: &str) -> i64 {
    let stem = key.strip_suffix(".md").unwrap_or(key);
    stem.rsplit_once(STAMP_SEPARATOR)
        .and_then(|(_, m)| m.parse().ok())
        .unwrap_or(0)
}
