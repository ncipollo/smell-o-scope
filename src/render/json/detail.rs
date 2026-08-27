//! A file's per-measure check-failure detail: named offenders for
//! `complexity`/`methods`, or the file's own value for `lines`/
//! `declarations`. Typed per measure rather than uniformly, since HTML
//! wants the whole-file number directly rather than unwrapping a
//! one-element array.

use serde::Serialize;
use serde::ser::SerializeMap;
use smell::Measure;

use crate::feature::aggregate::tree::{self, Detail as TreeDetail, Finding, MeasureLimit};

/// One entry per *configured* measure, keyed by [`Measure::name`] in the
/// order the run configured them: an array of [`Offender`]s for an
/// entries-shaped measure (`[]` when this file has none), or the failing
/// value for a whole-file measure (`null` when this file didn't fail it).
/// A measure with no [`Finding`] for this file still knows which empty
/// form to emit, via [`tree::shape`].
pub struct Detail<'a> {
    pub findings: &'a [Finding],
    pub measures: &'a [MeasureLimit],
}

#[derive(Serialize)]
pub struct Offender {
    pub name: String,
    pub value: usize,
}

impl Offender {
    fn new(offender: &tree::Offender) -> Offender {
        Offender {
            name: offender.name.clone(),
            value: offender.value,
        }
    }
}

impl Serialize for Detail<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.measures.len()))?;
        for limit in self.measures {
            entry(&mut map, limit.measure, self.findings)?;
        }
        map.end()
    }
}

fn entry<M>(map: &mut M, measure: Measure, findings: &[Finding]) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    let found = findings.iter().find(|finding| finding.measure == measure);
    match found {
        Some(finding) => entry_for_finding(map, measure, &finding.detail),
        None => entry_for_absence(map, measure),
    }
}

fn entry_for_finding<M>(map: &mut M, measure: Measure, detail: &TreeDetail) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    match detail {
        TreeDetail::Entries(offenders) => {
            let offenders: Vec<Offender> = offenders.iter().map(Offender::new).collect();
            map.serialize_entry(measure.name(), &offenders)
        }
        TreeDetail::Whole(value) => map.serialize_entry(measure.name(), &Some(*value)),
    }
}

fn entry_for_absence<M>(map: &mut M, measure: Measure) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    match tree::shape(measure) {
        tree::Shape::Entries => map.serialize_entry(measure.name(), &Vec::<Offender>::new()),
        tree::Shape::Whole => map.serialize_entry(measure.name(), &None::<usize>),
    }
}

#[cfg(test)]
mod tests {
    use smell::Measure;

    use super::*;
    use crate::feature::aggregate::tree::Offender as TreeOffender;
    use crate::testing::measure_limits;

    fn to_string(detail: &Detail) -> String {
        serde_json::to_string(detail).expect("detail serializes")
    }

    #[test]
    fn detail_lists_offenders_for_an_entries_measure() {
        let measures = measure_limits(&[(Measure::Complexity, 10)]);
        let findings = vec![Finding {
            measure: Measure::Complexity,
            detail: TreeDetail::Entries(vec![TreeOffender {
                name: "f".to_string(),
                value: 12,
            }]),
        }];
        let detail = Detail {
            findings: &findings,
            measures: &measures,
        };
        assert_eq!(
            to_string(&detail),
            r#"{"complexity":[{"name":"f","value":12}]}"#
        );
    }

    #[test]
    fn detail_is_an_empty_array_for_an_entries_measure_without_a_finding() {
        let measures = measure_limits(&[(Measure::Methods, 5)]);
        let detail = Detail {
            findings: &[],
            measures: &measures,
        };
        assert_eq!(to_string(&detail), r#"{"methods":[]}"#);
    }

    #[test]
    fn detail_is_the_value_for_a_whole_file_measure() {
        let measures = measure_limits(&[(Measure::Lines, 100)]);
        let findings = vec![Finding {
            measure: Measure::Lines,
            detail: TreeDetail::Whole(150),
        }];
        let detail = Detail {
            findings: &findings,
            measures: &measures,
        };
        assert_eq!(to_string(&detail), r#"{"lines":150}"#);
    }

    #[test]
    fn detail_is_null_for_a_whole_file_measure_without_a_finding() {
        let measures = measure_limits(&[(Measure::Declarations, 5)]);
        let detail = Detail {
            findings: &[],
            measures: &measures,
        };
        assert_eq!(to_string(&detail), r#"{"declarations":null}"#);
    }

    #[test]
    fn detail_keys_follow_configured_measure_order() {
        let measures = measure_limits(&[(Measure::Lines, 100), (Measure::Complexity, 10)]);
        let detail = Detail {
            findings: &[],
            measures: &measures,
        };
        assert_eq!(to_string(&detail), r#"{"lines":null,"complexity":[]}"#);
    }
}
