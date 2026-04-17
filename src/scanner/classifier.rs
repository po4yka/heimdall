//! ROADMAP Phase 2 — deterministic task-category classifier.
//!
//! Pure function: given the tools invoked in a turn (and optionally the
//! first user message of the session), assign one of 13 categories.
//! No LLM calls, no state, no I/O. Safe across threads.
//!
//! The rule set is intentionally starter-quality: it classifies every
//! turn heimdall can observe today using only `tool_name` plus `all_tools`
//! and an optional session prompt hint. Future phases can enrich the
//! inputs (tool arguments, bash command prefixes, file extensions)
//! without breaking this enum or the storage shape.

use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;

use regex::RegexSet;
use serde::{Deserialize, Serialize};

/// The 13 canonical task categories. Stable slugs are stored in
/// `turns.category`; changing a slug is a breaking migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskCategory {
    Coding,
    Debugging,
    FeatureDev,
    Testing,
    Git,
    Docs,
    Research,
    Refactor,
    DevOps,
    Config,
    Planning,
    Review,
    Other,
}

impl TaskCategory {
    pub const ALL: [TaskCategory; 13] = [
        TaskCategory::Coding,
        TaskCategory::Debugging,
        TaskCategory::FeatureDev,
        TaskCategory::Testing,
        TaskCategory::Git,
        TaskCategory::Docs,
        TaskCategory::Research,
        TaskCategory::Refactor,
        TaskCategory::DevOps,
        TaskCategory::Config,
        TaskCategory::Planning,
        TaskCategory::Review,
        TaskCategory::Other,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            TaskCategory::Coding => "coding",
            TaskCategory::Debugging => "debugging",
            TaskCategory::FeatureDev => "feature_dev",
            TaskCategory::Testing => "testing",
            TaskCategory::Git => "git",
            TaskCategory::Docs => "docs",
            TaskCategory::Research => "research",
            TaskCategory::Refactor => "refactor",
            TaskCategory::DevOps => "devops",
            TaskCategory::Config => "config",
            TaskCategory::Planning => "planning",
            TaskCategory::Review => "review",
            TaskCategory::Other => "other",
        }
    }
}

impl fmt::Display for TaskCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TaskCategory {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for cat in TaskCategory::ALL {
            if cat.as_str() == s {
                return Ok(cat);
            }
        }
        Err(())
    }
}

/// Regex patterns matched against a lower-cased user prompt. First match
/// wins; if none match, the tool-name pass decides.
fn prompt_patterns() -> &'static [(TaskCategory, &'static str)] {
    // Order matters: first match wins. Strong intent signals come first;
    // more generic ones (Refactor, Docs) are deprioritised so that e.g.
    // "add a test for the dedup logic" resolves to Testing, not Refactor.
    &[
        (
            TaskCategory::Debugging,
            r"\b(bug|fix|broken|crash|error|panic|traceback|stack\s*trace)\b",
        ),
        (
            TaskCategory::Testing,
            r"\b(test|spec|assert|coverage|flaky)",
        ),
        (
            TaskCategory::Planning,
            r"\b(plan|design|architect|roadmap|outline)",
        ),
        (TaskCategory::Review, r"\b(review|audit|lint)"),
        (
            TaskCategory::Git,
            r"\b(commit|branch|merge|rebase|pull\s*request|\bpr\b|push|cherry.?pick)",
        ),
        (
            TaskCategory::DevOps,
            r"\b(deploy|docker|kubernet|terraform|ansible|helm|ci\s*cd|pipeline)",
        ),
        (
            TaskCategory::Config,
            r"\b(config|settings|\.env|toml|yaml|ini)",
        ),
        (
            TaskCategory::Docs,
            r"\b(docs?|documentation|readme|javadoc|rustdoc)\b",
        ),
        (
            TaskCategory::FeatureDev,
            r"\b(add|implement|create|build|new)\s+(a|an|the|feature)",
        ),
        (
            TaskCategory::Refactor,
            r"\b(refactor|rename|extract|inline|simplif|cleanup|dedup)",
        ),
    ]
}

fn prompt_regex() -> &'static RegexSet {
    static SET: OnceLock<RegexSet> = OnceLock::new();
    SET.get_or_init(|| {
        RegexSet::new(prompt_patterns().iter().map(|(_, p)| *p))
            .expect("prompt_patterns must all be valid regex")
    })
}

