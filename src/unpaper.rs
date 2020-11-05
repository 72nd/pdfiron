use crate::error::ErrorMessage;
use crate::run::Run;

use std::thread;

use bus::Bus;

/// Name of the unpaper binary.
const UNPAPER_BINARY: &str = "unpaper";

/// Executes the document enhancement application unpaper. To speed up the process, multiple
/// threads (as many cores as the CPU offer) will be used at the same time. It's possible to use
/// additional unpaper arguments by submitting them via the options parameter.
pub fn execute<'a>(
    run: &Run,
    layout: Option<&'a str>,
    output_pages: Option<&'a str>,
    options: Option<&'a str>,
) -> Result<(), ErrorMessage> {
    run.stepper.log_step(
    Ok(())
}
