use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use rusqlite::Connection;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use super::{FileStat, remove, upsert};
use crate::{
    error::{Error, Result},
    frontmatter::{Document, now_local},
    id,
    store::{read_document, write_document},
};

#[derive(Debug, Default)]
pub struct Report {
    pub duplicates: Vec<Duplicate>,
}

/// A file whose id collided with an older note and was given a new one.
#[derive(Debug, Clone)]
pub struct Duplicate {
    pub path: String,
    pub new_id: String,
}

struct Entry {
    rel: String,
    abs: PathBuf,
    id: String,
    created: Option<OffsetDateTime>,
    stat: FileStat,
    /// None when the file is unchanged since it was indexed.
    doc: Option<Document>,
}

struct Known {
    id: String,
    created: Option<OffsetDateTime>,
    stat: FileStat,
}

/// Brings the index in line with the files under `root`. Only files whose
/// mtime or size differ from the index are parsed.
pub fn reconcile(conn: &mut Connection, root: &Path) -> Result<Report> {
    let files = scan(root)?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM errors", [])?;
    let known = load_known(&tx)?;

    let mut entries = Vec::new();
    for (rel, abs, stat) in files {
        if let Some(k) = known.get(&rel).filter(|k| k.stat == stat) {
            entries.push(Entry {
                rel,
                abs,
                id: k.id.clone(),
                created: k.created,
                stat,
                doc: None,
            });
            continue;
        }
        let mut doc = match read_document(&abs) {
            Ok(doc) => doc,
            Err(Error::CorruptFrontmatter(msg)) => {
                tx.execute(
                    "INSERT INTO errors (path, message) VALUES (?1, ?2)",
                    [&rel, &msg],
                )?;
                continue;
            }
            Err(e) => return Err(e),
        };
        let mut stat = stat;
        let id = match doc.id.clone() {
            Some(id) => id,
            None => {
                let new_id = id::generate();
                let now = now_local();
                doc.id = Some(new_id.clone());
                doc.created.get_or_insert(now);
                doc.updated.get_or_insert(now);
                write_document(&abs, &doc)?;
                stat = stat_of(&abs)?;
                new_id
            }
        };
        entries.push(Entry {
            rel,
            abs,
            id,
            created: doc.created,
            stat,
            doc: Some(doc),
        });
    }

    let duplicates = resolve_duplicates(&mut entries)?;

    for e in &entries {
        if let Some(doc) = &e.doc {
            upsert(&tx, &e.rel, &e.id, doc, e.stat)?;
        }
    }

    let on_disk: HashSet<&str> = entries.iter().map(|e| e.rel.as_str()).collect();
    let indexed: Vec<(String, String)> = tx
        .prepare("SELECT id, path FROM notes")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    for (id, path) in indexed {
        if !on_disk.contains(path.as_str()) {
            remove(&tx, &id)?;
        }
    }

    tx.commit()?;
    Ok(Report { duplicates })
}

/// For each id shared by several files, the earliest `created` (then oldest
/// mtime) keeps it; the others are rewritten with a fresh id.
fn resolve_duplicates(entries: &mut [Entry]) -> Result<Vec<Duplicate>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        groups.entry(e.id.clone()).or_default().push(i);
    }

    let mut fixed = Vec::new();
    for mut idxs in groups.into_values().filter(|g| g.len() > 1) {
        idxs.sort_by_key(|&i| {
            (
                entries[i].created.is_none(),
                entries[i].created,
                entries[i].stat.mtime,
            )
        });
        for &i in &idxs[1..] {
            let e = &mut entries[i];
            let mut doc = match e.doc.take() {
                Some(doc) => doc,
                None => read_document(&e.abs)?,
            };
            e.id = id::generate();
            doc.id = Some(e.id.clone());
            write_document(&e.abs, &doc)?;
            e.stat = stat_of(&e.abs)?;
            e.doc = Some(doc);
            fixed.push(Duplicate {
                path: e.rel.clone(),
                new_id: e.id.clone(),
            });
        }
    }
    Ok(fixed)
}

