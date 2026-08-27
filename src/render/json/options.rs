//! The `options` echo: what a run was configured with. See
//! [`crate::feature::scope::options::Settings`] for the asymmetry between
//! where the glob filters and the limits each come from.

use serde::Serialize;

use crate::feature::scope::options::Settings;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Options<'a> {
    pub rule: &'a str,
    pub include: &'a [String],
    pub exclude: &'a [String],
    pub branches: &'a [String],
    pub implements: &'a [String],
    pub max_complexity: Option<usize>,
    pub max_methods: Option<usize>,
    pub max_lines: Option<usize>,
    pub max_declarations: Option<usize>,
}

impl<'a> Options<'a> {
    pub fn new(settings: &'a Settings) -> Options<'a> {
        Options {
            rule: &settings.rule,
            include: &settings.include,
            exclude: &settings.exclude,
            branches: &settings.branches,
            implements: &settings.implements,
            max_complexity: settings.max_complexity,
            max_methods: settings.max_methods,
            max_lines: settings.max_lines,
            max_declarations: settings.max_declarations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Settings {
        Settings {
            rule: "default".to_string(),
            include: vec!["*.rs".to_string()],
            exclude: vec!["**/target/**".to_string()],
            branches: vec!["switch".to_string()],
            implements: vec!["Describe".to_string()],
            max_complexity: Some(10),
            max_methods: None,
            max_lines: Some(300),
            max_declarations: None,
        }
    }

    #[test]
    fn options_uses_camel_case_limit_keys() {
        let value = serde_json::to_value(Options::new(&settings())).expect("serializes");
        assert!(value.get("maxComplexity").is_some());
        assert!(value.get("maxLines").is_some());
        assert!(value.get("max_complexity").is_none());
    }

    #[test]
    fn options_echoes_null_for_unconfigured_limits() {
        let value = serde_json::to_value(Options::new(&settings())).expect("serializes");
        assert_eq!(value["maxMethods"], serde_json::Value::Null);
        assert_eq!(value["maxDeclarations"], serde_json::Value::Null);
    }

    #[test]
    fn options_echoes_globs_and_rule() {
        let value = serde_json::to_value(Options::new(&settings())).expect("serializes");
        assert_eq!(value["rule"], "default");
        assert_eq!(value["include"][0], "*.rs");
        assert_eq!(value["exclude"][0], "**/target/**");
        assert_eq!(value["branches"][0], "switch");
        assert_eq!(value["implements"][0], "Describe");
    }
}
