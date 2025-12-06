use std::{fs::File, path::Path};

use elf::{
    ElfStream,
    abi::{STB_GLOBAL, STV_DEFAULT},
    endian::LittleEndian,
};
use rustc_demangle::demangle;
use stringmap::StringMap;

pub fn scan_kernel(path: &Path) -> anyhow::Result<StringMap<usize>> {
    let file = File::open(path)?;
    let mut elf_stream = ElfStream::<LittleEndian, _>::open_stream(file)?;

    let mut symbols = StringMap::new();

    let (symtab, strtab) = elf_stream
        .symbol_table()?
        .ok_or(anyhow::anyhow!("Symbol table missing"))?;

    for symbol in symtab.iter() {
        if symbol.is_undefined() || symbol.st_bind() != STB_GLOBAL || symbol.st_vis() != STV_DEFAULT
        {
            continue;
        }

        let symbol_name = strtab.get(symbol.st_name as usize)?;
        let symbol_name = format!("{:#}", demangle(symbol_name));
        if !symbol_name.starts_with("zodiac")
            && !symbol_name.starts_with("<zodiac")
            && !symbol_name.starts_with("<alloc::sync::Arc")
        {
            continue;
        }

        symbols.insert(&symbol_name, symbol.st_value as usize);
    }

    Ok(symbols)
}
