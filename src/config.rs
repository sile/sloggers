// TODO: common
// - stdlog
// - level
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use slog::Logger;
use toml;

use Result;
use loggers::{NullLoggerBuilder, TerminalLoggerBuilder};

pub type LoggerId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub loggers: HashMap<LoggerId, LoggerConfig>,
}
impl Config {
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut f = track_try!(File::open(path));
        let mut toml = String::new();
        track_try!(f.read_to_string(&mut toml));
        track!(Self::from_toml(&toml))
    }
    pub fn from_toml(toml: &str) -> Result<Self> {
        let config = track_try!(toml::from_str(toml));
        Ok(config)
    }

    // TODO: `values`を取るようにする
    // Builderトレイトを定義して云々
    pub fn build(&self) -> Result<HashMap<LoggerId, Logger>> {
        self.loggers
            .iter()
            .map(|(id, config)| {
                     let logger = track_try!(config.build(), "id={:?}", id);
                     Ok((id.clone(), logger))
                 })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LoggerConfig {
    #[serde(rename = "null")]
    Null,

    #[serde(rename = "terminal")]
    Terminal(TerminalLoggerConfig),

    #[serde(rename = "file")]
    File,
}
impl LoggerConfig {
    pub fn build(&self) -> Result<Logger> {
        match *self {
            LoggerConfig::Null => Ok(NullLoggerBuilder::new().finish(o!())),
            LoggerConfig::Terminal(ref c) => Ok(TerminalLoggerBuilder::from_config(c).finish(o!())),
            LoggerConfig::File => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalLoggerConfig {
    pub format: TerminalFormat,
    pub timezone: Timezone,
    pub output: TerminalOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalFormat {
    #[serde(rename = "full")]
    Full,

    #[serde(rename = "compact")]
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Timezone {
    #[serde(rename = "utc")]
    Utc,

    #[serde(rename = "local")]
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalOutput {
    #[serde(rename = "stdout")]
    Stdout,

    #[serde(rename = "stderr")]
    Stderr,
}