fn load_known(conn: &Connection) -> Result<HashMap<String, Known>> {
    let mut stmt = conn.prepare("SELECT path, id, created, mtime, size FROM notes")?;
    let rows = stmt.query_map([], |r| {
        let created: Option<String> = r.get(2)?;
        Ok((
            r.get::<_, String>(0)?,
            Known {
                id: r.get(1)?,
                created: created.and_then(|s| OffsetDateTime::parse(&s, &Rfc3339).ok()),
                stat: FileStat {
                    mtime: r.get(3)?,
                    size: r.get(4)?,
                },
            },
        ))
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Every `<root>/<notebook>/*.md`, skipping dot-directories such as assets,
/// trash and cache. Returns (relative path, absolute path, stat).
fn scan(root: &Path) -> Result<Vec<(String, PathBuf, FileStat)>> {
    let io = |path: &Path, source| Error::Io {
        path: path.to_path_buf(),
        source,
    };
    let mut files = Vec::new();
    for nb in fs::read_dir(root).map_err(|e| io(root, e))? {
        let nb = nb.map_err(|e| io(root, e))?;
        let nb_name = nb.file_name().to_string_lossy().into_owned();
        if nb_name.starts_with('.') || !nb.path().is_dir() {
            continue;
        }
        for f in fs::read_dir(nb.path()).map_err(|e| io(&nb.path(), e))? {
            let path = f.map_err(|e| io(&nb.path(), e))?.path();
            if path.extension().is_some_and(|x| x == "md") && path.is_file() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                let rel = format!("{nb_name}/{name}");
                let stat = stat_of(&path)?;
                files.push((rel, path, stat));
            }
        }
    }
    Ok(files)
}

fn stat_of(path: &Path) -> Result<FileStat> {
    let meta = fs::metadata(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos() as i64);
    Ok(FileStat {
        mtime,
        size: meta.len() as i64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{self, Query, backlinks, errors, list, path_of};
    use std::fs;

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn note(id: &str, created: &str, tags: &str, body: &str) -> String {
        format!(
            "---\nid: {id}\ncreated: {created}\nupdated: {created}\ntags: [{tags}]\n---\n{body}"
        )
    }

    const T1: &str = "2025-01-01T00:00:00+08:00";
    const T2: &str = "2025-02-01T00:00:00+08:00";

    fn setup() -> (tempfile::TempDir, Connection) {
        (
            tempfile::tempdir().unwrap(),
            index::open_in_memory().unwrap(),
        )
    }

    fn all(conn: &Connection) -> Vec<index::NoteMeta> {
        list(conn, &Query::default()).unwrap()
    }

    fn search(conn: &Connection, text: &str) -> Vec<index::NoteMeta> {
        let q = Query {
            text: Some(text.into()),
            ..Default::default()
        };
        list(conn, &q).unwrap()
    }

    #[test]
    fn indexes_notes_and_skips_dot_dirs() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/aaa111.md",
            &note("aaa111", T1, "Rust", "# Hello\n\nfirst line\n"),
        );
        write(
            dir.path(),
            ".trash/zzz999.md",
            &note("zzz999", T1, "", "# Gone\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();

        let notes = all(&conn);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title.as_deref(), Some("Hello"));
        assert_eq!(notes[0].summary, "first line");
        assert_eq!(notes[0].notebook, "Inbox");
        assert_eq!(notes[0].tags, vec!["rust"]);
    }

    #[test]
    fn missing_id_is_filled_and_written_back() {
        let (dir, mut conn) = setup();
        write(dir.path(), "Inbox/plain.md", "# Plain\n");
        reconcile(&mut conn, dir.path()).unwrap();

        let notes = all(&conn);
        assert_eq!(notes.len(), 1);
        let on_disk = fs::read_to_string(dir.path().join("Inbox/plain.md")).unwrap();
        assert!(on_disk.contains(&format!("id: {}", notes[0].id)));

        reconcile(&mut conn, dir.path()).unwrap();
        assert_eq!(all(&conn)[0].id, notes[0].id);
    }

    #[test]
    fn duplicate_id_keeps_earliest_created() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("dup111", T2, "", "# Newer\n"),
        );
        write(
            dir.path(),
            "Work/b.md",
            &note("dup111", T1, "", "# Older\n"),
        );
        let report = reconcile(&mut conn, dir.path()).unwrap();

        assert_eq!(report.duplicates.len(), 1);
        assert_eq!(report.duplicates[0].path, "Inbox/a.md");
        assert_eq!(
            path_of(&conn, "dup111").unwrap().as_deref(),
            Some("Work/b.md")
        );
        let new_id = &report.duplicates[0].new_id;
        assert_eq!(
            path_of(&conn, new_id).unwrap().as_deref(),
            Some("Inbox/a.md")
        );
        let on_disk = fs::read_to_string(dir.path().join("Inbox/a.md")).unwrap();
        assert!(on_disk.contains(&format!("id: {new_id}")));
        assert_eq!(all(&conn).len(), 2);
    }

    #[test]
    fn moved_file_keeps_id_and_updates_path() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/m.md",
            &note("mov111", T1, "", "# Move\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();

        fs::create_dir_all(dir.path().join("Work")).unwrap();
        fs::rename(dir.path().join("Inbox/m.md"), dir.path().join("Work/m.md")).unwrap();
        let report = reconcile(&mut conn, dir.path()).unwrap();

        assert!(report.duplicates.is_empty());
        assert_eq!(all(&conn).len(), 1);
        assert_eq!(
            path_of(&conn, "mov111").unwrap().as_deref(),
            Some("Work/m.md")
        );
        assert_eq!(all(&conn)[0].notebook, "Work");
    }

    #[test]
    fn deleted_file_is_removed_from_index() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("del111", T1, "x", "# A\n\nsearchable\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();
        fs::remove_file(dir.path().join("Inbox/a.md")).unwrap();
        reconcile(&mut conn, dir.path()).unwrap();

        assert!(all(&conn).is_empty());
        assert!(search(&conn, "searchable").is_empty());
    }

    #[test]
    fn external_edit_is_picked_up() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("edt111", T1, "", "# Before\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("edt111", T1, "", "# After the edit\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();

        assert_eq!(all(&conn)[0].title.as_deref(), Some("After the edit"));
    }

    #[test]
    fn id_changed_at_same_path_replaces_old_row() {
        let (dir, mut conn) = setup();
        write(dir.path(), "Inbox/a.md", &note("old111", T1, "", "# A\n"));
        reconcile(&mut conn, dir.path()).unwrap();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("new111", T1, "", "# A changed\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();

        let notes = all(&conn);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, "new111");
    }

    #[test]
    fn corrupt_file_is_reported_not_indexed() {
        let (dir, mut conn) = setup();
        write(dir.path(), "Inbox/bad.md", "---\nid: a\nno closing\n");
        write(dir.path(), "Inbox/ok.md", &note("ok1111", T1, "", "# Ok\n"));
        reconcile(&mut conn, dir.path()).unwrap();

        assert_eq!(all(&conn).len(), 1);
        let errs = errors(&conn).unwrap();
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].0, "Inbox/bad.md");
        assert_eq!(
            fs::read_to_string(dir.path().join("Inbox/bad.md")).unwrap(),
            "---\nid: a\nno closing\n"
        );
    }

    #[test]
    fn note_turning_corrupt_leaves_index() {
        let (dir, mut conn) = setup();
        write(dir.path(), "Inbox/a.md", &note("cor111", T1, "", "# A\n"));
        reconcile(&mut conn, dir.path()).unwrap();
        write(dir.path(), "Inbox/a.md", "---\nid: cor111\nbroken\n");
        reconcile(&mut conn, dir.path()).unwrap();

        assert!(all(&conn).is_empty());
        assert_eq!(errors(&conn).unwrap().len(), 1);
    }

    #[test]
    fn backlinks_follow_links_and_survive_title_changes() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/t.md",
            &note("tgt111", T1, "", "# Target\n"),
        );
        write(
            dir.path(),
            "Inbox/s.md",
            &note("src111", T1, "", "# Source\n\nsee [[tgt111|Target]]\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();
        write(
            dir.path(),
            "Inbox/t.md",
            &note("tgt111", T1, "", "# Renamed\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();

        let links = backlinks(&conn, "tgt111").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].id, "src111");
        assert!(backlinks(&conn, "src111").unwrap().is_empty());
    }

    #[test]
    fn search_long_query_with_snippet() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("fts111", T1, "", "# A\n\nThe Quick brown fox\n"),
        );
        write(
            dir.path(),
            "Inbox/b.md",
            &note("fts222", T1, "", "# B\n\nnothing here\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();

        let hits = search(&conn, "quick BROWN");
        assert_eq!(hits.len(), 1);
        let s = hits[0].snippet.as_ref().unwrap();
        let chars: Vec<char> = s.text.chars().collect();
        assert_eq!(
            chars[s.start..s.end].iter().collect::<String>(),
            "Quick brown"
        );
    }

    #[test]
    fn search_matches_title() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("ttl111", T1, "", "# Unique heading\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();
        assert_eq!(search(&conn, "Unique head").len(), 1);
    }

    #[test]
    fn search_short_query_falls_back_to_like() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("shr111", T1, "", "# A\n\n今天天氣很好\n"),
        );
        write(
            dir.path(),
            "Inbox/b.md",
            &note("shr222", T1, "", "# B\n\n100% done_ok\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();

        assert_eq!(search(&conn, "天氣").len(), 1);
        assert_eq!(search(&conn, "好").len(), 1);
        assert_eq!(search(&conn, "%").len(), 1);
        assert_eq!(search(&conn, "_o").len(), 1);
    }

    #[test]
    fn search_long_cjk_query() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("cjk111", T1, "", "# A\n\n今天天氣很好\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();
        assert_eq!(search(&conn, "天氣很").len(), 1);
        assert!(search(&conn, "天氣不").is_empty());
    }

    #[test]
    fn search_query_with_quotes_does_not_break() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("qte111", T1, "", "# A\n\nsay \"hello\" now\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();
        assert_eq!(search(&conn, "\"hello\"").len(), 1);
    }

    #[test]
    fn filters_combine_with_search_and_tags_are_anded() {
        let (dir, mut conn) = setup();
        write(
            dir.path(),
            "Inbox/a.md",
            &note("flt111", T1, "x, y", "# A\n\nshared words\n"),
        );
        write(
            dir.path(),
            "Inbox/b.md",
            &note("flt222", T1, "x", "# B\n\nshared words\n"),
        );
        write(
            dir.path(),
            "Work/c.md",
            &note("flt333", T1, "x, y", "# C\n\nshared words\n"),
        );
        reconcile(&mut conn, dir.path()).unwrap();

        let q = |notebook: Option<&str>, tags: &[&str], text: Option<&str>| Query {
            notebook: notebook.map(Into::into),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            text: text.map(Into::into),
        };
        assert_eq!(list(&conn, &q(None, &["x"], None)).unwrap().len(), 3);
        assert_eq!(list(&conn, &q(None, &["X", "y"], None)).unwrap().len(), 2);
        assert_eq!(
            list(&conn, &q(Some("Inbox"), &["x", "y"], None))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            list(&conn, &q(Some("Work"), &[], Some("shared")))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn schema_version_mismatch_rebuilds() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("index.db");
        let mut conn = index::open(&db).unwrap();
        let notes = tempfile::tempdir().unwrap();
        write(notes.path(), "Inbox/a.md", &note("ver111", T1, "", "# A\n"));
        reconcile(&mut conn, notes.path()).unwrap();
        conn.pragma_update(None, "user_version", 0).unwrap();
        drop(conn);

        let conn = index::open(&db).unwrap();
        assert!(all(&conn).is_empty());
    }
}
