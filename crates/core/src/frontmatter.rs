use serde_yaml_ng::{Mapping, Value};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use unicode_normalization::UnicodeNormalization;

use crate::error::{Error, Result};

const DELIMITER: &str = "---";
const KEY_ID: &str = "id";
const KEY_CREATED: &str = "created";
const KEY_UPDATED: &str = "updated";
const KEY_TAGS: &str = "tags";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
}

#[derive(Debug, Clone)]
pub struct Document {
    /// None when the file has no frontmatter or no id field.
    pub id: Option<String>,
    pub created: Option<OffsetDateTime>,
    pub updated: Option<OffsetDateTime>,
    pub tags: Vec<String>,
    /// Unknown frontmatter fields, preserved on write.
    pub extra: Mapping,
    /// Always LF internally; converted according to `line_ending` on render.
    pub body: String,
    pub line_ending: LineEnding,
}

impl Document {
    pub fn new(id: String, body: &str) -> Self {
        let now = now_local();
        let mut body = body.replace("\r\n", "\n");
        if !body.ends_with('\n') {
            body.push('\n');
        }
        Self {
            id: Some(id),
            created: Some(now),
            updated: Some(now),
            tags: Vec::new(),
            extra: Mapping::new(),
            body,
            line_ending: LineEnding::Lf,
        }
    }

    pub fn parse(text: &str) -> Result<Self> {
        let line_ending = if text.contains("\r\n") {
            LineEnding::CrLf
        } else {
            LineEnding::Lf
        };
        let text = text.replace("\r\n", "\n");

        let mut doc = Self {
            id: None,
            created: None,
            updated: None,
            tags: Vec::new(),
            extra: Mapping::new(),
            body: String::new(),
            line_ending,
        };

        let Some(rest) = text
            .strip_prefix(DELIMITER)
            .and_then(|r| r.strip_prefix('\n'))
        else {
            doc.body = text;
            return Ok(doc);
        };

        let (yaml, body) =
            split_closing(rest).ok_or_else(|| corrupt("missing closing delimiter"))?;
        doc.body = body.to_string();

        let value: Value = serde_yaml_ng::from_str(yaml).map_err(|e| corrupt(e.to_string()))?;
        let mut map = match value {
            Value::Mapping(m) => m,
            Value::Null => Mapping::new(),
            _ => return Err(corrupt("frontmatter is not a mapping")),
        };

        if let Some(v) = map.remove(KEY_ID) {
            doc.id = Some(string_field(KEY_ID, v)?);
        }
        if let Some(v) = map.remove(KEY_CREATED) {
            doc.created = Some(time_field(KEY_CREATED, v)?);
        }
        if let Some(v) = map.remove(KEY_UPDATED) {
            doc.updated = Some(time_field(KEY_UPDATED, v)?);
        }
        if let Some(v) = map.remove(KEY_TAGS) {
            doc.tags = tags_field(v)?;
        }
        doc.extra = map;
        Ok(doc)
    }

    pub fn render(&self) -> Result<String> {
        let mut map = Mapping::new();
        if let Some(id) = &self.id {
            map.insert(KEY_ID.into(), id.clone().into());
        }
        if let Some(t) = self.created {
            map.insert(KEY_CREATED.into(), format_time(t)?.into());
        }
        if let Some(t) = self.updated {
            map.insert(KEY_UPDATED.into(), format_time(t)?.into());
        }
        map.insert(
            KEY_TAGS.into(),
            Value::Sequence(self.tags.iter().cloned().map(Value::from).collect()),
        );
        for (k, v) in &self.extra {
            map.insert(k.clone(), v.clone());
        }

        let yaml =
            serde_yaml_ng::to_string(&Value::Mapping(map)).map_err(|e| corrupt(e.to_string()))?;
        let out = format!("{DELIMITER}\n{yaml}{DELIMITER}\n{}", self.body);
        Ok(match self.line_ending {
            LineEnding::Lf => out,
            LineEnding::CrLf => out.replace('\n', "\r\n"),
        })
    }
}

pub fn normalize_tag(tag: &str) -> String {
    tag.trim().nfc().collect::<String>().to_lowercase()
}

pub fn normalize_tags<S: AsRef<str>>(tags: &[S]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tag in tags {
        let tag = normalize_tag(tag.as_ref());
        if !tag.is_empty() && !out.contains(&tag) {
            out.push(tag);
        }
    }
    out
}

pub fn now_local() -> OffsetDateTime {
    OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
}

fn corrupt(msg: impl Into<String>) -> Error {
    Error::CorruptFrontmatter(msg.into())
}

