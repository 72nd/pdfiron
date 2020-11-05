use crate::error::ErrorMessage;
use crate::run::{Run, CONVERT_OUTPUT, START_PDF};
use crate::util;

use std::process::Command;

/// Name of the convert binary.
const CONVERT_BINARY: &str = "convert";

/// The pages of the input document are extracted to images from the document. Takes an
/// optional name of the convert binary (specify by the command line argument).
pub fn execute<'a>(
    run: &Run,
    use_gray: bool,
    use_rgb: bool,
    resolution: Option<&'a str>,
    options: Option<&'a str>,
) -> Result<(), ErrorMessage> {
    run.log_step("Extracting images form input PDF");

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
    cmd.arg(run.prepend_with_temp_folder(START_PDF))
        .arg(run.prepend_with_temp_folder(
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

    util::run_cmd(cmd, CONVERT_BINARY)?;
    run.wait();
    Ok(())
}
