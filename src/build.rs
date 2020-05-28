use crate::file::FileLoggerBuilder;
use crate::misc;
use crate::null::NullLoggerBuilder;
use crate::terminal::TerminalLoggerBuilder;
#[cfg(feature = "slog-kvfilter")]
use crate::types::KVFilterParameters;
use crate::types::{OverflowStrategy, Severity, SourceLocation};
use crate::Result;
use slog::{Drain, FnValue, Logger};
use slog_async::Async;
#[cfg(feature = "slog-kvfilter")]
use slog_kvfilter::KVFilter;
use std::fmt::Debug;
use std::panic::{RefUnwindSafe, UnwindSafe};

/// This trait allows to build a logger instance.
pub trait Build {
    /// Builds a logger.
    fn build(&self) -> Result<Logger>;
}

/// Logger builder.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum LoggerBuilder {
    /// File logger.
    File(FileLoggerBuilder),

    /// Null logger.
    Null(NullLoggerBuilder),

    /// Terminal logger.
    Terminal(TerminalLoggerBuilder),
}
impl Build for LoggerBuilder {
    fn build(&self) -> Result<Logger> {
        match *self {
            LoggerBuilder::File(ref b) => track!(b.build()),
            LoggerBuilder::Null(ref b) => track!(b.build()),
            LoggerBuilder::Terminal(ref b) => track!(b.build()),
        }
    }
}

/// Common code for wrapping up a bare `Drain` into a finished `Logger`.
/// 
/// This is just a data structure and a shared `build_with_drain` routine. Individual logger builders need to expose methods that fill in the fields of this `struct`, and their `Build` implementation needs to call `BuilderCommon::build_with_drain` after building the basic drain.
#[derive(Debug)]
pub(crate) struct BuilderCommon {
    pub source_location: SourceLocation,
    pub overflow_strategy: OverflowStrategy,
    pub level: Severity,
    pub channel_size: usize,
    #[cfg(feature = "slog-kvfilter")]
    pub kvfilterparameters: Option<KVFilterParameters>,
}
impl Default for BuilderCommon {
    fn default() -> Self {
        BuilderCommon {
            source_location: SourceLocation::default(),
            overflow_strategy: OverflowStrategy::default(),
            level: Severity::default(),
            channel_size: 1024,
            #[cfg(feature = "slog-kvfilter")]
            kvfilterparameters: None,
        }
    }
}
impl BuilderCommon {
    pub fn build_with_drain<D>(&self, drain: D) -> Logger
    where
        D: Drain + Send + 'static,
        D::Err: Debug,
    {
        // async inside, level and key value filters outside for speed
        let drain = Async::new(drain.fuse())
            .chan_size(self.channel_size)
            .overflow_strategy(self.overflow_strategy.to_async_type())
            .build()
            .fuse();

        #[cfg(feature = "slog-kvfilter")]
        {
            if let Some(ref p) = self.kvfilterparameters {
                let kvdrain = KVFilter::new(drain, p.severity.as_level())
                    .always_suppress_any(p.always_suppress_any.clone())
                    .only_pass_any_on_all_keys(p.only_pass_any_on_all_keys.clone())
                    .always_suppress_on_regex(p.always_suppress_on_regex.clone())
                    .only_pass_on_regex(p.only_pass_on_regex.clone());
                self.build_logger(kvdrain)
            } else {
                self.build_logger(drain)
            }
        }

        #[cfg(not(feature = "slog-kvfilter"))]
        self.build_logger(drain)
    }

    fn build_logger<D>(&self, drain: D) -> Logger
    where
        D: Drain + Send + Sync + UnwindSafe + RefUnwindSafe + 'static,
        D::Err: Debug,
    {
        let drain = self.level.set_level_filter(drain.fuse());

        match self.source_location {
            SourceLocation::None => Logger::root(drain.fuse(), o!()),
            SourceLocation::ModuleAndLine => {
                Logger::root(drain.fuse(), o!("module" => FnValue(misc::module_and_line)))
            }
            SourceLocation::FileAndLine => {
                Logger::root(drain.fuse(), o!("module" => FnValue(misc::file_and_line)))
            }
            SourceLocation::LocalFileAndLine => Logger::root(
                drain.fuse(),
                o!("module" => FnValue(misc::local_file_and_line)),
            ),
        }
    }
}
