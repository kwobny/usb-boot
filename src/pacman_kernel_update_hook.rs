//! This module contains the logic for a program that copies all files
//! in /boot/usb-boot/* onto the usb to update them.

use std::{path::Path, fs};

pub struct Config {
    block_device_path: Box<Path>, // Path to block device on usb.
    from_directory: Box<Path>, // Path on this computer to directory containing usb boot stuff.
    to_directory: Box<Path>, // Path with mounted block device as root.
}

pub fn get_config() {
}

pub enum PermissionError {
    /// Could not access block device.
    AccessBlockDevice,
    /// Could not mount block device.
    MountBlockDevice,
}
pub enum UpdateUsbBootError {
    /// The block device file specified in the supplied config does not exist.
    /// This could mean the usb is not plugged in.
    BlockDeviceDoesNotExist,
    /// Could not perform update operation due to insufficient permissions.
    InsufficientPermissions(PermissionError),
    Other(anyhow::Error),
}
pub fn update_usb_boot(config: Config) -> Result<(), UpdateUsbBootError> {
    match fs::metadata(config.block_device_path) {
    }
}

