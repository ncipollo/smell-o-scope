//! Output generation: `json` renders the `--format json` document, `html`
//! wraps that same document in a self-contained `--format html` report.

use std::path::Path;

pub mod html;
pub mod json;

/// A node's display name: its file name, or the full path when it has none
/// (a root given as `.` or `/`).
pub(crate) fn display_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().to_string(),
    )
}
