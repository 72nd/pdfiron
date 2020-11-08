use crate::error::ErrorMessage;
use crate::run::{PortableAnymap, Run, START_PDF};
use crate::util;

use std::process::Command;

/// Name of the convert binary.
const CONVERT_BINARY: &str = "convert";

/// The pages of the input document are extracted to images from the document. Takes an
/// optional name of the convert binary (specify by the command line argument).
pub fn execute<'a>(
    run: &Run,
    resolution: Option<&'a str>,
    options: Option<&'a str>,
) -> Result<(), ErrorMessage> {
    run.log_step("Extracting images form input PDF");

    let mut cmd = Command::new(CONVERT_BINARY);
    cmd.arg("-units").arg("PixelsPerInch");

    // Color mode
    match run.format {
        PortableAnymap::Graymap => cmd
            .arg("-colorspace")
            .arg("gray")
            .arg("-depth")
            .arg("8")
            .arg("-background")
            .arg("white")
            .arg("-alpha")
            .arg("Off"),
        PortableAnymap::Bitmap => cmd
            .arg("-depth")
            .arg("8")
            .arg("-background")
            .arg("white")
            .arg("-alpha")
            .arg("Off"),
        PortableAnymap::Pixmap => cmd.arg("-type").arg("Bilevel"),
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
        .arg(run.build_path("a_%03d"));

    println!("{:?}", &cmd);

    util::run_cmd(cmd, CONVERT_BINARY)?;
    run.wait();
    Ok(())
}
