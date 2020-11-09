mod convert;
mod error;
mod pdf;
mod run;
mod tesseract;
mod unpaper;
mod util;

#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;
use clap::{App, ArgMatches};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    env_logger::Builder::new()
        .filter(
            None,
            match matches.is_present("verbose") {
                true => log::LevelFilter::Debug,
                false => log::LevelFilter::Info,
            },
        )
        .init();
    match convert(matches) {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    };
}

/// Does the conversion.
fn convert(matches: ArgMatches) -> Result<(), error::ErrorMessage> {
    let run = run::Run::new(
        matches.value_of("INPUT").unwrap(),
        matches.value_of("output"),
        matches.is_present("gray"),
        matches.is_present("rgb"),
        matches.is_present("step"),
    )?;

    convert::execute(
        &run,
        matches.value_of("resolution").map(|x| x.into()),
        matches.value_of("convert-options").map(|x| x.into()),
    )?;
    unpaper::execute(
        &run,
        matches.value_of("layout").map(|x| x.into()),
        matches.value_of("output-pages").map(|x| x.into()),
        matches.value_of("unpaper-options").map(|x| x.into()),
    )?;
    convert::prepare_for_tesseract(
        &run,
        matches.value_of("resolution").map(|x| x.into()),
        matches.is_present("disable-unpaper"),
        matches.is_present("disable-tesseract"),
    )?;
    tesseract::execute(
        &run,
        matches.is_present("disable-tesseract"),
        matches.value_of("lang").map(|x| x.into()),
        matches.value_of("tesseract-options").map(|x| x.into()),
        matches.value_of("tesseract-threads").map(|x| x.into()),
    )?;
    pdf::unite(&run)?;

    Ok(())
}
