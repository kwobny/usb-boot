//! This program is used for kexec-ing into the real kernel when
//! booting from usb into encrypted root, while in first initrd.
//!
//! The program does these things:
//!     1. Reads kernel command line from /proc/cmdline
//!     2. Parses command line and alters it according to specific parameters
//!     3. Runs kexec -l
//!     4. Runs systemctl kexec

fn main() -> anyhow::Result<()> {
    usb_boot_kexec::run()
}
