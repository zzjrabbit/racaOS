use std::path::PathBuf;

use clap::{Args, Parser, crate_version};
use serde::{Deserialize, Serialize};

use crate::{
    arch::Arch,
    command::{build, run},
    config::Config,
};

pub fn cli_main(config: &Config) {
    let cli = Cli::parse();
    let CargoSubcommand::Zoo(subcommand) = &cli.cargo_subcommand;

    match subcommand {
        ZooSubcommand::Build(args) => build(config, args),
        ZooSubcommand::Run(args) => run(config, args),
    }
}

#[derive(Debug, Parser)]
#[clap(display_name = "cargo", bin_name = "cargo")]
pub struct Cli {
    #[clap(subcommand)]
    cargo_subcommand: CargoSubcommand,
}

#[derive(Debug, Parser)]
enum CargoSubcommand {
    #[clap(subcommand, version = crate_version!())]
    Zoo(ZooSubcommand),
}

#[derive(Debug, Parser)]
pub enum ZooSubcommand {
    #[command(about = "Compile the project and its dependencies")]
    Build(BuildArgs),
    #[command(about = "Run the kernel with a VMM")]
    Run(RunArgs),
}

#[derive(Debug, Parser)]
pub struct ForwardedArguments {
    #[arg(
        help = "The full set of Cargo arguments",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub args: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct BuildArgs {
    #[command(flatten)]
    pub common_args: CommonArgs,
}

#[derive(Debug, Parser)]
pub struct RunArgs {
    #[command(flatten)]
    pub common_args: CommonArgs,
    #[arg(
        long = "cpu",
        short = 'c',
        help = "The number of CPUs to use",
        default_value = "0",
        global = true
    )]
    pub cpu: u32,
    #[arg(
        long = "memory",
        short = 'm',
        help = "The amount of memory to allocate",
        default_value = "",
        global = true
    )]
    pub memory: String,
    #[arg(
        long = "serial",
        short = 's',
        help = "The serial target of qemu",
        default_value = "",
        global = true
    )]
    pub serial: String,
}

#[derive(Debug, Args, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CargoArgs {
    #[arg(
        long,
        help = "The Cargo build profile (built-in candidates are 'dev', 'release', 'test' and 'bench')",
        conflicts_with = "release",
        global = true
    )]
    pub profile: Option<String>,
    #[arg(
        long,
        help = "Build artifacts in release mode",
        conflicts_with = "profile",
        global = true
    )]
    pub release: bool,
    #[arg(
        long,
        value_name = "FEATURES",
        help = "List of features to activate",
        value_delimiter = ',',
        num_args = 1..,
        global = true,
    )]
    pub features: Vec<String>,
    #[arg(long, help = "Do not activate the `default` features", global = true)]
    pub no_default_features: bool,
    #[arg(
        long = "config",
        help = "Override a configuration value",
        value_name = "KEY=VALUE",
        global = true
    )]
    pub override_configs: Vec<String>,
}

impl CargoArgs {
    pub fn profile(&self) -> Option<String> {
        if self.release {
            Some("release".to_owned())
        } else {
            self.profile.clone()
        }
    }
}

#[derive(Debug, Args)]
/// Common args used for build, run, test and debug subcommand
pub struct CommonArgs {
    #[command(flatten)]
    pub build_args: CargoArgs,
    #[arg(
        long = "target-arch",
        value_name = "ARCH",
        help = "The architecture to build for",
        global = true
    )]
    pub target_arch: Option<Arch>,
    #[arg(
        long = "kcmd-args",
        require_equals = true,
        help = "Extra or overriding command line arguments for guest kernel",
        value_name = "ARGS",
        global = true
    )]
    pub kcmd_args: Vec<String>,
    #[arg(
        long = "bootdev-append-options",
        help = "Additional QEMU `-drive` options for the boot device",
        value_name = "OPTIONS",
        global = true
    )]
    pub bootdev_append_options: Option<String>,
    #[arg(
        long = "qemu-exe",
        help = "The QEMU executable file",
        value_name = "FILE",
        global = true
    )]
    pub qemu_exe: Option<PathBuf>,
    #[arg(
        long = "qemu-args",
        require_equals = true,
        help = "Extra arguments or overriding arguments for running QEMU",
        value_name = "ARGS",
        global = true
    )]
    pub qemu_args: Vec<String>,
}
