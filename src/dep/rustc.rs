use std::path::{self, Component, Path as StdPath, PathBuf};

use colored::Colorize;

use super::rust_repo;

#[derive(Debug, PartialEq)]
pub(crate) struct Path<'p> {
    rustc_prefix: PathBuf,
    rust_repo_path: rust_repo::Path<'p>,
}

impl<'p> Path<'p> {
    pub(crate) fn from_std_path(path: &'p StdPath) -> Option<Self> {
        if !path.is_absolute() {
            return None;
        }

        let mut components = path.components();

        let mut rustc_prefix = PathBuf::new();
        while let Some(component) = components.next() {
            rustc_prefix.push(component);

            if let Component::Normal(component) = component {
                if component == "rustc" {
                    break;
                }
            }
        }

        let hash = super::get_component_normal(components.next()?)?.to_str()?;
        if !hash.chars().all(|c| char::is_ascii_hexdigit(&c)) {
            return None;
        }
        rustc_prefix.push(hash);

        let rust_repo_path = rust_repo::Path::from_std_path(components.as_path());

        Some(Path {
            rustc_prefix,
            rust_repo_path,
        })
    }

    pub(crate) fn format_short(&self) -> String {
        format!(
            "[rust]{}{}",
            path::MAIN_SEPARATOR,
            self.rust_repo_path.format()
        )
    }

    pub(crate) fn format_highlight(&self) -> String {
        format!(
            "{}{}{}",
            self.rustc_prefix.display().to_string().dimmed(),
            path::MAIN_SEPARATOR,
            self.rust_repo_path.format_highlight()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end() {
        let input = StdPath::new(
            "/rustc/9bc8c42bb2f19e745a63f3445f1ac248fb015e53/library/core/src/panicking.rs",
        );

        let path = Path::from_std_path(input).unwrap();
        let expected = Path {
            rustc_prefix: PathBuf::from("/rustc/9bc8c42bb2f19e745a63f3445f1ac248fb015e53"),
            rust_repo_path: rust_repo::Path::One52(rust_repo::One52Path {
                library: "library",
                crate_name: "core",
                path: StdPath::new("src/panicking.rs"),
            }),
        };

        assert_eq!(expected, path);

        let expected_str = "[rust]/library/core/src/panicking.rs";
        let formatted_str = path.format_short();

        assert_eq!(expected_str, formatted_str);
    }
}
