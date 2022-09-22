mod cmdline_parsing;
mod config_parsing;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::{io, ffi::OsString, env};
use std::borrow::Borrow;
use clap::{Parser, ErrorKind};
use cmdline_parsing::{Cli, Commands, KernelCommandsArgs};

use self::config_parsing::CompareKernels;

#[derive(Debug)]
pub enum CompareKernelsOption {
    Full,
    Efficient,
}

#[derive(Debug)]
pub struct ChangeKernel {
    pub source: String,
    pub destination: String,
    pub hard_link: bool,
    pub mkinitcpio_preset: String,
    pub compare_kernels: Option<CompareKernelsOption>,
}

#[derive(Debug)]
pub struct DeployBootFiles {
    /// The block device file that contains
    /// the filesystem to deploy the boot files to.
    pub destination_block_device: Option<PathBuf>,
    /// The place to mount the block device / the
    /// place where the block device is mounted on.
    pub block_device_mount_point: PathBuf,
    /// The path to the directory containing the boot
    /// files to copy from, relative to the root
    /// directory.
    pub boot_files_source: PathBuf,
    /// The path to the directory to copy the boot files
    /// to, relative to the block device mount point
    /// / the root of the block device file system.
    pub boot_files_destination: PathBuf,
}

#[derive(Debug)]
pub enum OperationRequest {
    ChangeKernel(ChangeKernel),
    DeployBootFiles(DeployBootFiles),
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
/// how to carry out the user's request. This function does no validation
/// of whether the request is valid or not. The request may contain
/// invalid data, so it is the calling code's job to make sure that
/// the request makes sense. The request directly represents what the
/// user tells the program to do, which may or may not be valid.
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
pub fn interact_with_user(default_config_file: &str)
    -> Result<OperationRequest, UserInteractError>
{
    interact_with_user_provided_cmdline(
        default_config_file, env::args_os(),
    )
}
fn interact_with_user_provided_cmdline<C, T>(default_config_file: &str, cmdline: C)
    -> Result<OperationRequest, UserInteractError> where
    C: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli_args = Cli::try_parse_from(cmdline).map_err(|err| {
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
                ref compare_kernels,
            },
            ..
        } |
        Commands::UpdateKernel {
            shared_args: KernelCommandsArgs {
                hard_link: hard_link_flag,
                no_hard_link: no_hard_link_flag,
                ref compare_kernels,
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
                _ => panic!(),
            };

            let boot_kernel = config_contents.boot_kernel;
            let mkinitcpio_preset = config_contents.mkinitcpio_preset;
            let hard_link_default = config_contents.default_options.hard_link;

            let do_hard_link = match (hard_link_flag, no_hard_link_flag) {
                (false, false) => hard_link_default,
                (a, b) if a ^ b => hard_link_flag,
                _ => panic!(), // if (true, true)
            };
            let compare_kernels = match compare_kernels {
                None => match config_contents.default_options.compare_kernels {
                    CompareKernels::False => None,
                    CompareKernels::Full => Some(CompareKernelsOption::Full),
                    CompareKernels::Efficient => Some(CompareKernelsOption::Efficient),
                },
                Some(x) => match x.borrow() {
                    "false" => None,
                    "full" => Some(CompareKernelsOption::Full),
                    "efficient" => Some(CompareKernelsOption::Efficient),
                    _ => panic!(),
                },
            };

            OperationRequest::ChangeKernel(ChangeKernel {
                source: source_kernel_file,
                destination: boot_kernel,
                hard_link: do_hard_link,
                mkinitcpio_preset,
                compare_kernels,
            })
        },
        Commands::DeployBootFiles => {
            let deploy_config = config_contents.deploy_boot_files;
            OperationRequest::DeployBootFiles(DeployBootFiles {
                destination_block_device: deploy_config
                    .destination_block_device.map(PathBuf::from),
                block_device_mount_point: deploy_config.mount_point.into(),
                boot_files_source: deploy_config.source_directory.into(),
                boot_files_destination: deploy_config.destination_directory.into(),
            })
        },
    };

    Ok(operation_request)
}
