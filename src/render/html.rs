//! `--format html` rendering: a single self-contained document — embedded
//! data, CSS, and JS, opening from `file://` with zero network requests.
//! The script is one shared closure split across `assets/script/*.js` for
//! file-size reasons; [`script`] reassembles it before embedding. Together
//! these build three ways to navigate the same embedded document: a
//! directory tree, a heatmap treemap (switchable via a toggle), and a
//! search box that jumps to a node in whichever view is active.

pub mod data;

use crate::feature::scope::Scan;
use crate::render::json;

const TEMPLATE: &str = include_str!("../../assets/template.html");
const STYLE: &str = include_str!("../../assets/style.css");
const SCRIPT_SHARED: &str = include_str!("../../assets/script/shared.js");
const SCRIPT_SHELL: &str = include_str!("../../assets/script/shell.js");
const SCRIPT_HEATMAP: &str = include_str!("../../assets/script/heatmap.js");
const SCRIPT_SEARCH: &str = include_str!("../../assets/script/search.js");
const SCRIPT_TOOLTIP: &str = include_str!("../../assets/script/tooltip.js");

/// Renders `scan` as the self-contained HTML document: `TEMPLATE` with its
/// style, script, and data placeholders filled in. Data is substituted
/// *last* — style and script must already be in place, so a path or
/// offender name that happens to contain the literal text `{{script}}`
/// lands as inert data rather than expanding into a second script body.
pub fn render(scan: &Scan) -> String {
    let document = TEMPLATE.replacen("{{style}}", STYLE, 1);
    let document = document.replacen("{{script}}", &script(), 1);
    document.replacen("{{data}}", &data::escape(&json::render_compact(scan)), 1)
}

/// Reassembles the one JS closure split across `assets/script/*.js`, in the
/// same order the original single file had them in — every function inside
/// is hoisted within the shared IIFE regardless of which part it came from,
/// so this reproduces that file's behavior exactly.
fn script() -> String {
    [
        SCRIPT_SHARED,
        SCRIPT_SHELL,
        SCRIPT_HEATMAP,
        SCRIPT_SEARCH,
        SCRIPT_TOOLTIP,
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use smell::{Measure, PathError};

    use super::*;
    use crate::feature::aggregate::{Mode, tree::AggregatedTree};
    use crate::feature::scope::options::Settings;
    use crate::testing::{aggregated_directory, aggregated_file, counts, measure_limits};

    fn settings() -> Settings {
        Settings {
            rule: "default".to_string(),
            include: vec![],
            exclude: vec![],
            branches: vec![],
            implements: vec![],
            max_complexity: Some(10),
            max_methods: None,
            max_lines: None,
            max_declarations: None,
        }
    }

    fn scan<'a>(tree: &'a AggregatedTree, errors: &'a [PathError]) -> Scan<'a> {
        Scan {
            mode: Mode::default(),
            settings: settings(),
            tree,
            errors,
        }
    }

    /// Pulls the text between the `smell-data` script tags out of a
    /// rendered document, mirroring how `app.js` reads it via `textContent`.
    fn extract_data(html: &str) -> &str {
        const OPEN: &str = "id=\"smell-data\">";
        let start = html.find(OPEN).expect("data script tag present") + OPEN.len();
        let rest = &html[start..];
        let end = rest.find("</script>").expect("closing script tag present");
        &rest[..end]
    }

    #[test]
    fn render_embeds_a_parsable_document() {
        let tree = AggregatedTree {
            measures: measure_limits(&[(Measure::Complexity, 10)]),
            roots: vec![aggregated_file("a.rs", Some(1), counts(&[]), vec![])],
        };
        let html = render(&scan(&tree, &[]));
        let value: serde_json::Value =
            serde_json::from_str(extract_data(&html)).expect("valid JSON");
        assert_eq!(value["version"], 1);
    }

    #[test]
    fn render_escapes_angle_brackets_in_the_data_block() {
        let tree = AggregatedTree {
            measures: measure_limits(&[]),
            roots: vec![aggregated_file(
                "</script>evil.rs",
                None,
                counts(&[]),
                vec![],
            )],
        };
        let html = render(&scan(&tree, &[]));
        assert!(!extract_data(&html).contains("</script>"));
        let value: serde_json::Value =
            serde_json::from_str(extract_data(&html)).expect("still valid JSON");
        assert_eq!(value["roots"][0]["path"], "</script>evil.rs");
    }

    #[test]
    fn render_makes_no_network_requests() {
        let html = render(&scan(&AggregatedTree::default(), &[]));
        for needle in [
            "http://",
            "https://",
            "//cdn",
            "<script src",
            "<link ",
            "@import",
            "url(",
        ] {
            assert!(
                !html.contains(needle),
                "unexpected network-capable content: {needle}"
            );
        }
    }

    #[test]
    fn render_inlines_the_style_and_script() {
        let html = render(&scan(&AggregatedTree::default(), &[]));
        assert!(html.contains("<style>:root"));
        assert!(html.contains("(function"));
        assert!(!html.contains("{{style}}"));
        assert!(!html.contains("{{script}}"));
        assert!(!html.contains("{{data}}"));
    }

    #[test]
    fn render_substitutes_data_last() {
        let tree = AggregatedTree {
            measures: measure_limits(&[]),
            roots: vec![aggregated_file("{{script}}.rs", None, counts(&[]), vec![])],
        };
        let html = render(&scan(&tree, &[]));
        assert_eq!(html.matches("<script>").count(), 1);
        assert!(extract_data(&html).contains("{{script}}.rs"));
    }

    #[test]
    fn script_never_writes_markup_directly() {
        let script = script();
        for needle in [
            "innerHTML",
            "outerHTML",
            "insertAdjacentHTML",
            "document.write",
        ] {
            assert!(
                !script.contains(needle),
                "app.js must build the DOM via textContent/createElement, found {needle}"
            );
        }
    }

    #[test]
    fn script_is_one_closed_iife() {
        let script = script();
        assert!(script.trim_start().starts_with("(function ()"));
        assert!(script.trim_end().ends_with("})();"));
    }

    #[test]
    fn script_assembles_every_part() {
        let script = script();
        for part in [
            SCRIPT_SHARED,
            SCRIPT_SHELL,
            SCRIPT_HEATMAP,
            SCRIPT_SEARCH,
            SCRIPT_TOOLTIP,
        ] {
            assert!(script.contains(part), "script part missing from the join");
        }
    }

    #[test]
    fn render_includes_a_labeled_search_input() {
        let tree = AggregatedTree {
            measures: measure_limits(&[]),
            roots: vec![aggregated_file("a.rs", Some(1), counts(&[]), vec![])],
        };
        let html = render(&scan(&tree, &[]));
        assert!(html.contains("smell-search"));
        assert!(html.contains("Search files and folders"));
        assert!(html.contains("combobox"));
        assert!(html.contains("listbox"));
    }

    #[test]
    fn render_includes_every_root() {
        let tree = AggregatedTree {
            measures: measure_limits(&[]),
            roots: vec![
                aggregated_directory("src", counts(&[]), vec![]),
                aggregated_directory("fixtures", counts(&[]), vec![]),
            ],
        };
        let html = render(&scan(&tree, &[]));
        let data = extract_data(&html);
        assert!(data.contains("\"src\""));
        assert!(data.contains("\"fixtures\""));
    }
}
