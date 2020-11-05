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
                    "Execution of {} failed {}",
                    cmd_name,
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

/// Provides a comfortable way to pause between the steps if the user did enable the function.
/// Contains a boolean whether the wait should be executed or not.
pub struct Stepper(bool);

impl Stepper {
    /// Returns a new instance of the Stepper object. Calculates the number of needed steps based
    /// on skipped steps.
    pub fn new(enabled: bool) -> Self {
        Self(enabled)
    }

    /// Outputs the path of the temporary folder to the user via the debug facility. This enables
    /// the user to find the correct temporary working folder. If the Stepper is disabled, the path
    /// will be outputted on Debug level thus only be visible when the verbose mode is enabled.
    /// Needs the path to the temporary folder.
    pub fn log_folder_path(&self, folder: PathBuf) {
        match self.0 {
            true => info!("File operations will be happening in {}", folder.display()),
            false => debug!("temporary folder is {}", folder.display()),
        }
    }

    /// The method logs the given step description. This helps the user to determine which steps
    /// was executed and thus when to look at the files and when to skip a pause without any
    /// further investigation. The log-level depends whether the stepper is enabled or not. Active
    /// it's info, otherwise it's debug.
    pub fn log_step<S: Into<String>>(&self, desc: S) {
        match self.0 {
            true => info!("{}...", desc.into()),
            false => debug!("{}...", desc.into()),
        }
    }

    /// If the step mode was enabled the method will wait until user hits enter. This is used for
    /// the pause between steps mode. Allowing the user to tweak the files in the temporary folder
    pub fn wait(&self) {
        match self.0 {
            true => {
                println!("Hit enter to proceed with next step...");
                let mut void = String::new();
                match io::stdin().read_line(&mut void) {
                    Ok(_) => {}
                    Err(e) => error!("couldn't read line, {}", e),
                }
            }
            false => {}
        }
    }
}
