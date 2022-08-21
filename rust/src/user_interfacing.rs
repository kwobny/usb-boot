mod cmdline_parsing;
mod config_parsing;

#[cfg(test)]
mod tests;

use std::io;
use clap::{Parser, ErrorKind};
use cmdline_parsing::{Cli, Commands, KernelCommandsArgs};

#[derive(Debug)]
pub enum OperationRequest {
    ChangeKernel {
        source: String,
        destination: String,
        hard_link: bool,
        mkinitcpio_preset: String,
    }
}

/// A type representing an error that occurred while trying to
/// interact with the user.
#[derive(Debug, thiserror::Error)]
pub enum UserInteractError {
    #[error("invalid command line arguments")]
    InvalidCommandLineArguments {
        #[source]
        details: clap::error::Error,
    },
    #[error("io error while handling command line arguments")]
    CliIOError {
        #[source]
        source: clap::error::Error,
    },
    #[error("there is a syntax error in the config file")]
    ConfigParseError {
        #[source]
        source: toml::de::Error,
    },
    /// Failed to access config file due to it not existing,
    /// permission errors, etc.
    #[error("failed to access config file")]
    ConfigAccessFailed {
        #[source]
        source: io::Error,
    },
}

/// This function interacts with the user.
/// It determines what the user wants the program to do,
/// based on command line arguments and a config file.
/// It also prints various things to the terminal depending
/// on the situation, e.g. to display help text in case
/// the command line arguments were invalid.
///
/// This function itself does not make any decisions on what the
/// program does. It just determines and returns the user's request.
/// It is up to the rest of the program's discretion whether to or
/// how to carry out the user's request.
/// Thus, this function does not read anything other than command
/// line arguments and a config file to determine what to return.
/// This function is designed so that if the caller just calls this
/// function and does nothing else, no changes will be made to the system,
/// no matter how many times this function is called. In other words,
/// this function is suitable for dry running purposes.
///
/// This function can succeed or fail. On success, it returns a
/// struct representing the user's request for the program.
/// On failure, it returns an error.
pub fn interact_with_user(default_config_file: &str) -> Result<OperationRequest, UserInteractError> {
    let cli_args = Cli::try_parse().map_err(|err| {
        match err.kind() {
            ErrorKind::Io | ErrorKind::Format =>
                UserInteractError::CliIOError {
                    source: err,
                },
            _ => UserInteractError::InvalidCommandLineArguments {
                details: err,
            },
        }
    })?;

    let config_file = match cli_args.config {
        Some(ref x) => x,
        None => default_config_file,
    };
    let config_contents = config_parsing::parse_config(config_file)?;

    let operation_request = match cli_args.command {
        Commands::ChangeKernel {
            shared_args: KernelCommandsArgs {
                hard_link: hard_link_flag,
                no_hard_link: no_hard_link_flag,
            },
            ..
        } |
        Commands::UpdateKernel {
            shared_args: KernelCommandsArgs {
                hard_link: hard_link_flag,
                no_hard_link: no_hard_link_flag,
            },
        } => {
            let source_kernel_file = match cli_args.command {
                Commands::ChangeKernel {
                    file,
                    ..
                } => file,
                Commands::UpdateKernel { .. } => {
                    config_contents.upstream_kernel
                },
            };

            let boot_kernel = config_contents.boot_kernel;
            let mkinitcpio_preset = config_contents.mkinitcpio_preset;
            let hard_link_default = config_contents.default_options.hard_link;

            let do_hard_link = match (hard_link_flag, no_hard_link_flag) {
                (false, false) => hard_link_default,
                (a, b) if a ^ b => hard_link_flag,
                _ => panic!(), // if (true, true)
            };

            OperationRequest::ChangeKernel {
                source: source_kernel_file,
                destination: boot_kernel,
                hard_link: do_hard_link,
                mkinitcpio_preset,
            }
        },
    };

    Ok(operation_request)
}
