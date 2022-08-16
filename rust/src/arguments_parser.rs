use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long, value_parser)]
    pub config: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
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

#[derive(Args)]
pub struct KernelCommandsArgs {
    #[clap(long = "hard-link", action, group = "hard_link")]
    hard_link: bool,

    #[clap(long = "no-hard-link", action, group = "hard_link")]
    no_hard_link: bool,
}
