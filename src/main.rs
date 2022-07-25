//! This program is used for kexec-ing into the real kernel when
//! booting from usb into encrypted root, while in first initrd.
//!
//! The program does these things:
//!     1. Reads kernel command line from /proc/cmdline
//!     2. Parses command line and alters it according to specific parameters
//!     3. Runs kexec -l
//!     4. Runs systemctl kexec

use std::env;

use anyhow::{Result, bail};
use usb_boot_kexec::initramfs_kexec_runner::TransformParameters;

fn main() -> Result<()> {
    /*
    let config = usb_boot_kexec::parse_args(env::args(), TransformParameters {
        additional_args: "--additional_args".to_string(),
        kernel: "--kernel".to_string(),
        initrd: "--initrd".to_string(),
    }.try_into().unwrap())?;

    usb_boot_kexec::run(config)?;
    Ok(())
    */

    bail!("test error message");
}