/// Classify a single turn.
///
/// Inputs:
/// - `tool_name`: the primary tool invoked (e.g. `Edit`, `Bash`, `Read`).
/// - `all_tools`: every tool name from the content blocks of this turn.
/// - `user_prompt_hint`: optional lowercase first user message of the
///   session. Provides intent signal that tool names alone cannot.
pub fn classify(
    tool_name: Option<&str>,
    all_tools: &[String],
    user_prompt_hint: Option<&str>,
) -> TaskCategory {
    // Prompt-hint pass first: it encodes intent, which dominates tool choice
    // for ambiguous cases (e.g. Bash could be git or tests or devops).
    if let Some(prompt) = user_prompt_hint {
        let lower = prompt.to_ascii_lowercase();
        let matches: Vec<usize> = prompt_regex().matches(&lower).into_iter().collect();
        if let Some(first) = matches.first() {
            return prompt_patterns()[*first].0;
        }
    }

    // Tool-only fallback. Collect the set of tool names (primary + content).
    let mut tools: Vec<&str> = Vec::new();
    if let Some(t) = tool_name {
        tools.push(t);
    }
    for t in all_tools {
        tools.push(t.as_str());
    }

    let has = |needle: &str| tools.iter().any(|t| t.eq_ignore_ascii_case(needle));

    if has("TodoWrite") || has("ExitPlanMode") {
        return TaskCategory::Planning;
    }
    if has("Write") {
        return TaskCategory::FeatureDev;
    }
    if has("Edit") || has("MultiEdit") || has("NotebookEdit") {
        return TaskCategory::Coding;
    }
    if has("Task") {
        // Agent delegation — usually a research or coding subtask. Default
        // to Research since delegation is typically "go find out".
        return TaskCategory::Research;
    }
    if has("Read") || has("Grep") || has("Glob") || has("LS") || has("WebFetch") || has("WebSearch")
    {
        return TaskCategory::Research;
    }

    TaskCategory::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(tools: &[&str]) -> Vec<String> {
        tools.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn slug_round_trip() {
        for cat in TaskCategory::ALL {
            let s = cat.as_str();
            assert_eq!(TaskCategory::from_str(s).unwrap(), cat);
        }
    }

    #[test]
    fn tool_only_edit_classifies_as_coding() {
        assert_eq!(classify(Some("Edit"), &[], None), TaskCategory::Coding);
    }

    #[test]
    fn tool_only_write_classifies_as_feature_dev() {
        assert_eq!(classify(Some("Write"), &[], None), TaskCategory::FeatureDev);
    }

    #[test]
    fn tool_only_read_only_classifies_as_research() {
        assert_eq!(
            classify(Some("Read"), &v(&["Grep", "Glob"]), None),
            TaskCategory::Research
        );
    }

    #[test]
    fn todo_write_wins_over_other_tools() {
        assert_eq!(
            classify(Some("Edit"), &v(&["TodoWrite", "Read"]), None),
            TaskCategory::Planning
        );
    }

    #[test]
    fn task_delegation_is_research() {
        assert_eq!(classify(Some("Task"), &[], None), TaskCategory::Research);
    }

    #[test]
    fn unknown_tool_falls_back_to_other() {
        assert_eq!(
            classify(Some("SomethingWeird"), &[], None),
            TaskCategory::Other
        );
    }

    #[test]
    fn no_tools_classifies_as_other() {
        assert_eq!(classify(None, &[], None), TaskCategory::Other);
    }

    #[test]
    fn prompt_hint_debug_wins_over_tool_coding() {
        // Even though Edit would say "coding", a prompt mentioning "fix
        // the bug" should classify the session as Debugging.
        assert_eq!(
            classify(Some("Edit"), &[], Some("please fix the bug in auth")),
            TaskCategory::Debugging
        );
    }

    #[test]
    fn prompt_hint_refactor_beats_edit_default() {
        assert_eq!(
            classify(
                Some("Edit"),
                &[],
                Some("refactor the parser into smaller fns")
            ),
            TaskCategory::Refactor
        );
    }

    #[test]
    fn prompt_hint_testing_is_detected() {
        assert_eq!(
            classify(Some("Bash"), &[], Some("add a test for the dedup logic")),
            TaskCategory::Testing
        );
    }

    #[test]
    fn prompt_hint_git_is_detected() {
        assert_eq!(
            classify(
                Some("Bash"),
                &[],
                Some("merge the feature branch into main")
            ),
            TaskCategory::Git
        );
    }

    #[test]
    fn prompt_hint_docs_is_detected() {
        assert_eq!(
            classify(
                Some("Write"),
                &[],
                Some("update the README with install notes")
            ),
            TaskCategory::Docs
        );
    }

    #[test]
    fn prompt_hint_devops_is_detected() {
        assert_eq!(
            classify(Some("Bash"), &[], Some("deploy the staging docker image")),
            TaskCategory::DevOps
        );
    }

    #[test]
    fn prompt_hint_config_is_detected() {
        assert_eq!(
            classify(Some("Edit"), &[], Some("tweak the toml config for logging")),
            TaskCategory::Config
        );
    }

    #[test]
    fn prompt_hint_planning_is_detected() {
        assert_eq!(
            classify(Some("TodoWrite"), &[], Some("plan the phase-0 refactor")),
            TaskCategory::Planning
        );
    }

    #[test]
    fn prompt_hint_review_is_detected() {
        assert_eq!(
            classify(Some("Read"), &[], Some("review this PR for regressions")),
            TaskCategory::Review
        );
    }

    #[test]
    fn prompt_hint_feature_dev_is_detected() {
        assert_eq!(
            classify(
                Some("Write"),
                &[],
                Some("add a new feature to the export module")
            ),
            TaskCategory::FeatureDev
        );
    }
}
