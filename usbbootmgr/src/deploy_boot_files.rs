use std::borrow::Cow;

use crate::user_interfacing::DeployBootFiles;
use crate::log;

#[derive(thiserror::Error, Debug)]
pub enum DeployBootFilesError {
}

pub fn deploy_boot_files(details: DeployBootFiles) -> Result<(), DeployBootFilesError> {
    log::info(format_args!(
        "\
Block device = {}
Mount point = {}
Source directory = {}
Destination directory = {}
",
        match details.destination_block_device {
            Some(ref x) => &x,
            None => "None",
        },
        details.block_device_mount_point,
        details.boot_files_source,
        details.boot_files_destination,
    ));

    todo!();
}
