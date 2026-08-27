//! Test-only fixture helpers shared across unit and end-to-end tests.

use std::path::PathBuf;

/// Resolves a path under `fixtures/` at the crate root, independent of the
/// current working directory a test runs from.
pub fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(relative)
}
