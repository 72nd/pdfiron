mod error;
mod run;

use std::error::Error;

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
    convert(matches);
}

/// Does the conversion.
fn convert(matches: ArgMatches) {
    let _run = match run::Run::new(matches.value_of("INPUT").unwrap(), matches.value_of("output")) {
        Ok(x) => x,
        Err(e) => {
            error!("{}", e);
            return
        },
    };
}
