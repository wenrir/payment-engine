use csv::Error as csv_error;
use std::io::Error as io_error;
use thiserror::Error;
#[derive(Error, Debug)]
pub(crate) enum AccountError {}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Unable read csv file: `{0}`")]
    CsvRead(#[from] csv_error),
    #[error("Unable write csv to stdout: `{0}`")]
    StdOut(#[from] io_error),
}
