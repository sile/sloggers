use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Serialize, Deserialize};
use toml;

use {Result, Build, LoggerBuilder};
use file::FileLoggerConfig;
use null::NullLoggerConfig;
use terminal::TerminalLoggerConfig;

pub trait Config: Sized + Serialize + for<'a> Deserialize<'a> {
    type Builder: Build;
    fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut f = track_try!(File::open(path));
        let mut toml = String::new();
        track_try!(f.read_to_string(&mut toml));
        track!(Self::from_toml(&toml))
    }
    fn from_toml(toml: &str) -> Result<Self> {
        let config = track_try!(toml::from_str(toml));
        Ok(config)
    }
    fn try_into_builder(self) -> Result<Self::Builder>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LoggerConfig {
    #[serde(rename = "file")]
    File(FileLoggerConfig),

    #[serde(rename = "null")]
    Null(NullLoggerConfig),

    #[serde(rename = "terminal")]
    Terminal(TerminalLoggerConfig),
}
impl Config for LoggerConfig {
    type Builder = LoggerBuilder;
    fn try_into_builder(self) -> Result<Self::Builder> {
        match self {
            LoggerConfig::File(c) => track!(c.try_into_builder()).map(LoggerBuilder::File),
            LoggerConfig::Null(c) => track!(c.try_into_builder()).map(LoggerBuilder::Null),
            LoggerConfig::Terminal(c) => track!(c.try_into_builder()).map(LoggerBuilder::Terminal),
        }
    }
}
