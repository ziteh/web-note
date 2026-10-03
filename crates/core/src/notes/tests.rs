use std::fs;

use super::*;

fn open() -> (tempfile::TempDir, Notes) {
    let dir = tempfile::tempdir().unwrap();
    let notes = Notes::open(dir.path(), 1024).unwrap();
    (dir, notes)
}

fn q(text: &str) -> Query {
    Query {
        text: Some(text.into()),
        ..Default::default()
    }
}

#[test]
fn open_creates_layout() {
    let (dir, _) = open();
    for d in [DEFAULT_NOTEBOOK, ASSETS_DIR, TRASH_DIR, CACHE_DIR] {
        assert!(dir.path().join(d).is_dir());
    }
}

#[test]
fn create_and_get() {
    let (dir, notes) = open();
    let n = notes
        .create(DEFAULT_NOTEBOOK, "# Hello\n\nworld", &["A".into()])
        .unwrap();
    assert_eq!(n.meta.title.as_deref(), Some("Hello"));
    assert_eq!(n.meta.tags, vec!["a"]);
    assert!(n.body.ends_with('\n'));
    assert!(dir.path().join(&n.meta.path).is_file());
    assert_eq!(notes.get(&n.meta.id).unwrap().version, n.version);
}

#[test]
fn create_in_missing_notebook_fails() {
    let (_dir, notes) = open();
    assert!(matches!(
        notes.create("Nope", "x", &[]),
        Err(Error::NotebookNotFound)
    ));
    assert!(matches!(
        notes.create("../x", "x", &[]),
        Err(Error::InvalidName)
    ));
}

#[test]
fn save_updates_title_and_timestamp_only_on_change() {
    let (_dir, notes) = open();
    let n = notes.create(DEFAULT_NOTEBOOK, "# Old\n", &[]).unwrap();

    let same = notes
        .save(&n.meta.id, &n.body, &[], &n.version, false)
        .unwrap();
    assert_eq!(same.version, n.version);
    assert_eq!(same.meta.updated, n.meta.updated);

    std::thread::sleep(std::time::Duration::from_millis(1100));
    let changed = notes
        .save(&n.meta.id, "# New\n", &[], &n.version, false)
        .unwrap();
    assert_eq!(changed.meta.title.as_deref(), Some("New"));
    assert_ne!(changed.meta.updated, n.meta.updated);
    assert_eq!(changed.meta.created, n.meta.created);
}

#[test]
fn save_conflict_and_force_keeps_overwritten_version_in_trash() {
    let (dir, notes) = open();
    let n = notes.create(DEFAULT_NOTEBOOK, "# A\n", &[]).unwrap();
    notes
        .save(&n.meta.id, "# Mine one\n", &[], &n.version, false)
        .unwrap();

    let stale = notes.save(&n.meta.id, "# Mine two\n", &[], &n.version, false);
    assert!(matches!(stale, Err(Error::Conflict)));

    let forced = notes
        .save(&n.meta.id, "# Mine two\n", &[], &n.version, true)
        .unwrap();
    assert_eq!(forced.meta.title.as_deref(), Some("Mine two"));
    let trashed = notes.trash().unwrap();
    assert_eq!(trashed.len(), 1);
    let (_, path) = notes.find_trashed(&trashed[0].key).unwrap();
    assert!(fs::read_to_string(path).unwrap().contains("Mine one"));
    assert!(dir.path().join(&forced.meta.path).is_file());
}

#[test]
fn save_preserves_unknown_fields_and_crlf() {
    let (dir, notes) = open();
    let n = notes.create(DEFAULT_NOTEBOOK, "# A\n", &[]).unwrap();
    let abs = dir.path().join(&n.meta.path);
    let text = fs::read_to_string(&abs)
        .unwrap()
        .replace("tags: []", "tags: []\nmood: calm");
    fs::write(&abs, text.replace('\n', "\r\n")).unwrap();

    let cur = notes.get(&n.meta.id).unwrap();
    notes
        .save(&n.meta.id, "# B\n", &[], &cur.version, false)
        .unwrap();
    let after = fs::read_to_string(&abs).unwrap();
    assert!(after.contains("mood: calm"));
    assert!(after.contains("\r\n"));
    assert!(!after.replace("\r\n", "").contains('\n'));
}

