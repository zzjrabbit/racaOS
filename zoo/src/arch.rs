use clap::{ValueEnum, builder::PossibleValue};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    process::Command,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Arch {
    #[serde(rename = "x86_64")]
    X86_64,
}

impl ValueEnum for Arch {
    fn value_variants<'a>() -> &'a [Self] {
        &[Arch::X86_64]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Arch::X86_64 => Some(PossibleValue::new(self.to_str())),
        }
    }
}

impl Arch {
    /// Get the target triple for the architecture.
    pub fn triple(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64-unknown-none",
        }
    }

    pub fn system_qemu(&self) -> &'static str {
        match self {
            Arch::X86_64 => "qemu-system-x86_64",
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
        }
    }
}

impl Display for Arch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

pub fn get_default_arch() -> Arch {
    if let Ok(arch) = std::env::var("OSDK_TARGET_ARCH") {
        return match arch.as_str() {
            "x86_64" => Arch::X86_64,
            _ => panic!(
                "The environment variable `OSDK_TARGET_ARCH` specifies an unsupported native architecture"
            ),
        };
    };

    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .expect("Failed to run rustc to get the host target");
    let output =
        std::str::from_utf8(&output.stdout).expect("`rustc -vV` didn't return utf8 output");

    let field = "host: ";
    let host = output
        .lines()
        .find(|l| l.starts_with(field))
        .map(|l| &l[field.len()..])
        .expect("`rustc -vV` didn't give a line for host")
        .to_string();

    match host.split('-').next() {
        Some(host_arch) => match host_arch {
            "x86_64" => Arch::X86_64,
            _ => panic!("The host has an unsupported native architecture"),
        },
        None => panic!("`rustc -vV` gave a host with unknown format"),
    }
}
