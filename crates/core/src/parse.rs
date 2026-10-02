use std::ops::Range;

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

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
        assert_eq!(title("intro\n\n# Hello\n\n# Second\n").as_deref(), Some("Hello"));
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
        assert_eq!(title("# Use `cargo` now\n").as_deref(), Some("Use cargo now"));
    }

    #[test]
    fn wikilinks_with_and_without_alias() {
        let links = wikilinks("see [[abc123|My Note]] and [[def456]]");
        assert_eq!(
            links,
            vec![
                WikiLink { id: "abc123".into(), alias: Some("My Note".into()) },
                WikiLink { id: "def456".into(), alias: None },
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
