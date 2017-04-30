extern crate clap;
#[macro_use]
extern crate slog;
extern crate sloggers;
#[macro_use]
extern crate trackable;

use clap::{App, Arg};
use sloggers::Config;

fn main() {
    let matches = App::new("hello")
        .arg(Arg::with_name("CONFIG_FILE").index(1).required(true))
        .get_matches();
    let config_file = matches.value_of("CONFIG_FILE").unwrap();

    let config = track_try_unwrap!(Config::from_toml_file(config_file));
    let loggers = track_try_unwrap!(config.build());
    for (id, logger) in loggers {
        info!(logger, "Hello {:?}", id);
    }
}
