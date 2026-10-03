use std::collections::HashMap;

use rusqlite::{Connection, params_from_iter};
use unicode_normalization::UnicodeNormalization;

use crate::{error::Result, frontmatter::normalize_tag};

/// Trigram FTS needs at least this many characters; shorter queries use LIKE.
const MIN_FTS_CHARS: usize = 3;
const SNIPPET_BEFORE: usize = 30;
const SNIPPET_AFTER: usize = 60;

#[derive(Debug, Clone, Default)]
pub struct Query {
    pub notebook: Option<String>,
    /// All tags must match.
    pub tags: Vec<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NoteMeta {
    pub id: String,
    pub path: String,
    pub notebook: String,
    pub title: Option<String>,
    pub summary: String,
    pub tags: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub snippet: Option<Snippet>,
}

/// `start..end` are character offsets of the match within `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

pub fn list(conn: &Connection, q: &Query) -> Result<Vec<NoteMeta>> {
    let mut clauses = Vec::new();
    let mut params = Vec::new();

    if let Some(nb) = &q.notebook {
        clauses.push("n.notebook = ?".to_string());
        params.push(nb.nfc().collect::<String>());
    }
    for tag in &q.tags {
        clauses.push("EXISTS (SELECT 1 FROM note_tags t WHERE t.id = n.id AND t.tag = ?)".into());
        params.push(normalize_tag(tag));
    }

    let text = q
        .text
        .as_deref()
        .map(|t| t.nfc().collect::<String>())
        .filter(|t| !t.is_empty());
    if let Some(text) = &text {
        if text.chars().count() >= MIN_FTS_CHARS {
            clauses.push("n.rowid IN (SELECT rowid FROM notes_fts WHERE notes_fts MATCH ?)".into());
            params.push(format!("\"{}\"", text.replace('"', "\"\"")));
        } else {
            clauses.push(
                "n.rowid IN (SELECT rowid FROM notes_fts
                             WHERE title LIKE ? ESCAPE '\\' OR body LIKE ? ESCAPE '\\')"
                    .into(),
            );
            let like = format!("%{}%", escape_like(text));
            params.push(like.clone());
            params.push(like);
        }
    }

    let mut notes = fetch(conn, &clauses, params)?;
    if let Some(text) = &text {
        for (rowid, meta) in &mut notes {
            let body: String = conn.query_row(
                "SELECT body FROM notes_fts WHERE rowid = ?1",
                [*rowid],
                |r| r.get(0),
            )?;
            meta.snippet = snippet(&body, text);
        }
    }
    Ok(notes.into_iter().map(|(_, m)| m).collect())
}

pub fn backlinks(conn: &Connection, id: &str) -> Result<Vec<NoteMeta>> {
    let clauses = ["n.id IN (SELECT src FROM links WHERE dst = ?)".to_string()];
    let notes = fetch(conn, &clauses, vec![id.to_string()])?;
    Ok(notes.into_iter().map(|(_, m)| m).collect())
}

fn fetch(
    conn: &Connection,
    clauses: &[String],
    params: Vec<String>,
) -> Result<Vec<(i64, NoteMeta)>> {
    let mut sql = String::from(
        "SELECT n.rowid, n.id, n.path, n.notebook, n.title, n.summary, n.created, n.updated
         FROM notes n",
    );
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }

    let mut tags = all_tags(conn)?;
    let rows = conn
        .prepare(&sql)?
        .query_map(params_from_iter(params), |r| {
            let id: String = r.get(1)?;
            Ok((
                r.get(0)?,
                NoteMeta {
                    tags: tags.remove(&id).unwrap_or_default(),
                    id,
                    path: r.get(2)?,
                    notebook: r.get(3)?,
                    title: r.get(4)?,
                    summary: r.get(5)?,
                    created: r.get(6)?,
                    updated: r.get(7)?,
                    snippet: None,
                },
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

fn all_tags(conn: &Connection) -> Result<HashMap<String, Vec<String>>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let mut stmt = conn.prepare("SELECT id, tag FROM note_tags ORDER BY tag")?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
        let (id, tag) = row?;
        map.entry(id).or_default().push(tag);
    }
    Ok(map)
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn fold(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn snippet(body: &str, needle: &str) -> Option<Snippet> {
    let hay: Vec<char> = body.chars().collect();
    let needle: Vec<char> = needle.chars().map(fold).collect();
    let pos = (0..hay.len().checked_sub(needle.len())? + 1).find(|&i| {
        hay[i..i + needle.len()]
            .iter()
            .zip(&needle)
            .all(|(h, n)| fold(*h) == *n)
    })?;

    let from = pos.saturating_sub(SNIPPET_BEFORE);
    let to = (pos + needle.len() + SNIPPET_AFTER).min(hay.len());
    let text = hay[from..to]
        .iter()
        .map(|&c| if c == '\n' { ' ' } else { c })
        .collect();
    Some(Snippet {
        text,
        start: pos - from,
        end: pos - from + needle.len(),
    })
}
