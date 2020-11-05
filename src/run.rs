use crate::error::ErrorMessage;
use crate::util::Stepper;

use std::env;
use std::fs::{self};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use shellexpand;
use tempfile::{Builder, TempDir};

/// Name of the start file in the temporary folder.
pub const START_PDF: &str = "input.pdf";
/// Output file definition for PDF to images conversion.
pub const CONVERT_OUTPUT: &str = "a_%03d";

/// This struct contains all the needed information and states to go trough the different
/// conversion steps. The object manages the temporary folder.
pub struct Run {
    /// Absolute path to the input file.
    input: PathBuf,
    /// Absolute path to the output file.
    output: Option<PathBuf>,
    /// Temporary folder where the conversion happens.
    folder: TempDir,
    /// Stepper to pause the execution between the steps if needed.
    pub stepper: Stepper,
}

impl Run {
    /// Returns a new run based on an input file and an optional output file path. The given input
    /// path will get shell-expanded and normalized as absolute path relative to the current
    /// working directory. The shell-expansion enables the usage of the tilde (`~`) as abbreviation
    /// of the home folder and environment variables. The existence of the input file is tested.
    /// The `do_step` states whether to pause between the steps.
    ///
    /// To modify the object further use the build pattern and finish the configuration with
    /// init().
    pub fn new<S: Into<String>>(input: S, do_step: bool) -> Result<Self, ErrorMessage> {
        let input = Run::expand_input_path(input.into())?;
        Run::validate_input_file(&input)?;

        let rsl = Self {
            input: input.clone(),
            output: None,
            folder: match Builder::new().prefix("pdfiron-").tempdir() {
                Ok(x) => x,
                Err(e) => {
                    return Err(ErrorMessage::new(format!(
                        "Couldn't create temp folder, {}",
                        e
                    )))
                }
            },
            stepper: Stepper::new(do_step),
        };

        rsl.stepper.log_folder_path(rsl.folder.path().to_path_buf());

        let in_dst = rsl.prepend_with_temp_folder(START_PDF);
        debug!("copy {} to {}", input.display(), in_dst.display());
        match fs::copy(&input, &in_dst) {
            Ok(_) => Ok(rsl),
            Err(e) => Err(ErrorMessage::new(format!(
                "Couldn't copy input file to {}, {}",
                in_dst.display(),
                e
            ))),
        }
    }

    /// Returns the path to the temporary folder with some path appended.
    pub fn prepend_with_temp_folder<'a, S: Into<&'a str>>(&self, path: S) -> PathBuf {
        self.folder.path().join(path.into())
    }

    /// Returns the output path based on the absolute input path. Folder stays the same,
    /// the suffix -ironed is added to the filename.
    fn default_output_path(&self) -> PathBuf {
        let mut rsl = PathBuf::new();
        rsl.push(self.input.parent().unwrap());
        let name = self.input.file_name().unwrap().to_string_lossy();
        let name_ele = name.split(".").collect::<Vec<&str>>();
        let name_parts = name_ele.split_first().unwrap();
        rsl.push(format!(
            "{}-ironed.{}",
            name_parts.0,
            name_parts.1.join(".")
        ));
        rsl
    }

    /// Shell expands the input path and normalize it to an absolute path.
    fn expand_input_path(file: String) -> Result<PathBuf, ErrorMessage> {
        let expanded = match shellexpand::full(&file) {
            Ok(x) => x,
            Err(e) => {
                return Err(ErrorMessage::new(format!(
                    "Couldn't shell-expand given input path {}, {}.",
                    file, e
                )))
            }
        };
        let mut expanded_input = String::new();
        expanded_input.push_str(&expanded);
        let mut input = PathBuf::new();
        input.push(expanded_input);

        match input.is_absolute() {
            true => Ok(input),
            false => match env::current_dir() {
                Ok(x) => Ok(x.join(input)),
                Err(e) => Err(ErrorMessage::new(format!(
                    "Couldn't determine working directory to normalize input file, {}.",
                    e,
                ))),
            },
        }
    }

    /// Checks if a file exists and is a PDF file.
    fn validate_input_file(file: &PathBuf) -> Result<(), ErrorMessage> {
        let not_pdf_err = ErrorMessage::new(format!(
            "Given input file {} isn't a PDF file",
            file.display()
        ));
        match file.exists() {
            true => match file.extension() {
                Some(x) => match x.to_string_lossy().to_lowercase() == "pdf" {
                    true => Ok(()),
                    false => Err(not_pdf_err),
                },
                None => Err(not_pdf_err),
            },
            false => Err(ErrorMessage::new(format!(
                "Given input file {} doesn't exist",
                file.display(),
            ))),
        }
    }
}

// If no output path was specified,
// it will be determined depending on the input path: The output will be placed in the same
// folder and with `-ironed` as suffix. The function also creates a temporary folder for the
// file operations and copies the input PDF file into this location as `input.pdf`.
