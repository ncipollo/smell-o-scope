//! Escapes a JSON document for embedding inside an HTML `<script>` block.
//!
//! `<` never appears in JSON's own structural syntax (`{}[]:,"` and its bare
//! literals) — every `<` in a rendered document is already inside a string,
//! where `\u003c` is a valid escape `JSON.parse` decodes straight back to
//! `<`. A blind, whole-document replace therefore round-trips exactly, and
//! it neutralizes every hazardous sequence at once (`</script`, `<script`,
//! `<!--`) rather than just `</` as the source data might contain any of them.

/// Escapes every `<` in `json` so it can't close the surrounding `<script>`
/// tag or otherwise confuse the HTML tokenizer.
pub fn escape(json: &str) -> String {
    json.replace('<', "\\u003c")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_replaces_every_angle_bracket() {
        assert_eq!(escape("</script><!--"), "\\u003c/script>\\u003c!--");
    }

    #[test]
    fn escape_leaves_other_text_alone() {
        assert_eq!(escape(r#"{"name":"a.rs"}"#), r#"{"name":"a.rs"}"#);
    }

    #[test]
    fn escaped_json_still_parses() {
        let json = serde_json::to_string(&serde_json::json!({"name": "</script>evil"}))
            .expect("serializes");
        let escaped = escape(&json);
        assert!(!escaped.contains("</script>"));
        let value: serde_json::Value = serde_json::from_str(&escaped).expect("still valid JSON");
        assert_eq!(value["name"], "</script>evil");
    }
}
