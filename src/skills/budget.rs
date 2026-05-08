use serde::Serialize;

/// Context window sizes paired with model labels used in the default budget table.
pub const DEFAULT_CONTEXT_SIZES: &[(&str, usize)] = &[
    ("Sonnet 4.6 (200K)", 200_000),
    ("Sonnet 4.6 / Opus 4.7 (1M)", 1_000_000),
];

#[derive(Debug, Clone, Serialize)]
pub struct BudgetRow {
    pub model_label: &'static str,
    pub context_size: usize,
    pub fraction: f64,
    pub budget_tokens: usize,
    pub used_tokens: usize,
    /// Positive = headroom remaining; negative = over budget.
    pub headroom_tokens: i64,
    pub simulated_drop_count: usize,
    /// Up to 10 skill names that would be dropped (alphabetical simulation order).
    pub simulated_drop_order: Vec<String>,
}

/// Compute budget rows for the given per-skill token counts.
///
/// `fraction` is `skillListingBudgetFraction` (e.g. `0.01` = 1 %).
/// `context_sizes` is typically [`DEFAULT_CONTEXT_SIZES`].
///
/// Skills are sorted alphabetically to simulate the drop priority because we
/// have no access to real invocation frequency or recency data.  The output
/// documents this as a simulation.
pub fn compute_budget(
    skill_names: &[&str],
    skill_tokens: &[usize],
    fraction: f64,
    context_sizes: &[(&'static str, usize)],
) -> Vec<BudgetRow> {
    debug_assert_eq!(skill_names.len(), skill_tokens.len());
    let used: usize = skill_tokens.iter().sum();

    context_sizes
        .iter()
        .map(|(label, ctx)| {
            let ctx = *ctx;
            let budget = ((ctx as f64 * fraction) as usize).max(1);
            let headroom = budget as i64 - used as i64;

            let (drop_count, drop_order) = if headroom < 0 {
                // Sort alphabetically (as a proxy for least-recently-used).
                let mut indexed: Vec<(&&str, &usize)> =
                    skill_names.iter().zip(skill_tokens.iter()).collect();
                indexed.sort_by_key(|(name, _)| **name);

                let mut remaining = used;
                let mut dropped: Vec<String> = Vec::new();
                for (name, tokens) in &indexed {
                    if remaining <= budget {
                        break;
                    }
                    remaining = remaining.saturating_sub(**tokens);
                    dropped.push((*name).to_string());
                }
                let count = dropped.len();
                dropped.truncate(10);
                (count, dropped)
            } else {
                (0, vec![])
            };

            BudgetRow {
                model_label: label,
                context_size: ctx,
                fraction,
                budget_tokens: budget,
                used_tokens: used,
                headroom_tokens: headroom,
                simulated_drop_count: drop_count,
                simulated_drop_order: drop_order,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SIZES: &[(&str, usize)] = &[
        ("Sonnet 200K", 200_000),
        ("Sonnet 1M", 1_000_000),
    ];

    #[test]
    fn no_drop_when_within_budget() {
        // 10 skills × 100 tokens = 1 000; 1% of 200K = 2 000 → headroom = 1 000
        let names: Vec<&str> = (0..10).map(|_| "skill").collect();
        let tokens: Vec<usize> = vec![100; 10];
        let rows = compute_budget(&names, &tokens, 0.01, SIZES);
        let row_200k = &rows[0];
        assert_eq!(row_200k.used_tokens, 1_000);
        assert_eq!(row_200k.budget_tokens, 2_000);
        assert_eq!(row_200k.headroom_tokens, 1_000);
        assert_eq!(row_200k.simulated_drop_count, 0);
        assert!(row_200k.simulated_drop_order.is_empty());
    }

    #[test]
    fn drops_skills_when_over_budget() {
        // 100 skills × 100 tokens = 10 000; 1% of 200K = 2 000 → need to drop 80
        let names: Vec<&str> = (0..100).map(|_| "s").collect();
        let tokens: Vec<usize> = vec![100; 100];
        let rows = compute_budget(&names, &tokens, 0.01, SIZES);
        let row_200k = &rows[0];
        assert_eq!(row_200k.simulated_drop_count, 80);
        // drop_order capped at 10
        assert_eq!(row_200k.simulated_drop_order.len(), 10);
    }

    #[test]
    fn no_drop_on_large_context() {
        // 100 skills × 100 tokens = 10 000; 1% of 1M = 10 000 → headroom = 0
        let names: Vec<&str> = (0..100).map(|_| "x").collect();
        let tokens: Vec<usize> = vec![100; 100];
        let rows = compute_budget(&names, &tokens, 0.01, SIZES);
        let row_1m = &rows[1];
        assert_eq!(row_1m.simulated_drop_count, 0);
        assert_eq!(row_1m.headroom_tokens, 0);
    }

    #[test]
    fn negative_headroom_when_over_budget() {
        // 1 skill × 5 000 tokens; 1% of 200K = 2 000 → headroom = -3 000
        let names = ["big-skill"];
        let tokens = [5_000usize];
        let rows = compute_budget(&names, &tokens, 0.01, &[("Sonnet 200K", 200_000)]);
        assert_eq!(rows[0].headroom_tokens, -3_000);
    }
}