#[test]
fn rename_h1_move_notebook_links_still_resolve() {
    let (_dir, notes) = open();
    notes.create_notebook("Work").unwrap();
    let target = notes.create(DEFAULT_NOTEBOOK, "# Target\n", &[]).unwrap();
    let body = format!("# Source\n\n[[{}|Target]]\n", target.meta.id);
    notes.create(DEFAULT_NOTEBOOK, &body, &[]).unwrap();

    let t = notes
        .save(&target.meta.id, "# Renamed\n", &[], &target.version, false)
        .unwrap();
    let moved = notes.move_note(&target.meta.id, "Work").unwrap();
    assert_eq!(moved.meta.notebook, "Work");
    assert_eq!(moved.meta.updated, t.meta.updated);
    assert_eq!(notes.backlinks(&target.meta.id).unwrap().len(), 1);
    assert_eq!(
        notes.get(&target.meta.id).unwrap().meta.title.as_deref(),
        Some("Renamed")
    );
}

#[test]
fn external_modification_is_noticed_on_get() {
    let (dir, notes) = open();
    let n = notes.create(DEFAULT_NOTEBOOK, "# A\n", &[]).unwrap();
    let abs = dir.path().join(&n.meta.path);
    let text = fs::read_to_string(&abs)
        .unwrap()
        .replace("# A", "# Edited elsewhere");
    fs::write(&abs, text).unwrap();

    let got = notes.get(&n.meta.id).unwrap();
    assert_eq!(got.meta.title.as_deref(), Some("Edited elsewhere"));
    assert_ne!(got.version, n.version);
    assert!(matches!(
        notes.save(&n.meta.id, "# Mine\n", &[], &n.version, false),
        Err(Error::Conflict)
    ));
}

#[test]
fn delete_to_trash_and_restore() {
    let (_dir, notes) = open();
    let n = notes
        .create(DEFAULT_NOTEBOOK, "# Gone\n\nfindable text\n", &[])
        .unwrap();
    notes.delete(&n.meta.id).unwrap();
    assert!(matches!(notes.get(&n.meta.id), Err(Error::NoteNotFound)));
    assert!(notes.list(&q("findable")).unwrap().is_empty());

    let items = notes.trash().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title.as_deref(), Some("Gone"));
    let restored = notes.restore(&items[0].key).unwrap();
    assert_eq!(restored.meta.id, n.meta.id);
    assert!(notes.trash().unwrap().is_empty());
    assert_eq!(notes.list(&q("findable")).unwrap().len(), 1);
}

#[test]
fn restore_with_taken_id_gets_fresh_id() {
    let (_dir, notes) = open();
    let n = notes.create(DEFAULT_NOTEBOOK, "# A\n", &[]).unwrap();
    notes.save(&n.meta.id, "# B\n", &[], "stale", true).unwrap();
    let key = notes.trash().unwrap().remove(0).key;
    let restored = notes.restore(&key).unwrap();
    assert_ne!(restored.meta.id, n.meta.id);
    assert_eq!(notes.list(&Query::default()).unwrap().len(), 2);
}

#[test]
fn purge_and_empty_trash() {
    let (_dir, notes) = open();
    for _ in 0..2 {
        let n = notes.create(DEFAULT_NOTEBOOK, "# A\n", &[]).unwrap();
        notes.delete(&n.meta.id).unwrap();
    }
    let items = notes.trash().unwrap();
    notes.purge(&items[0].key).unwrap();
    assert_eq!(notes.trash().unwrap().len(), 1);
    assert!(matches!(
        notes.purge("../../etc/passwd"),
        Err(Error::TrashItemNotFound)
    ));
    notes.empty_trash().unwrap();
    assert!(notes.trash().unwrap().is_empty());
}

