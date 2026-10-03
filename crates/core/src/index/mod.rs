mod query;
mod reconcile;

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    error::Result,
    frontmatter::{Document, format_time},
    parse,
};

pub use query::{NoteMeta, Query, Snippet, backlinks, list};
pub use reconcile::{Duplicate, Report, reconcile};

/// Bump when the schema changes; a mismatch drops and recreates all tables.
const SCHEMA_VERSION: i64 = 1;
const SUMMARY_CHARS: usize = 100;

const DROP: &str = "
DROP TABLE IF EXISTS notes;
DROP TABLE IF EXISTS note_tags;
DROP TABLE IF EXISTS links;
DROP TABLE IF EXISTS errors;
DROP TABLE IF EXISTS notes_fts;
";

const SCHEMA: &str = "
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    notebook TEXT NOT NULL,
    title TEXT,
    summary TEXT NOT NULL,
    created TEXT,
    updated TEXT,
    mtime INTEGER NOT NULL,
    size INTEGER NOT NULL
);
CREATE INDEX notes_path ON notes(path);
CREATE TABLE note_tags (
    id TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (id, tag)
) WITHOUT ROWID;
CREATE INDEX note_tags_tag ON note_tags(tag);
CREATE TABLE links (
    src TEXT NOT NULL,
    dst TEXT NOT NULL,
    PRIMARY KEY (src, dst)
) WITHOUT ROWID;
CREATE INDEX links_dst ON links(dst);
CREATE TABLE errors (
    path TEXT PRIMARY KEY,
    message TEXT NOT NULL
);
CREATE VIRTUAL TABLE notes_fts USING fts5(title, body, tokenize = 'trigram');
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileStat {
    pub mtime: i64,
    pub size: i64,
}

pub fn open(path: &Path) -> Result<Connection> {
    init(Connection::open(path)?)
}

pub fn open_in_memory() -> Result<Connection> {
    init(Connection::open_in_memory()?)
}

fn init(conn: Connection) -> Result<Connection> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version != SCHEMA_VERSION {
        conn.execute_batch(DROP)?;
        conn.execute_batch(SCHEMA)?;
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    }
    Ok(conn)
}

/// Inserts or updates a note. `rel` is the path relative to the notes root,
/// with `/` separators. A different note previously indexed at the same path
/// is removed.
pub fn upsert(
    conn: &Connection,
    rel: &str,
    id: &str,
    doc: &Document,
    stat: FileStat,
) -> Result<()> {
    let stale: Vec<String> = conn
        .prepare_cached("SELECT id FROM notes WHERE path = ?1 AND id <> ?2")?
        .query_map(params![rel, id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for stale_id in stale {
        remove(conn, &stale_id)?;
    }

    let notebook = rel.split('/').next().unwrap_or_default();
    let title = parse::title(&doc.body);
    conn.execute(
        "INSERT INTO notes (id, path, notebook, title, summary, created, updated, mtime, size)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
             path = excluded.path, notebook = excluded.notebook, title = excluded.title,
             summary = excluded.summary, created = excluded.created, updated = excluded.updated,
             mtime = excluded.mtime, size = excluded.size",
        params![
            id,
            rel,
            notebook,
            title,
            summary(&doc.body),
            doc.created.map(format_time).transpose()?,
            doc.updated.map(format_time).transpose()?,
            stat.mtime,
            stat.size,
        ],
    )?;

    let rowid: i64 = conn.query_row("SELECT rowid FROM notes WHERE id = ?1", [id], |r| r.get(0))?;
    conn.execute("DELETE FROM notes_fts WHERE rowid = ?1", [rowid])?;
    conn.execute(
        "INSERT INTO notes_fts (rowid, title, body) VALUES (?1, ?2, ?3)",
        params![rowid, title.as_deref().unwrap_or(""), doc.body],
    )?;

    conn.execute("DELETE FROM note_tags WHERE id = ?1", [id])?;
    for tag in &doc.tags {
        conn.execute(
            "INSERT OR IGNORE INTO note_tags (id, tag) VALUES (?1, ?2)",
            params![id, tag],
        )?;
    }
    conn.execute("DELETE FROM links WHERE src = ?1", [id])?;
    for link in parse::wikilinks(&doc.body) {
        conn.execute(
            "INSERT OR IGNORE INTO links (src, dst) VALUES (?1, ?2)",
            params![id, link.id],
        )?;
    }
    Ok(())
}

/// Removes a note from the index. Links pointing at it are kept, so they
/// show up as broken rather than disappearing.
pub fn remove(conn: &Connection, id: &str) -> Result<()> {
    let rowid: Option<i64> = conn
        .query_row("SELECT rowid FROM notes WHERE id = ?1", [id], |r| r.get(0))
        .optional()?;
    if let Some(rowid) = rowid {
        conn.execute("DELETE FROM notes_fts WHERE rowid = ?1", [rowid])?;
    }
    conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
    conn.execute("DELETE FROM note_tags WHERE id = ?1", [id])?;
    conn.execute("DELETE FROM links WHERE src = ?1", [id])?;
    Ok(())
}

pub fn path_of(conn: &Connection, id: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT path FROM notes WHERE id = ?1", [id], |r| r.get(0))
        .optional()?)
}

/// Files that failed to load during the last reconcile, as (path, message).
pub fn errors(conn: &Connection) -> Result<Vec<(String, String)>> {
    let rows = conn
        .prepare("SELECT path, message FROM errors ORDER BY path")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

fn summary(body: &str) -> String {
    body.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.chars().take(SUMMARY_CHARS).collect())
        .unwrap_or_default()
}
