//! Output generation. `json` is the real `--format json` document; `debug`
//! is a placeholder until the real HTML document lands (#4-6).

use std::path::Path;

pub mod debug;
pub mod json;

/// A node's display name: its file name, or the full path when it has none
/// (a root given as `.` or `/`).
pub(crate) fn display_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().to_string(),
    )
}
