use crate::entities::EngineEvent;
use csv::Error as csv_error;
use std::io::Error as io_error;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinError;
#[derive(Error, Debug)]
#[allow(unused)]
/// Processing related errors
pub(crate) enum ProcessError {
    #[error("Not possible create account `{0}`")]
    AccountCreation(String),
}
#[derive(Error, Debug)]
/// File related errors.
pub enum FileError {
    #[error("Unable read csv file: `{0}`")]
    CsvRead(#[from] csv_error),
    #[error("Unable write csv to stdout: `{0}`")]
    StdOut(#[from] io_error),
}
#[derive(Error, Debug)]
/// Account related errors.
pub(crate) enum AccountError {}
#[derive(Error, Debug)]
/// Engine related errors.
pub enum EngineError {
    #[error(transparent)]
    File(#[from] FileError),
    #[error("Invalid row in csv file: ${0}")]
    ParseRow(#[from] csv_error),
    #[error("Failed to send transaction onto channel: ${0}")]
    ChannelSend(#[from] SendError<EngineEvent>),
    #[error("Failed to terminate engine runner: ${0}")]
    Terminate(#[from] JoinError),
    #[error("Unknown event ${0}")]
    Event(String),
}
#[cfg(test)]
#[derive(Error, Debug)]
/// Test related errors.
pub(crate) enum TestError {
    #[error("Unexpected io error: `{0}`")]
    StdOut(#[from] io_error),
}
