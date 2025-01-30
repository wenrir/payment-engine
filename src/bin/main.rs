//! Main entrypoint for binary.
use paymentlib::run_from_csv;
use std::{fs::exists, panic};
/// Main entrypoint of the binary.
/// Reads file path as an argument from user, returns output to stdout.
///
/// Requires : User provides file path in terms of the first argument to the application.
/// Does : Application provides validated data to stdout.
/// Ensures :
/// + Argument exists on 1st spot, store as path.
/// + File exists in path.
/// + Call and read output from library.
/// + Provides output to stdout.
///
/// # Examples
///
/// Basic usage:
///
/// ``` sh
/// cargo run -- transactions.csv > accounts.csv
/// ```
#[tokio::main]
async fn main() {
    match std::env::args().nth(1) {
        Some(path) => {
            assert!(
                exists(&path).expect("File does not exist"),
                "Assertion failed in main: File {:?} does not exist, please make sure to provide a valid path.",
                &path.as_str()
            );
            run_from_csv(&path).await.expect(
                "Unable to finish reading tx from csv. [engine failed]",
            );
        }
        None => {
            panic!(
                "Please provide path, usage example : cargo run -- transactions.csv > accounts.csv"
            );
        }
    };

    //println!("path: {:?}", path)
}
