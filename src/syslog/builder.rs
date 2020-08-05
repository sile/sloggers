use super::adapter::{Adapter, DefaultAdapter};
use super::{Facility, Priority};
use crate::build::BuilderCommon;
#[cfg(feature = "slog-kvfilter")]
use crate::types::KVFilterParameters;
use crate::types::{OverflowStrategy, Severity, SourceLocation};
use crate::Build;
use crate::Result;
use slog::{Logger, OwnedKVList, Record};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fmt::{self, Debug};
use std::sync::Arc;

type InnerBuilder = slog_syslog::SyslogBuilder<Arc<dyn Adapter + Send + Sync + 'static>>;

/// A logger builder which builds loggers that send log records to a syslog server.
///
/// All settings have sensible defaults. Simply calling
/// `SyslogBuilder::new().build()` will yield a functional, reasonable
/// `Logger` in most situations. However, most applications will want to set
/// the `facility`.
///
/// The resulting logger will work asynchronously (the default channel size is 1024).
///
/// # Example
///
/// ```
/// use slog::info;
/// use sloggers::Build;
/// use sloggers::types::Severity;
/// use sloggers::syslog::{Facility, SyslogBuilder};
/// use std::ffi::CStr;
///
/// # fn main() -> Result<(), sloggers::Error> {
/// let logger = SyslogBuilder::new()
///     .facility(Facility::User)
///     .level(Severity::Debug)
///     .ident(CStr::from_bytes_with_nul(b"sloggers-example-app\0").unwrap())
///     .build()?;
///
/// info!(logger, "Hello, world! This is a test message from `sloggers::syslog`.");
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SyslogBuilder {
    common: BuilderCommon,
    inner: Option<InnerBuilder>,
}

impl Default for SyslogBuilder {
    fn default() -> Self {
        SyslogBuilder {
            common: BuilderCommon::default(),
            inner: Some(slog_syslog::SyslogBuilder::new().adapter(Arc::new(DefaultAdapter))),
        }
    }
}

impl From<InnerBuilder> for SyslogBuilder {
    fn from(builder: InnerBuilder) -> Self {
        SyslogBuilder {
            common: BuilderCommon::default(),
            inner: Some(builder),
        }
    }
}

impl SyslogBuilder {
    /// Makes a new `SyslogBuilder` instance.
    pub fn new() -> Self {
        SyslogBuilder::default()
    }

    fn take_inner(&mut self) -> InnerBuilder {
        self.inner
            .take()
            .expect("`SyslogBuilder` in invalid state caused by prior panic")
    }

    fn borrow_inner(&self) -> &InnerBuilder {
        self.inner
            .as_ref()
            .expect("`SyslogBuilder` in invalid state caused by prior panic")
    }

    /// Sets the source code location type this logger will use.
    pub fn source_location(&mut self, source_location: SourceLocation) -> &mut Self {
        self.common.source_location = source_location;
        self
    }

    /// Sets the syslog facility to send logs to.
    ///
    /// By default, this is [`Facility::User`].
    ///
    /// [`Facility::User`]: enum.Facility.html#variant.User
    pub fn facility(&mut self, facility: Facility) -> &mut Self {
        self.inner = Some(self.take_inner().facility(facility));
        self
    }

    /// Sets the overflow strategy for the logger.
    pub fn overflow_strategy(&mut self, overflow_strategy: OverflowStrategy) -> &mut Self {
        self.common.overflow_strategy = overflow_strategy;
        self
    }

    /// Sets the name of this program, for inclusion with log messages.
    /// (POSIX calls this the “tag”.)
    ///
    /// The supplied string must not contain any zero (ASCII NUL) bytes.
    ///
    /// # Default value
    ///
    /// If a name is not given, the default behavior depends on the libc
    /// implementation in use.
    ///
    /// BSD, GNU, and Apple libc use the actual process name. µClibc uses the
    /// constant string `syslog`. Fuchsia libc and musl libc use no name at
    /// all.
    ///
    /// # When to use
    ///
    /// This method converts the given string to a C-compatible string at run
    /// time. It should only be used if the process name is obtained
    /// dynamically, such as from a configuration file.
    ///
    /// If the process name is constant, use the `ident` method instead.
    ///
    /// # Panics
    ///
    /// This method panics if the supplied string contains any null bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use sloggers::Build;
    /// use sloggers::syslog::SyslogBuilder;
    ///
    /// # let some_string = "hello".to_string();
    /// let my_ident: String = some_string;
    ///
    /// let logger = SyslogBuilder::new()
    ///     .ident_str(my_ident)
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// # Data use and lifetime
    ///
    /// This method takes an ordinary Rust string, copies it into a
    /// [`CString`] (which appends a null byte on the end), and passes that to
    /// the `ident` method.
    ///
    /// [`CString`]: https://doc.rust-lang.org/std/ffi/struct.CString.html
    pub fn ident_str(&mut self, ident: impl AsRef<str>) -> &mut Self {
        let cs = CString::new(ident.as_ref()).expect(
            "`sloggers::syslog::SyslogBuilder::ident` called with string that contains null bytes",
        );

        self.ident(cs)
    }

