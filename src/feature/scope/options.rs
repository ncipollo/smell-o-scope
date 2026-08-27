use smell::{AnalysisOptions, Overrides};

use crate::feature::scope::request::Request;

/// Globs excluded from every analysis unless the user overrides them with
/// their own `--exclude`. Each ends in `/**` so smell prunes the whole
/// subtree during traversal instead of visiting every file inside it.
pub const DEFAULT_EXCLUDES: [&str; 5] = [
    "**/.*/**",
    "**/node_modules/**",
    "**/target/**",
    "**/build/**",
    "**/dist/**",
];

/// Maps a [`Request`] onto the [`Overrides`] smell resolves against.
pub fn overrides(request: &Request) -> Overrides {
    Overrides {
        include: request.include.clone(),
        exclude: excludes(&request.exclude),
        branches: request.branches.clone(),
        implements: request.implements.clone(),
        max_complexity: request.max_complexity,
        max_methods: request.max_methods,
        max_lines: request.max_lines,
        max_declarations: request.max_declarations,
        rule: request.rule.clone(),
    }
}

/// smell's `resolve_options` replaces a rule's `exclude` outright with any
/// non-empty override, rather than concatenating the two. Given no
/// `--exclude`, this always sends a non-empty list (the defaults), so a
/// `smell.toml` rule's own `exclude` is shadowed whenever the defaults
/// apply. There's no way around that without a second resolve pass; the
/// clean fix is an additive `Overrides` field in `smell` itself. Passing any
/// `--exclude` replaces the defaults entirely, matching smell's own
/// override semantics so the flag behaves identically in both tools.
fn excludes(user: &[String]) -> Vec<String> {
    if user.is_empty() {
        DEFAULT_EXCLUDES
            .iter()
            .map(|glob| glob.to_string())
            .collect()
    } else {
        user.to_vec()
    }
}

/// What a run was configured with, for echoing back in a rendered document.
///
/// `rule`/`include`/`exclude`/`branches`/`implements` come from `Overrides`
/// (what was requested, with our default excludes applied) rather than the
/// resolved `AnalysisOptions`: `smell` compiles those into opaque globs, so
/// a `smell.toml` rule's own patterns can't be recovered here and won't
/// appear. The `max_*` limits are the opposite: they come from the resolved
/// `AnalysisOptions` rather than `Overrides`, since a limit set by a
/// `smell.toml` rule (rather than a flag) would otherwise echo as `null`
/// while the run's `measures` list shows it as configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub rule: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub branches: Vec<String>,
    pub implements: Vec<String>,
    pub max_complexity: Option<usize>,
    pub max_methods: Option<usize>,
    pub max_lines: Option<usize>,
    pub max_declarations: Option<usize>,
}

