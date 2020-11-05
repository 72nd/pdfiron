/// The actual work.
use crate::error::GenericError;

use std::env;
use std::path::{Path, PathBuf};

use shellexpand;

/// This struct contains all the needed information and states to go trough the different
/// conversion steps.
pub struct Run {
    /// Absolute path to the input file.
    input: PathBuf,
    /// Absolute path to the output file.
    output: PathBuf,
}

impl Run {
    /// Returns a new run based on an input file and an optional output file path. The given input
    /// path will get shell-expanded and normalized as absolute path relative to the current
    /// working directory. The shell-expansion enables the usage of the tilde (`~`) as abbreviation
    /// of the home folder and environment variables. The existence of the input file is tested.
    /// If no output path is specified, it will be determined depending on the input path: The
    /// output will be placed in the same folder and with `-ironed` as suffix.
    pub fn new<S: Into<String>>(input: S, output: Option<S>) -> Result<Self, GenericError> {
        let input = Run::expand_input_path(input.into())?;
        Run::validate_input_file(&input)?;
        Ok(Self {
            input: input.clone(),
            output: match output {
                Some(x) => Path::new(&x.into()).to_path_buf(),
                None => Run::output_paht(&input),
            },
        })
    }

    /// Shell expands the input path and normalize it to an absolute path.
    fn expand_input_path(file: String) -> Result<PathBuf, GenericError> {
        let expanded = match shellexpand::full(&file) {
            Ok(x) => x,
            Err(e) => {
                return Err(GenericError::new(format!(
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
                Err(e) => Err(GenericError::new(format!(
                    "Couldn't determine working directory to normalize input file, {}.",
                    e,
                ))),
            },
        }
    }

    /// Checks if a file exists and is a PDF file.
    fn validate_input_file(file: &PathBuf) -> Result<(), GenericError> {
        let not_pdf_err = GenericError::new(format!(
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
            false => Err(GenericError::new(format!(
                "Given input file {} doesn't exist",
                file.display(),
            ))),
        }
    }

    /// Returns the output path based on the absolute input path. Folder stays the same,
    /// the suffix -ironed is added to the filename.
    fn output_paht(input: &PathBuf) -> PathBuf {
        let mut rsl = PathBuf::new();
        rsl.push(input.parent().unwrap());
        let name = input.file_name().unwrap().to_string_lossy();
        let name_ele = name.split(".").collect::<Vec<&str>>();
        let name_parts = name_ele.split_first().unwrap();
        rsl.push(format!(
            "{}-ironed.{}",
            name_parts.0,
            name_parts.1.join(".")
        ));
        rsl
    }
}
