// SPDX-License-Identifier: MPL-2.0

//! Component system
//!

#![no_std]
#![deny(unsafe_code)]

extern crate alloc;

use alloc::{fmt::Debug, string::String, vec::Vec};

pub use comp_macro::*;
use log::{debug, error, info};

type ComponentFunction = dyn Fn() -> Result<(), ComponentInitError> + Sync;

/// The initialization stages of the component system.
///
/// - `Bootstrap`: The earliest stage, called after OSTD initialization is
///   complete but before kernel subsystem initialization begins. This stage
///   runs on the BSP (Bootstrap Processor) only, before SMP (Symmetric
///   Multi-Processing) is enabled. Components in this stage can initialize
///   core kernel services that other components depend on.
/// - `Kthread`: The kernel thread stage, initialized after SMP is enabled
///   and the first kernel thread is spawned. This stage runs in the context
///   of the first kernel thread on the BSP.
/// - `Process`: The process stage, initialized after the first user process
///   is created. This stage runs in the context of the first user process,
///   and prepares the system for user-space execution.
#[derive(Debug, PartialEq, Eq)]
pub enum InitStage {
    Bootstrap,
    Kthread,
    Process,
}

#[derive(Debug)]
pub enum ComponentInitError {
    UninitializedDependencies(String),
    Unknown,
}

pub struct ComponentRegistry {
    stage: InitStage,
    function: &'static ComponentFunction,
    path: &'static str,
}

impl ComponentRegistry {
    pub const fn new(
        stage: InitStage,
        function: &'static ComponentFunction,
        path: &'static str,
    ) -> Self {
        Self {
            stage,
            function,
            path,
        }
    }
}

impl Debug for ComponentRegistry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field("stage", &self.stage)
            .field("path", &self.path)
            .finish()
    }
}

#[derive(Debug)]
pub enum ComponentSystemInitError {
    FileNotValid,
    NotIncludeAllComponent(String),
}

/// Initializes the component system for a specific stage.
///
/// It collects all functions marked with the `init_component` macro, filters them
/// according to the given stage, and invokes them in the correct order while honoring
/// dependencies and priorities between crates.
///
/// The collection of ComponentRegistry usually generate by `parse_metadata` macro.
///
/// ```rust
///     component::init_all(component::InitStage::Bootstrap, component::parse_metadata!());
/// ```
///
pub fn init_all(
    stage: InitStage,
    components: Vec<&ComponentRegistry>,
) -> Result<(), ComponentSystemInitError> {
    match_and_call(stage, components)?;
    Ok(())
}

/// Match the ComponentInfo with ComponentRegistry. The key is the relative path of one component
fn match_and_call(
    stage: InitStage,
    components: Vec<&ComponentRegistry>,
) -> Result<(), ComponentSystemInitError> {
    let mut components_to_init = Vec::new();
    for component in components {
        if component.stage != stage {
            continue;
        }

        components_to_init.push(component);
    }

    debug!("component infos: {components_to_init:?}");
    info!("Components initializing in {stage:?} stage...");

    for component in components_to_init {
        info!("Component initializing:{:?}", component);
        if let Err(res) = (component.function)() {
            error!("Component initialize error:{:?}", res);
        } else {
            info!("Component initialize complete");
        }
    }
    info!("All components initialization in {stage:?} stage completed");
    Ok(())
}

#[doc(hidden)]
#[macro_export]
macro_rules! get_component {
    ($name: ident) => {
        unsafe { &$name::__COMPONENT_REGISTRY }
    };
}

#[macro_export]
macro_rules! component_list {
    ($($name: ident), *) => {
        [
            $(component::get_component!($name)),*
        ].into()
    };
}
