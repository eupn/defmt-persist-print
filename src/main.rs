mod cli;
mod dep;
mod elf;
mod storage;

use std::{
    env, fs,
    io::{self, Write as _},
    path::Path,
    process,
};

use anyhow::{anyhow, bail};
use colored::Colorize as _;
use defmt_decoder::Locations;

use crate::elf::Elf;
use crate::storage::FlashStorage;

use defmt_persist::{LogManager, StorageHelper};
use std::io::BufRead;

const BUF_LEN: usize = 1024 * 10; // should be enough for everybody (c)

fn main() -> anyhow::Result<()> {
    cli::handle_arguments().map(|code| process::exit(code))
}

fn run_target_program(elf_path: &Path, opts: &cli::Opts) -> anyhow::Result<i32> {
    if !elf_path.exists() {
        return Err(anyhow!(
            "can't find ELF file at `{}`; are you sure you got the right path?",
            elf_path.display()
        ));
    }

    let elf_bytes = fs::read(elf_path)?;
    let elf = &Elf::parse(&elf_bytes)?;

    let current_dir = &env::current_dir()?;

    extract_and_print_logs(elf, opts, current_dir)?;

    print_separator();

    Ok(0)
}

fn extract_and_print_logs(
    elf: &Elf,
    opts: &cli::Opts,
    current_dir: &Path,
) -> Result<(), anyhow::Error> {
    print_separator();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut read_buf = [0u8; BUF_LEN];
    let mut defmt_buffer = vec![];

    let input_buf = stdin_dump_to_bytes()?;
    let input_buf = input_buf.as_slice();
    let mut storage = FlashStorage::new(input_buf);
    let mut sh = StorageHelper::try_new(&mut storage).unwrap();

    match &elf.defmt_table {
        Some(table) => {
            let num_bytes_read =
                LogManager::retrieve_frames_helper(&mut sh, &mut storage, &mut read_buf).unwrap();
            defmt_buffer.extend_from_slice(&read_buf[..num_bytes_read]);

            decode_and_print_defmt_logs(
                &mut defmt_buffer,
                table,
                elf.defmt_locations.as_ref(),
                current_dir,
                opts,
            )?;
        }

        _ => {
            stdout.write_all(&read_buf)?;
            stdout.flush()?;
        }
    }

    Ok(())
}

fn stdin_dump_to_bytes() -> Result<Vec<u8>, anyhow::Error> {
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let mut bytes = vec![];

    for line in stdin.lines().flatten() {
        let words = line.split(' ').collect::<Vec<_>>();
        let mut words = words.into_iter();
        if let Some("Addr") = words.next() {
            if let Some(s) = words.last() {
                let word = &s[2..];
                let word = u32::from_str_radix(word, 16)?;
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
    }

    if bytes.is_empty() {
        bail!("No stdin dump provided");
    }

    Ok(bytes)
}

fn decode_and_print_defmt_logs(
    buffer: &mut Vec<u8>,
    table: &defmt_decoder::Table,
    locations: Option<&Locations>,
    current_dir: &Path,
    opts: &cli::Opts,
) -> Result<(), anyhow::Error> {
    let mut frame_buf = [0u8; BUF_LEN];

    for buf in buffer.split_mut(|b| *b == 0xFF).filter(|f| f.len() > 1) {
        let len = cobs::decode_with_sentinel(buf, &mut frame_buf, 0xFF);

        if let Ok(len) = len {
            let buf = &frame_buf[..len];

            match table.decode(buf) {
                Ok((frame, _)) => {
                    // NOTE(`[]` indexing) all indices in `table` have already been verified to exist in
                    // the `locations` map
                    let (file, line, mod_path) = locations
                        .map(|locations| &locations[&frame.index()])
                        .map(|location| {
                            let path = if let Ok(relpath) = location.file.strip_prefix(&current_dir)
                            {
                                relpath.display().to_string()
                            } else {
                                let dep_path = dep::Path::from_std_path(&location.file);

                                if opts.shorten_paths {
                                    dep_path.format_short()
                                } else {
                                    dep_path.format_highlight()
                                }
                            };

                            (
                                Some(path),
                                Some(location.line as u32),
                                Some(&*location.module),
                            )
                        })
                        .unwrap_or((None, None, None));

                    // Forward the defmt frame to our logger.
                    defmt_decoder::log::log_defmt(&frame, file.as_deref(), line, mod_path);
                }

                Err(defmt_decoder::DecodeError::UnexpectedEof) => {
                    log::error!("Unexpected end of defmt frame");
                }

                Err(defmt_decoder::DecodeError::Malformed) => {
                    log::error!("failed to decode defmt data: {:x?}", buffer);
                    return Err(defmt_decoder::DecodeError::Malformed.into());
                }
            }
        } else {
            log::error!("COBS decoding error. Raw frame: {:?}", frame_buf);
        }
    }

    Ok(())
}

/// Print a line to separate different execution stages.
fn print_separator() {
    println!("{}", "─".repeat(80).dimmed());
}
