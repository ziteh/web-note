mod assets;
mod notebooks;
mod trash;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, PoisonError},
};

use rusqlite::Connection;

use crate::{
    error::{Error, Result},
    frontmatter::{Document, normalize_tag, normalize_tags, now_local},
    id,
    index::{self, Duplicate, NoteMeta, Query},
    store::{read_document, write_document},
};

pub use trash::TrashItem;

pub const DEFAULT_NOTEBOOK: &str = "Inbox";
const ASSETS_DIR: &str = ".assets";
const TRASH_DIR: &str = ".trash";
const CACHE_DIR: &str = ".cache";
const DB_FILE: &str = "index.db";
const VERSION_CHARS: usize = 16;

#[derive(Debug, Clone)]
pub struct Note {
    pub meta: NoteMeta,
    pub body: String,
    /// Content hash of the file as read; pass back to `save` for the
    /// optimistic lock.
    pub version: String,
}

#[derive(Debug, Clone, Default)]
pub struct Status {
    /// Files that failed to load, as (path, message).
    pub errors: Vec<(String, String)>,
    pub duplicates: Vec<Duplicate>,
}

pub struct Notes {
    root: PathBuf,
    max_asset_bytes: u64,
    /// Serializes every operation, which also serializes writes per note.
    conn: Mutex<Connection>,
    duplicates: Mutex<Vec<Duplicate>>,
}

impl Notes {
    pub fn open(root: &Path, max_asset_bytes: u64) -> Result<Self> {
        for dir in [
            root.join(DEFAULT_NOTEBOOK),
            root.join(ASSETS_DIR),
            root.join(TRASH_DIR),
            root.join(CACHE_DIR),
        ] {
            fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;
        }
        let conn = index::open(&root.join(CACHE_DIR).join(DB_FILE))?;
        let notes = Self {
            root: root.to_path_buf(),
            max_asset_bytes,
            conn: Mutex::new(conn),
            duplicates: Mutex::new(Vec::new()),
        };
        notes.rescan()?;
        Ok(notes)
    }

    pub fn max_asset_bytes(&self) -> u64 {
        self.max_asset_bytes
    }

    pub fn rescan(&self) -> Result<Status> {
        let mut conn = self.conn();
        self.reconcile(&mut conn)?;
        self.status_locked(&conn)
    }

    pub fn status(&self) -> Result<Status> {
        self.status_locked(&self.conn())
    }

    pub fn dismiss_duplicates(&self) {
        self.duplicates
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }

    pub fn list(&self, query: &Query) -> Result<Vec<NoteMeta>> {
        index::list(&self.conn(), query)
    }

    pub fn backlinks(&self, id: &str) -> Result<Vec<NoteMeta>> {
        index::backlinks(&self.conn(), id)
    }

    pub fn get(&self, id: &str) -> Result<Note> {
        let mut conn = self.conn();
        self.load(&mut conn, id)
    }

    pub fn create(&self, notebook: &str, body: &str, tags: &[String]) -> Result<Note> {
        let notebook = self.existing_notebook(notebook)?;
        let id = id::generate();
        let rel = format!("{notebook}/{id}.md");
        let mut doc = Document::new(id.clone(), body);
        doc.tags = normalize_tags(tags);

        let mut conn = self.conn();
        self.write_and_index(&conn, &rel, &id, &doc)?;
        self.load(&mut conn, &id)
    }

    /// Saves body and tags. Nothing is written when neither changed. When the
    /// file differs from `base_version`, fails with `Conflict` unless `force`
    /// is set, in which case the overwritten version is kept in the trash.
    pub fn save(
        &self,
        id: &str,
        body: &str,
        tags: &[String],
        base_version: &str,
        force: bool,
    ) -> Result<Note> {
        let mut conn = self.conn();
        let rel = self.locate(&mut conn, id)?;
        let abs = self.root.join(&rel);
        let text = fs::read_to_string(&abs).map_err(|e| io_err(&abs, e))?;

        if version_of(&text) != base_version {
            if !force {
                return Err(Error::Conflict);
            }
            self.trash_copy(&rel, &text)?;
        }

        let mut doc = Document::parse(&text)?;
        let body = body.replace("\r\n", "\n");
        let tags = normalize_tags(tags);
        if doc.body != body || doc.tags != tags {
            doc.body = body;
            doc.tags = tags;
            doc.updated = Some(now_local());
            self.write_and_index(&conn, &rel, id, &doc)?;
        }
        self.load(&mut conn, id)
    }

