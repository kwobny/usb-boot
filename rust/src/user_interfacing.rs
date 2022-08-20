mod cmdline_parsing;
mod config_parsing;

use std::{fmt::Display, error::Error, io};

use clap::{Parser, ErrorKind};

use cmdline_parsing::{Cli, Commands, KernelCommandsArgs};
use config_parsing::ConfigContentsKey;

pub use config_parsing::ConfigFileInfo;

pub struct OperationRequest {
    pub request_kind: RequestKind,

    pub config_file_existed: bool,
}
pub enum RequestKind {
    ChangeKernel {
        source: String,
        destination: String,
        hard_link: bool,
        mkinitcpio_preset: String,
    }
}

/// All key fields are relative to the root of the config file.
/// Keys in tables use dots as separators between key components.
#[derive(Debug)]
pub enum InvalidInputKind {
    InvalidCommandLineArguments {
        details: clap::error::Error,
    },
    RequiredKeyMissingInConfig {
        key: String,
    },
    /// The command to execute requires a certain key from the
    /// config file, but there is no config file.
    NoConfigButKeyRequired {
        key: String,
    },
    /// An unrecognized key was present in the config.
    UnknownKeyInConfig {
        key: String,
    },
    InvalidConfigSyntax {
        cause: toml::de::Error,
    },
    /// A value in the config file was expected to be one type
    /// but was found to be another type.
    /// This error is detected when the config is parsed, regardless
    /// of if the program actually queries/needs the key/value.
    UnexpectedValueType {
        key: String,
        expected_type: &'static str,
        actual_type: &'static str,
    },
}
#[derive(Debug)]
pub enum IOErrorKind {
    /// Failed to access config file due to it not existing,
    /// permission errors, etc.
    ConfigAccessFailed {
        source: io::Error,
    },
    Other {
        source: anyhow::Error,
    },
}
#[derive(Debug)]
/// A type representing an error that occurred while trying to
/// interact with the user.
/// Use InvalidUserInput for when the CONTENT itself of the user's
/// input is invalid.
/// Use IOError for when the program COULD NOT ACCESS the content.
/// So for example, a config file not existing or readable would be
/// an IOError.
pub enum UserInteractError {
    /// An error caused by invalid input from the user.
    InvalidUserInput(InvalidInputKind),
    /// An error in communication with the user.
    IOError(IOErrorKind),
}
impl Display for UserInteractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserInteractError::InvalidUserInput(kind) => {
                write!(f, "invalid user input/configuration: ")?;
                match kind {
                    InvalidInputKind::InvalidCommandLineArguments { .. } =>
                        write!(f, "invalid command line arguments")?,
                    InvalidInputKind::RequiredKeyMissingInConfig {
                        key,
                    } => {
                        write!(f,
                            "the key \"{}\" is required to execute the \
                            specified command, but it is not present \
                            in the config file",
                            key,
                        )?;
                    },
                    InvalidInputKind::NoConfigButKeyRequired { key } =>
                        write!(f,
                            "the key \"{key}\" is required, but there is \
                            no config file",
                        )?,
                    InvalidInputKind::UnknownKeyInConfig {
                        key,
                    } => {
                        write!(f,
                            "the key \"{}\" was found in the config file,\
                            but it is not a valid/recognized config key",
                            key,
                        )?;
                    },
                    InvalidInputKind::InvalidConfigSyntax { .. } =>
                        write!(f,
                            "there is a syntax error in the config file",
                        )?,
                    InvalidInputKind::UnexpectedValueType {
                        key,
                        expected_type,
                        actual_type,
                    } => {
                        write!(f,
                            "expected a \"{expected_type}\" for the type \
                            of the value of the key \"{key}\", but got a \
                            \"{actual_type}\" instead",
                        )?;
                    },
                };
            },
            UserInteractError::IOError(kind) => {
                write!(f, "io error when trying to interact with user: ")?;
                match kind {
                    IOErrorKind::ConfigAccessFailed { source } =>
                        write!(f, "failed to access config file")?,
                    IOErrorKind::Other { source } =>
                        write!(f, "other error")?,
                };
            },
        }

        Ok(())
    }
}
impl std::error::Error for UserInteractError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UserInteractError::InvalidUserInput(kind) => match kind {
                InvalidInputKind::InvalidCommandLineArguments { details } =>
                    Some(details),
                InvalidInputKind::InvalidConfigSyntax { cause } =>
                    Some(cause),
                _ => None,
            },
            UserInteractError::IOError(kind) => match kind {
                IOErrorKind::ConfigAccessFailed { source } => Some(source),
                IOErrorKind::Other { source } =>
                    Some(source.as_ref()),
            }
        }
    }
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
            ErrorKind::Io | ErrorKind::Format =>
                UserInteractError::IOError(IOErrorKind::Other {
                    source: err.into(),
                }),
            _ => UserInteractError::InvalidUserInput(
                InvalidInputKind::InvalidCommandLineArguments {
                    details: err,
                }
            ),
        }
    })?;

    let config_file = match cli_args.config {
        Some(ref x) => &x,
        None => config_file_info.default_file,
    };
    let config_must_exist = config_file_info.file_must_exist || cli_args.config_must_exist;
    let config_contents = config_parsing::parse_config(
        config_file_info,
        config_must_exist,
        config_file,
    )?;

    macro_rules! get_config_key {
        ($type:ty, $key:tt) => {
            config_contents.get::<$type>(ConfigContentsKey::$key)?
        };
        ($key:tt) => {
            config_contents.get(ConfigContentsKey::$key)?
        };
    }

    let operation_kind = match cli_args.command {
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
                    get_config_key!(String, UpstreamKernel).to_owned()
                },
            };

            let boot_kernel = get_config_key!(String, BootKernel)
                .to_owned();
            let mkinitcpio_preset = get_config_key!(String, MkinitcpioPreset)
                .to_owned();
            let hard_link_default = *get_config_key!(DefaultHardLink);

            let do_hard_link = match (hard_link_flag, no_hard_link_flag) {
                (false, false) => hard_link_default,
                (a, b) if a ^ b => hard_link_flag,
                _ => panic!(), // if (true, true)
            };

            RequestKind::ChangeKernel {
                source: source_kernel_file,
                destination: boot_kernel,
                hard_link: do_hard_link,
                mkinitcpio_preset,
            }
        },
    };

    Ok(OperationRequest {
        request_kind: operation_kind,
        config_file_existed: config_contents.config_file_exists(),
    })
}
