//! The translation boundary from `smell`'s check-failure vocabulary into
//! this crate's [`Finding`]/[`Detail`] shape. This is the only module that
//! reads `smell::Subject` — every aggregator builds on this index rather
//! than matching on `Subject` itself.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use smell::{CheckResult, Subject};

use crate::feature::aggregate::tree::{Detail, Finding, MeasureLimit, Offender};

/// Every file's [`Finding`]s, indexed by path.
pub struct FailureIndex {
    by_path: HashMap<PathBuf, Vec<Finding>>,
}

impl FailureIndex {
    /// Indexes every `CheckFailure` across every measure by the path it
    /// failed on, translating each into a [`Finding`].
    pub fn build(checks: &[CheckResult]) -> FailureIndex {
        let mut by_path: HashMap<PathBuf, Vec<Finding>> = HashMap::new();
        for check in checks {
            for failure in &check.failures {
                by_path
                    .entry(failure.path.clone())
                    .or_default()
                    .push(Finding {
                        measure: check.measure,
                        detail: detail(&failure.subject),
                    });
            }
        }
        FailureIndex { by_path }
    }

    /// The findings recorded for `path`, or an empty slice when it had none.
    pub fn findings(&self, path: &Path) -> &[Finding] {
        self.by_path.get(path).map_or(&[], Vec::as_slice)
    }
}

/// The measures a run had a configured limit for, in `smell`'s flag order.
/// A measure that ran and passed (empty `failures`) is still included —
/// that's what distinguishes "ran and passed" from "never configured".
pub fn limits(checks: &[CheckResult]) -> Vec<MeasureLimit> {
    checks
        .iter()
        .map(|check| MeasureLimit {
            measure: check.measure,
            limit: check.limit,
        })
        .collect()
}

/// The only place `smell::Subject` is matched: `Entries` keeps its named
/// offenders, `File` becomes a [`Detail::Whole`] — never
/// `subject.entries().len()`, which is `0` for a `File` subject.
fn detail(subject: &Subject) -> Detail {
    match subject {
        Subject::Entries(offenders) => Detail::Entries(
            offenders
                .iter()
                .map(|offender| Offender {
                    name: offender.name.clone(),
                    value: offender.value,
                })
                .collect(),
        ),
        Subject::File(value) => Detail::Whole(*value),
    }
}

#[cfg(test)]
mod tests {
    use smell::Measure;

    use super::*;
    use crate::testing::{check_result, entries_failure, file_failure};

    #[test]
    fn index_groups_findings_by_path() {
        let checks = vec![check_result(
            Measure::Complexity,
            10,
            vec![
                entries_failure("a.rs", &[("f", 12)]),
                entries_failure("b.rs", &[("g", 15)]),
            ],
        )];
        let index = FailureIndex::build(&checks);
        assert_eq!(index.findings(Path::new("a.rs")).len(), 1);
        assert_eq!(index.findings(Path::new("b.rs")).len(), 1);
    }

    #[test]
    fn index_keeps_a_finding_per_measure_for_one_path() {
        let checks = vec![
            check_result(
                Measure::Complexity,
                10,
                vec![entries_failure("a.rs", &[("f", 12)])],
            ),
            check_result(Measure::Lines, 100, vec![file_failure("a.rs", 150)]),
        ];
        let index = FailureIndex::build(&checks);
        let findings = index.findings(Path::new("a.rs"));
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].measure, Measure::Complexity);
        assert_eq!(findings[1].measure, Measure::Lines);
    }

    #[test]
    fn index_returns_nothing_for_an_unknown_path() {
        let index = FailureIndex::build(&[]);
        assert!(index.findings(Path::new("missing.rs")).is_empty());
    }

    #[test]
    fn index_maps_a_file_subject_to_a_whole_detail() {
        let checks = vec![check_result(
            Measure::Lines,
            100,
            vec![file_failure("a.rs", 150)],
        )];
        let index = FailureIndex::build(&checks);
        assert_eq!(
            index.findings(Path::new("a.rs"))[0].detail,
            Detail::Whole(150)
        );
    }

    #[test]
    fn limits_lists_configured_measures_in_check_order() {
        let checks = vec![
            check_result(Measure::Complexity, 10, vec![]),
            check_result(Measure::Lines, 100, vec![]),
        ];
        assert_eq!(
            limits(&checks),
            vec![
                MeasureLimit {
                    measure: Measure::Complexity,
                    limit: 10,
                },
                MeasureLimit {
                    measure: Measure::Lines,
                    limit: 100,
                },
            ]
        );
    }

    #[test]
    fn limits_includes_a_measure_that_ran_and_passed() {
        let checks = vec![check_result(Measure::Complexity, 10, vec![])];
        assert_eq!(limits(&checks).len(), 1);
    }

    /// Guards against `smell` ever changing a measure's subject kind
    /// without `tree::shape` following: every `Detail` this module builds
    /// must have the shape `tree::shape` predicts for its measure.
    #[test]
    fn finding_shape_matches_its_measure() {
        use crate::feature::aggregate::tree;

        let entries = detail(&Subject::Entries(vec![]));
        assert_eq!(entries.shape(), tree::shape(Measure::Complexity));
        assert_eq!(entries.shape(), tree::shape(Measure::Methods));

        let whole = detail(&Subject::File(0));
        assert_eq!(whole.shape(), tree::shape(Measure::Lines));
        assert_eq!(whole.shape(), tree::shape(Measure::Declarations));
    }
}
