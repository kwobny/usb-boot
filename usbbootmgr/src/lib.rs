mod user_interfacing;

use std::{process::Command, fs};
use std::io;
use std::path::Path;

use anyhow::Context;
use user_interfacing::{OperationRequest, UserInteractError, ChangeKernel};

const DEFAULT_CONFIG_FILE: &str = "/etc/usb-boot/usbbootmgr.toml";
const FILE_UTILITY: &str = "/usr/bin/file";
const MKINITCPIO_PROGRAM: &str = "/usr/bin/mkinitcpio";
const MKINITCPIO_PRESETS_DIR: &str = "/etc/mkinitcpio.d";

fn handle_user_interact_error(err: UserInteractError) -> Result<(), anyhow::Error> {
    use UserInteractError::*;
    match err {
        InvalidCommandLineArguments { details: source } |
        CliIOError { source } => {
            source.exit();
        },
        ConfigParseError { source } => {
            eprintln!("{source}");
            Ok(())
        },
        ConfigAccessFailed { source } => {
            eprintln!("{source}");
            Ok(())
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum ChangeKernelError {
    /// A file that's supposed to be a kernel image
    /// is not accessible, doesn't exist, or is not a kernel image.
    #[error("the file \"{file}\" is not accessible or is not a kernel image")]
    FileNotAccessibleKernelImage {
        file: String,
        #[source]
        source: Option<io::Error>,
    },
}

fn handle_change_kernel(details: ChangeKernel) -> Result<(), anyhow::Error> {
    // Check that the source file is accessible and is a kernel image.
    // The destination does not have to exist or be a kernel image.
    for file in [&details.source] {
        // First, check if the file is accessible.
        if !Path::new(&file).exists() {
            return Err(ChangeKernelError::FileNotAccessibleKernelImage {
                file: file.to_owned(),
                source: None,
            }.into());
        }
        // Next, call the "file" program and get its output.
        let output = Command::new(FILE_UTILITY)
            .arg(&file)
            .output().context("failed to collect output from file program")?;
        let output = String::from_utf8(output.stdout)
            .context("failed to convert output from file program to string")?;
        // Check if the following strings are in the output
        // from the "file" program. All of the strings must
        // be in the output, or else it's an error.
        for find_str in ["kernel", "executable"] {
            if output.find(find_str).is_none() {
                return Err(ChangeKernelError::FileNotAccessibleKernelImage {
                    file: file.to_owned(),
                    source: None,
                }.into());
            }
        }
    }
    // Now we've confirmed that the source file is all good
    // (it's accessible and it's a kernel file). Moving on.

    // Ensure mkinitcpio preset exists.
    if !Path::new(
        &format!("{}/{}.preset", MKINITCPIO_PRESETS_DIR, details.mkinitcpio_preset)
    ).exists() {
        anyhow::bail!(
            "the mkinitcpio preset \"{}\" does not exist",
            details.mkinitcpio_preset,
        );
    }

    // Delete destination before copying / hard linking.
    fs::remove_file(&details.destination)
        .context("failed to unlink destination file")?;

    // Copy/hard link the source file to destination.
    if details.hard_link {
        fs::hard_link(&details.source, &details.destination)
            .context("failed to create hard link at destination to the source file")?;
    } else {
        fs::copy(&details.source, &details.destination)
            .context("failed to copy source file to destination")?;
    }

    // Regenerate usb boot initramfs.
    let exit_status = Command::new(MKINITCPIO_PROGRAM)
        .args(["--preset", &details.mkinitcpio_preset])
        .status().context("failed to execute mkinitcpio")?;
    if !exit_status.success() {
        anyhow::bail!("failed to regenerate usb boot initramfs images");
    }

    Ok(())
}

pub fn run() -> Result<(), anyhow::Error> {
    let operation = user_interfacing::interact_with_user(DEFAULT_CONFIG_FILE);
    let operation = match operation {
        Err(err) => {
            return handle_user_interact_error(err);
        },
        Ok(x) => x,
    };

    match operation {
        OperationRequest::ChangeKernel(details) => {
            handle_change_kernel(details)
        },
    }
}
