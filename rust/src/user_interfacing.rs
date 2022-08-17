mod cmdline_parsing;
mod config_parsing;

use std::fs;

use toml::Value;
use clap::{Parser, ErrorKind};

use cmdline_parsing::{Cli, Commands, KernelCommandsArgs};
use config_parsing::ConfigFileInfo;

pub enum OperationRequest {
    ChangeKernel {
        source: String,
        destination: String,
        hard_link: bool,
        mkinitcpio_preset: String,
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
pub fn interact_with_user(config_file_info: ConfigFileInfo) -> Result<OperationRequest, UserInteractError> {
    let cli_args = Cli::try_parse().map_err(|err| {
        match err.kind() {
            ErrorKind::Io | ErrorKind::Format => UserInteractError::IOError,
            _ => UserInteractError::UserInputError,
        }
    })?;

    let config_file = match cli_args.config {
        Some(ref x) => &x,
        None => config_file_info.default_file,
    };
    let config = fs::read_to_string(config_file)
        .map_err(|_| UserInteractError::IOError)?
        .parse::<Value>()
        .map_err(|_| UserInteractError::UserInputError)?;
    let config_contents = config_parsing::parse_config(config_file_info, config)?;

    match cli_args.command {
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
                        .ok_or(UserInteractError::UserInputError)?
                },
            };

            let boot_kernel = config_contents.boot_kernel
                .ok_or(UserInteractError::UserInputError)?;
            let mkinitcpio_preset = config_contents.mkinitcpio_preset
                .ok_or(UserInteractError::UserInputError)?;
            let hard_link_default = config_contents.hard_link_default
                .ok_or(UserInteractError::UserInputError)?;

            let do_hard_link = match (hard_link_flag, no_hard_link_flag) {
                (false, false) => hard_link_default,
                (a, b) if a ^ b => hard_link_flag,
                _ => panic!(), // if (true, true)
            };

            Ok(OperationRequest::ChangeKernel {
                source: source_kernel_file,
                destination: boot_kernel,
                hard_link: do_hard_link,
                mkinitcpio_preset,
            })
        },
    }
}
