//! Aggregates smell's unaggregated per-file check results up the directory
//! tree. `violations` (a plain count of findings) is the only mode today;
//! adding another means a new `Mode` variant, a new match arm here, and a
//! new module — `violations` itself is never touched.
//!
//! Note on overlapping roots: `smell::TreeAnalysis` deliberately preserves
//! provenance, so the same file can appear under two different roots and
//! be counted in both. That's correct per-root behavior; the whole-run
//! total, [`tree::AggregatedTree::totals`], dedups by path the way
//! `TreeAnalysis::reports` does, so an overlap doesn't inflate it.

use smell::{CheckResult, TreeAnalysis};

pub mod failures;
pub mod tree;
pub mod violations;

pub use tree::AggregatedTree;
use violations::Violations;

/// Aggregates a `TreeAnalysis` and its `CheckResult`s into an
/// [`AggregatedTree`]. Each aggregation strategy is its own type
/// implementing this trait, selected by [`Mode`].
pub trait Aggregator {
    fn aggregate(&self, tree: &TreeAnalysis, checks: &[CheckResult]) -> AggregatedTree;
}

/// The aggregation strategies a scan can run. A `--mode` flag to select one
/// is deferred to a later issue; today this only ever resolves to the
/// default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Violations,
}

impl Mode {
    /// A stable, lowercase identifier safe to store or match on, mirroring
    /// `smell::Measure::name()`.
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Violations => "violations",
        }
    }
}

/// Runs `tree`/`checks` through the aggregator `mode` selects.
pub fn aggregate(mode: Mode, tree: &TreeAnalysis, checks: &[CheckResult]) -> AggregatedTree {
    match mode {
        Mode::Violations => Violations.aggregate(tree, checks),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::tree;

    struct Stub;

    impl Aggregator for Stub {
        fn aggregate(&self, _tree: &TreeAnalysis, _checks: &[CheckResult]) -> AggregatedTree {
            AggregatedTree::default()
        }
    }

    #[test]
    fn a_second_mode_only_needs_the_aggregator_trait() {
        let aggregators: Vec<Box<dyn Aggregator>> = vec![Box::new(Violations), Box::new(Stub)];
        let empty = tree(vec![]);
        for aggregator in aggregators {
            aggregator.aggregate(&empty, &[]);
        }
    }

    #[test]
    fn an_aggregated_tree_is_constructible_without_violations_data() {
        let aggregated = AggregatedTree::default();
        assert!(aggregated.roots.is_empty());
        assert!(aggregated.measures.is_empty());
    }

    #[test]
    fn mode_defaults_to_violations() {
        assert_eq!(Mode::default(), Mode::Violations);
    }

    #[test]
    fn mode_name_is_violations() {
        assert_eq!(Mode::Violations.name(), "violations");
    }

    #[test]
    fn aggregate_dispatches_the_default_mode_to_violations() {
        let empty = tree(vec![]);
        assert_eq!(
            aggregate(Mode::default(), &empty, &[]),
            Violations.aggregate(&empty, &[])
        );
    }
}
