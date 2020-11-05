mod convert;
mod error;
mod run;
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
        matches.is_present("step"),
    )?;

    convert::execute(
        &run,
        matches.is_present("gray"),
        matches.is_present("rgb"),
        matches.value_of("resolution"),
        matches.value_of("convert_options"),
    )?;

    Ok(())
}
