use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use web_note_core as core;

#[derive(Serialize, ToSchema)]
pub struct Snippet {
    pub text: String,
    /// Character offsets of the match within `text`.
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, ToSchema)]
pub struct Todo {
    pub done: u32,
    pub total: u32,
}

#[derive(Serialize, ToSchema)]
pub struct NoteMeta {
    pub id: String,
    pub path: String,
    pub notebook: String,
    pub title: Option<String>,
    pub summary: String,
    pub tags: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    /// Present when the note starts with a `TODO:` checklist.
    pub todo: Option<Todo>,
    pub snippet: Option<Snippet>,
}

impl From<core::index::NoteMeta> for NoteMeta {
    fn from(m: core::index::NoteMeta) -> Self {
        Self {
            id: m.id,
            path: m.path,
            notebook: m.notebook,
            title: m.title,
            summary: m.summary,
            tags: m.tags,
            created: m.created,
            updated: m.updated,
            todo: m.todo.map(|t| Todo {
                done: t.done,
                total: t.total,
            }),
            snippet: m.snippet.map(|s| Snippet {
                text: s.text,
                start: s.start,
                end: s.end,
            }),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct Note {
    pub meta: NoteMeta,
    pub body: String,
    /// Pass back as `base_version` when saving.
    pub version: String,
}

impl From<core::notes::Note> for Note {
    fn from(n: core::notes::Note) -> Self {
        Self {
            meta: n.meta.into(),
            body: n.body,
            version: n.version,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateNote {
    pub notebook: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SaveNote {
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub base_version: String,
    /// Overwrite even if the note changed since `base_version`.
    #[serde(default)]
    pub force: bool,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct FormatBody {
    pub body: String,
}

#[derive(Deserialize, ToSchema)]
pub struct MoveNote {
    pub notebook: String,
}

#[derive(Serialize, ToSchema)]
pub struct Notebook {
    pub name: String,
    pub count: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct NotebookName {
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct Tag {
    pub tag: String,
    pub count: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct RenameTag {
    pub from: String,
    pub to: String,
}

#[derive(Serialize, ToSchema)]
pub struct RenameTagResult {
    pub affected: usize,
}

#[derive(Serialize, ToSchema)]
pub struct AssetUploaded {
    pub name: String,
    /// Relative path to use inside a note.
    pub path: String,
}

#[derive(Serialize, ToSchema)]
pub struct OrphanAssetsTrashed {
    pub count: usize,
}

#[derive(Serialize, ToSchema)]
pub struct TrashItem {
    pub key: String,
    pub notebook: String,
    pub id: Option<String>,
    pub title: Option<String>,
    pub trashed_at_millis: i64,
}

impl From<core::notes::TrashItem> for TrashItem {
    fn from(t: core::notes::TrashItem) -> Self {
        Self {
            key: t.key,
            notebook: t.notebook,
            id: t.id,
            title: t.title,
            trashed_at_millis: t.trashed_at_millis,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct LoadError {
    pub path: String,
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct Duplicate {
    pub path: String,
    pub new_id: String,
}

#[derive(Serialize, ToSchema)]
pub struct Status {
    pub errors: Vec<LoadError>,
    pub duplicates: Vec<Duplicate>,
}

impl From<core::notes::Status> for Status {
    fn from(s: core::notes::Status) -> Self {
        Self {
            errors: s
                .errors
                .into_iter()
                .map(|(path, message)| LoadError { path, message })
                .collect(),
            duplicates: s
                .duplicates
                .into_iter()
                .map(|d| Duplicate {
                    path: d.path,
                    new_id: d.new_id,
                })
                .collect(),
        }
    }
}
