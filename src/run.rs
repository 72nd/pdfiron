/// The actual work.
use crate::error::ErrorMessage;

use std::env;
use std::fs::{self};
use std::path::{Path, PathBuf};

use shellexpand;
use tempfile::{Builder, TempDir};

/// Name of the start file in the temporary folder.
const START_PDF: &str = "input.pdf";

/// This struct contains all the needed information and states to go trough the different
/// conversion steps.
pub struct Run {
    /// Absolute path to the input file.
    input: PathBuf,
    /// Absolute path to the output file.
    output: Option<PathBuf>,
    /// Temporary folder where the conversion happens.
    folder: Option<TempDir>,
    /// Optional parameters for the convert call.
    convert_options: Option<Vec<String>>,
    /// Whether to run tesseract-ocr or not.
    do_tesseract: bool,
    /// Whether to run unpaper or not.
    do_unpaper: bool,
}

impl Run {
    /// Returns a new run based on an input file and an optional output file path. The given input
    /// path will get shell-expanded and normalized as absolute path relative to the current
    /// working directory. The shell-expansion enables the usage of the tilde (`~`) as abbreviation
    /// of the home folder and environment variables. The existence of the input file is tested.
    ///
    /// To modify the object further use the build pattern and finish the configuration with
    /// init().
    pub fn new<S: Into<String>>(input: S) -> Result<Self, ErrorMessage> {
        let input = Run::expand_input_path(input.into())?;
        Run::validate_input_file(&input)?;

        Ok(Self {
            input: input.clone(),
            output: None,
            folder: None,
            convert_options: None,
            do_tesseract: false,
            do_unpaper: false,
        })
    }

    /// Sets a specific output file for the conversion. If no output is specified by using this
    /// method the init() method will use a default path based on the input path.
    pub fn output<S: Into<String>>(&mut self, output: Option<S>) -> &mut Self {
        match output {
            Some(x) => self.output = Some(Path::new(&x.into()).to_path_buf()),
            None => {}
        };
        self
    }

    /// Sets the optional parameters for the convert call.
    pub fn convert_options<S: Into<String>>(&mut self, opt: Option<S>) -> &mut Self {
        self.convert_options = match opt {
            Some(x) => Some(x.into().split(" ").map(|x| x.to_string()).collect()),
            None => None,
        };
        self
    }

    /// Sets whether to run tesseract or not.
    pub fn do_tesseract(&mut self, do_tesseract: bool) -> &mut Self {
        self.do_tesseract = do_tesseract;
        self
    }

    /// Sets whether to run unpaper or not.
    pub fn do_unpaper(&mut self, do_unpaper: bool) -> &mut Self {
        self.do_unpaper = do_unpaper;
        self
    }

    /// Initializes the instance and creates the temporary folder. The input PDF is copied into the
    /// folder and the pages are extracted to images from the document.
    pub fn init(&mut self) -> Result<(), ErrorMessage> {
        let folder = match Builder::new().prefix("pdfiron-").tempdir() {
            Ok(x) => x,
            Err(e) => {
                return Err(ErrorMessage::new(format!(
                    "Couldn't create temp folder, {}",
                    e
                )))
            }
        };
        debug!("created new temporary folder {}", folder.path().display());

        let in_dst = folder.path().join(START_PDF);
        debug!("copy {} to {}", self.input.display(), in_dst.display());
        match fs::copy(&self.input, &in_dst) {
            Ok(_) => {}
            Err(e) => {
                return Err(ErrorMessage::new(format!(
                    "Couldn't copy input file to {}, {}",
                    in_dst.display(),
                    e
                )))
            }
        };

        self.folder = Some(folder);
        Ok(())
    }

    /// Returns the output path based on the absolute input path. Folder stays the same,
    /// the suffix -ironed is added to the filename.
    fn default_output_paht(&self) -> PathBuf {
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
