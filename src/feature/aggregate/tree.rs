//! Mirrors `smell`'s `TreeNode` shape, attaching per-node violation counts
//! (per-measure + total) and, for files, the check-failure detail that
//! produced them.
//!
//! These types are owned by this crate rather than reusing `smell::Subject`/
//! `Offender` directly: those derive only `Debug + Clone`, so reusing them
//! would keep `AggregatedTree` — the render input model every future
//! JSON/HTML output builds on — from ever deriving `PartialEq` or
//! `Serialize`.

use std::path::{Path, PathBuf};

use smell::Measure;

/// Every measure `smell` can check, in the order `smell::check` runs them.
/// [`Counts`] is a fixed 4-slot array keyed by this list rather than a
/// `HashMap<Measure, usize>`: `Measure` has no `Hash`/`Ord` impl, and there
/// are only ever four of them.
pub const MEASURES: [Measure; 4] = [
    Measure::Complexity,
    Measure::Methods,
    Measure::Lines,
    Measure::Declarations,
];

/// Per-measure violation counts for one node, plus their [`Counts::total`].
/// [`slot`] is an exhaustive match on `Measure`, so a fifth `smell` measure
/// fails this crate to compile rather than silently going uncounted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts([usize; 4]);

impl Counts {
    pub fn get(&self, measure: Measure) -> usize {
        self.0[slot(measure)]
    }

    pub fn total(&self) -> usize {
        self.0.iter().sum()
    }

    pub fn increment(&mut self, measure: Measure, by: usize) {
        self.0[slot(measure)] += by;
    }

    pub fn merge(&mut self, other: Counts) {
        for (mine, theirs) in self.0.iter_mut().zip(other.0) {
            *mine += theirs;
        }
    }
}

fn slot(measure: Measure) -> usize {
    match measure {
        Measure::Complexity => 0,
        Measure::Methods => 1,
        Measure::Lines => 2,
        Measure::Declarations => 3,
    }
}

/// A measure a run had a configured limit for, and what that limit was.
/// Lives on [`AggregatedTree`] rather than per-node: it's a property of the
/// run, not of any one file or folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasureLimit {
    pub measure: Measure,
    pub limit: usize,
}

/// One named entry inside a file that exceeded its measure's limit — a
/// function over `max_complexity`, a type over `max_methods`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offender {
    pub name: String,
    pub value: usize,
}

/// What exceeded a measure's limit in a file: named entries inside it, or
/// the file as a whole (over `max_lines` / `max_declarations`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detail {
    Entries(Vec<Offender>),
    Whole(usize),
}

impl Detail {
    /// One violation per named entry; the whole-file case is one violation
    /// regardless of its value, since there's nothing to enumerate.
    pub fn count(&self) -> usize {
        match self {
            Detail::Entries(offenders) => offenders.len(),
            Detail::Whole(_) => 1,
        }
    }
}

/// One measure's failure detail for a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub measure: Measure,
    pub detail: Detail,
}

/// The result of running an [`crate::feature::aggregate::Aggregator`] over a
/// `TreeAnalysis`: the measures the run configured, and one aggregated node
/// per `smell` root.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AggregatedTree {
    pub measures: Vec<MeasureLimit>,
    pub roots: Vec<AggregatedNode>,
}

/// Mirrors `smell::TreeNode`: a directory or a file, each carrying its own
/// aggregated [`Counts`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregatedNode {
    Directory(AggregatedDirectory),
    File(AggregatedFile),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregatedDirectory {
    pub path: PathBuf,
    pub counts: Counts,
    pub children: Vec<AggregatedNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregatedFile {
    pub path: PathBuf,
    pub counts: Counts,
    pub findings: Vec<Finding>,
}

impl AggregatedNode {
    pub fn path(&self) -> &Path {
        match self {
            AggregatedNode::Directory(directory) => &directory.path,
            AggregatedNode::File(file) => &file.path,
        }
    }

    pub fn counts(&self) -> Counts {
        match self {
            AggregatedNode::Directory(directory) => directory.counts,
            AggregatedNode::File(file) => file.counts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_increment_accumulates_per_measure() {
        let mut counts = Counts::default();
        counts.increment(Measure::Complexity, 2);
        counts.increment(Measure::Complexity, 3);
        assert_eq!(counts.get(Measure::Complexity), 5);
    }

    #[test]
    fn counts_get_returns_zero_for_an_unrecorded_measure() {
        let counts = Counts::default();
        assert_eq!(counts.get(Measure::Lines), 0);
    }

    #[test]
    fn counts_total_sums_every_measure() {
        let mut counts = Counts::default();
        counts.increment(Measure::Complexity, 2);
        counts.increment(Measure::Lines, 1);
        assert_eq!(counts.total(), 3);
    }

    #[test]
    fn counts_merge_adds_each_measure() {
        let mut a = Counts::default();
        a.increment(Measure::Complexity, 2);
        let mut b = Counts::default();
        b.increment(Measure::Complexity, 1);
        b.increment(Measure::Methods, 4);
        a.merge(b);
        assert_eq!(a.get(Measure::Complexity), 3);
        assert_eq!(a.get(Measure::Methods), 4);
    }

    #[test]
    fn detail_counts_each_entry_as_one_violation() {
        let detail = Detail::Entries(vec![
            Offender {
                name: "a".to_string(),
                value: 1,
            },
            Offender {
                name: "b".to_string(),
                value: 2,
            },
        ]);
        assert_eq!(detail.count(), 2);
    }

    #[test]
    fn detail_counts_a_whole_file_subject_as_one_violation() {
        assert_eq!(Detail::Whole(150).count(), 1);
    }

    #[test]
    fn node_counts_and_path_delegate_to_each_variant() {
        let mut counts = Counts::default();
        counts.increment(Measure::Lines, 1);

        let file = AggregatedNode::File(AggregatedFile {
            path: PathBuf::from("src/a.rs"),
            counts,
            findings: vec![],
        });
        assert_eq!(file.path(), Path::new("src/a.rs"));
        assert_eq!(file.counts().get(Measure::Lines), 1);

        let directory = AggregatedNode::Directory(AggregatedDirectory {
            path: PathBuf::from("src"),
            counts,
            children: vec![],
        });
        assert_eq!(directory.path(), Path::new("src"));
        assert_eq!(directory.counts().total(), 1);
    }
}