#[test]
fn notebook_lifecycle() {
    let (_dir, notes) = open();
    notes.create_notebook("  工作 ").unwrap();
    assert!(matches!(
        notes.create_notebook("工作"),
        Err(Error::NotebookExists)
    ));
    let n = notes.create("工作", "# A\n", &[]).unwrap();
    assert!(matches!(
        notes.delete_notebook("工作"),
        Err(Error::NotebookNotEmpty)
    ));

    notes.rename_notebook("工作", "Work Stuff").unwrap();
    assert_eq!(notes.get(&n.meta.id).unwrap().meta.notebook, "Work Stuff");
    assert!(matches!(
        notes.rename_notebook(DEFAULT_NOTEBOOK, "X"),
        Err(Error::ProtectedNotebook)
    ));
    assert!(matches!(
        notes.delete_notebook(DEFAULT_NOTEBOOK),
        Err(Error::ProtectedNotebook)
    ));

    notes.move_note(&n.meta.id, DEFAULT_NOTEBOOK).unwrap();
    notes.delete_notebook("Work Stuff").unwrap();
    let names: Vec<_> = notes
        .notebooks()
        .unwrap()
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    assert_eq!(names, vec![DEFAULT_NOTEBOOK]);
}

#[test]
fn notebook_names_are_validated() {
    let (_dir, notes) = open();
    for bad in ["", "  ", ".hidden", "a/b", "a\\b", "..", "a\nb"] {
        assert!(
            matches!(notes.create_notebook(bad), Err(Error::InvalidName)),
            "{bad:?}"
        );
    }
}

#[test]
fn rename_and_merge_tags() {
    let (_dir, notes) = open();
    notes
        .create(DEFAULT_NOTEBOOK, "# A\n", &["old".into(), "keep".into()])
        .unwrap();
    notes
        .create(DEFAULT_NOTEBOOK, "# B\n", &["old".into(), "new".into()])
        .unwrap();
    notes
        .create(DEFAULT_NOTEBOOK, "# C\n", &["other".into()])
        .unwrap();

    assert_eq!(notes.rename_tag("OLD", "new").unwrap(), 2);
    let tags = notes.tags().unwrap();
    assert_eq!(
        tags,
        vec![("keep".into(), 1), ("new".into(), 2), ("other".into(), 1)]
    );
    assert_eq!(notes.rename_tag("same", "same").unwrap(), 0);
    assert!(matches!(
        notes.rename_tag("a", " "),
        Err(Error::InvalidName)
    ));
}

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nrest";

#[test]
fn assets_dedupe_and_validate() {
    let (dir, notes) = open();
    let a = notes.add_asset(PNG).unwrap();
    let b = notes.add_asset(PNG).unwrap();
    assert_eq!(a, b);
    assert!(a.ends_with(".png"));
    assert_eq!(
        fs::read_dir(dir.path().join(ASSETS_DIR)).unwrap().count(),
        1
    );

    let (bytes, ct) = notes.read_asset(&a).unwrap();
    assert_eq!((bytes.as_slice(), ct), (PNG, "image/png"));

    assert!(matches!(
        notes.add_asset(b"not an image"),
        Err(Error::UnsupportedAsset)
    ));
    assert!(matches!(
        notes.add_asset(&vec![0; 2048]),
        Err(Error::AssetTooLarge)
    ));
    assert!(matches!(
        notes.read_asset("../../x.png"),
        Err(Error::AssetNotFound)
    ));
    assert!(matches!(
        notes.read_asset("abc.png"),
        Err(Error::AssetNotFound)
    ));
}

#[test]
fn rescan_reports_duplicates_until_dismissed() {
    let (dir, notes) = open();
    let n = notes.create(DEFAULT_NOTEBOOK, "# A\n", &[]).unwrap();
    fs::copy(
        dir.path().join(&n.meta.path),
        dir.path().join(DEFAULT_NOTEBOOK).join("copy.md"),
    )
    .unwrap();

    let status = notes.rescan().unwrap();
    assert_eq!(status.duplicates.len(), 1);
    assert_eq!(notes.status().unwrap().duplicates.len(), 1);
    notes.dismiss_duplicates();
    assert!(notes.status().unwrap().duplicates.is_empty());
}

#[test]
fn reopen_reuses_index_and_sees_files() {
    let dir = tempfile::tempdir().unwrap();
    let id = {
        let notes = Notes::open(dir.path(), 1024).unwrap();
        notes
            .create(DEFAULT_NOTEBOOK, "# Persist\n", &[])
            .unwrap()
            .meta
            .id
    };
    let notes = Notes::open(dir.path(), 1024).unwrap();
    assert_eq!(
        notes.get(&id).unwrap().meta.title.as_deref(),
        Some("Persist")
    );
}
