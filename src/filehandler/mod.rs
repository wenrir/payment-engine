//! Filehandler logic, for reading csv files and writing to stdout.

use crate::entities::account::Account;

use crate::errors::FileError;
use csv::{Reader, ReaderBuilder, Trim::All, Writer};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::Path;
#[allow(unused)]
/// Reads a csv file.
/// Expects a valid path csv as input
/// Returns a reader with the content of csv file.
/// Will panic if file does not exists or wrong extension.
pub(crate) fn read_csv(file_path: &str) -> Result<Reader<File>, FileError> {
    let path = Path::new(file_path);
    assert!(path.exists());
    assert!(path.is_file());
    assert_eq!(path.extension(), Some(OsStr::new("csv")));
    //https://docs.rs/csv/latest/csv/struct.ReaderBuilder.html
    let mut binding = ReaderBuilder::new();
    let rdr = binding
        .delimiter(b',')
        .has_headers(true)
        .trim(All)
        .comment(Some(b'#'))
        .flexible(true) // This is somewhat difficult to understand from the requirement, since some inputs does not have `amount` do I still force `,` ?
        .double_quote(false)
        .quoting(false);
    Ok(rdr.from_path(path)?)
}
/// Write account information to stdout
/// Considerations:
/// + Maybe use AsyncWrite instead?
/// + Should this be in filehandler?
#[allow(dead_code)]
pub(crate) fn csv_to_stdout<S: Write>(accounts: Vec<&Account>, stream: S) -> Result<(), FileError> {
    let mut wtr = Writer::from_writer(stream);
    for account in accounts {
        wtr.serialize(account)?;
    }
    wtr.flush()?;
    Ok(())
}
