//! This module is responsible for parsing the config file
//! for this program. That is all this module does. Dealing
//! with the config file is the sole responsibility of this
//! module.

use std::fs;

use toml::{Value, value::Table};

use super::UserInteractError;

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

/// This function reads the file provided, and returns
/// the root table of the config.
fn get_config_from_file(config_file: &str) -> Result<Value, UserInteractError> {
    fs::read_to_string(config_file)
        .map_err(|_| UserInteractError::IOError)?
        .parse::<Value>()
        .map_err(|_| UserInteractError::UserInputError)
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

pub struct ConfigContents {
    pub boot_kernel: Option<String>,
    pub upstream_kernel: Option<String>,
    pub mkinitcpio_preset: Option<String>,

    pub hard_link_default: Option<bool>,
}

/// This function parses the config file for this program,
/// and returns a struct representing its contents.
/// This function reads nothing but the config file.
pub fn parse_config(config_info: ConfigFileInfo, config_file: &str) ->
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

    let config_root = get_config_from_file(config_file)?;
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
