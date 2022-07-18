mod utils;

use std::{fs, process::Command};
use anyhow::Result;
use common::{AggregateError, size_based_container::SizeBasedContainer};

#[derive(Debug, PartialEq)]
pub struct Config {
    pub transform_parameters: UniqueTransformParameters,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TransformParameters {
    pub additional_args: String,
    pub kernel: String,
    pub initrd: String,
}
#[derive(PartialEq, Debug, Clone)]
pub struct UniqueTransformParameters(TransformParameters);
impl TryFrom<TransformParameters> for UniqueTransformParameters {
    type Error = ();
    fn try_from(transform_parameters: TransformParameters) -> Result<Self, ()> {
        if elements_are_unique(&[
            &transform_parameters.additional_args,
            &transform_parameters.kernel,
            &transform_parameters.initrd,
        ]) {
            Ok(UniqueTransformParameters(transform_parameters))
        } else {
            Err(())
        }
    }
}

/// Tests whether there are any two elements in the slice that are equal
/// to each other.
/// Returns true if every element in the slice is unique, i.e. there are no two elements in
/// the slice that are equal to each other.
/// Returns false if there are at least two elements in the slice that are equal to each other.
fn elements_are_unique<T: Eq>(elements: &[T]) -> bool {
    for base in 0..(elements.len()-1) {
        let compare_to = &elements[base];
        for elem in &elements[base+1..] {
            if *elem == *compare_to {
                return false;
            }
        }
    }
    true
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum TransformCommandLineError {
    #[error("missing required parameter: {parameter}")]
    MissingRequiredParameter {
        parameter: String,
    },
    #[error("required parameter set multiple times: {parameter}")]
    RequiredParameterSetMultipleTimes {
        parameter: String,
    },
}
#[derive(Debug, PartialEq)]
struct KexecArgs {
    kernel: String,
    initrd: String,
    command_line: String,
}
fn transform_command_line(command_line: &str, transform_parameters: UniqueTransformParameters) -> Result<KexecArgs, AggregateError<TransformCommandLineError>> {
    let transform_parameters = transform_parameters.0;

    let mut new_cmdline = String::new();
    let mut kernel: Option<&str> = None;
    let mut initrd: Option<&str> = None;

    let mut errors = Vec::new();

    // For every parameter in the kernel command line, check if the key matches
    // one of the transform parameters. If it does, do the corresponding special action.
    // If not, then add it to the new_cmdline.
    'args_loop: for parameter in utils::split_at_unquoted_spaces(command_line) {
        // Parameter only matches if it is in the form of "key=value"
        // and the key is equal to one of the transform parameters.
        if let Some((key, mut value)) = parameter.split_once('=') {
            if key == transform_parameters.additional_args {
                // If the additional arguments are wrapped in single quotes or double quotes,
                // remove the outer pair of quotes before pushing the arguments onto the new
                // command line.
                let mut is_wrapped = false;
                for c in ['\'', '"'] {
                    is_wrapped = is_wrapped ||
                        (value.starts_with(c) && value.ends_with(c));
                }
                if is_wrapped {
                    let mut chars = value.chars();
                    chars.next();
                    chars.next_back();
                    value = chars.as_str();
                }

                new_cmdline.push_str(value);
                new_cmdline.push(' ');
                continue 'args_loop;
            }
            else {
                for (key_name, set_var) in [
                    (&transform_parameters.kernel, &mut kernel),
                    (&transform_parameters.initrd, &mut initrd),
                ] {
                    if key == key_name {
                        if set_var.is_some() {
                            errors.push(TransformCommandLineError::RequiredParameterSetMultipleTimes {
                                parameter: key_name.clone(),
                            });
                        }
                        *set_var = Some(value);
                        continue 'args_loop;
                    }
                }
            }
        }
        // Parameter did not match any of the keys.
        // So just add it onto the new cmdline.
        new_cmdline.push_str(parameter);
        new_cmdline.push(' ');
    }

    // If kernel or initramfs are not provided on the kernel command line,
    // return an error.
    for (value, parameter_str) in [
        (kernel, transform_parameters.kernel),
        (initrd, transform_parameters.initrd),
    ] {
        if value.is_none() {
            errors.push(TransformCommandLineError::MissingRequiredParameter {
                parameter: parameter_str,
            });
        }
    }
    if let Ok(aggregate) = AggregateError::try_from(errors) {
        return Err(aggregate);
    }

    Ok(KexecArgs {
        command_line: new_cmdline,
        kernel: kernel.unwrap().to_string(),
        initrd: initrd.unwrap().to_string(),
    })
}

pub fn run(config: Config) -> Result<()> {
    // Get current kernel command line
    let kernel_command_line = fs::read_to_string("/proc/cmdline")?;

    // Transform command line
    let new_command_line = transform_command_line(&kernel_command_line, config.transform_parameters)?;

    // Invoke kexec -l
    let success = Command::new("kexec")
        .args([
            "-l",
            &new_command_line.kernel,
            &format!("--initrd={}", new_command_line.initrd),
            &format!("--append={}", new_command_line.command_line),
        ])
        .spawn()?
        .wait()?
        .success();
    if !success {
        anyhow::bail!("failed to kexec load");
    }

    // Invoke systemctl kexec
    let success = Command::new("systemctl")
        .arg("kexec")
        .spawn()?
        .wait()?
        .success();
    if !success {
        anyhow::bail!("failed to systemctl kexec");
    }

    Ok(())
}

/// Represents an error that occurred while executing the [`parse_args`] function.
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseArgsError {
    /// The key for an option was given, but no value was provided for it.<br>
    /// This occurs when the option is specified in the form of `key value` (2 arguments),
    /// but `key` is the last argument and there is no additional argument for the value after that.<br>
    /// All options must have both a key and value.
    ///
    /// # Example
    ///     --option1 value1 --option2=value2 --option3
    /// Here, `--option3` is given, but no value is provided for it.
    #[error("the option \"{key}\" was given but no value was provided for it")]
    KeyWithoutValue {
        key: String,
    },
    /**
     * An option was given multiple times on the command line. All options should be given exactly
     * once.
     *
     * # Example
     *     --option1=value1 --option2=value2 --option1=value3
     * Here, `--option1` is given twice. This is invalid.
     */
    #[error("the option \"{option}\" was set multiple times")]
    OptionSetMultipleTimes {
        option: String,
    },
    /**
     * An argument could not be processed because it was not a known option. All arguments have to
     * specify options that are one of the 3 fields in the `option_names` parameter.
     *
     * # Example
     *     --option1=value1 --option2=value2 --unknown-option=value3
     * Assuming none of the fields in the `option_names` parameter of the [`parse_args`] function
     * contained `--unknown-option`, this command line would be invalid because `--unknown-option`
     * is not a valid option.
     */
    #[error("unknown argument: {argument}")]
    UnknownArgument {
        argument: String,
    },
    /**
     * A required option was not specified on the command line. All 3 options indicated by the 3
     * fields of the `option_names` parameter must be specified on the command line.
     *
     * # Example
     *     --additional-args-option=value1 --initrd-option=value2
     * Assuming the kernel field of the `option_names` parameter is `--kernel-option`, this example
     * would result in an error because the `--kernel-option` option is not given on the command
     * line.
     */
    #[error("the required option \"{option}\" was not provided")]
    MissingRequiredOption {
        option: String,
    },
    #[error("multiple options were set to the same value")]
    MultipleOptionSameValue,
}

/// This function parses the command line arguments of this program.
/// There must be exactly three options specified, with one option for each option name / key in
/// the `option_names` parameter.
/// Each option must be in the form of "key=value" (1 argument) or "key value" (2 arguments).
/// The 3 options are the strings stored in the 3 fields of the `option_names` parameter of
/// this function.
///
/// # Errors:
///   - All 3 options are required. If one or more of the options are missing, the function raises a
///     [`MissingRequiredOption`](ParseArgsError::MissingRequiredOption) for each missing option.
///   - asdf
pub fn parse_args(args: impl IntoIterator<Item=String>, option_names: UniqueTransformParameters) -> Result<Config, AggregateError<ParseArgsError>> {
    let option_names = option_names.0;

    let mut additional_args = None;
    let mut kernel = None;
    let mut initrd = None;

    let mut errors = Vec::new();

    // This is an array containing mappings of possible options,
    // and variables to set to the value of the option if the option
    // matches.
    let mut mappings = [
        (option_names.additional_args, &mut additional_args),
        (option_names.kernel, &mut kernel),
        (option_names.initrd, &mut initrd),
    ];

    // This is basically a for loop over the args argument.
    // It is done this way to allow access to the iterator.
    let mut args = args.into_iter();
    'args_loop: loop {
        let arg = match args.next() {
            Some(x) => x,
            None => break,
        };
        // For each possible option, check if the argument matches the option.
        for (key_name, set_var) in mappings.iter_mut() {
            // There are two ways to specify an option with value on the command line.
            // Check if the current argument matches the current option
            // in any of the two ways, and set the value variable to the option's value
            // if it matches.
            // If it does not match in any of the two ways, move on to the next possible option.
            let value;
            if arg == **key_name {
                // The option is specified in the form of "--option value", with the key
                // in one argument and the value of the option in the next.
                // Get the next argument and set the value variable to that.
                value = match args.next() {
                    Some(x) => x,
                    None => {
                        // If there is no next argument (the iterator is exhausted),
                        // raise an error and stop iteration over arguments.
                        errors.push(ParseArgsError::KeyWithoutValue { key: key_name.clone() });
                        break 'args_loop;
                    },
                };
            }
            else {
                // Check if the option is specified in the
                // form of "--option=value", all in one argument.
                let beginning_part = format!("{}=", key_name);
                if !arg.starts_with(&beginning_part) {
                    continue;
                }
                value = arg[beginning_part.len()..].to_string();
            }

            // If the corresponding variable to set has
            // already been set, this means the current option
            // has been specified more than once.
            // This is invalid, raise an error.
            if set_var.is_some() {
                errors.push(ParseArgsError::OptionSetMultipleTimes {
                    option: key_name.clone(),
                });
            }
            **set_var = Some(value);
            continue 'args_loop;
        }

        // The current argument did not match any of the possible options,
        // raise an error.
        errors.push(ParseArgsError::UnknownArgument { argument: arg });
    }

