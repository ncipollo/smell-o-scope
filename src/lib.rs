//! Track down and visualize code smell. The `smell-o-scope` binary is a thin
//! CLI over this library.

/// The tool's name and version, shown until real functionality lands.
pub fn banner() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::banner;

    #[test]
    fn banner_includes_name_and_version() {
        let banner = banner();
        assert!(banner.starts_with("smell-o-scope "));
        assert!(banner.ends_with(env!("CARGO_PKG_VERSION")));
    }
}
