//! A measure-keyed count map (`{"total": N, "complexity": N, ...}`), used
//! for both the document's `totals` and every node's `violations`. Named
//! after `render::debug::violations::breakdown`, the text-render
//! equivalent, and distinct from `aggregate::tree::Counts`, the plain
//! fixed-slot value it wraps.

use serde::Serialize;
use serde::ser::SerializeMap;

use crate::feature::aggregate::tree::{Counts, MeasureLimit};

/// Serializes as `total` followed by one entry per *configured* measure —
/// never the full four-measure set — keyed by [`smell::Measure::name`], in
/// the order the run configured them. Built with `serialize_map` rather
/// than a `serde_json::Value::Object` intermediate: the latter is backed by
/// a `BTreeMap` and would sort keys alphabetically, losing the measure
/// order that makes this diffable across runs.
pub struct Breakdown<'a> {
    pub counts: Counts,
    pub measures: &'a [MeasureLimit],
}

impl Serialize for Breakdown<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.measures.len() + 1))?;
        map.serialize_entry("total", &self.counts.total())?;
        for limit in self.measures {
            map.serialize_entry(limit.measure.name(), &self.counts.get(limit.measure))?;
        }
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use smell::Measure;

    use super::*;
    use crate::testing::{counts, measure_limits};

    fn to_string(breakdown: &Breakdown) -> String {
        serde_json::to_string(breakdown).expect("breakdown serializes")
    }

    #[test]
    fn breakdown_puts_total_first_then_configured_measures() {
        let measures = measure_limits(&[(Measure::Complexity, 10), (Measure::Lines, 100)]);
        let breakdown = Breakdown {
            counts: counts(&[(Measure::Complexity, 2), (Measure::Lines, 1)]),
            measures: &measures,
        };
        assert_eq!(
            to_string(&breakdown),
            r#"{"total":3,"complexity":2,"lines":1}"#
        );
    }

    #[test]
    fn breakdown_omits_unconfigured_measures() {
        let measures = measure_limits(&[(Measure::Complexity, 10)]);
        let breakdown = Breakdown {
            counts: counts(&[(Measure::Complexity, 2)]),
            measures: &measures,
        };
        assert!(!to_string(&breakdown).contains("lines"));
    }

    #[test]
    fn breakdown_of_no_measures_is_total_only() {
        let breakdown = Breakdown {
            counts: Counts::default(),
            measures: &[],
        };
        assert_eq!(to_string(&breakdown), r#"{"total":0}"#);
    }
}
