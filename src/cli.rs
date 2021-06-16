use std::path::PathBuf;

use defmt_decoder::DEFMT_VERSION;
use git_version::git_version;
use log::Level;
use structopt::{clap::AppSettings, StructOpt};

/// Successful termination of process.
const EXIT_SUCCESS: i32 = 0;

/// Prints defmt logs saved by `defmt-persist`.
#[derive(StructOpt)]
#[structopt(name = "defmt-print", setting = AppSettings::TrailingVarArg)]
pub(crate) struct Opts {
    /// Path to an ELF firmware file.
    #[structopt(name = "ELF", parse(from_os_str), required_unless_one(&["version"]))]
    elf: Option<PathBuf>,

    /// Enable more verbose logging.
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u32,

    /// Prints version information
    #[structopt(short = "V", long)]
    version: bool,

    /// Whether to shorten paths (e.g. to crates.io dependencies) in backtraces and defmt logs
    #[structopt(long)]
    pub(crate) shorten_paths: bool,

    /// Arguments passed after the ELF file path are discarded
    #[structopt(name = "REST")]
    _rest: Vec<String>,
}

pub(crate) fn handle_arguments() -> anyhow::Result<i32> {
    let opts: Opts = Opts::from_args();
    let verbose = opts.verbose;

    defmt_decoder::log::init_logger(verbose >= 1, move |metadata| {
        if defmt_decoder::log::is_defmt_frame(metadata) {
            true // We want to display *all* defmt frames.
        } else {
            // Log depending on how often the `--verbose` (`-v`) cli-param is supplied:
            //   * 0: log everything from defmt-print, with level "info" or higher
            //   * 1: log everything from defmt-print
            //   * 2 or more: log everything
            if verbose >= 2 {
                true
            } else if verbose >= 1 {
                metadata.target().starts_with("defmt_print")
            } else {
                metadata.target().starts_with("defmt_print") && metadata.level() <= Level::Info
            }
        }
    });

    if opts.version {
        print_version();
        Ok(EXIT_SUCCESS)
    } else if let Some(elf) = opts.elf.as_deref() {
        crate::run_target_program(elf, &opts)
    } else {
        unreachable!("due to `StructOpt` constraints")
    }
}

/// The string reported by the `--version` flag
fn print_version() {
    /// Version from `Cargo.toml` e.g. `"0.1.4"`
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    /// `""` OR git hash e.g. `"34019f8"`
    ///
    /// `git describe`-docs:
    /// > The command finds the most recent tag that is reachable from a commit. (...)
    /// It suffixes the tag name with the number of additional commits on top of the tagged object
    /// and the abbreviated object name of the most recent commit.
    //
    // The `fallback` is `"--"`, cause this will result in "" after `fn extract_git_hash`.
    const GIT_DESCRIBE: &str = git_version!(fallback = "--", args = ["--long"]);
    // Extract the "abbreviated object name"
    let hash = extract_git_hash(GIT_DESCRIBE);

    println!(
        "{} {}\nsupported defmt version: {}",
        VERSION, hash, DEFMT_VERSION
    );
}

/// Extract git hash from a `git describe` statement
fn extract_git_hash(git_describe: &str) -> &str {
    git_describe.split('-').nth(2).unwrap()
}
