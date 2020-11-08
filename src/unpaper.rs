use crate::error::ErrorMessage;
use crate::run::Run;
use crate::util;

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use num_cpus;

/// Name of the unpaper binary.
const UNPAPER_BINARY: &str = "unpaper";

/// Executes the document enhancement application unpaper. To speed up the process, multiple
/// threads (as many cores as the CPU offer) will be used at the same time. It's possible to use
/// additional unpaper arguments by submitting them via the options parameter.
pub fn execute(
    run: &Run,
    layout: Option<String>,
    output_pages: Option<String>,
    options: Option<String>,
) -> Result<(), ErrorMessage> {
    run.log_step("Enhance with unpaper");

    let input_files = run.image_files("a_")?;
    let mut files: Vec<(PathBuf, PathBuf)> = vec![];
    for input in input_files {
        files.push((
            input.clone(),
            run.build_path(format!("b_{}_%03d", util::file_name(input)), None),
        ));
    }

    let files_arc = Arc::new(Mutex::new(files));
    let layout = Arc::new(layout);
    let output_pages = Arc::new(output_pages);
    let options = Arc::new(options);
    let mut handles = vec![];

    for _ in 1..num_cpus::get() {
        let files_arc = Arc::clone(&files_arc);
        let layout = Arc::clone(&layout);
        let output_pages = Arc::clone(&output_pages);
        let options = Arc::clone(&options);
        let handle =
            thread::spawn(move || unpaper_thread(files_arc, layout, output_pages, options));
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    run.wait();
    Ok(())
}

/// An unpaper execution thread. Takes one image from the shared vector and process it. When
/// unpaper finishes the next image will be pulled from the vector of tuples. The first element in
/// the tuple is the input path, the second points to the output file using the unpaper number
/// format. When the bus is empty, the thread terminates.
fn unpaper_thread(
    input: Arc<Mutex<Vec<(PathBuf, PathBuf)>>>,
    layout: Arc<Option<String>>,
    output_pages: Arc<Option<String>>,
    options: Arc<Option<String>>,
) {
    loop {
        let mut files = input.lock().unwrap();
        let file = match files.pop() {
            Some(x) => x,
            None => return,
        };
        drop(files);

        let mut cmd = Command::new(UNPAPER_BINARY);
        cmd.args(match *options {
            Some(ref x) => x.split(" ").collect::<Vec<&str>>(),
            None => vec![],
        });
        match *layout {
            Some(ref x) => {
                cmd.arg("--layout").arg(x);
            }
            None => {}
        };
        match *output_pages {
            Some(ref x) => {
                cmd.arg("--output-pages").arg(x);
            }
            None => {}
        };
        cmd.arg(&file.0);
        cmd.arg(&file.1);

        debug!("Going to enhance {} with unpaper", &file.0.display());
        match util::run_cmd(cmd, UNPAPER_BINARY) {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        };
        debug!(
            "{} was enhanced and saved as {}",
            &file.0.display(),
            &file.1.display()
        );
    }
}
