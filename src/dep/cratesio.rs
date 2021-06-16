use std::path::{self, Component, Path as StdPath, PathBuf};

use colored::Colorize as _;

#[derive(Debug, PartialEq)]
pub(crate) struct Path<'p> {
    registry_prefix: PathBuf,
    crate_name_version: &'p str,
    path: &'p StdPath,
}

impl<'p> Path<'p> {
    pub(crate) fn from_std_path(path: &'p StdPath) -> Option<Self> {
        if !path.is_absolute() {
            return None;
        }

        let mut components = path.components();

        let mut registry_prefix = PathBuf::new();
        while let Some(component) = components.next() {
            registry_prefix.push(component.as_os_str());

            if let Component::Normal(component) = component {
                if component == "registry" {
                    break;
                }
            }
        }

        let src = super::get_component_normal(components.next()?)?;
        if src != "src" {
            return None;
        }
        registry_prefix.push(src);

        let github = super::get_component_normal(components.next()?)?.to_str()?;
        if !github.starts_with("github.com-") {
            return None;
        }
        registry_prefix.push(github);

        let crate_name_version = super::get_component_normal(components.next()?)?.to_str()?;

        Some(Path {
            registry_prefix,
            crate_name_version,
            path: components.as_path(),
        })
    }

    pub(crate) fn format_short(&self) -> String {
        format!(
            "[{}]{}{}",
            self.crate_name_version,
            path::MAIN_SEPARATOR,
            self.path.display()
        )
    }

    pub(crate) fn format_highlight(&self) -> String {
        format!(
            "{}{sep}{}{sep}{}",
            self.registry_prefix.display().to_string().dimmed(),
            self.crate_name_version.bold(),
            self.path.display(),
            sep = path::MAIN_SEPARATOR,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end() {
        let input = StdPath::new(
        "/home/user/.cargo/registry/src/github.com-1ecc6299db9ec823/cortex-m-rt-0.6.13/src/lib.rs",
    );

        let path = Path::from_std_path(input).unwrap();
        let expected = Path {
            registry_prefix: PathBuf::from(
                "/home/user/.cargo/registry/src/github.com-1ecc6299db9ec823/",
            ),
            crate_name_version: "cortex-m-rt-0.6.13",
            path: StdPath::new("src/lib.rs"),
        };

        assert_eq!(expected, path);

        let expected_str = "[cortex-m-rt-0.6.13]/src/lib.rs";
        let formatted_str = path.format_short();

        assert_eq!(expected_str, formatted_str);
    }
}
