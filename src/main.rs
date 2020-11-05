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
    let mut run = run::Run::new(
        matches.value_of("INPUT").unwrap(),
        matches.is_present("step"),
    )?;
    run.output(matches.value_of("output"))
        .do_tesseract(matches.is_present("do_tesseract"))
        .do_unpaper(matches.is_present("do_unpaper"));

    run.init()?;

    run.convert_to_img(
        matches.is_present("gray"),
        matches.is_present("rgb"),
        matches.value_of("resolution"),
        matches.value_of("convert_options"),
    )?;

    Ok(())
}
