use crate::error::ErrorMessage;

use std::env;
use std::fs;
use std::io::stdin;
use std::path::PathBuf;

use shellexpand;
use tempfile::{Builder, TempDir};

/// Name of the start file in the temporary folder.
pub const START_PDF: &str = "input.pdf";

/// Enumeration of the three possible image formats used within the process.
#[derive(Debug, Clone, Copy)]
pub enum Format {
    /// RGB images, PBM.
    Bitmap,
    /// Gray images, PGM.
    Graymap,
    /// White/Black only images, PPM.
    Pixmap,
    /// TIFF used for Tesseract.
    Tiff,
}

impl Format {
    /// Returns the appropriate image format based on the user input.
    fn from(use_gray: bool, use_rgb: bool) -> Self {
        match (use_gray, use_rgb) {
            (true, false) => Format::Graymap,
            (false, true) => Format::Pixmap,
            (_, _) => Format::Bitmap,
        }
    }

    /// Returns the file extension for a given Anymap format.
    pub fn extension<'a>(self) -> &'a str {
        match self {
            Format::Bitmap => "pbm",
            Format::Graymap => "pgm",
            Format::Pixmap => "ppm",
            Format::Tiff => "tiff",
        }
    }
}

/// This struct contains all the needed information and states to go trough the different
/// conversion steps. The object manages the temporary folder.
pub struct Run {
    /// Absolute path to the input file.
    input: PathBuf,
    /// Absolute path to the output file.
    output: Option<PathBuf>,
    /// Temporary folder where the conversion happens.
    folder: TempDir,
    /// Provides a comfortable way to pause between the steps if the user did enable the function.
    /// Contains a boolean whether the wait should be executed or not.
    do_step: bool,
    /// Image file format used internally.
    pub format: Format,
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
    pub fn new<S: Into<String>>(
        input: S,
        output: Option<S>,
        use_gray: bool,
        use_rgb: bool,
        do_step: bool,
    ) -> Result<Self, ErrorMessage> {
        let input = Run::expand_path(input.into())?;
        Run::validate_input_file(&input)?;

        let rsl = Self {
            input: input.clone(),
            output: match output {
                Some(x) => Some(Run::expand_path(x.into())?),
                None => None,
            },
            folder: match Builder::new().prefix("pdfiron-").tempdir() {
                Ok(x) => x,
                Err(e) => {
                    return Err(ErrorMessage::new(format!(
                        "Couldn't create temp folder, {}",
                        e
                    )))
                }
            },
            do_step: do_step,
            format: Format::from(use_gray, use_rgb),
        };

        rsl.log_folder_path(rsl.folder.path().to_path_buf());

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
    pub fn prepend_with_temp_folder<'a, S: Into<String>>(&self, path: S) -> PathBuf {
        self.folder.path().join(path.into())
    }

    /// Returns a Vector with all paths of the files in the temporary folder with a given prefix
    /// and a optional file ending.
    pub fn query_files<'a>(&self, starts_with: &'a str) -> Result<Vec<PathBuf>, ErrorMessage> {
        let elements = match fs::read_dir(&self.folder) {
            Ok(x) => x,
            Err(e) => {
                return Err(ErrorMessage::new(format!(
                    "Couldn't read content of temp folder, {}",
                    e
                )))
            }
        };

        Ok(elements
            .map(|x| x.unwrap().path())
            .filter(|x| match x.file_name() {
                Some(z) => z.to_string_lossy().starts_with(starts_with),
                None => false,
            })
            .collect())
    }

    /// Joins (in this order) the temporary folder path with the given filename and the given
    /// extension. If None the extension will be determined.
    pub fn build_path<'a, S: Into<String>>(
        &self,
        filename: S,
        extension: Option<&'a str>,
    ) -> PathBuf {
        let mut pth = self.prepend_with_temp_folder(filename);
        pth.set_extension(match extension {
            Some(x) => x,
            None => self.format.extension(),
        });
        pth
    }

    /// Outputs the path of the temporary folder to the user via the debug facility. This enables
    /// the user to find the correct temporary working folder. If the Stepper is disabled, the path
    /// will be outputted on Debug level thus only be visible when the verbose mode is enabled.
    /// Needs the path to the temporary folder.
    pub fn log_folder_path(&self, folder: PathBuf) {
        match self.do_step {
            true => info!("File operations will be happening in {}", folder.display()),
            false => debug!("temporary folder is {}", folder.display()),
        }
    }

    /// The method logs the given step description. This helps the user to determine which steps
    /// was executed and thus when to look at the files and when to skip a pause without any
    /// further investigation. The log-level depends whether the stepper is enabled or not. Active
    /// it's info, otherwise it's debug.
    pub fn log_step<S: Into<String>>(&self, desc: S) {
        match self.do_step {
            true => info!("{}...", desc.into()),
            false => debug!("{}...", desc.into()),
        }
    }

    /// If the step mode was enabled the method will wait until user hits enter. This is used for
    /// the pause between steps mode. Allowing the user to tweak the files in the temporary folder
    pub fn wait(&self) {
        match self.do_step {
            true => {
                println!("Hit enter to proceed with next step...");
                let mut void = String::new();
                match stdin().read_line(&mut void) {
                    Ok(_) => {}
                    Err(e) => error!("couldn't read line, {}", e),
                }
            }
            false => {}
        }
    }

    /// Returns the output path for the PDF. If the user didn't specify a path, a default path will
    /// be used in the same folder as the input file.
    pub fn output_path(&self) -> PathBuf {
        match &self.output {
            Some(x) => x.clone(),
            None => {
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
        }
    }

    /// Shell expands a path and normalize it to an absolute path.
    fn expand_path(file: String) -> Result<PathBuf, ErrorMessage> {
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
