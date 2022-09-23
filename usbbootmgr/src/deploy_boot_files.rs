use crate::user_interfacing::DeployBootFiles;
use crate::logging;

#[derive(thiserror::Error, Debug)]
pub enum DeployBootFilesError {
}

pub fn deploy_boot_files(details: DeployBootFiles) -> Result<(), DeployBootFilesError> {
    logging::info(format_args!(""));

    todo!();
}
