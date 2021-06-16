use std::{env, ops::Deref};

use defmt_decoder::{Locations, Table};
use object::read::File as ObjectFile;

pub(crate) struct Elf<'file> {
    elf: ObjectFile<'file>,
    pub(crate) defmt_table: Option<Table>,
    pub(crate) defmt_locations: Option<Locations>,
}

impl<'file> Elf<'file> {
    pub(crate) fn parse(elf_bytes: &'file [u8]) -> Result<Self, anyhow::Error> {
        let elf = ObjectFile::parse(elf_bytes)?;

        let (defmt_table, defmt_locations) = extract_defmt_info(elf_bytes)?;

        Ok(Self {
            elf,
            defmt_table,
            defmt_locations,
        })
    }
}

impl<'elf> Deref for Elf<'elf> {
    type Target = ObjectFile<'elf>;

    fn deref(&self) -> &ObjectFile<'elf> {
        &self.elf
    }
}

fn extract_defmt_info(elf_bytes: &[u8]) -> anyhow::Result<(Option<Table>, Option<Locations>)> {
    let defmt_table = match env::var("PROBE_RUN_IGNORE_VERSION").as_deref() {
        Ok("true") | Ok("1") => defmt_decoder::Table::parse_ignore_version(elf_bytes)?,
        _ => defmt_decoder::Table::parse(elf_bytes)?,
    };

    let mut defmt_locations = None;

    if let Some(table) = defmt_table.as_ref() {
        let locations = table.get_locations(elf_bytes)?;

        if !table.is_empty() && locations.is_empty() {
            log::warn!("insufficient DWARF info; compile your program with `debug = 2` to enable location info");
        } else if table
            .indices()
            .all(|idx| locations.contains_key(&(idx as u64)))
        {
            defmt_locations = Some(locations);
        } else {
            log::warn!("(BUG) location info is incomplete; it will be omitted from the output");
        }
    }

    Ok((defmt_table, defmt_locations))
}
