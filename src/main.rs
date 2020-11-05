mod error;
mod run;

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
            match matches.is_present("debug") {
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
    let _run = run::Run::new(matches.value_of("INPUT").unwrap())?
        .output(matches.value_of("output"))
        .init();
    Ok(())
}
