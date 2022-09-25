use std::process::Command;
use std::{fs, io};

use anyhow::Context;

use crate::{MOUNT_PROGRAM, UNMOUNT_PROGRAM};
use crate::user_interfacing::DeployBootFiles;
use crate::log;

#[derive(thiserror::Error, Debug)]
pub enum DeployBootFilesError {
}

pub fn deploy_boot_files(details: DeployBootFiles) -> Result<(), anyhow::Error> {
    let mut cleanup_level: u8 = 0;

    // Ok(true) means the thing ran well.
    // Ok(false) means there was a user error along the way.
    let result = (|| -> Result<bool, anyhow::Error> {
        macro_rules! advance_cleanup_level {
            () => {
                cleanup_level += 1;
            };
        }

        // Print operation details before continuing.
        log::info(format_args!(
            "\
    Operation details:
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

        // Mount block device.
        if let Some(device) = details.destination_block_device {
            log::info("Mounting block device");

            let status = Command::new(MOUNT_PROGRAM)
                .args([&device, &details.block_device_mount_point])
                .status()
                .context("failed to invoke mount program")?;
            if !status.success() {
                log::error(status.to_string());
                log::error("Failed to mount block device.");

                return Ok(false);
            }

            advance_cleanup_level!();
        }

        let combined_path = format_args!(
            "{}/{}",
            details.block_device_mount_point,
            details.boot_files_destination,
        ).to_string();

        // Remove old files.
        log::info("Removing all old boot files from usb");
        let result = remove_all_in_directory(&combined_path);
        if let Err(error) = result {
            log::error(error.to_string());
            log::error("Failed to remove old usb boot files.");
            return Ok(false);
        }

        // Copy new files.
        log::info("Copying new boot files onto usb");

        let copy_operation_result = (|| -> Result<bool, anyhow::Error> {
            let entries_result = infallible_directory_entries(&details.boot_files_source);
            let entries = match entries_result {
                Err(err) => {
                    log::error(err.to_string());
                    log::error("Failed to get entries of source directory.");
                    return Ok(false);
                },
                Ok(x) => x,
            };
            let status = Command::new("cp")
                .args(["--dereference", "-t", &details.boot_files_destination])
                .args(entries.into_iter().map(|x| x.path()))
                .status()
                .context("failed to invoke cp program")?;
            if !status.success() {
                log::error(status.to_string());
                return Ok(false);
            }

            Ok(true)
        })();
        let copy_operation_result = copy_operation_result?;
        if copy_operation_result == false {
            log::error("Failed to copy new boot files to usb.");
            return Ok(false);
        }

        log::info("Successfully updated usb boot files. Exiting");

        Ok(true)
    })();
    let cleanup_result = (|| -> Result<(), anyhow::Error> {
        macro_rules! cleanup_fence {
            () => {
                if cleanup_level == 0 {
                    return Ok(());
                } else {
                    cleanup_level -= 1;
                }
            };
        }

        cleanup_fence!();
        // Unmount block device.
        log::info("Unmounting block device");

        todo!();
    })();

    match result {
        Ok(_) => {
            log::info("Successfully updated usb boot files. Exiting");
        },
        Err(_) => {
            log::error("Failed to update usb boot files. Exiting");
        },
    }

    todo!();
}

fn infallible_directory_entries(path: &str) -> io::Result<Vec<fs::DirEntry>> {
    let mut stuff = Vec::new();
    for entry in fs::read_dir(path)? {
        stuff.push(entry?);
    }
    Ok(stuff)
}

// This function removes all stuff in a directory,
// but not the directory itself.
// Currently, this function fails fast.
fn remove_all_in_directory(path: &str) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let entry_path = entry.path();

        if file_type.is_dir() {
            fs::remove_dir_all(entry_path)?;
        } else {
            fs::remove_file(entry_path)?;
        }
    }

    Ok(())
}
