use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use crate::feature::scope;
use crate::feature::scope::output::{self, Format};
use crate::feature::scope::request::Request;

#[derive(Parser)]
#[command(name = "smell-o-scope", version, about)]
struct Cli {
    /// Source files or directories to analyze (directories are searched
    /// recursively); propagated straight to `smell`.
    #[arg(value_name = "PATH", num_args = 1.., required = true)]
    paths: Vec<PathBuf>,

    /// Only analyze files matching this glob (repeatable).
    #[arg(long)]
    include: Vec<String>,

    /// Skip files matching this glob (repeatable). Replaces smell-o-scope's
    /// default excludes outright rather than adding to them.
    #[arg(long)]
    exclude: Vec<String>,

    /// Count only these branch kinds, comma-separated (see `smell --info branches`).
    #[arg(long, value_delimiter = ',')]
    branches: Vec<String>,

    /// Only analyze types implementing/extending this interface, protocol,
    /// trait, or superclass (repeatable).
    #[arg(long, value_name = "NAME")]
    implements: Vec<String>,

    /// Report functions whose complexity exceeds this limit.
    #[arg(long, value_name = "N")]
    max_complexity: Option<usize>,

    /// Report types whose method count exceeds this limit.
    #[arg(long, value_name = "N")]
    max_methods: Option<usize>,

    /// Report files whose line count exceeds this limit.
    #[arg(long, value_name = "N")]
    max_lines: Option<usize>,

    /// Report files whose declaration count exceeds this limit.
    #[arg(long, value_name = "N")]
    max_declarations: Option<usize>,

    /// Use the named rule from smell.toml instead of the "default" rule.
    #[arg(long, value_name = "NAME")]
    rule: Option<String>,

    /// Output document format.
    #[arg(long, value_enum, default_value_t = Format::default())]
    format: Format,

    /// Where to write the document; `-` means stdout. Defaults to stdout for
    /// json and `smell-o-scope.html` for html.
    #[arg(long, value_name = "PATH")]
    output: Option<PathBuf>,
}

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let request = request(cli);
    let outcome = match scope::run(&request) {
        Ok(outcome) => outcome,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    for message in &outcome.errors {
        eprintln!("{message}");
    }
    if let Err(error) = output::emit(&outcome.destination, &outcome.document) {
        eprintln!("error: writing output: {error}");
        return ExitCode::FAILURE;
    }
    exit_code(&outcome)
}

fn request(cli: Cli) -> Request {
    Request {
        paths: cli.paths,
        include: cli.include,
        exclude: cli.exclude,
        branches: cli.branches,
        implements: cli.implements,
        max_complexity: cli.max_complexity,
        max_methods: cli.max_methods,
        max_lines: cli.max_lines,
        max_declarations: cli.max_declarations,
        rule: cli.rule,
        format: cli.format,
        output: cli.output,
    }
}

/// Check failures are reported but don't fail the run — smell-o-scope
/// produces a report artifact, not a CI gate. The one exception: every
/// requested path failed, so there's nothing to show for the run.
fn exit_code(outcome: &scope::Outcome) -> ExitCode {
    if outcome.analyzed_nothing() && !outcome.errors.is_empty() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn requires_at_least_one_path() {
        assert!(Cli::try_parse_from(["smell-o-scope"]).is_err());
    }

    #[test]
    fn parses_multiple_paths() {
        let cli = Cli::try_parse_from(["smell-o-scope", "src", "lib.rs"]).expect("paths parse");
        assert_eq!(
            cli.paths,
            vec![PathBuf::from("src"), PathBuf::from("lib.rs")]
        );
    }

    #[test]
    fn parses_repeatable_include_and_exclude() {
        let cli = Cli::try_parse_from([
            "smell-o-scope",
            "src",
            "--include",
            "*.rs",
            "--exclude",
            "**/gen/**",
        ])
        .expect("globs parse");
        assert_eq!(cli.include, vec!["*.rs"]);
        assert_eq!(cli.exclude, vec!["**/gen/**"]);
    }

    #[test]
    fn parses_comma_separated_branches() {
        let cli = Cli::try_parse_from(["smell-o-scope", "src", "--branches", "switch,loop"])
            .expect("branches parse");
        assert_eq!(cli.branches, vec!["switch", "loop"]);
    }

    #[test]
    fn parses_repeatable_implements() {
        let cli = Cli::try_parse_from(["smell-o-scope", "src", "--implements", "Describe"])
            .expect("implements parse");
        assert_eq!(cli.implements, vec!["Describe"]);
    }

    #[test]
    fn parses_max_limits() {
        let cli = Cli::try_parse_from([
            "smell-o-scope",
            "src",
            "--max-complexity",
            "10",
            "--max-methods",
            "8",
            "--max-lines",
            "300",
            "--max-declarations",
            "5",
        ])
        .expect("limits parse");
        assert_eq!(cli.max_complexity, Some(10));
        assert_eq!(cli.max_methods, Some(8));
        assert_eq!(cli.max_lines, Some(300));
        assert_eq!(cli.max_declarations, Some(5));
    }

    #[test]
    fn parses_rule_name() {
        let cli =
            Cli::try_parse_from(["smell-o-scope", "src", "--rule", "swift"]).expect("rule parses");
        assert_eq!(cli.rule, Some("swift".to_string()));
    }

    #[test]
    fn format_defaults_to_html() {
        let cli = Cli::try_parse_from(["smell-o-scope", "src"]).expect("path parses");
        assert_eq!(cli.format, Format::Html);
    }

    #[test]
    fn parses_json_format() {
        let cli = Cli::try_parse_from(["smell-o-scope", "src", "--format", "json"])
            .expect("format parses");
        assert_eq!(cli.format, Format::Json);
    }

    #[test]
    fn rejects_unknown_format() {
        assert!(Cli::try_parse_from(["smell-o-scope", "src", "--format", "xml"]).is_err());
    }

    #[test]
    fn parses_output_path() {
        let cli = Cli::try_parse_from(["smell-o-scope", "src", "--output", "report.html"])
            .expect("output parses");
        assert_eq!(cli.output, Some(PathBuf::from("report.html")));
    }

    #[test]
    fn output_defaults_to_none() {
        let cli = Cli::try_parse_from(["smell-o-scope", "src"]).expect("path parses");
        assert_eq!(cli.output, None);
    }
}
