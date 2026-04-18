//! Grade calculation for the optimize report.
//!
//! Rules (deterministic; same inputs always produce the same output):
//!
//! | Condition                              | Grade |
//! |----------------------------------------|-------|
//! | No findings at all                     |  A    |
//! | Only Low findings, count ≤ 2           |  B    |
//! | Only Low findings, count ≥ 3           |  C    |
//! | Any Medium finding (and no High)       |  D    |
//! | Any High finding                       |  F    |
//! | All other cases (shouldn't occur)      |  C    |

use super::{Finding, Severity};

/// Compute an A–F letter grade from a slice of findings.
///
/// See module-level doc for the rule table.
pub fn compute_grade(findings: &[Finding]) -> char {
    if findings.is_empty() {
        return 'A';
    }

    let has_high = findings.iter().any(|f| f.severity == Severity::High);
    if has_high {
        return 'F';
    }

    let has_medium = findings.iter().any(|f| f.severity == Severity::Medium);
    if has_medium {
        return 'D';
    }

    // All remaining findings are Low.
    if findings.len() <= 2 { 'B' } else { 'C' }
}

// ---------------------------------------------------------------------------
// Tests — every branch of the rule table
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::Finding;

    fn low(n: usize) -> Vec<Finding> {
        (0..n)
            .map(|i| Finding {
                detector: format!("det_{i}"),
                severity: Severity::Low,
                title: format!("title {i}"),
                detail: "detail".into(),
                estimated_monthly_waste_nanos: 0,
            })
            .collect()
    }

    fn medium() -> Finding {
        Finding {
            detector: "med".into(),
            severity: Severity::Medium,
            title: "medium".into(),
            detail: "detail".into(),
            estimated_monthly_waste_nanos: 1_000_000_000,
        }
    }

    fn high() -> Finding {
        Finding {
            detector: "hi".into(),
            severity: Severity::High,
            title: "high".into(),
            detail: "detail".into(),
            estimated_monthly_waste_nanos: 5_000_000_000,
        }
    }

    #[test]
    fn no_findings_is_a() {
        assert_eq!(compute_grade(&[]), 'A');
    }

    #[test]
    fn one_low_is_b() {
        assert_eq!(compute_grade(&low(1)), 'B');
    }

    #[test]
    fn two_low_is_b() {
        assert_eq!(compute_grade(&low(2)), 'B');
    }

    #[test]
    fn three_low_is_c() {
        assert_eq!(compute_grade(&low(3)), 'C');
    }

    #[test]
    fn many_low_is_c() {
        assert_eq!(compute_grade(&low(10)), 'C');
    }

    #[test]
    fn one_medium_no_high_is_d() {
        let mut findings = low(1);
        findings.push(medium());
        assert_eq!(compute_grade(&findings), 'D');
    }

    #[test]
    fn medium_alone_is_d() {
        assert_eq!(compute_grade(&[medium()]), 'D');
    }

    #[test]
    fn one_high_is_f() {
        assert_eq!(compute_grade(&[high()]), 'F');
    }

    #[test]
    fn high_plus_medium_is_f() {
        // High takes precedence over medium.
        let findings = vec![medium(), high()];
        assert_eq!(compute_grade(&findings), 'F');
    }

    #[test]
    fn high_plus_low_is_f() {
        let mut findings = low(2);
        findings.push(high());
        assert_eq!(compute_grade(&findings), 'F');
    }
}
