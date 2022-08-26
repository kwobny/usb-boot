//! This module is responsible for parsing the config file
//! for this program. That is all this module does. Dealing
//! with the config file is the sole responsibility of this
//! module.

use std::fs;
use std::borrow::Cow;
use serde::Deserialize;
use super::UserInteractError;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigContents {
    pub boot_kernel: String,
    pub upstream_kernel: String,
    pub mkinitcpio_preset: String,

    #[serde(default)]
    pub default_options: DefaultOptions,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct DefaultOptions {
    pub hard_link: bool,
    pub compare_kernels: CompareKernels,
}
impl Default for DefaultOptions {
    fn default() -> Self {
        DefaultOptions {
            hard_link: false,
            compare_kernels: CompareKernels::False,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CompareKernels {
    False,
    Full,
    Efficient,
}

/// This function parses the config file for this program,
/// and returns a struct representing its contents.
/// This function reads nothing but the config file.
pub fn parse_config(config_file: &str) ->
    Result<ConfigContents, UserInteractError>
{
    let config_file_string = fs::read_to_string(config_file)
        .map_err(|err|
            UserInteractError::ConfigAccessFailed {
                source: err,
            }
        )?;

    toml::from_str(&config_file_string).map_err(|err|
        UserInteractError::ConfigParseError {
            source: err,
        }
    )
}
