use crate::error::ErrorMessage;
use crate::run::{Format, Run, START_PDF};
use crate::util;

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use num_cpus;

/// Name of the convert binary.
const CONVERT_BINARY: &str = "convert";

/// The pages of the input document are extracted to images from the document. Takes an
/// optional name of the convert binary (specify by the command line argument).
pub fn execute<'a>(
    run: &Run,
    resolution: Option<String>,
    options: Option<String>,
) -> Result<(), ErrorMessage> {
    run.log_step("Extracting images form input PDF");

    let pages = util::count_pdf_pages(run.prepend_with_temp_folder(START_PDF))?;
    let input_files: Vec<(PathBuf, PathBuf)> = (0..pages)
        .map(|x| {
            (
                run.prepend_with_temp_folder(format!("{}[{}]", START_PDF, x)),
                run.build_path(format!("a_{}", x), None),
            )
        })
        .collect();

    let files_arc = Arc::new(Mutex::new(input_files));
    let format = Arc::new(run.format);
    let resolution = Arc::new(resolution);
    let options = Arc::new(options);
    let mut handles = vec![];

    for _ in 1..num_cpus::get() {
        let files_arc = Arc::clone(&files_arc);
        let format = Arc::clone(&format);
        let resolution = Arc::clone(&resolution);
        let options = Arc::clone(&options);

        let handle = thread::spawn(move || convert_thread(files_arc, format, resolution, options));
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap()?;
    }

    run.wait();
    Ok(())
}

/// Converts the intermediate portable anymaps (ether convert output or unpaper output depending
/// whether unpaper was executed or not) and converts them into tiff's for Tesseract. Otherwise the
/// resolution and or size could be affected.
pub fn prepare_for_tesseract<'a>(
    run: &Run,
    resolution: Option<String>,
    disable_unpaper: bool,
    disable_tesseract: bool,
) -> Result<(), ErrorMessage> {
    if disable_tesseract {
        return Ok(());
    }
    run.log_step("Converting images for Tesseract input");

    let input_files = run.image_files(match disable_unpaper {
        true => "a_",
        false => "b_",
    })?;
    let mut files: Vec<(PathBuf, PathBuf)> = vec![];
    for input in input_files {
        files.push((
            input.clone(),
            run.build_path(format!("c_{}_%03d", util::file_name(input)), Some("tiff")),
        ));
    }

    let files_arc = Arc::new(Mutex::new(files));
    let format = Arc::new(Format::Tiff);
    let resolution = Arc::new(resolution);
    let options = Arc::new(None);
    let mut handles = vec![];

    for _ in 1..num_cpus::get() {
        let files_arc = Arc::clone(&files_arc);
        let format = Arc::clone(&format);
        let resolution = Arc::clone(&resolution);
        let options = Arc::clone(&options);

        let handle = thread::spawn(move || convert_thread(files_arc, format, resolution, options));
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap()?;
    }

    run.wait();
    Ok(())
}

/// An convert execution thread to run multiple convert instances in parallel. Takes a tuple with
/// the input and output path and executes the conversion. Will return if no more files are
/// available to be processed.
fn convert_thread<'a>(
    input: Arc<Mutex<Vec<(PathBuf, PathBuf)>>>,
    format: Arc<Format>,
    resolution: Arc<Option<String>>,
    options: Arc<Option<String>>,
) -> Result<(), ErrorMessage> {
    loop {
        let mut files = input.lock().unwrap();
        let file = match files.pop() {
            Some(x) => x,
            None => return Ok(()),
        };
        drop(files);
        let mut cmd = Command::new(CONVERT_BINARY);
        cmd.arg("-units").arg("PixelsPerInch");

        // Color mode
        match *format {
            Format::Graymap => cmd
                .arg("-colorspace")
                .arg("gray")
                .arg("-depth")
                .arg("8")
                .arg("-background")
                .arg("white")
                .arg("-alpha")
                .arg("Off"),
            Format::Bitmap => cmd
                .arg("-depth")
                .arg("8")
                .arg("-background")
                .arg("white")
                .arg("-alpha")
                .arg("Off"),
            Format::Pixmap => cmd.arg("-type").arg("Bilevel"),
            Format::Tiff => &cmd,
        };

        set_density(&mut cmd, Arc::clone(&resolution))?;

        // Optional arguments
        cmd.args(match *format {
            Format::Tiff => vec![],
            _ => match *options {
                Some(ref x) => x.split(" ").collect::<Vec<&str>>(),
                None => vec![],
            },
        });

        cmd.arg(&file.0);
        cmd.arg(&file.1);

        debug!("Going to convert {}", &file.0.display());
        util::run_cmd(cmd, CONVERT_BINARY)?;
        debug!(
            "{} was converted to {}",
            &file.0.display(),
            &file.1.display()
        );
    }
}

/// Adds the density argument for a given command.
fn set_density<'a>(cmd: &mut Command, resolution: Arc<Option<String>>) -> Result<(), ErrorMessage> {
    let res = match *resolution {
        Some(ref x) => match x.parse::<u64>() {
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
    Ok(())
}
