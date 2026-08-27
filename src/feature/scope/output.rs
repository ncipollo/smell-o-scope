use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::ValueEnum;

/// Written to when `--output` is a file, and no path is given for `--format html`.
pub const DEFAULT_HTML_OUTPUT: &str = "smell-o-scope.html";

/// The document format to render: `json` is the raw document (see
/// [`crate::render::json`]); `html` wraps that same document in a
/// self-contained report (see [`crate::render::html`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Json,
    #[default]
    Html,
}

/// Where a rendered document is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    Stdout,
    File(PathBuf),
}

/// Resolves `--output` against `--format`: an explicit path wins (`-` means
/// stdout); otherwise json defaults to stdout and html defaults to
/// [`DEFAULT_HTML_OUTPUT`].
pub fn destination(format: Format, output: Option<&Path>) -> Destination {
    match output {
        Some(path) if path == Path::new("-") => Destination::Stdout,
        Some(path) => Destination::File(path.to_path_buf()),
        None => match format {
            Format::Json => Destination::Stdout,
            Format::Html => Destination::File(PathBuf::from(DEFAULT_HTML_OUTPUT)),
        },
    }
}

/// Writes `document` to `destination`, noting a file write on stderr so a
/// run isn't silent while stdout stays clean for `--format json`.
pub fn emit(destination: &Destination, document: &str) -> io::Result<()> {
    match destination {
        Destination::Stdout => {
            println!("{document}");
            Ok(())
        }
        Destination::File(path) => {
            fs::write(path, format!("{document}\n"))?;
            eprintln!("wrote {}", path.display());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_defaults_to_stdout_for_json() {
        assert_eq!(destination(Format::Json, None), Destination::Stdout);
    }

    #[test]
    fn destination_defaults_to_html_file_for_html() {
        assert_eq!(
            destination(Format::Html, None),
            Destination::File(PathBuf::from(DEFAULT_HTML_OUTPUT))
        );
    }

    #[test]
    fn destination_uses_explicit_output_for_json() {
        let path = Path::new("report.json");
        assert_eq!(
            destination(Format::Json, Some(path)),
            Destination::File(PathBuf::from("report.json"))
        );
    }

    #[test]
    fn destination_uses_explicit_output_for_html() {
        let path = Path::new("report.html");
        assert_eq!(
            destination(Format::Html, Some(path)),
            Destination::File(PathBuf::from("report.html"))
        );
    }

    #[test]
    fn destination_treats_dash_as_stdout() {
        assert_eq!(
            destination(Format::Html, Some(Path::new("-"))),
            Destination::Stdout
        );
    }
}
