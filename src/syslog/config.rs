use super::SyslogBuilder;
use crate::types::{OverflowStrategy, Severity, SourceLocation};
use crate::Config;
use serde::{Deserialize, Serialize};
use slog_syslog::config::ConfiguredMsgFormat;
use slog_syslog::format::MsgFormat;
use std::sync::Arc;

/// The configuration of `SyslogBuilder`.
///
/// # TOML Example
///
/// ```
/// # use sloggers::syslog::SyslogConfig;
/// # use sloggers::types::Severity;
/// # use slog_syslog::Facility;
/// #
/// # const TOML_CONFIG: &'static str = r#"
/// format = "basic"
/// ident = "foo"
/// facility = "daemon"
/// log_pid = true
/// level = "warning"
/// # "#;
/// #
/// # let config: SyslogConfig = toml::de::from_str(TOML_CONFIG).expect("deserialization failed");
/// # assert_eq!(config.level, Severity::Warning);
/// # assert_eq!(config.syslog_settings.facility, Facility::Daemon);
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(default)]
pub struct SyslogConfig {
    /// Settings specific to syslog.
    ///
    /// This field is [flattened]: in a configuration file, the syslog settings
    /// should appear directly at this level, not in a `syslog_settings`
    /// subsection.
    ///
    /// [Flattened]: https://serde.rs/field-attrs.html#flatten
    #[serde(flatten)]
    pub syslog_settings: slog_syslog::config::SyslogConfig,

    /// Log level.
    pub level: Severity,

    /// Source code location
    pub source_location: SourceLocation,

    /// Asynchronous channel size
    pub channel_size: usize,

    /// Whether to drop logs on overflow.
    ///
    /// The possible values are `drop`, `drop_and_report`, or `block`.
    ///
    /// The default value is `drop_and_report`.
    pub overflow_strategy: OverflowStrategy,
}

impl SyslogConfig {
    /// Creates a new `SyslogConfig` with default settings.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for SyslogConfig {
    fn default() -> Self {
        SyslogConfig {
            syslog_settings: slog_syslog::config::SyslogConfig::default(),
            level: Severity::default(),
            source_location: SourceLocation::default(),
            channel_size: 1024,
            overflow_strategy: OverflowStrategy::default(),
        }
    }
}

impl Config for SyslogConfig {
    type Builder = SyslogBuilder;

    fn try_to_builder(&self) -> crate::Result<Self::Builder> {
        let inner_config = self.syslog_settings.clone();
        let config_format = ConfiguredMsgFormat::from(inner_config.format.clone());

        let inner_builder: slog_syslog::SyslogBuilder<Arc<dyn MsgFormat + Send + Sync + 'static>> =
            inner_config.into_builder().format(Arc::new(config_format));

        let mut b = SyslogBuilder::from(inner_builder);

        b.level(self.level);
        b.source_location(self.source_location);
        b.channel_size(self.channel_size);
        b.overflow_strategy(self.overflow_strategy);

        Ok(b)
    }
}