    // For each required option, check if the option was set.
    // If not, raise an error.
    for (key_name, set_var) in mappings {
        if set_var.is_none() {
            errors.push(ParseArgsError::MissingRequiredOption { option: key_name.clone() });
        }
    }

    // Check if any errors have been raised.
    // If so, exit the function with an error.
    if let Ok(aggregate) = AggregateError::try_from(errors) {
        return Err(aggregate);
    }

    let unique_transform_parameters = TransformParameters {
        additional_args: additional_args.unwrap(),
        kernel: kernel.unwrap(),
        initrd: initrd.unwrap(),
    }.try_into();

    match unique_transform_parameters {
        Ok(x) => Ok(Config {
            transform_parameters: x,
        }),
        Err(_) => Err(SizeBasedContainer::from_single(ParseArgsError::MultipleOptionSameValue)
                      .try_into()
                      .unwrap()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elements_are_unique() {
        let test_cases: &[(&[i32], bool)] = &[
            (&[2, 5, 3], true),
            (&[2, 2, 3], false),
            (&[2, 5, 2], false),
            (&[2, 3, 3], false),

            (&[2, 5, 3, 10], true),
            (&[4, 4, 4, 4], false),
        ];
        for (elements, are_unique) in test_cases {
            assert_eq!(elements_are_unique(elements), *are_unique);
        }
    }

    #[test]
    fn unique_transform_parameters_try_from() {
        let unique = TransformParameters {
            additional_args: "hello".to_string(),
            kernel: "goodbye".to_string(),
            initrd: "cheese".to_string(),
        };
        let not_unique = TransformParameters {
            additional_args: "hello".to_string(),
            kernel: "hello".to_string(),
            initrd: "cheese".to_string(),
        };

        assert_eq!(UniqueTransformParameters::try_from(unique.clone()), Ok(UniqueTransformParameters(unique)));
        assert_eq!(UniqueTransformParameters::try_from(not_unique), Err(()));
    }

    #[test]
    fn test_transform_command_line() {
        let transform_parameters: UniqueTransformParameters = TransformParameters {
            additional_args: "--asdf".to_string(),
            kernel: "--kernel-lol".to_string(),
            initrd: "--see-initrd".to_string(),
        }.try_into().unwrap();

        let working_command_line = r#"2312 --kernel-lol=tty390=zxcvr lol=5 --asdf="tee=4 sasd=1 83      dfds 983=5=das"     see 3 cx=8ijds --see-initrd=--kernel-lol"#;
        let working_expected = Ok(KexecArgs {
            kernel: "tty390=zxcvr".to_string(),
            initrd: "--kernel-lol".to_string(),
            command_line: "2312 lol=5 tee=4 sasd=1 83      dfds 983=5=das see 3 cx=8ijds ".to_string(),
        });

        let missing_kernel_command_line = r#"2312 --kernel-lol lol=5 --asdf="tee=4 sasd=1 83      dfds 983=5=das"     see 3 cx=8ijds --see-initrd=--kernel-lol"#;
        let missing_kernel_expected = Err(
            SizeBasedContainer::from_single(
                TransformCommandLineError::MissingRequiredParameter {
                    parameter: "--kernel-lol".to_string(),
                }
            ).try_into().unwrap()
        );

        let set_multiple_times_command_line = r#"2312 --kernel-lol=tty390=zxcvr lol=5 --see-initrd="tee=4 sasd=1 83      dfds 983=5=das"     see 3 cx=8ijds --see-initrd=--kernel-lol"#;
        let set_multiple_times_expected = Err(
            SizeBasedContainer::from_single(
                TransformCommandLineError::RequiredParameterSetMultipleTimes {
                    parameter: "--see-initrd".to_string(),
                }
            ).try_into().unwrap()
        );

        let no_additional_args_command_line = r#"lololololol --kernel-lol= --see-initrd="#;
        let no_additional_args_expected = Ok(KexecArgs {
            kernel: "".to_string(),
            initrd: "".to_string(),
            command_line: "lololololol ".to_string(),
        });

        let additional_args_quotes_command_line = r#"an_option="32 cxds" 'jcxn ewi' --kernel-lol= --see-initrd= --asdf="lol=3" ewji  --asdf=""fdji   e32 cx=3"" --asdf="'hello goodbye c32=gfda'" --asdf="x="hello    fdjs"  id=4"   ejkncxv"#;
        let additional_args_quotes_expected = Ok(KexecArgs {
            kernel: "".to_string(),
            initrd: "".to_string(),
            command_line: r#"an_option="32 cxds" 'jcxn ewi' lol=3 ewji "fdji   e32 cx=3" 'hello goodbye c32=gfda' x="hello    fdjs"  id=4 ejkncxv "#.to_string(),
        });

        for (command_line, expected) in [
            (working_command_line, working_expected),
            (missing_kernel_command_line, missing_kernel_expected),
            (set_multiple_times_command_line, set_multiple_times_expected),
            (no_additional_args_command_line, no_additional_args_expected),
            (additional_args_quotes_command_line, additional_args_quotes_expected),
        ] {
            assert_eq!(transform_command_line(command_line, transform_parameters.clone()), expected);
        }
    }

    #[test]
    fn test_parse_args() {
        let option_names: UniqueTransformParameters = TransformParameters {
            additional_args: "--add-args".to_string(),
            kernel: "--popcorn-kernel".to_string(),
            initrd: "--initramfs".to_string(),
        }.try_into().unwrap();

        let working_command_line = "--add-args --cpio --popcorn-kernel=--casdf --initramfs --9anime.to";
        let working_expected = Ok(
            Config {
                transform_parameters: TransformParameters {
                    additional_args: "--cpio".to_string(),
                    kernel: "--casdf".to_string(),
                    initrd: "--9anime.to".to_string(),
                }.try_into().unwrap(),
            }
        );

        let excessive_args_command_line = "--add-args --cpio --add-rgs --popcorn-kernel=--casdf --initramfs --9anime.to";
        let excessive_args_expected = Err(
            SizeBasedContainer::from_single(
                ParseArgsError::UnknownArgument {
                    argument: "--add-rgs".to_string(),
                }
            ).try_into().unwrap()
        );

        let duplicate_option_command_line = "--add-args --cpio --popcorn-kernel=--casdf --add-args hello --initramfs --9anime.to";
        let duplicate_option_expected = Err(
            SizeBasedContainer::from_single(
                ParseArgsError::OptionSetMultipleTimes {
                    option: "--add-args".to_string(),
                }
            ).try_into().unwrap()
        );

        let key_without_value_command_line = "--add-args --cpio --popcorn-kernel=--casdf --initramfs";
        let key_without_value_expected = Err(
            SizeBasedContainer::from_single(
                ParseArgsError::KeyWithoutValue {
                    key: "--initramfs".to_string(),
                }
            ).try_into().unwrap()
        );

        let missing_options_command_line = "--popcorn-kernel=--casdf --initramfs --9anime.to";
        let missing_options_expected = Err(
            SizeBasedContainer::from_single(
                ParseArgsError::MissingRequiredOption {
                    option: "--add-args".to_string(),
                }
            ).try_into().unwrap()
        );

        let same_value_command_line = "--add-args --cpio --popcorn-kernel=--9anime.to --initramfs --9anime.to";
        let same_value_expected = Err(
            SizeBasedContainer::from_single(
                ParseArgsError::MultipleOptionSameValue
            ).try_into().unwrap()
        );

        for (command_line, expected) in [
            (working_command_line, working_expected),
            (excessive_args_command_line, excessive_args_expected),
            (duplicate_option_command_line, duplicate_option_expected),
            (key_without_value_command_line, key_without_value_expected),
            (missing_options_command_line, missing_options_expected),
            (same_value_command_line, same_value_expected),
        ] {
            assert_eq!(parse_args(command_line.split_whitespace().map(|x| x.to_string()), option_names.clone()), expected);
        }
    }
}
