use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::{Serialize, Deserialize};
use toml;

use {Result, Build, LoggerBuilder};
use file::FileLoggerConfig;
use null::NullLoggerConfig;
use terminal::TerminalLoggerConfig;

/// Configuration of a logger builder.
pub trait Config: Sized + Serialize + for<'a> Deserialize<'a> {
    /// Logger builder.
    type Builder: Build;

    /// Makes a configuration from the specified TOML file.
    fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut f = track_try!(File::open(path));
        let mut toml = String::new();
        track_try!(f.read_to_string(&mut toml));
        track!(Self::from_toml(&toml))
    }

    /// Makes a configuration from the TOML text.
    fn from_toml(toml: &str) -> Result<Self> {
        let config = track_try!(toml::from_str(toml));
        Ok(config)
    }

    /// Converts to TOML text.
    fn to_toml(&self) -> Result<String> {
        let toml = track_try!(toml::to_string(self));
        Ok(toml)
    }

    /// Makes a logger builder associated with this configuration.
    fn try_into_builder(self) -> Result<Self::Builder>;
}

/// The configuration of `LoggerBuilder`.
///
/// # Examples
///
/// Null logger.
///
/// ```
/// use sloggers::{Config, LoggerConfig};
///
/// let toml = r#"
/// type = "null"
/// "#;
/// let _config = LoggerConfig::from_toml(toml).unwrap();
/// ```
///
/// Terminal logger.
///
/// ```
/// use sloggers::{Config, LoggerConfig};
///
/// let toml = r#"
/// type = "terminal"
/// level = "warning"
/// "#;
/// let _config = LoggerConfig::from_toml(toml).unwrap();
/// ```
///
/// File logger.
///
/// ```
/// use sloggers::{Config, LoggerConfig};
///
/// let toml = r#"
/// type = "file"
/// path = "/path/to/file.log"
/// timezone = "utc"
/// "#;
/// let _config = LoggerConfig::from_toml(toml).unwrap();
/// ```
#[allow(missing_docs)]
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
