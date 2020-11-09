use crate::error::ErrorMessage;
use crate::run::Run;
use crate::util;

use std::process::Command;

/// Name of the pdfunite binary.
const PDFUNITE_BINARY: &str = "pdfunite";

/// Unites the PDF-files into one file.
pub fn unite(run: &Run) -> Result<(), ErrorMessage> {
    run.log_step("Combine PDF");

    let mut cmd = Command::new(PDFUNITE_BINARY);
    let mut inputs: Vec<_> = run.query_files("d_")?;
    inputs.sort();
    cmd.args(inputs);
    cmd.arg(run.output_path());

    util::run_cmd(cmd, PDFUNITE_BINARY)?;
    run.wait();
    Ok(())
}
