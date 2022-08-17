use std::fs;

use toml::{Value, value::Table};
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

struct KeyHandler<'a> {
    key: &'a str,
    handler: &'a mut dyn FnMut(&mut ConfigContents, Value) -> Result<(), UserInteractError>,
}
fn parse_config_recursive (
    config_contents: &mut ConfigContents,
    config_tree: Table,
    possible_keys: &mut [KeyHandler],
) -> Result<(), UserInteractError> {
    'outer: for (key, value) in config_tree {
        for handler in &mut *possible_keys {
            if key == handler.key {
                (handler.handler)(config_contents, value)?;
                continue 'outer;
            }
        }
        return Err(UserInteractError::UserInputError);
    }
    return Ok(());
}

#[derive(Clone, Copy, Debug)]
pub struct ConfigFileInfo<'a> {
    pub default_file: &'a str,

    pub default_options_table_name: &'a str,
    pub default_hard_link_key: &'a str,

    pub boot_kernel_key: &'a str,
    pub upstream_kernel_key: &'a str,
    pub mkinitcpio_preset_key: &'a str,
}

struct ConfigContents {
    boot_kernel: Option<String>,
    upstream_kernel: Option<String>,
    mkinitcpio_preset: Option<String>,

    hard_link_default: Option<bool>,
}

fn parse_config(config_info: ConfigFileInfo, config_root: Value) ->
Result<ConfigContents, UserInteractError> {
    macro_rules! unwrap_variant {
        ($value:expr, $variant:path) => {
            match $value {
                $variant(x) => x,
                _ => return Err(UserInteractError::UserInputError),
            }
        };
    }
    macro_rules! set_config_content {
        ($field:ident, $expected_data_type:path) => {
            &mut |contents, value| {
                let value = unwrap_variant!(value, $expected_data_type);
                contents.$field = Some(value);
                Ok(())
            }
        };
    }

    let mut config_contents = ConfigContents {
        boot_kernel: None,
        upstream_kernel: None,
        mkinitcpio_preset: None,

        hard_link_default: None,
    };

    let root_table = match config_root {
        Value::Table(x) => x,
        _ => return Err(UserInteractError::UserInputError),
    };

    parse_config_recursive(&mut config_contents, root_table, &mut [
        KeyHandler {
            key: config_info.boot_kernel_key,
            handler: set_config_content!(boot_kernel, Value::String),
        },
        KeyHandler {
            key: config_info.upstream_kernel_key,
            handler: set_config_content!(upstream_kernel, Value::String),
        },
        KeyHandler {
            key: config_info.mkinitcpio_preset_key,
            handler: set_config_content!(mkinitcpio_preset, Value::String),
        },
        KeyHandler {
            key: config_info.default_options_table_name,
            handler: &mut |contents, value| {
                let default_options_table =
                    unwrap_variant!(value, Value::Table);
                parse_config_recursive(contents, default_options_table, &mut [
                    KeyHandler {
                        key: config_info.default_hard_link_key,
                        handler: set_config_content!(hard_link_default, Value::Boolean),
                    },
                ])
            },
        },
    ])?;

    Ok(config_contents)
}

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
    let config_contents = parse_config(config_file_info, config)?;

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