    /// Sets the name of this program, for inclusion with log messages.
    /// (POSIX calls this the “tag”.)
    ///
    /// # Default value
    ///
    /// If a name is not given, the default behavior depends on the libc
    /// implementation in use.
    ///
    /// BSD, GNU, and Apple libc use the actual process name. µClibc uses the
    /// constant string `syslog`. Fuchsia libc and musl libc use no name at
    /// all.
    ///
    /// # When to use
    ///
    /// This method should be used if you already have a C-compatible string to
    /// use for the process name, or if the process name is constant (as
    /// opposed to taken from a configuration file or command line parameter).
    ///
    /// # Data use and lifetime
    ///
    /// This method takes a C-compatible string, either owned or with the
    /// `'static` lifetime. This ensures that the string remains available for
    /// the entire time that the system libc might need it (until `closelog` is
    /// called, which happens when the built `Logger` is dropped).
    ///
    /// # Example
    ///
    /// ```
    /// use sloggers::Build;
    /// use sloggers::syslog::SyslogBuilder;
    /// use std::ffi::CStr;
    /// #
    /// # assert_eq!(
    /// #     CStr::from_bytes_with_nul("sloggers-example-app\0".as_bytes()).unwrap(),
    /// #     std::ffi::CString::new("sloggers-example-app").unwrap().as_c_str()
    /// # );
    ///
    /// let logger = SyslogBuilder::new()
    ///     .ident(CStr::from_bytes_with_nul("sloggers-example-app\0".as_bytes()).unwrap())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn ident(&mut self, ident: impl Into<Cow<'static, CStr>>) -> &mut Self {
        self.inner = Some(self.take_inner().ident(ident));
        self
    }

    // The `log_*` flag methods are all `#[inline]` because, in theory, the
    // optimizer could collapse several flag method calls into a single store
    // operation, which would be much faster…but it can only do that if the
    // calls are all inlined.

    /// Include the process ID in log messages.
    #[inline]
    pub fn log_pid(&mut self) -> &mut Self {
        self.inner = Some(self.take_inner().log_pid());
        self
    }

    /// Immediately open a connection to the syslog server, instead of waiting
    /// until the first log message is sent.
    ///
    /// `log_ndelay` and `log_odelay` are mutually exclusive, and one of them
    /// is the default. Exactly which one is the default depends on the
    /// platform, but on most platforms, `log_odelay` is the default.
    ///
    /// On OpenBSD 5.6 and newer, this setting has no effect, because that
    /// platform uses a dedicated system call instead of a socket for
    /// submitting syslog messages.
    #[inline]
    pub fn log_ndelay(&mut self) -> &mut Self {
        self.inner = Some(self.take_inner().log_ndelay());
        self
    }

    /// *Don't* immediately open a connection to the syslog server. Wait until
    /// the first log message is sent before connecting.
    ///
    /// `log_ndelay` and `log_odelay` are mutually exclusive, and one of them
    /// is the default. Exactly which one is the default depends on the
    /// platform, but on most platforms, `log_odelay` is the default.
    ///
    /// On OpenBSD 5.6 and newer, this setting has no effect, because that
    /// platform uses a dedicated system call instead of a socket for
    /// submitting syslog messages.
    #[inline]
    pub fn log_odelay(&mut self) -> &mut Self {
        self.inner = Some(self.take_inner().log_odelay());
        self
    }

    /// If a child process is created to send a log message, don't wait for
    /// that child process to exit.
    ///
    /// This option is highly unlikely to have any effect on any modern system.
    /// On a modern system, spawning a child process for every single log
    /// message would be extremely slow. This option only ever existed as a
    /// [workaround for limitations of the 2.11BSD kernel][2.11BSD wait call],
    /// and was already [deprecated as of 4.4BSD][4.4BSD deprecation notice].
    /// It is included here only for completeness because, unfortunately,
    /// [POSIX defines it].
    ///
    /// [2.11BSD wait call]: https://www.retro11.de/ouxr/211bsd/usr/src/lib/libc/gen/syslog.c.html#n:176
    /// [4.4BSD deprecation notice]: https://github.com/sergev/4.4BSD-Lite2/blob/50587b00e922225c62f1706266587f435898126d/usr/src/sys/sys/syslog.h#L164
    /// [POSIX defines it]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/closelog.html
    #[inline]
    pub fn log_nowait(&mut self) -> &mut Self {
        self.inner = Some(self.take_inner().log_nowait());
        self
    }

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
    #[inline]
    pub fn log_perror(&mut self) -> &mut Self {
        self.inner = Some(self.take_inner().log_perror());
        self
    }

    /// Set the [`Adapter`], which handles formatting and message priorities.
    ///
    /// See also the [`format`] and [`priority`] methods, which offer a
    /// convenient way to customize formatting or priority mapping.
    ///
    /// The default is [`DefaultAdapter`].
    ///
    /// This method wraps the format in an `Arc`. If your format is alrady
    /// wrapped in an `Arc`, call the `format_arc` method instead.
    ///
    /// # Example
    ///
    /// ```
    /// use sloggers::Build;
    /// use sloggers::syslog::adapter::BasicAdapter;
    /// use sloggers::syslog::SyslogBuilder;
    ///
    /// let logger = SyslogBuilder::new()
    ///     .adapter(BasicAdapter)
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// [`Adapter`]: adapter/trait.Adapter.html
    /// [`DefaultAdapter`]: adapter/struct.DefaultAdapter.html
    /// [`format`]: #method.format
    /// [`priority`]: #method.priority
    pub fn adapter(&mut self, adapter: impl Adapter + Send + Sync + 'static) -> &mut Self {
        self.adapter_arc(Arc::new(adapter))
    }

    /// Set the [`Adapter`], which handles formatting and message priorities.
    ///
    /// See also the [`format`] and [`priority`] methods, which offer a
    /// convenient way to customize formatting or priority mapping.
    ///
    /// The default is [`DefaultAdapter`].
    ///
    /// This method takes the [`Adapter`] wrapped in an `Arc`. Call this if
    /// your [`Adapter`] is already wrapped in an `Arc`. If not, call the
    /// [`adapter`][`Self::adapter`] method instead.
    ///
    /// # Example
    ///
    /// ```
    /// use sloggers::Build;
    /// use sloggers::syslog::adapter::BasicAdapter;
    /// use sloggers::syslog::SyslogBuilder;
    /// use std::sync::Arc;
    ///
    /// let adapter = Arc::new(BasicAdapter);
    ///
    /// let logger = SyslogBuilder::new()
    ///     .adapter_arc(adapter.clone())
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// [`Self::adapter`]: #method.adapter
    /// [`Adapter`]: adapter/trait.Adapter.html
    /// [`DefaultAdapter`]: adapter/struct.DefaultAdapter.html
    /// [`format`]: #method.format
    /// [`priority`]: #method.priority
    pub fn adapter_arc(&mut self, adapter: Arc<dyn Adapter + Send + Sync + 'static>) -> &mut Self {
        self.inner = Some(self.take_inner().adapter(adapter));
        self
    }

    /// Modify the [`Adapter`], which handles formatting and message
    /// priorities.
    ///
    /// This method supplies the current [`Adapter`] to the provided closure,
    /// then replaces the current [`Adapter`] with the one that the closure
    /// returns.
    ///
    /// The [`adapter`][`Self::adapter`], [`format`], and [`priority`] methods
    /// all call this method to replace the [`Adapter`].
    ///
    /// # Example
    ///
    /// ```
    /// use sloggers::Build;
    /// use sloggers::syslog::adapter::{Adapter, DefaultAdapter};
    /// use sloggers::syslog::SyslogBuilder;
    /// use std::sync::Arc;
    ///
    /// let logger = SyslogBuilder::new()
    ///     .map_adapter(|adapter| Arc::new(adapter.with_fmt(|f, record, _| {
    ///         write!(f, "here's a message: {}", record.msg())?;
    ///         Ok(())
    ///     })))
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// [`Self::adapter`]: #method.adapter
    /// [`Adapter`]: adapter/trait.Adapter.html
    /// [`format`]: #method.format
    /// [`priority`]: #method.priority
    pub fn map_adapter(
        &mut self,
        replacer: impl FnOnce(Arc<dyn Adapter + Send + Sync + 'static>) -> Arc<dyn Adapter + Send + Sync + 'static>,
    ) -> &mut Self {
        self.inner = Some(self.take_inner().map_adapter(replacer));
        self
    }

    /// Use custom message formatting.
    ///
    /// Syslog does not support structured log data. If Slog key-value pairs
    /// are to be included with log messages, they must be included as part of
    /// the message. The closure provided to this method decides if and how
    /// this will be done.
    ///
    /// # Example
    ///
    /// This formatting function simply prepends `here's a message: ` to each
    /// log message:
    ///
    /// ```
    /// use sloggers::Build;
    /// use sloggers::syslog::SyslogBuilder;
    ///
    /// let logger = SyslogBuilder::new()
    ///     .format(|f, record, _| {
    ///         write!(f, "here's a message: {}", record.msg())?;
    ///         Ok(())
    ///     })
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// Note the use of the `?` operator. The closure is expected to return
    /// `Result<(), slog::Error>`, not the `Result<(), std::fmt::Error>` that
    /// `write!` returns. `slog::Error` does have a conversion from
    /// `std::fmt::Error`, which the `?` operator will automatically perform.
    ///
    /// [`Adapter::with_fmt`]: adapter/trait.Adapter.html#method.with_fmt
    pub fn format(
        &mut self,
        fmt_fn: impl (Fn(&mut fmt::Formatter, &Record, &OwnedKVList) -> slog::Result) + Send + Sync + 'static,
    ) -> &mut Self {
        self.map_adapter(|adapter| Arc::new(adapter.with_fmt(fmt_fn)))
    }

    /// Customize mapping of [`slog::Level`]s to
    /// [syslog priorities][`Priority`].
    ///
    /// # Example
    ///
    /// ## Force all messages to [`Level::Err`]
    ///
    /// This uses the default message formatting, but makes all syslog messages
    /// be [`Level::Err`]:
    ///
    /// ```
    /// use sloggers::Build;
    /// use sloggers::syslog::{Level, SyslogBuilder};
    ///
    /// let logger = SyslogBuilder::new()
    ///     .priority(|record, _| {
    ///         Level::Err.into()
    ///     })
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// Notice the use of `into()`. [`Level`] can be converted directly into
    /// [`Priority`].
    ///
    /// ## Override level and facility
    ///
    /// In this example, [`slog::Level::Info`] messages from the `my_app::mail`
    /// module are logged as [`Level::Notice`] instead of the default
    /// [`Level::Info`], and all messages from that module are logged with a
    /// different facility:
    ///
    /// ```
    /// use sloggers::Build;
    /// use sloggers::syslog::{Facility, Level, Priority, SyslogBuilder};
    ///
    /// let drain = SyslogBuilder::new()
    ///     .facility(Facility::Daemon)
    ///     .priority(|record, _| {
    ///         Priority::new(
    ///             match record.level() {
    ///                 slog::Level::Info => Level::Notice,
    ///                 other => Level::from_slog(other),
    ///             },
    ///             match record.module() {
    ///                 "my_app::mail" => Some(Facility::Mail),
    ///                 _ => None,
    ///             },
    ///         )
    ///     })
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// [`Level`]: enum.Level.html
    /// [`Level::Err`]: enum.Level.html#variant.Err
    /// [`Level::Info`]: enum.Level.html#variant.Info
    /// [`Level::Notice`]: enum.Level.html#variant.Notice
    /// [`Priority`]: struct.Priority.html
    /// [`slog::Level`]: https://docs.rs/slog/2/slog/enum.Level.html
    /// [`slog::Level::Info`]: https://docs.rs/slog/2/slog/enum.Level.html#variant.Info
    /// [`SyslogBuilder::priority`]: struct.SyslogBuilder.html#method.priority
    pub fn priority(
        &mut self,
        priority_fn: impl (Fn(&Record, &OwnedKVList) -> Priority) + Send + Sync + 'static,
    ) -> &mut Self {
        self.map_adapter(|adapter| Arc::new(adapter.with_priority(priority_fn)))
    }

    /// Sets the log level of this logger.
    ///
    /// Not to be confused with the [`priority`] method. This method *filters*
    /// log messages; [`priority`] *changes* their [syslog priority].
    ///
    /// [`priority`]: #method.priority
    /// [syslog priority]: struct.Priority.html
    pub fn level(&mut self, severity: Severity) -> &mut Self {
        self.common.level = severity;
        self
    }

    /// Sets the size of the asynchronous channel of this logger.
    pub fn channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.common.channel_size = channel_size;
        self
    }

    /// Sets [`KVFilter`].
    ///
    /// [`KVFilter`]: https://docs.rs/slog-kvfilter/0.6/slog_kvfilter/struct.KVFilter.html
    #[cfg(feature = "slog-kvfilter")]
    pub fn kvfilter(&mut self, parameters: KVFilterParameters) -> &mut Self {
        self.common.kvfilterparameters = Some(parameters);
        self
    }
}

impl Build for SyslogBuilder {
    fn build(&self) -> Result<Logger> {
        let drain = self.borrow_inner().clone().build();
        let logger = self.common.build_with_drain(drain);
        Ok(logger)
    }
}