/// Splits `rest` (text after the opening delimiter line) into yaml and body.
fn split_closing(rest: &str) -> Option<(&str, &str)> {
    let mut pos = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches('\n') == DELIMITER {
            return Some((&rest[..pos], &rest[pos + line.len()..]));
        }
        pos += line.len();
    }
    None
}

fn string_field(key: &str, v: Value) -> Result<String> {
    match v {
        Value::String(s) if !s.is_empty() => Ok(s),
        _ => Err(corrupt(format!("`{key}` must be a non-empty string"))),
    }
}

fn time_field(key: &str, v: Value) -> Result<OffsetDateTime> {
    let s = string_field(key, v)?;
    OffsetDateTime::parse(&s, &Rfc3339)
        .map_err(|_| corrupt(format!("`{key}` is not a valid RFC 3339 timestamp")))
}

fn tags_field(v: Value) -> Result<Vec<String>> {
    let items = match v {
        Value::Null => return Ok(Vec::new()),
        Value::Sequence(s) => s,
        _ => return Err(corrupt("`tags` must be a list")),
    };
    let mut raw = Vec::new();
    for item in items {
        let Value::String(s) = item else {
            return Err(corrupt("`tags` must contain only strings"));
        };
        raw.push(s);
    }
    Ok(normalize_tags(&raw))
}

pub fn format_time(t: OffsetDateTime) -> Result<String> {
    t.format(&Rfc3339).map_err(|e| corrupt(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "---\nid: abc123\ncreated: 2025-01-02T03:04:05+08:00\nupdated: 2025-01-02T03:04:05+08:00\ntags:\n- Foo\nmood: happy\n---\n# Title\n\nbody\n";

    #[test]
    fn parses_known_fields() {
        let d = Document::parse(SAMPLE).unwrap();
        assert_eq!(d.id.as_deref(), Some("abc123"));
        assert_eq!(d.tags, vec!["foo"]);
        assert_eq!(d.body, "# Title\n\nbody\n");
        assert!(d.created.is_some() && d.updated.is_some());
    }

    #[test]
    fn preserves_unknown_fields() {
        let d = Document::parse(SAMPLE).unwrap();
        let out = d.render().unwrap();
        assert!(out.contains("mood: happy"));
        let again = Document::parse(&out).unwrap();
        assert_eq!(again.extra, d.extra);
    }

    #[test]
    fn preserves_offset() {
        let out = Document::parse(SAMPLE).unwrap().render().unwrap();
        assert!(out.contains("2025-01-02T03:04:05+08:00"));
    }

    #[test]
    fn preserves_crlf() {
        let text = SAMPLE.replace('\n', "\r\n");
        let out = Document::parse(&text).unwrap().render().unwrap();
        assert!(out.contains("\r\n"));
        assert!(!out.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn preserves_missing_trailing_newline() {
        let text = SAMPLE.trim_end_matches('\n');
        let d = Document::parse(text).unwrap();
        assert!(!d.render().unwrap().ends_with('\n'));
    }

    #[test]
    fn new_document_is_lf_with_trailing_newline() {
        let d = Document::new("abc".into(), "hello");
        let out = d.render().unwrap();
        assert!(out.ends_with("hello\n"));
        assert!(!out.contains('\r'));
    }

    #[test]
    fn no_frontmatter_has_no_id() {
        let d = Document::parse("# Just text\n").unwrap();
        assert!(d.id.is_none());
        assert_eq!(d.body, "# Just text\n");
    }

    #[test]
    fn missing_closing_delimiter_is_corrupt() {
        let r = Document::parse("---\nid: a\n# Title\n");
        assert!(matches!(r, Err(Error::CorruptFrontmatter(_))));
    }

    #[test]
    fn invalid_yaml_is_corrupt() {
        let r = Document::parse("---\nid: [unclosed\n---\nbody\n");
        assert!(matches!(r, Err(Error::CorruptFrontmatter(_))));
    }

    #[test]
    fn duplicate_id_is_corrupt() {
        let r = Document::parse("---\nid: a\nid: b\n---\nbody\n");
        assert!(matches!(r, Err(Error::CorruptFrontmatter(_))));
    }

    #[test]
    fn invalid_timestamp_is_corrupt() {
        let r = Document::parse("---\nid: a\ncreated: yesterday\n---\n");
        assert!(matches!(r, Err(Error::CorruptFrontmatter(_))));
    }

    #[test]
    fn tags_are_normalized_and_deduped() {
        let d = Document::parse("---\nid: a\ntags: [Foo, foo, ' BAR ']\n---\n").unwrap();
        assert_eq!(d.tags, vec!["foo", "bar"]);
    }

    #[test]
    fn tags_are_nfc_normalized() {
        assert_eq!(normalize_tag("e\u{301}"), "\u{e9}");
    }
}
