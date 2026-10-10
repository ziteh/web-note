use std::ops::Range;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLink {
    pub id: String,
    pub alias: Option<String>,
}

/// Text of the first H1, ignoring headings inside code blocks.
pub fn title(body: &str) -> Option<String> {
    let mut in_h1 = false;
    let mut text = String::new();
    for event in Parser::new(body) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => in_h1 = true,
            Event::End(TagEnd::Heading(HeadingLevel::H1)) => {
                let t = text.trim();
                return (!t.is_empty()).then(|| t.to_string());
            }
            Event::Text(t) | Event::Code(t) if in_h1 => text.push_str(&t),
            _ => {}
        }
    }
    None
}

/// Checkbox counts of a leading `TODO:` paragraph followed by a task list,
/// placed before the first H1. `None` when the note is not a todo note.
pub fn todo(body: &str) -> Option<(u32, u32)> {
    let mut events = Parser::new_ext(body, Options::ENABLE_TASKLISTS);
    let is_header = matches!(events.next(), Some(Event::Start(Tag::Paragraph)))
        && matches!(events.next(), Some(Event::Text(t)) if t.as_ref() == "TODO:")
        && matches!(events.next(), Some(Event::End(TagEnd::Paragraph)));
    if !is_header {
        return None;
    }
    let (mut done, mut total) = (0, 0);
    for event in events {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => break,
            Event::TaskListMarker(checked) => {
                total += 1;
                done += u32::from(checked);
            }
            _ => {}
        }
    }
    (total > 0).then_some((done, total))
}

/// `[[id]]` and `[[id|alias]]` links outside code blocks and inline code.
pub fn wikilinks(body: &str) -> Vec<WikiLink> {
    let code = code_ranges(body);
    let in_code = |pos: usize| code.iter().any(|r| r.contains(&pos));

    let mut links = Vec::new();
    let mut from = 0;
    while let Some(off) = body[from..].find("[[") {
        let start = from + off;
        from = start + 2;
        if in_code(start) {
            continue;
        }
        let Some(len) = body[from..].find("]]") else {
            break;
        };
        let inner = &body[from..from + len];
        if inner.contains(['\n', '[']) {
            continue;
        }
        let (id, alias) = match inner.split_once('|') {
            Some((id, alias)) => (id, Some(alias.to_string())),
            None => (inner, None),
        };
        if id::is_valid(id) {
            links.push(WikiLink {
                id: id.to_string(),
                alias,
            });
            from += len + 2;
        }
    }
    links
}

fn code_ranges(body: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut block_start = None;
    for (event, range) in Parser::new(body).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => block_start = Some(range.start),
            Event::End(TagEnd::CodeBlock) => {
                if let Some(s) = block_start.take() {
                    ranges.push(s..range.end);
                }
            }
            Event::Code(_) => ranges.push(range),
            _ => {}
        }
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_is_first_h1() {
        assert_eq!(
            title("intro\n\n# Hello\n\n# Second\n").as_deref(),
            Some("Hello")
        );
    }

    #[test]
    fn title_skips_code_blocks() {
        let body = "```\n# not a title\n```\n\n# Real\n";
        assert_eq!(title(body).as_deref(), Some("Real"));
    }

    #[test]
    fn title_none_without_h1() {
        assert_eq!(title("## only h2\n\ntext"), None);
        assert_eq!(title("```\n# fake\n```\n"), None);
    }

    #[test]
    fn title_keeps_inline_code_text() {
        assert_eq!(
            title("# Use `cargo` now\n").as_deref(),
            Some("Use cargo now")
        );
    }

    #[test]
    fn todo_counts_checkboxes_before_h1() {
        let body = "TODO:\n\n- [x] a\n- [ ] b\n\n# Title\n\n- [ ] not counted\n";
        assert_eq!(todo(body), Some((1, 2)));
    }

    #[test]
    fn todo_requires_leading_header_and_items() {
        assert_eq!(todo("# Title\n\n- [ ] a\n"), None);
        assert_eq!(todo("intro\n\nTODO:\n\n- [ ] a\n"), None);
        assert_eq!(todo("TODO:\n\n# Title\n"), None);
    }

    #[test]
    fn wikilinks_with_and_without_alias() {
        let links = wikilinks("see [[abc123|My Note]] and [[def456]]");
        assert_eq!(
            links,
            vec![
                WikiLink {
                    id: "abc123".into(),
                    alias: Some("My Note".into())
                },
                WikiLink {
                    id: "def456".into(),
                    alias: None
                },
            ]
        );
    }

    #[test]
    fn wikilinks_skip_code() {
        let body = "`[[aaa111]]`\n\n```\n[[bbb222]]\n```\n\n[[ccc333]]\n";
        let links = wikilinks(body);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].id, "ccc333");
    }

    #[test]
    fn wikilinks_reject_invalid_ids() {
        assert!(wikilinks("[[Not Valid]] [[]] [[a/b]]").is_empty());
    }
}
