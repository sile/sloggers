use super::format::MsgFormatConfig;
use super::{Facility, SyslogBuilder};
use crate::types::{OverflowStrategy, Severity, SourceLocation};
use crate::Config;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ffi::CStr;

/// The configuration of `SyslogBuilder`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(default)]
pub struct SyslogConfig {
    /// Log level.
    pub level: Severity,

    /// How to format syslog messages with structured data.
    ///
    /// Possible values are `default` and `basic`.
    ///
    /// See [`MsgFormat`] for more information.
    ///
    /// [`MsgFormat`]: format/trait.MsgFormat.html
    pub format: MsgFormatConfig,

    /// Source code location
    pub source_location: SourceLocation,

    /// The syslog facility to send logs to.
    pub facility: Facility,

    /// Asynchronous channel size
    pub channel_size: usize,

    /// Whether to drop logs on overflow.
    ///
    /// The possible values are `drop`, `drop_and_report`, or `block`.
    ///
    /// The default value is `drop_and_report`.
    pub overflow_strategy: OverflowStrategy,

    /// The name of this program, for inclusion with log messages. (POSIX calls
    /// this the “tag”.)
    ///
    /// The string must not contain any zero (ASCII NUL) bytes.
    ///
    /// # Default value
    ///
    /// If a name is not given, the default behavior depends on the libc
    /// implementation in use.
    ///
    /// BSD, GNU, and Apple libc use the actual process name. µClibc uses the
    /// constant string `syslog`. Fuchsia libc and musl libc use no name at
    /// all.
    pub ident: Option<Cow<'static, CStr>>,

    /// Include the process ID in log messages.
    pub log_pid: bool,

    /// Whether to immediately open a connection to the syslog server.
    ///
    /// If true, a connection will be immediately opened. If false, the
    /// connection will only be opened when the first log message is submitted.
    ///
    /// The default is platform-defined, but on most platforms, the default is
    /// `true`.
    ///
    /// On OpenBSD 5.6 and newer, this setting has no effect, because that
    /// platform uses a dedicated system call instead of a socket for
    /// submitting syslog messages.
    pub log_delay: Option<bool>,

    /// Also emit log messages on `stderr` (**see warning**).
    ///
    /// # Warning
    ///
    /// The libc `syslog` function is not subject to the global mutex that
    /// Rust uses to synchronize access to `stderr`. As a result, if one thread
    /// writes to `stderr` at the same time as another thread emits a log
    /// message with this option, the log message may appear in the middle of
    /// the other thread's output.
    ///
    /// Note that this problem is not specific to Rust or this crate. Any
    /// program in any language that writes to `stderr` in one thread and logs
    /// to `syslog` with `LOG_PERROR` in another thread at the same time will
    /// have the same problem.
    ///
    /// The exception is the `syslog` implementation in GNU libc, which
    /// implements this option by writing to `stderr` through the C `stdio`
    /// API (as opposed to the `write` system call), which has its own mutex.
    /// As long as all threads write to `stderr` using the C `stdio` API, log
    /// messages on this platform will never appear in the middle of other
    /// `stderr` output. However, Rust does not use the C `stdio` API for
    /// writing to `stderr`, so even on GNU libc, using this option may result
    /// in garbled output.
    pub log_perror: bool,
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
            level: Severity::default(),
            format: MsgFormatConfig::default(),
            source_location: SourceLocation::default(),
            facility: Facility::default(),
            channel_size: 1024,
            overflow_strategy: OverflowStrategy::default(),
            ident: None,
            log_pid: false,
            log_delay: None,
            log_perror: false,
        }
    }
}

impl Config for SyslogConfig {
    type Builder = SyslogBuilder;

    fn try_to_builder(&self) -> crate::Result<Self::Builder> {
        let mut b = SyslogBuilder::new();

        b.level(self.level);
        b.source_location(self.source_location);
        b.facility(self.facility);
        b.channel_size(self.channel_size);
        b.overflow_strategy(self.overflow_strategy);

        // Don't make this call if not using a non-default format, or there
        // will be an unnecessary extra allocation. `SyslogBuilder::new`
        // already allocates an `Arc<dyn MsgFormat>`, and this call allocates
        // another one.
        if self.format != MsgFormatConfig::Default {
            b.format_arc((&self.format).into());
        }

        if let Some(ident) = &self.ident {
            b.ident(ident.clone());
        }

        if self.log_pid {
            b.log_pid();
        }

        if let Some(log_delay) = self.log_delay {
            if log_delay {
                b.log_odelay();
            } else {
                b.log_ndelay();
            }
        }

        if self.log_perror {
            b.log_perror();
        }

        Ok(b)
    }
}
