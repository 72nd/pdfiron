use crate::error::{ErrorMessage, ExecutableNameMethod, ExecutableNotFound};
/// Contains some methods used by the run module. They wore excluded to this file to tidy up the
/// run module a little bit.
use std::env;
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::process::{Command, Output};

/// Runs a Command and handles the outcome of it.
pub fn run_cmd<'a>(mut cmd: Command, cmd_name: &'a str) -> Result<(), ErrorMessage> {
    match cmd.output() {
        Ok(x) => match x.status.success() {
            true => debug!(
                "{}",
                match String::from_utf8(x.stdout) {
                    Ok(x) => x,
                    Err(_) => String::from("<Error converting stdout to string>"),
                }
            ),
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

