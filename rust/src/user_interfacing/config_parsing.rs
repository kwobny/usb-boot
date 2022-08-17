//! This module is responsible for parsing the config file
//! for this program. That is all this module does. Dealing
//! with the config file is the sole responsibility of this
//! module.

use std::{fs, collections::HashMap};

use toml::{Value, value::Table};

use super::{UserInteractError, InvalidInputKind};

macro_rules! direct_mapping {
    ($config_key:expr => $contents_key:tt) => {
        ConfigMapping {
            config_key: $config_key,
            kind: MappingKind::Direct(ConfigContentsKey::$contents_key),
        }
    };
}
macro_rules! nested_mapping {
    ($config_key:expr, $($sub_mapping:expr),* $(,)?) => {
        ConfigMapping {
            config_key: $config_key,
            kind: MappingKind::Nest(Nest {
                mappings: Box::new([
                    $($sub_mapping)*,
                ]),
            }),
        }
    };
}

macro_rules! impl_from_toml_value {
    // The parameter $type_str represents the type of $impl_target.
    ($($impl_target:ty, $type_str:literal, $value_variant:path),+ $(,)?) => {
        $(impl FromTomlValue for $impl_target {
            fn convert(value: &Value) -> Result<&Self, &'static str> {
                match value {
                    $value_variant(v) => Ok(v),
                    _ => Err($type_str),
                }
            }
        })+
    };
}

//------------------ BEGIN CONFIG CONSTANTS AREA -------------------

#[derive(Clone, Copy, Debug)]
pub struct ConfigFileInfo<'a> {
    pub default_file: &'a str,

    pub default_options_table_name: &'a str,
    // Each of the default options keys must be relative to the
    // default options table name, not the root table.
    pub default_hard_link_key: &'a str,

    pub boot_kernel_key: &'a str,
    pub upstream_kernel_key: &'a str,
    pub mkinitcpio_preset_key: &'a str,
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum ConfigContentsKey {
    BootKernel,
    UpstreamKernel,
    MkinitcpioPreset,

    DefaultHardLink,
}

fn config_mapping_from_config_info(config_info: ConfigFileInfo) -> Nest {
    Nest {
        mappings: Box::new([
            direct_mapping!(
                config_info.boot_kernel_key.to_string()
                => BootKernel
            ),
            direct_mapping!(
                config_info.upstream_kernel_key.to_string()
                => UpstreamKernel
            ),
            direct_mapping!(
                config_info.mkinitcpio_preset_key.to_string()
                => MkinitcpioPreset
            ),
            nested_mapping!(
                config_info.default_options_table_name.to_string(),
                direct_mapping!(
                    config_info.default_hard_link_key.to_string()
                    => DefaultHardLink
                ),
            ),
        ]),
    }
}

impl_from_toml_value![
    String, "String", Value::String,
    bool, "Boolean", Value::Boolean,
    Table, "Table", Value::Table,
];

//---------------- END CONFIG CONSTANTS AREA ------------------

struct Nest {
    mappings: Box<[ConfigMapping]>,
}
enum MappingKind {
    Direct(ConfigContentsKey),
    Nest(Nest),
}
struct ConfigMapping {
    // This is the key relative to the table that the key is under,
    // not to the root of the config file.
    config_key: String,
    kind: MappingKind,
}

trait FromTomlValue {
    // The error variant of the return type
    // should be a string representing the expected value type,
    // i.e. the type of Self.
    fn convert(value: &Value) -> Result<&Self, &'static str>;
}
fn unwrap_toml_value<R, T>(key: T, value: &Value)
    -> Result<&R, UserInteractError> where
    R: FromTomlValue,
    T: FnOnce() -> String,
{
    R::convert(value).map_err(|expected| UserInteractError::InvalidUserInput(
        InvalidInputKind::UnexpectedValueType {
            key: key(),
            expected_type: expected,
            actual_type: value.type_str(),
        }
    ))
}

/// This function reads the file provided, and returns
/// the root table of the config.
fn get_config_from_file(config_file: &str) -> Result<Value, UserInteractError> {
    fs::read_to_string(config_file)
        .map_err(|err| UserInteractError::IOError {
            cause: err.into(),
        })?
        .parse::<Value>()
        .map_err(|err| UserInteractError::InvalidUserInput(
            InvalidInputKind::InvalidConfigSyntax {
                cause: err,
            }
        ))
}

