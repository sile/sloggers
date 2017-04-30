use slog::{Logger, OwnedKV, SendSyncRefUnwindSafeKV, Drain};
use slog_term::{TermDecorator, CompactFormat, FullFormat};
use slog_async::Async;

use config::{TerminalLoggerConfig, TerminalFormat, Timezone, TerminalOutput};

pub struct TerminalLoggerBuilder {
    is_full: bool,
    use_utc_time: bool,
    use_stdout: bool,
}
impl TerminalLoggerBuilder {
    pub fn new() -> TerminalLoggerBuilder {
        TerminalLoggerBuilder {
            is_full: true,
            use_utc_time: false,
            use_stdout: true,
        }
    }
    pub fn from_config(config: &TerminalLoggerConfig) -> Self {
        let mut builder = Self::new();
        match config.format {
            TerminalFormat::Full => builder.full(),
            TerminalFormat::Compact => builder.compact(),
        };
        match config.timezone {
            Timezone::Utc => builder.utc_time(),
            Timezone::Local => builder.local_time(),
        };
        match config.output {
            TerminalOutput::Stdout => builder.stdout(),
            TerminalOutput::Stderr => builder.stderr(),
        };
        builder
    }

    pub fn full(&mut self) -> &mut Self {
        self.is_full = true;
        self
    }
    pub fn compact(&mut self) -> &mut Self {
        self.is_full = false;
        self
    }
    pub fn utc_time(&mut self) -> &mut Self {
        self.use_utc_time = true;
        self
    }
    pub fn local_time(&mut self) -> &mut Self {
        self.use_utc_time = false;
        self
    }
    pub fn stdout(&mut self) -> &mut Self {
        self.use_stdout = true;
        self
    }
    pub fn stderr(&mut self) -> &mut Self {
        self.use_stdout = false;
        self
    }

    pub fn finish<T>(&self, values: OwnedKV<T>) -> Logger
        where T: SendSyncRefUnwindSafeKV + 'static
    {
        let decorator = TermDecorator::new();
        let decorator = if self.use_stdout {
            decorator.stdout().build()
        } else {
            decorator.stderr().build()
        };
        if self.is_full {
            let format = if self.use_utc_time {
                FullFormat::new(decorator).use_utc_timestamp()
            } else {
                FullFormat::new(decorator).use_local_timestamp()
            };
            let drain = Async::default(format.build().fuse()).fuse();
            Logger::root(drain, values)
        } else {
            let format = if self.use_utc_time {
                CompactFormat::new(decorator).use_utc_timestamp()
            } else {
                CompactFormat::new(decorator).use_local_timestamp()
            };
            let drain = Async::default(format.build().fuse()).fuse();
            Logger::root(drain, values)
        }
    }
}
