//! The flat, self-contained input a renderer needs for one scan — no
//! `TreeAnalysis` walk required alongside it.

use smell::PathError;

use crate::feature::aggregate::{AggregatedTree, Mode};
use crate::feature::scope::options::Settings;

/// Everything a document needs about one scan: how it was aggregated, what
/// it was configured with, the aggregated tree, and any path-level errors.
/// Borrows rather than owns since rendering happens while these are still
/// alive in `scope::run_in`.
pub struct Scan<'a> {
    pub mode: Mode,
    pub settings: Settings,
    pub tree: &'a AggregatedTree,
    pub errors: &'a [PathError],
}
