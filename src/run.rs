/// The actual work.
use crate::error::ErrorMessage;
use crate::util::{self, Stepper};

use std::env;
use std::ffi::OsStr;
use std::fs::{self};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use shellexpand;
use tempfile::{Builder, TempDir};

/// Name of the start file in the temporary folder.
const START_PDF: &str = "input.pdf";
/// Binary name of convert.
const CONVERT_BINARY: &str = "convert";
/// Output file definition for PDF to images conversion.
const CONVERT_OUTPUT: &str = "a_%03d";

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
    /// Stepper to pause the execution between the steps if needed.
    stepper: Stepper,
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
        Ok(Self {
            input: input.clone(),
            output: None,
            folder: None,
            convert_options: None,
            do_tesseract: false,
            do_unpaper: false,
            stepper: Stepper::new(do_step),
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

    /// Initializes the instance and creates the temporary folder. The input PDF is copied into
    /// this folder.
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
        self.stepper.log_folder_path(folder.path().to_path_buf());

        self.folder = Some(folder);
        let in_dst = self.append_to_temp(START_PDF);
        debug!("copy {} to {}", self.input.display(), in_dst.display());
        match fs::copy(&self.input, &in_dst) {
            Ok(_) => Ok(()),
            Err(e) => Err(ErrorMessage::new(format!(
                "Couldn't copy input file to {}, {}",
                in_dst.display(),
                e
            ))),
        }
    }

    /// The pages of the input document are extracted to images from the document. Takes an
    /// optional name of the convert binary (specify by the command line argument).
    pub fn convert_to_img<'a>(
        &self,
        use_gray: bool,
        use_rgb: bool,
        resolution: Option<&'a str>,
        options: Option<&'a str>,
    ) -> Result<(), ErrorMessage> {
        self.stepper.log_step("Extracting images form input PDF");

        let mut cmd = Command::new(CONVERT_BINARY);
        cmd.arg("-units").arg("PixelsPerInch");

        // Color mode
        match (use_gray, use_rgb) {
            (true, false) => cmd
                .arg("-colorspace")
                .arg("gray")
                .arg("-depth")
                .arg("8")
                .arg("-background")
                .arg("white")
                .arg("-flatten")
                .arg("-alpha")
                .arg("Off"),
            (false, true) => cmd
                .arg("-depth")
                .arg("8")
                .arg("-background")
                .arg("white")
                .arg("flatten")
                .arg("alpha")
                .arg("Off")
                .arg("-density"),
            (_, _) => cmd.arg("-type").arg("Bilevel"),
        };

        // Density
        let res = match resolution {
            Some(x) => match x.parse::<u64>() {
                Ok(x) => x,
                Err(_) => {
                    return Err(ErrorMessage::new(
                        "Invalid resolution argument, has to be positive int",
                    ))
                }
            },
            None => 300 as u64,
        };
        cmd.arg("-density").arg(format!("{}x{}", res, res));

        // Optional arguments
        cmd.args(match options {
            Some(x) => x.split(" ").collect::<Vec<&str>>(),
            None => vec![],
        });

        // Input and output
        cmd.arg(self.append_to_temp(START_PDF))
            .arg(self.append_to_temp(
                &format!(
                    "{}.{}",
                    CONVERT_OUTPUT,
                    match (use_gray, use_rgb) {
                        (true, false) => "pgm",
                        (false, true) => "ppm",
                        (_, _) => "pbm",
                    }
                )[..],
            ));

        run_cmd(cmd, CONVERT_BINARY)?;
        self.stepper.wait();
        Ok(())
    }

    /// Returns the path to the temporary folder with some path appended.
    fn append_to_temp<'a, S: Into<&'a str>>(&self, path: S) -> PathBuf {
        self.folder.as_ref().unwrap().path().join(path.into())
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

/// Runs a Command and handles the outcome of it.
fn run_cmd<'a>(mut cmd: Command, cmd_name: &'a str) -> Result<(), ErrorMessage> {
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

// If no output path was specified,
// it will be determined depending on the input path: The output will be placed in the same
// folder and with `-ironed` as suffix. The function also creates a temporary folder for the
// file operations and copies the input PDF file into this location as `input.pdf`.
