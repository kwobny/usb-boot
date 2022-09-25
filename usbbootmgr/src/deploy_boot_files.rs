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
    struct CleanupContext {
        cleanup_level: u8,
    }
    let mut cleanup_context = CleanupContext {
        cleanup_level: 0,
    };

    macro_rules! advance_cleanup_level {
        () => {
            cleanup_context.cleanup_level += 1;
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
        log::info("Mounting block device.");

        let status = Command::new(MOUNT_PROGRAM)
            .args([&device, &details.block_device_mount_point])
            .status()
            .context("failed to invoke mount program")?;
        if !status.success() {
            log::error(status.to_string());
            anyhow::bail!("failed to mount block device");
        }

        advance_cleanup_level!();
    }

    let combined_path = format_args!(
        "{}/{}",
        details.block_device_mount_point,
        details.boot_files_destination,
    ).to_string();

    // Remove old files.
    log::info("Removing all old boot files from usb.");
    remove_all_in_directory(&combined_path)
        .context("failed to remove all old boot files from usb")?;

    // Copy new files.
    log::info("Copying new boot files onto usb.");

    let entries = infallible_directory_entries(&details.boot_files_source)
        .context("failed to get entries of source directory")?;
    let status = Command::new("cp")
        .args(["--dereference", "-t", &details.boot_files_destination])
        .args(entries.into_iter().map(|x| x.path()))
        .status()
        .context("failed to invoke cp program")?;
    if !status.success() {
        log::error(status.to_string());
        anyhow::bail!("failed to copy new boot files to usb");
    }

    todo!();

    impl Drop for CleanupContext {
        fn drop(&mut self) {
            macro_rules! cleanup_fence {
                () => {
                    if self.cleanup_level == 0 {
                        return;
                    } else {
                        self.cleanup_level -= 1;
                    }
                };
            }

            cleanup_fence!();
            // Unmount block device.
        }
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
