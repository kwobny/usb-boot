mod utils;

use std::{fs, process::Command};

use thiserror::Error;
use anyhow::Result;

pub struct Config<'a> {
    pub transform_parameters: CmdlineTransformParameters<'a>,
}

struct KexecArgs {
    kernel: String,
    initrd: String,
    command_line: String,
}

pub struct CmdlineTransformParameters<'a> {
    pub additional_args: &'a str,
    pub kernel: &'a str,
    pub initrd: &'a str,
}

#[derive(Error, Debug)]
enum TransformCmdlineError {
    #[error("missing one or more required parameters (kernel/initramfs)")]
    MissingParameters,
}

fn transform_command_line(command_line: &str, transform_parameters: CmdlineTransformParameters) -> Result<KexecArgs> {
    let mut new_cmdline = String::new();
    let mut kernel: Option<&str> = None;
    let mut initrd: Option<&str> = None;

    // Return the value of a parameter if the parameter's key is the same
    // as the provided key.
    // If the key is not the same, or the parameter is not in the form of
    // key=value, then return None.
    fn value_if_key<'a>(parameter: &'a str, key: &'a str) -> Option<&'a str> {
        let key = format!("{}=", key);
        if parameter.starts_with(&key) {
            let value = &parameter[key.len()..];
            return Some(value);
        }
        return None;
    }
    // For every parameter in the kernel command line, check if the key matches
    // one of the transform parameters. If it does, do the corresponding special action.
    // If not, then add it to the new_cmdline.
    for parameter in utils::split_at_unquoted_spaces(command_line) {
        if let Some(value) = value_if_key(parameter, transform_parameters.additional_args) {
            new_cmdline.push_str(value);
            new_cmdline.push(' ');
        }
        else if let Some(value) = value_if_key(parameter, transform_parameters.kernel) {
            kernel = Some(value);
        }
        else if let Some(value) = value_if_key(parameter, transform_parameters.initrd) {
            initrd = Some(value);
        }
        else {
            // Parameter did not match any of the keys.
            // So just add it onto the new cmdline.
            new_cmdline.push_str(parameter);
            new_cmdline.push(' ');
        }
    }

    // If kernel or initramfs are not provided on the kernel command line,
    // return an error.
    macro_rules! unwrap_option {
        ($value:expr) => {
            $value.ok_or(TransformCmdlineError::MissingParameters)?
        };
    }

    let kernel_binary = unwrap_option!(kernel);
    let initramfs = unwrap_option!(initrd);

    Ok(KexecArgs {
        command_line: new_cmdline,
        kernel: kernel_binary.to_string(),
        initrd: initramfs.to_string(),
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
