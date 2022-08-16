use std::fs;

use toml::Value;
use clap::{Args, Parser, Subcommand, ErrorKind};

#[derive(Parser)]
struct Cli {
    #[clap(short, long, value_parser)]
    config: Option<String>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
struct KernelCommandsArgs {
    #[clap(long = "hard-link", action, group = "hard_link")]
    hard_link: bool,

    #[clap(long = "no-hard-link", action, group = "hard_link")]
    no_hard_link: bool,
}

pub enum OperationRequest {
    ChangeKernel {
        source: String,
        destination: String,
        hard_link: bool,
    }
}

pub enum UserInteractError {
    /// An error caused by invalid input from the user.
    UserInputError,
    /// An error in communication with the user.
    IOError,
}

/// This function interacts with the user.
/// It determines what the user wants the program to do,
/// based on command line arguments and a config file.
/// It also prints various things to the terminal depending
/// on the situation, e.g. to display help text in case
/// the command line arguments were invalid.<br>
/// This function itself does not make any decisions on what the
/// program does. It just determines and returns the user's request.
/// It is up to the rest of the program's discretion whether to or
/// how to carry out the user's request.
/// Thus, this function does not read anything other than command
/// line arguments and a config file to determine what to return.<br>
/// This function can succeed or fail. On success, it returns a
/// struct representing the user's request for the program.
/// On failure, it returns an error.
pub fn interact_with_user(default_config_file: &str) -> Result<OperationRequest, UserInteractError> {
    let cli_args = Cli::try_parse().map_err(|err| {
        match err.kind() {
            ErrorKind::Io | ErrorKind::Format => UserInteractError::IOError,
            _ => UserInteractError::UserInputError,
        }
    })?;

    let config_file = match cli_args.config {
        Some(ref x) => &x,
        None => default_config_file,
    };
    let config = fs::read_to_string(config_file)
        .map_err(|_| UserInteractError::IOError)?
        .parse::<Value>()
        .map_err(|_| UserInteractError::IOError)?;

    match cli_args.command {
        Commands::ChangeKernel {
            shared_args: KernelCommandsArgs {
                hard_link,
                no_hard_link,
            },
            file,
        } => {
        },
        _ => panic!(),
    }

    todo!();
}
