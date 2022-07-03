mod utils;

use std::{fs, process::Command};

struct KexecArgs {
    kernel: String,
    initrd: String,
    command_line: String,
}

fn transform_command_line(command_line: &str, additional_args_parameter: &str) -> KexecArgs {
    let mut new_cmdline = String::new();

    for parameter in utils::split_at_unquoted_spaces(command_line) {
        let key = format!("{additional_args_parameter}=");
        if parameter.starts_with(&key) {
            let value = &parameter[key.len()..];
        } else {
            new_cmdline.push_str(parameter);
        }
    }

    todo!();
}

pub fn run(additional_args_parameter: &str) -> anyhow::Result<()> {
    // Get current kernel command line
    let kernel_command_line = fs::read_to_string("/proc/cmdline")?;

    // Transform command line
    let new_command_line = transform_command_line(&kernel_command_line, additional_args_parameter);

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
