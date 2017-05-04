use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use slog::{Logger, Drain, FnValue};
use slog_async::Async;
use slog_term::{PlainDecorator, CompactFormat, FullFormat};

use {Result, Build, Config};
use misc::module_and_line;
use types::{Severity, Format, Timezone};

#[derive(Debug)]
pub struct FileLoggerBuilder {
    format: Format,
    timezone: Timezone,
    level: Severity,
    appender: FileAppender,
}
impl FileLoggerBuilder {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileLoggerBuilder {
            format: Format::default(),
            timezone: Timezone::default(),
            level: Severity::default(),
            appender: FileAppender::new(path),
        }
    }
    pub fn format(&mut self, format: Format) -> &mut Self {
        self.format = format;
        self
    }
    pub fn full(&mut self) -> &mut Self {
        self.format(Format::Full)
    }
    pub fn compact(&mut self) -> &mut Self {
        self.format(Format::Compact)
    }
    pub fn timezone(&mut self, timezone: Timezone) -> &mut Self {
        self.timezone = timezone;
        self
    }
    pub fn utc_time(&mut self) -> &mut Self {
        self.timezone(Timezone::Utc)
    }
    pub fn local_time(&mut self) -> &mut Self {
        self.timezone(Timezone::Local)
    }
    pub fn level(&mut self, severity: Severity) -> &mut Self {
        self.level = severity;
        self
    }
    fn build_with_drain<D>(&self, drain: D) -> Logger
        where D: Drain + Send + 'static,
              D::Err: Debug
    {
        let drain = Async::default(drain.fuse()).fuse();
        let drain = self.level.set_level_filter(drain).fuse();
        Logger::root(drain, o!("module" => FnValue(module_and_line)))
    }
}
impl Build for FileLoggerBuilder {
    fn build(&self) -> Result<Logger> {
        let decorator = PlainDecorator::new(self.appender.clone());
        let timestamp = self.timezone.to_timestamp_fn();
        let logger = match self.format {
            Format::Full => {
                let format = FullFormat::new(decorator).use_custom_timestamp(timestamp);
                self.build_with_drain(format.build())
            }
            Format::Compact => {
                let format = CompactFormat::new(decorator).use_custom_timestamp(timestamp);
                self.build_with_drain(format.build())
            }
        };
        Ok(logger)
    }
}

#[derive(Debug)]
struct FileAppender {
    path: PathBuf,
    file: Option<File>,
}
impl Clone for FileAppender {
    fn clone(&self) -> Self {
        FileAppender {
            path: self.path.clone(),
            file: None,
        }
    }
}
impl FileAppender {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileAppender {
            path: path.as_ref().to_path_buf(),
            file: None,
        }
    }
    fn reopen_if_needed(&mut self) -> io::Result<()> {
        if !self.path.exists() || self.file.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .write(true)
                .open(&self.path)?;
            self.file = Some(file);
        }
        Ok(())
    }
}
impl Write for FileAppender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.reopen_if_needed()?;
        if let Some(ref mut f) = self.file {
            f.write(buf)
        } else {
            Err(io::Error::new(io::ErrorKind::Other,
                               format!("Cannot open file: {:?}", self.path)))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut f) = self.file {
            f.flush()?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FileLoggerConfig {
    // TODO: default
    pub level: Severity,
    pub format: Format,
    pub timezone: Timezone,
    pub path: PathBuf,
}
impl Config for FileLoggerConfig {
    type Builder = FileLoggerBuilder;
    fn try_into_builder(self) -> Result<Self::Builder> {
        let mut builder = FileLoggerBuilder::new(self.path);
        builder.level(self.level);
        builder.format(self.format);
        builder.timezone(self.timezone);
        Ok(builder)
    }
}
