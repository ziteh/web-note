use rumdl_lib::config::{Config, default_registry};
use rumdl_lib::fix_coordinator::FixCoordinator;
use rumdl_lib::markdownlint_config::MarkdownlintConfig;
use rumdl_lib::rules::{all_rules, filter_rules};

const RULES: &str = include_str!("../../../.markdownlint.yaml");

const MAX_ITERATIONS: usize = 10;

/// Applies every auto-fixable markdownlint rule to a note body.
pub fn markdown(body: &str) -> String {
    let config = rules_config();
    let rules = filter_rules(&all_rules(&config), &config.global);
    let mut content = body.to_string();
    FixCoordinator::new()
        .apply_fixes_iterative(&rules, &[], &mut content, &config, MAX_ITERATIONS, None)
        .expect("bundled rule config is valid");
    content
}

fn rules_config() -> Config {
    let parsed: MarkdownlintConfig =
        serde_yaml_ng::from_str(RULES).expect("bundled rule config is valid");
    let (config, _) = parsed
        .map_to_sourced_rumdl_config(None)
        .validate_into(default_registry())
        .expect("bundled rule config is valid");
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixes_common_problems() {
        let out = markdown("# Title\ntext\n\n\n\n* a\n* b\n");
        assert_eq!(out, "# Title\n\ntext\n\n- a\n- b\n");
    }

    #[test]
    fn keeps_wikilinks() {
        let src = "see [[abc123|alias]] and [[def456]]\n";
        assert_eq!(markdown(src), src);
    }

    #[test]
    fn keeps_task_checkboxes() {
        let src = "- [ ] todo\n- [x] done\n";
        assert_eq!(markdown(src), src);
    }

    #[test]
    fn is_idempotent() {
        let once = markdown("# T\n* a\n\n\n*  b\ntext\n```\ncode\n```\n");
        assert_eq!(markdown(&once), once);
    }

    #[test]
    fn empty_body() {
        assert_eq!(markdown(""), "");
    }
}
