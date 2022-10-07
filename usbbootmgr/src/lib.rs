mod log;
mod misc;
mod user_interfacing;
mod change_kernel;
mod deploy_boot_files;

use std::process::ExitCode;

use user_interfacing::{OperationRequest, UserInteractError};

const DEFAULT_CONFIG_FILE: &str = "/etc/usb-boot/usbbootmgr.toml";

const FILE_UTILITY: &str = "/usr/bin/file";
const MKINITCPIO_PROGRAM: &str = "/usr/bin/mkinitcpio";
const MKINITCPIO_PRESETS_DIR: &str = "/etc/mkinitcpio.d";

const MOUNT_PROGRAM: &str = "/usr/bin/mount";
const UNMOUNT_PROGRAM: &str = "/usr/bin/umount";

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

pub fn run() -> Result<ExitCode, anyhow::Error> {
    let operation = user_interfacing::interact_with_user(DEFAULT_CONFIG_FILE);
    let operation = match operation {
        Err(err) => {
            handle_user_interact_error(err)?;
            return Ok(ExitCode::FAILURE);
        },
        Ok(x) => x,
    };

    match operation {
        OperationRequest::ChangeKernel(details) => {
            change_kernel::handle_change_kernel(details)?;
            return Ok(ExitCode::SUCCESS);
        },
        OperationRequest::DeployBootFiles(details) => {
            match deploy_boot_files::deploy_boot_files(details) {
                Ok(x) => Ok(ExitCode::from(x)),
                Err(x) => Err(x.into()),
            }
        },
    }
}
