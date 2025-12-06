use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;
use zodiac::ZodiacError;

use crate::{files, init_modules};

mod loader;
mod macros;
mod symbols;

static MODULES: Mutex<Vec<Arc<Module>>> = Mutex::new(Vec::new());

files!(
    BOOT_FILES,
    c"symbols.sym",
    c"modules/core_dylib.km",
    c"modules/logger.km",
    c"modules/errors.km",
    c"modules/memory.km",
    c"modules/filesystem.km",
    c"modules/task.km",
);

pub fn init() -> Result<(), ZodiacError> {
    init_modules!(BOOT_FILES)?;
    Ok(())
}

pub struct Module {
    entry: fn(),
    name: &'static str,
}

impl Module {
    pub fn load(data: &[u8]) -> Result<Arc<Self>, ZodiacError> {
        let module = Self::load_module(data)?;
        (module.entry)();
        Ok(module)
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}
