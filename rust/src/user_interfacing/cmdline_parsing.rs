//! This module parses the command line arguments. This module
//! does nothing else besides that. Parsing command line arguments
//! is the sole responsibility of this module.

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(short, long, value_parser)]
    pub config: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(name = "change-kernel")]
    ChangeKernel {
        #[clap(flatten)]
        shared_args: KernelCommandsArgs,

        #[clap(value_parser)]
        file: String,
    },
    #[clap(name = "update-kernel")]
    UpdateKernel {
        #[clap(flatten)]
        shared_args: KernelCommandsArgs,
    }
}

#[derive(Args, Debug)]
pub struct KernelCommandsArgs {
    #[clap(long = "hard-link", action, group = "hard_link")]
    pub hard_link: bool,

    #[clap(long = "no-hard-link", action, group = "hard_link")]
    pub no_hard_link: bool,

    #[clap(long = "compare-kernels",
           value_parser = ["false", "full", "efficient"])]
    pub compare_kernels: Option<String>,
}
