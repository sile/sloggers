//! Commonly used types.
use slog::{Level, LevelFilter, Drain};

/// The severity of a log record.
///
/// # Examples
///
/// The default value:
///
/// ```
/// use sloggers::types::Severity;
///
/// assert_eq!(Severity::default(), Severity::Info);
/// ```
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}
impl Severity {
    /// Converts `Severity` to `Level`.
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

    /// Sets `LevelFilter` to `drain`.
    pub fn set_level_filter<D: Drain>(&self, drain: D) -> LevelFilter<D> {
        LevelFilter::new(drain, self.as_level())
    }
}
impl Default for Severity {
    fn default() -> Self {
        Severity::Info
    }
}

/// The format of log records.
///
/// # Examples
///
/// The default value:
///
/// ```
/// use sloggers::types::Format;
///
/// assert_eq!(Format::default(), Format::Full);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// Full format.
    Full,

    /// Compact format.
    Compact,
}
impl Default for Format {
    fn default() -> Self {
        Format::Full
    }
}

/// Time Zone.
///
/// # Examples
///
/// The default value:
///
/// ```
/// use sloggers::types::TimeZone;
///
/// assert_eq!(TimeZone::default(), TimeZone::Local);
/// ```
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeZone {
    Utc,
    Local,
}
impl Default for TimeZone {
    fn default() -> Self {
        TimeZone::Local
    }
}
