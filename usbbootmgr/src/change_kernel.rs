use std::{process::Command, fs::{self, File}};
use std::io::{self, BufReader, Read};
use std::path::Path;
use std::os::linux::fs::MetadataExt;

use anyhow::Context;

use crate::{FILE_UTILITY, MKINITCPIO_PROGRAM, MKINITCPIO_PRESETS_DIR};
use crate::user_interfacing::{ChangeKernel, CompareKernelsOption};

/// This function checks if the contents of two files are identical.
/// Returns true if the contents of both files are identical.
/// Returns false if the contents differ between the files.
fn file_contents_are_identical(efficient: bool, left: &str, right: &str) -> Result<bool, anyhow::Error> {
    let filenames = [left, right];

    // First, check if the sizes of both files are different.
    // If the sizes are different, then we immediately know the
    // contents of the files are also different (assuming the sizes
    // are accurate). If there was an error with getting the size
    // of a file, then we abort and compare files the traditional way.
    let mut file_metadata = [0; 2].map(|_| None);
    if efficient {
        let mut size_of_files = [0; 2];
        let mut do_comparison = true;
        for (i, file) in filenames.iter().enumerate() {
            match fs::metadata(file) {
                Ok(metadata) => {
                    size_of_files[i] = metadata.len();
                    file_metadata[i] = Some(metadata);
                },
                Err(_) => {
                    do_comparison = false;
                },
            };
        }
        if do_comparison && size_of_files[0] != size_of_files[1] {
            return Ok(false);
        }
    }

    // Do preparation that is needed to compare both files, namely
    // obtaining readers for both files.
    let mut readers = [0; 2].map(|_| None);
    for (i, (file, possible_metadata)) in (filenames.iter())
        .zip(file_metadata).enumerate()
    {
        // Open both files.
        let file_handle = File::open(file)
            .with_context(||
                format!("failed to open the file \"{file}\" when \
                comparing contents of two files")
            )?;

        // Get the metadata of each file. If there is already
        // metadata from when doing file size comparison, then use that,
        // otherwise query metadata from the file.
        let metadata = match possible_metadata {
            None => {
                file_handle.metadata().with_context(|| format!(
                    "failed to get metadata of the file \"{file}\""
                ))?
            },
            Some(x) => x,
        };

        // Construct a new BufReader with capacity set to the io
        // block size of the file.
        readers[i] = Some(BufReader::with_capacity(
            metadata.st_blksize().try_into().unwrap(), file_handle,
        ));
    }
    // Now do the actual comparison. Iterate over the bytes of each
    // file, comparing them one by one. I.e do a lexicographic comparison.
    let mut byte_iterators = readers.map(|reader|
        reader.unwrap().bytes().peekable());
    loop {
        let mut done = None;
        for byte_iter in byte_iterators.iter_mut() {
            let current_is_done = byte_iter.peek().is_none();
            match done {
                None => done = Some(current_is_done),
                Some(x) => if x ^ current_is_done {
                    return Ok(false);
                },
            }
        }
        if done.unwrap() {
            break;
        }
        let bytes_are_equal = byte_iterators[0].next().unwrap()?
            == byte_iterators[1].next().unwrap()?;
        if !bytes_are_equal {
            return Ok(false);
        }
    }

    // If the file contents weren't equal, the function must have returned
    // false by now. If the function has gotten to this point, that means
    // the file contents are equal.

    Ok(true)
}

#[derive(thiserror::Error, Debug)]
pub enum ChangeKernelError {
    /// A file that's supposed to be a kernel image
    /// is not accessible, doesn't exist, or is not a kernel image.
    #[error("the file \"{file}\" is not accessible or is not a kernel image")]
    FileNotAccessibleKernelImage {
        file: String,
        #[source]
        source: Option<io::Error>,
    },
}

pub fn handle_change_kernel(details: ChangeKernel) -> Result<(), anyhow::Error> {
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

    // If compare kernels is on, check if the source and destination
    // have the same contents. If they do, then don't regenerate initramfs
    // at the end.
    let regenerate_initramfs = if let Some(x) = details.compare_kernels {
        let efficient = match x {
            CompareKernelsOption::Full => false,
            CompareKernelsOption::Efficient => true,
        };
        !file_contents_are_identical(efficient, &details.source, &details.destination)?
    } else {
        true
    };

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
    if regenerate_initramfs {
        let exit_status = Command::new(MKINITCPIO_PROGRAM)
            .args(["--preset", &details.mkinitcpio_preset])
            .status().context("failed to execute mkinitcpio")?;
        if !exit_status.success() {
            anyhow::bail!("failed to regenerate usb boot initramfs images");
        }
    }

    Ok(())
}
