use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub build: BuildConfig,
    pub qemu: Option<QemuConfig>,
}

impl Config {
    pub fn parse(config: String) -> Self {
        toml::from_str(&config).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BuildConfig {
    pub kernel_crate: String,
    pub profile: Option<String>,
    pub features: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct QemuConfig {
    pub args: Vec<String>,
    pub hw_virt: Option<bool>,
    pub serial_target: Option<String>,
    pub smp_cores: Option<u32>,
    pub memory_size: Option<String>,
}
