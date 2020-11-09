use crate::error::ErrorMessage;
use crate::run::Run;
use crate::util;

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

/// Name of the Tesseract binary.
const TESSERACT_BINARY: &str = "tesseract";

/// Takes the prepared tiff files and runs the OCR with Tesseract on each file.
pub fn execute(
    run: &Run,
    disable_tesseract: bool,
    lang: Option<String>,
    options: Option<String>,
    threads: Option<String>,
) -> Result<(), ErrorMessage> {
    if disable_tesseract {
        return Ok(());
    }
    run.log_step("OCR");
    let mut files: Vec<(PathBuf, PathBuf)> = vec![];
    for input in run.query_files("c_")? {
        files.push((
            input.clone(),
            run.build_path(format!("d_{}", util::file_name(input)), Some("pdf")),
        ));
    }

    let files = Arc::new(Mutex::new(files));
    let lang = Arc::new(lang);
    let options = Arc::new(options);
    let mut handles = vec![];

    let threads = match threads {
        Some(x) => x.parse::<u64>().unwrap(),
        None => 2,
    };
    for _ in 0..threads {
        let files = Arc::clone(&files);
        let lang = Arc::clone(&lang);
        let options = Arc::clone(&options);
        let handle = thread::spawn(move || tesseract_thread(files, lang, options));
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap()?;
    }

    run.wait();
    Ok(())
}

/// Tesseract execution thread. Takes one image from the shared vector and process it. When
/// Tesseract finishes the next image will be pulled from the vector of tuples. The first element in
/// the tuple is the input path, the second points to the output file using the Tesseract number
/// format. When the bus is empty, the thread terminates.
fn tesseract_thread(
    input: Arc<Mutex<Vec<(PathBuf, PathBuf)>>>,
    lang: Arc<Option<String>>,
    options: Arc<Option<String>>,
) -> Result<(), ErrorMessage> {
    loop {
        let mut files = input.lock().unwrap();
        let file = match files.pop() {
            Some(x) => x,
            None => return Ok(()),
        };
        drop(files);

        let mut cmd = Command::new(TESSERACT_BINARY);
        cmd.arg("-l").arg(match *lang {
            Some(ref x) => x,
            None => "eng",
        });
        cmd.args(match *options {
            Some(ref x) => x.split(" ").collect::<Vec<&str>>(),
            None => vec![],
        });
        cmd.arg(&file.0);
        cmd.arg(&file.1);
        cmd.arg("pdf");

        debug!("Going to execute OCR on {}", &file.0.display());
        util::run_cmd(cmd, TESSERACT_BINARY)?;
        debug!(
            "OCR result of {} was written to {}",
            &file.0.display(),
            &file.1.display()
        );
    }
}
