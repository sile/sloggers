use std::io::{self, Write};
use slog::{Level, LevelFilter, Drain};
use slog_term;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    #[serde(rename = "trace")]
    Trace,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "critical")]
    Critical,
}
impl Severity {
    pub fn as_level(&self) -> Level {
        match *self {
            Severity::Trace => Level::Trace,
            Severity::Debug => Level::Debug,
            Severity::Info => Level::Info,
            Severity::Warning => Level::Warning,
            Severity::Error => Level::Error,
            Severity::Critical => Level::Critical,
        }
    }
    pub fn set_level_filter<D: Drain>(&self, drain: D) -> LevelFilter<D> {
        LevelFilter::new(drain, self.as_level())
    }
}
impl Default for Severity {
    fn default() -> Self {
        Severity::Info
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    #[serde(rename = "full")]
    Full,

    #[serde(rename = "compact")]
    Compact,
}
impl Default for Format {
    fn default() -> Self {
        Format::Full
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Timezone {
    #[serde(rename = "utc")]
    Utc,

    #[serde(rename = "local")]
    Local,
}
impl Default for Timezone {
    fn default() -> Self {
        Timezone::Local
    }
}
impl Timezone {
    pub fn to_timestamp_fn(&self) -> fn(&mut Write) -> io::Result<()> {
        match *self {
            Timezone::Utc => slog_term::timestamp_utc,
            Timezone::Local => slog_term::timestamp_local,
        }
    }
}
