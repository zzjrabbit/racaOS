use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;
use zodiac::ZodiacError;

use crate::init_modules;

mod loader;
mod macros;
mod symbols;

static MODULES: Mutex<Vec<Arc<Module>>> = Mutex::new(Vec::new());

pub fn init() {
    symbols::init().unwrap();
    init_modules!(core_dylib, memory, hello).unwrap();
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
