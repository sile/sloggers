extern crate clap;
#[macro_use]
extern crate slog;
extern crate sloggers;
extern crate tomlconv;
#[macro_use]
extern crate trackable;

use clap::{App, Arg};
use sloggers::{Build, Config, LoggerConfig};
use tomlconv::FromToml;

fn main() {
    let matches = App::new("hello")
        .arg(Arg::with_name("CONFIG_FILE").index(1).required(true))
        .get_matches();
    let config_file = matches.value_of("CONFIG_FILE").unwrap();

    let config = track_try_unwrap!(LoggerConfig::from_toml_file(config_file));
    let builder = track_try_unwrap!(config.try_into_builder());
    let logger = track_try_unwrap!(builder.build());
    info!(logger, "Hello World!");
}
