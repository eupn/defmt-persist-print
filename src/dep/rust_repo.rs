use std::path::{self, Path as StdPath};

use colored::Colorize as _;

/// Representation of a rust-lang/rust repo path
#[derive(Debug, PartialEq)]
pub(crate) enum Path<'p> {
    One52(One52Path<'p>),
    Verbatim(&'p StdPath),
}

impl<'p> Path<'p> {
    pub(crate) fn from_std_path(path: &'p StdPath) -> Self {
        if let Some(path) = One52Path::from_std_path(path) {
            Path::One52(path)
        } else {
            Path::Verbatim(path)
        }
    }

    pub(crate) fn format(&self) -> String {
        match self {
            Path::One52(path) => path.format(),
            Path::Verbatim(path) => path.display().to_string(),
        }
    }

    pub(crate) fn format_highlight(&self) -> String {
        match self {
            Path::One52(path) => path.format_highlight(),
            Path::Verbatim(path) => path.display().to_string(),
        }
    }
}

/// rust-lang/repo path format as of 1.52 e.g. "library/core/src/panic.rs"
#[derive(Debug, PartialEq)]
pub(crate) struct One52Path<'p> {
    pub(crate) library: &'p str,
    pub(crate) crate_name: &'p str,
    pub(crate) path: &'p StdPath,
}

impl<'p> One52Path<'p> {
    fn from_std_path(path: &'p StdPath) -> Option<Self> {
        let mut components = path.components();

        let library = super::get_component_normal(components.next()?)?.to_str()?;
        if library != "library" {
            return None;
        }

        let crate_name = super::get_component_normal(components.next()?)?.to_str()?;

        Some(One52Path {
            library,
            crate_name,
            path: components.as_path(),
        })
    }

    fn format_highlight(&self) -> String {
        format!(
            "{}{sep}{}{sep}{}",
            self.library,
            self.crate_name.bold(),
            self.path.display(),
            sep = path::MAIN_SEPARATOR
        )
    }

    fn format(&self) -> String {
        format!(
            "{}{sep}{}{sep}{}",
            self.library,
            self.crate_name,
            self.path.display(),
            sep = path::MAIN_SEPARATOR
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_52_path() {
        let path = StdPath::new("library/core/src/sync/atomic.rs");

        let rust_repo_path = Path::from_std_path(path);
        let expected = Path::One52(One52Path {
            library: "library",
            crate_name: "core",
            path: StdPath::new("src/sync/atomic.rs"),
        });

        assert_eq!(expected, rust_repo_path);

        let expected_str = "library/core/src/sync/atomic.rs";
        let formatted_str = rust_repo_path.format();

        assert_eq!(expected_str, formatted_str);
    }

    #[test]
    fn v1_0_path() {
        let path = StdPath::new("src/libcore/atomic.rs");

        let rust_repo_path = Path::from_std_path(path);
        let expected = Path::Verbatim(path);

        assert_eq!(expected, rust_repo_path);

        let expected_str = "src/libcore/atomic.rs";
        let formatted_str = rust_repo_path.format();

        assert_eq!(expected_str, formatted_str);
    }
}
