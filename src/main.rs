mod error;

#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;
use clap::App;

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
    let _err = error::GenericError::new("hoi");
}