    /// Moving does not touch `updated`.
    pub fn move_note(&self, id: &str, notebook: &str) -> Result<Note> {
        let notebook = self.existing_notebook(notebook)?;
        let mut conn = self.conn();
        let rel = self.locate(&mut conn, id)?;
        let name = rel.rsplit('/').next().unwrap_or_default();
        let new_rel = format!("{notebook}/{name}");
        if new_rel != rel {
            let new_abs = self.root.join(&new_rel);
            fs::rename(self.root.join(&rel), &new_abs).map_err(|e| io_err(&new_abs, e))?;
            let doc = read_document(&new_abs)?;
            index::upsert(&conn, &new_rel, id, &doc, index::stat_of(&new_abs)?)?;
        }
        self.load(&mut conn, id)
    }

    pub fn tags(&self) -> Result<Vec<(String, i64)>> {
        index::tag_counts(&self.conn())
    }

    /// Renames `from` to `to` in every note; merges when `to` already exists.
    /// Returns the number of notes rewritten.
    pub fn rename_tag(&self, from: &str, to: &str) -> Result<usize> {
        let (from, to) = (normalize_tag(from), normalize_tag(to));
        if to.is_empty() {
            return Err(Error::InvalidName);
        }
        if from == to {
            return Ok(0);
        }
        let mut conn = self.conn();
        let ids = index::ids_with_tag(&conn, &from)?;
        for id in &ids {
            let rel = self.locate(&mut conn, id)?;
            let mut doc = read_document(&self.root.join(&rel))?;
            let renamed: Vec<String> = doc
                .tags
                .iter()
                .map(|t| if *t == from { to.clone() } else { t.clone() })
                .collect();
            doc.tags = normalize_tags(&renamed);
            doc.updated = Some(now_local());
            self.write_and_index(&conn, &rel, id, &doc)?;
        }
        Ok(ids.len())
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn reconcile(&self, conn: &mut Connection) -> Result<()> {
        let report = index::reconcile(conn, &self.root)?;
        self.duplicates
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend(report.duplicates);
        Ok(())
    }

    fn status_locked(&self, conn: &Connection) -> Result<Status> {
        Ok(Status {
            errors: index::errors(conn)?,
            duplicates: self
                .duplicates
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone(),
        })
    }

    /// Relative path of a note, reconciling first if the file no longer
    /// matches what the index recorded.
    fn locate(&self, conn: &mut Connection, id: &str) -> Result<String> {
        if let Some((rel, stat)) = index::indexed(conn, id)?
            && index::stat_of(&self.root.join(&rel)).is_ok_and(|s| s == stat)
        {
            return Ok(rel);
        }
        self.reconcile(conn)?;
        index::path_of(conn, id)?.ok_or(Error::NoteNotFound)
    }

    fn load(&self, conn: &mut Connection, id: &str) -> Result<Note> {
        let rel = self.locate(conn, id)?;
        let abs = self.root.join(&rel);
        let text = fs::read_to_string(&abs).map_err(|e| io_err(&abs, e))?;
        let doc = Document::parse(&text)?;
        let meta = index::get(conn, id)?.ok_or(Error::NoteNotFound)?;
        Ok(Note {
            meta,
            body: doc.body,
            version: version_of(&text),
        })
    }

    fn write_and_index(
        &self,
        conn: &Connection,
        rel: &str,
        id: &str,
        doc: &Document,
    ) -> Result<()> {
        let abs = self.root.join(rel);
        write_document(&abs, doc)?;
        index::upsert(conn, rel, id, doc, index::stat_of(&abs)?)
    }
}

fn version_of(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex()[..VERSION_CHARS].to_string()
}

fn io_err(path: &Path, source: std::io::Error) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;