fn parse_config_recursive(
    mappings: &Nest,
    current_key_section: &str,
    hash_map: &mut HashMap<ConfigContentsKey, Value>,
    table: Value,
) -> Result<(), UserInteractError> {
    let mappings = &mappings.mappings;
    let table = Table::convert(&table)
        .map_err(|expected|
            UserInteractError::InvalidUserInput(
                InvalidInputKind::UnexpectedValueType {
                    key: current_key_section.to_string(),
                    expected_type: expected,
                    actual_type: table.type_str(),
                }
            )
        )?;

    for (key, value) in table {
        let is_valid_key = mappings.iter().find(
            |mapping| mapping.config_key == *key
        );
        let is_valid_key = is_valid_key.ok_or_else(||
            UserInteractError::InvalidUserInput(
                InvalidInputKind::UnknownKeyInConfig {
                    key: key.to_owned(),
                }
            )
        )?;
        match &is_valid_key.kind {
            MappingKind::Direct(to) => {
                hash_map.insert(*to, value.clone());
            },
            MappingKind::Nest(nested_mappings) => {
                parse_config_recursive(
                    nested_mappings,
                    &is_valid_key.config_key,
                    hash_map,
                    value.clone(),
                ).map_err(|err|
                    match err {
                        UserInteractError::InvalidUserInput(
                            InvalidInputKind::UnexpectedValueType {
                                key: sub_key,
                                expected_type,
                                actual_type,
                            }
                        ) => UserInteractError::InvalidUserInput(
                            InvalidInputKind::UnexpectedValueType {
                                key: format!("{}.{}", current_key_section, sub_key),
                                expected_type,
                                actual_type,
                            }
                        ),
                        x => x,
                    }
                )?;
            },
        };
    }

    Ok(())
}

/// This function parses the config file for this program,
/// and returns a struct representing its contents.
/// This function reads nothing but the config file.
pub fn parse_config(config_info: ConfigFileInfo, config_file: &str)
    -> Result<ConfigContents, UserInteractError>
{
    let mut hash_map = HashMap::new();
    let config_mapping = config_mapping_from_config_info(config_info);
    let config_root = get_config_from_file(config_file)?;

    parse_config_recursive(&config_mapping, "", &mut hash_map, config_root)?;

    Ok(ConfigContents {
        data: hash_map,
        config_mapping,
    })
}

pub struct ConfigContents {
    data: HashMap<ConfigContentsKey, Value>,
    config_mapping: Nest,
}
impl ConfigContents {
    fn get_key_name_from_mapping_recursive<'a>(
        nest_name: &'a str,
        nest: &'a Nest,
        key: ConfigContentsKey,
    ) -> Option<String> {
        for mapping in nest.mappings.iter() {
            match &mapping.kind {
                MappingKind::Direct(enum_key) => {
                    if *enum_key == key {
                        return Some(mapping.config_key.to_owned());
                    }
                },
                MappingKind::Nest(sub_nest) => {
                    let sub_key_name = ConfigContents::get_key_name_from_mapping_recursive(
                        &mapping.config_key,
                        &sub_nest,
                        key,
                    );
                    let name_including_current = sub_key_name
                        .map(|x| format!("{}.{}", mapping.config_key, x));
                    if name_including_current.is_some() {
                        return name_including_current;
                    }
                },
            }
        }

        None
    }
    fn get_key_name_from_enum_key(&self, key: ConfigContentsKey) -> String {
        ConfigContents::get_key_name_from_mapping_recursive("", &self.config_mapping, key)
            .expect(&format!("The key {:?} is a variant of ConfigContentsKey,\
            but there exists no corresponding entry for it in the config key\
            mapping.", key))
    }
    pub fn get<T>(&self, key: ConfigContentsKey) -> Result<&T, UserInteractError> where
        T: FromTomlValue,
    {
        let possible_value = self.data.get(&key);
        let value = possible_value.ok_or_else(||
            UserInteractError::InvalidUserInput(
                InvalidInputKind::RequiredKeyMissingInConfig {
                    key: self.get_key_name_from_enum_key(key),
                }
            )
        )?;

        let converted_value = unwrap_toml_value(
            || self.get_key_name_from_enum_key(key),
            value,
        )?;

        Ok(converted_value)
    }
}
