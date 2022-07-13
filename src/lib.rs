mod utils;

use std::{fs, process::Command};
use anyhow::Result;
use common::AggregateError;

pub struct Config {
    pub transform_parameters: CmdlineTransformParameters,
}

struct KexecArgs {
    kernel: String,
    initrd: String,
    command_line: String,
}

pub struct CmdlineTransformParameters {
    pub additional_args: String,
    pub kernel: String,
    pub initrd: String,
}

#[derive(thiserror::Error, Debug)]
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
fn transform_command_line(command_line: &str, transform_parameters: CmdlineTransformParameters) -> Result<KexecArgs, AggregateError<TransformCommandLineError>> {
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
        if let Some((key, value)) = parameter.split_once('=') {
            if key == transform_parameters.additional_args {
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

#[derive(thiserror::Error, Debug)]
pub enum ParseArgsError {
    #[error("the option \"{key}\" was given but no value was provided for it")]
    KeyWithoutValue {
        key: String,
    },
    #[error("the option \"{option}\" was set multiple times")]
    OptionSetMultipleTimes {
        option: String,
    },
    #[error("unknown argument: {argument}")]
    UnknownArgument {
        argument: String,
    },
    #[error("the required option \"{option}\" was not provided")]
    MissingRequiredOption {
        option: String,
    },
}

/// This function parses the command line arguments.
/// There must be three options specified, each setting a particular key.
/// Each option must be in the form of "key=value" (1 argument) or "key value" (2 arguments).
pub fn parse_args(args: impl IntoIterator<Item=String>, option_names: CmdlineTransformParameters) -> Result<Config, AggregateError<ParseArgsError>> {
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

    Ok(Config {
        transform_parameters: CmdlineTransformParameters {
            additional_args: additional_args.unwrap(),
            kernel: kernel.unwrap(),
            initrd: initrd.unwrap(),
        }
    })
}
