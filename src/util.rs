use crate::error::ErrorMessage;

use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use regex::Regex;

/// Name of the pdfinfo binary.
const PDFINFO_BINARY: &str = "pdfinfo";

/// Runs a Command and handles the outcome of it.
pub fn run_cmd<'a>(mut cmd: Command, cmd_name: &'a str) -> Result<(), ErrorMessage> {
    match cmd.output() {
        Ok(x) => match x.status.success() {
            true => /*debug!(
                "{}",
                match String::from_utf8(x.stdout) {
                    Ok(x) => x,
                    Err(_) => String::from("<Error converting stdout to string>"),
                }
            )*/{},
            false => {
                return Err(ErrorMessage::new(format!(
                    "Execution of {} failed {} {}",
                    cmd_name,
                    match String::from_utf8(x.stdout) {
                        Ok(x) => x,
                        Err(_) => String::from("<Error converting stdout to string>"),
                    },
                    match String::from_utf8(x.stderr) {
                        Ok(x) => x,
                        Err(_) => String::from("<Error converting stderr to string>"),
                    }
                )))
            }
        },
        Err(e) => {
            if let ErrorKind::NotFound = e.kind() {
                return Err(ErrorMessage::new(format!(
                    "couldn't find the convert binary ({}) on your system",
                    cmd_name,
                )));
            } else {
                return Err(ErrorMessage::new(format!(
                    "Failed to call {}, {}",
                    cmd_name, e
                )));
            }
        }
    }
    Ok(())
}

/// Returns the filename (without any extension) of a path.
pub fn file_name(path: PathBuf) -> String {
    let name = path.file_name().unwrap().to_string_lossy();
    let name_ele = name.split(".").collect::<Vec<&str>>();
    name_ele.split_first().unwrap().0.to_string()
}

/// Determines the number of pages a given PDF file contains. Uses pdfinfo.
pub fn count_pdf_pages(file: PathBuf) -> Result<u64, ErrorMessage> {
    let mut cmd = Command::new(PDFINFO_BINARY);
    cmd.arg(file);
    let out = match cmd.output() {
        Ok(x) => match x.status.success() {
            true => match String::from_utf8(x.stdout) {
                Ok(x) => x,
                Err(_) => return Err(ErrorMessage::new("Couldn't convert stdout to string")),
            },
            false => {
                return Err(ErrorMessage::new(format!(
                    "Execution of {} failed {}",
                    PDFINFO_BINARY,
                    match String::from_utf8(x.stderr) {
                        Ok(x) => x,
                        Err(_) => String::from("<Error converting stderr to string>"),
                    }
                )))
            }
        },
        Err(e) => {
            if let ErrorKind::NotFound = e.kind() {
                return Err(ErrorMessage::new(format!(
                    "couldn't find the convert binary ({}) on your system",
                    PDFINFO_BINARY,
                )));
            } else {
                return Err(ErrorMessage::new(format!(
                    "Failed to call {}, {}",
                    PDFINFO_BINARY, e
                )));
            }
        }
    };

    let re = match Regex::new(r#"Pages:\s*([0-9]*)"#) {
        Ok(x) => x,
        Err(e) => {
            return Err(ErrorMessage::new(format!(
                "Couldn't compile regular expression, {}",
                e
            )))
        }
    };
    match re.captures(&out) {
        Some(x) => match x.get(1) {
            Some(x) => Ok(x.as_str().parse::<u64>().unwrap()),
            None => Err(ErrorMessage::new(format!(
                "Couldn't find number of pages in pdfinfo output"
            ))),
        },
        None => Err(ErrorMessage::new(format!(
            "Couldn't find number of pages in pdfinfo output"
        ))),
    }
}
