use std::process::Command;
use std::{fs, io};
use std::fs::FileType;
use std::os::unix::fs::FileTypeExt;

use anyhow::Context;

use crate::misc::AggregateError;
use crate::{MOUNT_PROGRAM, UNMOUNT_PROGRAM};
use crate::user_interfacing::DeployBootFiles;
use crate::log;

#[derive(thiserror::Error, Debug)]
pub enum DeployBootFilesError {
}

macro_rules! check_file_explicit {
    // Arguments are evaluated lazily.
    (
        $path:expr,
        $expected_filetype_filter:expr,
        $not_found_message:expr,
        $check_failed_message:expr,
        $unexpected_filetype_message:expr $(,)?
    ) => {
        let metadata = fs::metadata($path);
        let metadata = match metadata {
            Err(err) => {
                match err.kind() {
                    io::ErrorKind::NotFound => {
                        log::error($not_found_message);
                    },
                    _ => {
                        log::error(format_args!("{}", err));
                        log::error($check_failed_message);
                    },
                }
                return Ok(false);
            },
            Ok(x) => x,
        };
        let filetype_is_as_expected = $expected_filetype_filter(&metadata.file_type());
        if ! filetype_is_as_expected {
            log::error($unexpected_filetype_message);
            return Ok(false);
        }
    };
}

/// This function deploys boot files to the usb.
/// It returns a Result that represents how the operation went.
/// Ok(true) means everything went well.
/// Ok(false) means that the operation failed, but the failure was
/// an expected possibility and handled by this function.
/// Err means that the operation failed because of an
/// unexpected possibility that needs to be handled by the calling code.
///
/// ## Explanation of return type:
/// This function is not just a function that deploys boot files,
/// it's a function that also interacts with the user.
/// In the Result type, Err generally means that something happened,
/// which the calling code needs to handle. Ok means that there is
/// nothing which the calling code needs to handle. An error could
/// have occurred, but if the function handled and resolved it,
/// that still leads to an Ok value.
/// In this operation, there are certain errors which are predictable
/// errors expected to occur once in a while. For these, this function
/// itself handles it, and prints an error message. This function's
/// job is to interact with the user, so that makes sense. These
/// kinds of errors map to Ok(false) values because an error occurred,
/// but this function handled and resolved them.
/// But there are also errors which are possible but unexpected. Since
/// they are unexpected to this function, they are not handled
/// by this function and must be handled by the calling code.
/// Therefore, they map to Err values.
pub fn deploy_boot_files(details: DeployBootFiles) -> Result<bool, anyhow::Error> {
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

        // Verify that files exist.
        if let Some(ref block_device) = details.destination_block_device {
            check_file_explicit!(
                block_device,
                FileTypeExt::is_block_device,
                format_args!(
                    "The block device file \"{}\" does not exist. \
                    Perhaps the usb is unplugged?",
                    block_device,
                ),
                "Failed to check if the block device exists.",
                format_args!(
                    "The file {} is not a block device file.",
                    block_device,
                ),
            );
        }
        check_file_explicit!(
            &details.block_device_mount_point,
            FileType::is_dir,
            "The mount point does not exist.",
            "Failed to check if the mount point exists.",
            "The mount point is not a directory.",
        );
        check_file_explicit!(
            &details.boot_files_source,
            FileType::is_dir,
            "The source directory does not exist.",
            "Failed to check if the source directory exists.",
            "The source directory is not a directory."
        );

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

        let combined_path = format!(
            "{}/{}",
            details.block_device_mount_point,
            details.boot_files_destination,
        );
        check_file_explicit!(
            &combined_path,
            FileType::is_dir,
            "The destination directory does not exist.",
            "Failed to check if the destination directory exists.",
            "The destination directory is not a directory.",
        );

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
            let status = Command::new("/usr/bin/cp")
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

        Ok(true)
    })();
    let cleanup_result = (|| -> Result<bool, anyhow::Error> {
        macro_rules! cleanup_fence {
            () => {
                if cleanup_level == 0 {
                    return Ok(true);
                } else {
                    cleanup_level -= 1;
                }
            };
        }

        cleanup_fence!();

        // Unmount block device.
        log::info("Unmounting block device");
        let status = Command::new(UNMOUNT_PROGRAM)
            .arg(&details.block_device_mount_point)
            .status()
            .context("failed to invoke umount program")?;
        if !status.success() {
            log::error(status.to_string());
            log::error("Failed to unmount block device.");

            return Ok(false);
        }

        Ok(true)
    })();

    let mut errors = Vec::new();
    let mut all_ended_well = true;
    for res in [result, cleanup_result] {
        match res {
            Ok(x) => {
                all_ended_well = all_ended_well && x;
            },
            Err(err) => {
                errors.push(err);
            },
        }
    }

    let possible_aggregate_error = AggregateError::new(errors);

    if all_ended_well && possible_aggregate_error.is_none() {
        log::info("Successfully updated usb boot files. Exiting");
    } else {
        log::error("Failed to update usb boot files. Exiting");
    }

    match possible_aggregate_error {
        Some(aggregate_error) => Err(aggregate_error.into()),
        None => Ok(all_ended_well),
    }
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
