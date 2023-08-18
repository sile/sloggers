#[macro_use]
extern crate slog;
#[macro_use]
extern crate trackable;

use clap::{Arg, Command};
use sloggers::{Build, Config, LoggerConfig};

fn main() {
    let matches = Command::new("hello")
        .arg(Arg::new("CONFIG_FILE").index(1).required(true))
        .get_matches();
    let config_file = matches.get_one::<String>("CONFIG_FILE").unwrap();

    let config: LoggerConfig = track_try_unwrap!(serdeconv::from_toml_file(config_file));
    let builder = track_try_unwrap!(config.try_to_builder());
    let logger = track_try_unwrap!(builder.build());
    info!(logger, "Hello World!");
}