/// Builds the [`Settings`] echo from the `Overrides` a run sent to `smell`
/// and the `AnalysisOptions` it resolved to.
pub fn settings(overrides: &Overrides, options: &AnalysisOptions) -> Settings {
    Settings {
        rule: overrides
            .rule
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        include: overrides.include.clone(),
        exclude: overrides.exclude.clone(),
        branches: overrides.branches.clone(),
        implements: overrides.implements.clone(),
        max_complexity: options.max_complexity,
        max_methods: options.max_methods,
        max_lines: options.max_lines,
        max_declarations: options.max_declarations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Request {
        Request {
            include: vec!["*.rs".to_string()],
            exclude: vec!["custom/**".to_string()],
            branches: vec!["switch".to_string()],
            implements: vec!["Describe".to_string()],
            max_complexity: Some(10),
            max_methods: Some(8),
            max_lines: Some(300),
            max_declarations: Some(5),
            rule: Some("swift".to_string()),
            ..Request::default()
        }
    }

    #[test]
    fn overrides_copies_include_globs() {
        assert_eq!(overrides(&request()).include, vec!["*.rs".to_string()]);
    }

    #[test]
    fn overrides_copies_branches() {
        assert_eq!(overrides(&request()).branches, vec!["switch".to_string()]);
    }

    #[test]
    fn overrides_copies_implements() {
        assert_eq!(
            overrides(&request()).implements,
            vec!["Describe".to_string()]
        );
    }

    #[test]
    fn overrides_copies_max_limits() {
        let overrides = overrides(&request());
        assert_eq!(overrides.max_complexity, Some(10));
        assert_eq!(overrides.max_methods, Some(8));
        assert_eq!(overrides.max_lines, Some(300));
        assert_eq!(overrides.max_declarations, Some(5));
    }

    #[test]
    fn overrides_copies_rule() {
        assert_eq!(overrides(&request()).rule, Some("swift".to_string()));
    }

    #[test]
    fn overrides_defaults_are_empty_when_no_flags_given() {
        let overrides = overrides(&Request::default());
        assert!(overrides.include.is_empty());
        assert!(overrides.branches.is_empty());
        assert!(overrides.implements.is_empty());
        assert_eq!(overrides.max_complexity, None);
        assert_eq!(overrides.max_methods, None);
        assert_eq!(overrides.max_lines, None);
        assert_eq!(overrides.max_declarations, None);
        assert_eq!(overrides.rule, None);
    }

    #[test]
    fn overrides_uses_default_excludes_when_none_given() {
        let overrides = overrides(&Request::default());
        let expected: Vec<String> = DEFAULT_EXCLUDES
            .iter()
            .map(|glob| glob.to_string())
            .collect();
        assert_eq!(overrides.exclude, expected);
    }

    #[test]
    fn user_excludes_replace_defaults() {
        let overrides = overrides(&request());
        assert_eq!(overrides.exclude, vec!["custom/**".to_string()]);
        for default in DEFAULT_EXCLUDES {
            assert!(!overrides.exclude.contains(&default.to_string()));
        }
    }

    #[test]
    fn default_excludes_cover_dot_dirs_node_modules_and_build_outputs() {
        assert_eq!(
            DEFAULT_EXCLUDES,
            [
                "**/.*/**",
                "**/node_modules/**",
                "**/target/**",
                "**/build/**",
                "**/dist/**",
            ]
        );
    }

    #[test]
    fn default_excludes_all_end_in_subtree_suffix() {
        for pattern in DEFAULT_EXCLUDES {
            assert!(pattern.ends_with("/**"), "{pattern} lacks subtree suffix");
        }
    }

    #[test]
    fn settings_defaults_the_rule_name() {
        let overrides = overrides(&Request::default());
        let settings = settings(&overrides, &AnalysisOptions::default());
        assert_eq!(settings.rule, "default");
    }

    #[test]
    fn settings_echoes_the_requested_rule() {
        let overrides = overrides(&request());
        let settings = settings(&overrides, &AnalysisOptions::default());
        assert_eq!(settings.rule, "swift");
    }

    #[test]
    fn settings_echoes_the_default_excludes() {
        let overrides = overrides(&Request::default());
        let settings = settings(&overrides, &AnalysisOptions::default());
        let expected: Vec<String> = DEFAULT_EXCLUDES
            .iter()
            .map(|glob| glob.to_string())
            .collect();
        assert_eq!(settings.exclude, expected);
    }

    #[test]
    fn settings_echoes_user_excludes() {
        let overrides = overrides(&request());
        let settings = settings(&overrides, &AnalysisOptions::default());
        assert_eq!(settings.exclude, vec!["custom/**".to_string()]);
    }

    #[test]
    fn settings_echoes_branches_and_implements() {
        let overrides = overrides(&request());
        let settings = settings(&overrides, &AnalysisOptions::default());
        assert_eq!(settings.branches, vec!["switch".to_string()]);
        assert_eq!(settings.implements, vec!["Describe".to_string()]);
    }

    #[test]
    fn settings_takes_limits_from_the_resolved_options_not_the_overrides() {
        let overrides = overrides(&request());
        let options = AnalysisOptions {
            max_complexity: Some(1),
            max_methods: Some(2),
            max_lines: Some(3),
            max_declarations: None,
            ..AnalysisOptions::default()
        };
        let settings = settings(&overrides, &options);
        assert_eq!(settings.max_complexity, Some(1));
        assert_eq!(settings.max_methods, Some(2));
        assert_eq!(settings.max_lines, Some(3));
        assert_eq!(settings.max_declarations, None);
    }
}
