//! File logger.
use crate::build::BuilderCommon;
use crate::permissions::restrict_file_permissions;
#[cfg(feature = "slog-kvfilter")]
use crate::types::KVFilterParameters;
use crate::types::{Format, OverflowStrategy, Severity, SourceLocation, TimeZone};
use crate::{misc, BuildWithCustomFormat};
use crate::{Build, Config, ErrorKind, Result};
use chrono::{DateTime, Local, TimeZone as ChronoTimeZone, Utc};
#[cfg(feature = "libflate")]
use libflate::gzip::Encoder as GzipEncoder;
use serde::{Deserialize, Serialize};
use slog::{Drain, Logger};
use slog_term::{CompactFormat, FullFormat, PlainDecorator};
use std::fmt::Debug;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
#[cfg(feature = "libflate")]
use std::sync::mpsc;
#[cfg(feature = "libflate")]
use std::thread;
use std::time::{Duration, Instant};

/// A logger builder which build loggers that write log records to the specified file.
///
/// The resulting logger will work asynchronously (the default channel size is 1024).
#[derive(Debug)]
pub struct FileLoggerBuilder {
    common: BuilderCommon,
    format: Format,
    timezone: TimeZone,
    appender: FileAppender,
}

impl FileLoggerBuilder {
    /// Makes a new `FileLoggerBuilder` instance.
    ///
    /// This builder will create a logger which uses `path` as
    /// the output destination of the log records.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileLoggerBuilder {
            common: BuilderCommon::default(),
            format: Format::default(),
            timezone: TimeZone::default(),
            appender: FileAppender::new(path),
        }
    }

    /// Sets the format of log records.
    pub fn format(&mut self, format: Format) -> &mut Self {
        self.format = format;
        self
    }

    /// Sets the source code location type this logger will use.
    pub fn source_location(&mut self, source_location: SourceLocation) -> &mut Self {
        self.common.source_location = source_location;
        self
    }

    /// Sets the overflow strategy for the logger.
    pub fn overflow_strategy(&mut self, overflow_strategy: OverflowStrategy) -> &mut Self {
        self.common.overflow_strategy = overflow_strategy;
        self
    }

    /// Sets the time zone which this logger will use.
    pub fn timezone(&mut self, timezone: TimeZone) -> &mut Self {
        self.timezone = timezone;
        self
    }

    /// Sets the log level of this logger.
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

    /// By default, logger just appends log messages to file.
    /// If this method called, logger truncates the file to 0 length when opening.
    pub fn truncate(&mut self) -> &mut Self {
        self.appender.truncate = true;
        self
    }

    /// Sets the threshold used for determining whether rotate the current log file.
    ///
    /// If the byte size of the current log file exceeds this value, the file will be rotated.
    /// The name of the rotated file will be `"${ORIGINAL_FILE_NAME}.0"`.
    /// If there is a previously rotated file,
    /// it will be renamed to `"${ORIGINAL_FILE_NAME}.1"` before rotation of the current log file.
    /// This process is iterated recursively until log file names no longer conflict or
    /// [`rotate_keep`] limit reached.
    ///
    /// The default value is `std::u64::MAX`.
    ///
    /// [`rotate_keep`]: ./struct.FileLoggerBuilder.html#method.rotate_keep
    pub fn rotate_size(&mut self, size: u64) -> &mut Self {
        self.appender.rotate_size = size;
        self
    }

    /// Sets the maximum number of rotated log files to keep.
    ///
    /// If the number of rotated log files exceed this value, the oldest log file will be deleted.
    ///
    /// The default value is `8`.
    pub fn rotate_keep(&mut self, count: usize) -> &mut Self {
        self.appender.rotate_keep = count;
        self
    }

    /// Sets whether to compress or not compress rotated files.
    ///
    /// If `true` is specified, rotated files will be compressed by GZIP algorithm and
    /// the suffix ".gz" will be appended to those file names.
    ///
    /// The default value is `false`.
    #[cfg(feature = "libflate")]
    pub fn rotate_compress(&mut self, compress: bool) -> &mut Self {
        self.appender.rotate_compress = compress;
        self
    }

    /// Sets whether the log files should have restricted permissions.
    ///
    /// If `true` is specified, new log files will be created with the `600` octal permission
    /// on unix systems.
    /// On Windows systems, new log files will have an ACL which just contains the SID of
    /// the owner.
    ///
    /// The default value is `false`.
    pub fn restrict_permissions(&mut self, restrict: bool) -> &mut Self {
        self.appender.restrict_permissions = restrict;
        self
    }
}

impl Build for FileLoggerBuilder {
    fn build(&self) -> Result<Logger> {
        let timestamp = misc::timezone_to_timestamp_fn(self.timezone);
        let logger = match self.format {
            Format::Full => {
                let decorator = PlainDecorator::new(self.appender.clone());
                let format = FullFormat::new(decorator).use_custom_timestamp(timestamp);
                self.common.build_with_drain(format.build())
            }
            Format::Compact => {
                let decorator = PlainDecorator::new(self.appender.clone());
                let format = CompactFormat::new(decorator).use_custom_timestamp(timestamp);
                self.common.build_with_drain(format.build())
            }
            #[cfg(feature = "json")]
            Format::Json => {
                let drain = slog_json::Json::new(self.appender.clone())
                    .set_flush(true)
                    .add_default_keys()
                    .build();
                self.common.build_with_drain(drain)
            }
        };
        Ok(logger)
    }
}
impl BuildWithCustomFormat for FileLoggerBuilder {
    type Decorator = PlainDecorator<FileAppender>;

    fn build_with_custom_format<F, D>(&self, f: F) -> Result<Logger>
    where
        F: FnOnce(Self::Decorator) -> Result<D>,
        D: Drain + Send + 'static,
        D::Err: Debug,
    {
        let decorator = PlainDecorator::new(self.appender.clone());
        let drain = track!(f(decorator))?;
        Ok(self.common.build_with_drain(drain))
    }
}

#[derive(Debug)]
pub struct FileAppender {
    path: PathBuf,
    file: Option<BufWriter<File>>,
    truncate: bool,
    written_size: u64,
    rotate_size: u64,
    rotate_keep: usize,
    #[cfg(feature = "libflate")]
    rotate_compress: bool,
    #[cfg(feature = "libflate")]
    wait_compression: Option<mpsc::Receiver<io::Result<()>>>,
    next_reopen_check: Instant,
    reopen_check_interval: Duration,
    restrict_permissions: bool,
}

impl Clone for FileAppender {
    fn clone(&self) -> Self {
        FileAppender {
            path: self.path.clone(),
            file: None,
            truncate: self.truncate,
            written_size: 0,
            rotate_size: self.rotate_size,
            rotate_keep: self.rotate_keep,
            #[cfg(feature = "libflate")]
            rotate_compress: self.rotate_compress,
            #[cfg(feature = "libflate")]
            wait_compression: None,
            next_reopen_check: Instant::now(),
            reopen_check_interval: self.reopen_check_interval,
            restrict_permissions: self.restrict_permissions,
        }
    }
}

impl FileAppender {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileAppender {
            path: path.as_ref().to_path_buf(),
            file: None,
            truncate: false,
            written_size: 0,
            rotate_size: default_rotate_size(),
            rotate_keep: default_rotate_keep(),
            #[cfg(feature = "libflate")]
            rotate_compress: false,
            #[cfg(feature = "libflate")]
            wait_compression: None,
            next_reopen_check: Instant::now(),
            reopen_check_interval: Duration::from_millis(1000),
            restrict_permissions: false,
        }
    }

    fn reopen_if_needed(&mut self) -> io::Result<()> {
        // See issue #18
        // Basically, path.exists() is VERY slow on windows, so we just
        // can't check on every write. Limit checking to a predefined interval.
        // This shouldn't create problems neither for users, nor for logrotate et al.,
        // as explained in the issue.
        let now = Instant::now();
        let path_exists = if now >= self.next_reopen_check {
            self.next_reopen_check = now + self.reopen_check_interval;
            self.path.exists()
        } else {
            // Pretend the path exists without any actual checking.
            true
        };

        if self.file.is_none() || !path_exists {
            let mut file_builder = OpenOptions::new();
            file_builder.create(true);
            if self.truncate {
                file_builder.truncate(true);
            }
            // If the old file was externally deleted and we attempt to open a new one before releasing the old
            // handle, we get a Permission denied on Windows. Release the handle.
            self.file = None;

            let mut file = file_builder
                .append(!self.truncate)
                .write(true)
                .open(&self.path)?;

            if self.restrict_permissions {
                file = restrict_file_permissions(&self.path, file)?;
            }
            self.written_size = file.metadata()?.len();
            self.file = Some(BufWriter::new(file));
        }
        Ok(())
    }

    fn rotate(&mut self) -> io::Result<()> {
        #[cfg(feature = "libflate")]
        {
            if let Some(ref mut rx) = self.wait_compression {
                use std::sync::mpsc::TryRecvError;
                match rx.try_recv() {
                    Err(TryRecvError::Empty) => {
                        // The previous compression is in progress
                        return Ok(());
                    }
                    Err(TryRecvError::Disconnected) => {
                        let e = io::Error::new(
                            io::ErrorKind::Other,
                            "Log file compression thread aborted",
                        );
                        return Err(e);
                    }
                    Ok(result) => {
                        result?;
                    }
                }
            }
            self.wait_compression = None;
        }

        let _ = self.file.take();

        #[cfg(windows)]
        {
            if let Err(err) = self.rotate_old_files() {
                const ERROR_SHARING_VIOLATION: i32 = 32;

                // To avoid the problem reported by https://github.com/sile/sloggers/issues/43,
                // we ignore the error if its code is 32.
                if err.raw_os_error() != Some(ERROR_SHARING_VIOLATION) {
                    return Err(err);
                }
            }
        }
        #[cfg(not(windows))]
        self.rotate_old_files()?;

        self.written_size = 0;
        self.next_reopen_check = Instant::now();
        self.reopen_if_needed()?;

        Ok(())
    }
    fn rotate_old_files(&mut self) -> io::Result<()> {
        for i in (1..=self.rotate_keep).rev() {
            let from = self.rotated_path(i)?;
            let to = self.rotated_path(i + 1)?;
            if from.exists() {
                fs::rename(from, to)?;
            }
        }
        if self.path.exists() {
            let rotated_path = self.rotated_path(1)?;
            #[cfg(feature = "libflate")]
            {
                if self.rotate_compress {
                    let (plain_path, temp_gz_path) = self.rotated_paths_for_compression()?;
                    let (tx, rx) = mpsc::channel();
                    let restrict_perms = self.restrict_permissions;

                    fs::rename(&self.path, &plain_path)?;
                    thread::spawn(move || {
                        let result =
                            Self::compress(plain_path, temp_gz_path, rotated_path, restrict_perms);
                        let _ = tx.send(result);
                    });

                    self.wait_compression = Some(rx);
                } else {
                    fs::rename(&self.path, rotated_path)?;
                }
            }
            #[cfg(not(feature = "libflate"))]
            fs::rename(&self.path, rotated_path)?;
        }

        let delete_path = self.rotated_path(self.rotate_keep + 1)?;
        if delete_path.exists() {
            fs::remove_file(delete_path)?;
        }

        Ok(())
    }
    fn rotated_path(&self, i: usize) -> io::Result<PathBuf> {
        let path = self.path.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Non UTF-8 log file path: {:?}", self.path),
            )
        })?;
        #[cfg(feature = "libflate")]
        {
            if self.rotate_compress {
                Ok(PathBuf::from(format!("{}.{}.gz", path, i)))
            } else {
                Ok(PathBuf::from(format!("{}.{}", path, i)))
            }
        }
        #[cfg(not(feature = "libflate"))]
        Ok(PathBuf::from(format!("{}.{}", path, i)))
    }
    #[cfg(feature = "libflate")]
    fn rotated_paths_for_compression(&self) -> io::Result<(PathBuf, PathBuf)> {
        let path = self.path.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Non UTF-8 log file path: {:?}", self.path),
            )
        })?;
        Ok((
            PathBuf::from(format!("{}.1", path)),
            PathBuf::from(format!("{}.1.gz.temp", path)),
        ))
    }
    #[cfg(feature = "libflate")]
    fn compress(
        input_path: PathBuf,
        temp_path: PathBuf,
        output_path: PathBuf,
        restrict_perms: bool,
    ) -> io::Result<()> {
        let mut input = File::open(&input_path)?;
        let mut temp = File::create(&temp_path)?;
        if restrict_perms {
            temp = restrict_file_permissions(&temp_path, temp)?;
        }
        let mut output = GzipEncoder::new(temp)?;
        io::copy(&mut input, &mut output)?;
        output.finish().into_result()?;

        fs::rename(temp_path, output_path)?;
        fs::remove_file(input_path)?;
        Ok(())
    }
}

impl Write for FileAppender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.reopen_if_needed()?;
        let size = if let Some(ref mut f) = self.file {
            f.write(buf)?
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Cannot open file: {:?}", self.path),
            ));
        };

        self.written_size += size as u64;
        Ok(size)
    }
    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut f) = self.file {
            f.flush()?;
        }
        if self.written_size >= self.rotate_size {
            self.rotate()?;
        }
        Ok(())
    }
}

/// The configuration of `FileLoggerBuilder`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FileLoggerConfig {
    /// Log level.
    #[serde(default)]
    pub level: Severity,

    /// Log record format.
    #[serde(default)]
    pub format: Format,

    /// Source code location
    #[serde(default)]
    pub source_location: SourceLocation,

    /// Time Zone.
    #[serde(default)]
    pub timezone: TimeZone,

    /// Format string for the timestamp in the path.
    /// The string is formatted using [strftime](https://docs.rs/chrono/0.4.6/chrono/format/strftime/index.html#specifiers)
    ///
    /// Default: "%Y%m%d_%H%M", example: "20180918_1127"
    #[serde(default = "default_timestamp_template")]
    pub timestamp_template: String,

    /// Log file path template.
    ///
    /// It will be used as-is, with the following transformation:
    ///
    /// All occurrences of the substring "{timestamp}" will be replaced with the current timestamp
    /// formatted according to `timestamp_template`. The timestamp will respect the `timezone` setting.
    pub path: PathBuf,

    /// Asynchronous channel size
    #[serde(default = "default_channel_size")]
    pub channel_size: usize,

    /// Truncate the file or not
    #[serde(default)]
    pub truncate: bool,

    /// Log file rotation size.
    ///
    /// For details, see the documentation of [`rotate_size`].
    ///
    /// [`rotate_size`]: ./struct.FileLoggerBuilder.html#method.rotate_size
    #[serde(default = "default_rotate_size")]
    pub rotate_size: u64,

    /// Maximum number of rotated log files to keep.
    ///
    /// For details, see the documentation of [`rotate_keep`].
    ///
    /// [`rotate_keep`]: ./struct.FileLoggerBuilder.html#method.rotate_keep
    #[serde(default = "default_rotate_keep")]
    pub rotate_keep: usize,

    /// Whether to compress or not compress rotated files.
    ///
    /// For details, see the documentation of [`rotate_compress`].
    ///
    /// [`rotate_compress`]: ./struct.FileLoggerBuilder.html#method.rotate_compress
    ///
    /// The default value is `false`.
    #[serde(default)]
    #[cfg(feature = "libflate")]
    pub rotate_compress: bool,

    /// Whether to drop logs on overflow.
    ///
    /// The possible values are `drop`, `drop_and_report`, or `block`.
    ///
    /// The default value is `drop_and_report`.
    #[serde(default)]
    pub overflow_strategy: OverflowStrategy,

    /// Whether to restrict the permissions of log files.
    ///
    /// For details, see the documentation of [`restict_permissions`].
    ///
    /// [`restrict_permissions`]: ./struct.FileLoggerBuilder.html#method.restrict_permissions
    #[serde(default)]
    pub restrict_permissions: bool,
}

impl FileLoggerConfig {
    /// Creates a new `FileLoggerConfig` with default settings.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Config for FileLoggerConfig {
    type Builder = FileLoggerBuilder;
    fn try_to_builder(&self) -> Result<Self::Builder> {
        let now = Utc::now();
        let path_template = self.path.to_str().ok_or(ErrorKind::Invalid)?;
        let path =
            path_template_to_path(path_template, &self.timestamp_template, self.timezone, now);
        let mut builder = FileLoggerBuilder::new(path);
        builder.level(self.level);
        builder.format(self.format);
        builder.source_location(self.source_location);
        builder.timezone(self.timezone);
        builder.overflow_strategy(self.overflow_strategy);
        builder.channel_size(self.channel_size);
        builder.rotate_size(self.rotate_size);
        builder.rotate_keep(self.rotate_keep);
        #[cfg(feature = "libflate")]
        builder.rotate_compress(self.rotate_compress);
        builder.restrict_permissions(self.restrict_permissions);
        if self.truncate {
            builder.truncate();
        }
        Ok(builder)
    }
}

impl Default for FileLoggerConfig {
    fn default() -> Self {
        FileLoggerConfig {
            level: Severity::default(),
            format: Format::default(),
            source_location: SourceLocation::default(),
            overflow_strategy: OverflowStrategy::default(),
            timezone: TimeZone::default(),
            path: PathBuf::default(),
            timestamp_template: default_timestamp_template(),
            channel_size: default_channel_size(),
            truncate: false,
            rotate_size: default_rotate_size(),
            rotate_keep: default_rotate_keep(),
            #[cfg(feature = "libflate")]
            rotate_compress: false,
            restrict_permissions: false,
        }
    }
}

fn path_template_to_path(
    path_template: &str,
    timestamp_template: &str,
    timezone: TimeZone,
    date_time: DateTime<Utc>,
) -> PathBuf {
    let timestamp_string = match timezone {
        TimeZone::Local => {
            let local_timestamp = Local.from_utc_datetime(&date_time.naive_utc());
            local_timestamp.format(timestamp_template)
        }
        TimeZone::Utc => date_time.format(timestamp_template),
    }
    .to_string();
    let path_string = path_template.replace("{timestamp}", &timestamp_string);
    PathBuf::from(path_string)
}

fn default_channel_size() -> usize {
    1024
}

fn default_rotate_size() -> u64 {
    use std::u64;

    u64::MAX
}

fn default_rotate_keep() -> usize {
    8
}

fn default_timestamp_template() -> String {
    "%Y%m%d_%H%M".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Build, ErrorKind};
    use chrono::NaiveDateTime;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile::{Builder as TempDirBuilder, TempDir};

    #[test]
    fn test_reopen_if_needed() {
        let dir = tempdir();
        let log_path = &dir.path().join("foo.log");
        let logger = FileLoggerBuilder::new(log_path).build().unwrap();

        info!(logger, "Goodbye");
        thread::sleep(Duration::from_millis(50));
        assert!(log_path.exists());
        fs::remove_file(log_path).unwrap();
        assert!(!log_path.exists());

        thread::sleep(Duration::from_millis(100));
        info!(logger, "cruel");
        assert!(!log_path.exists()); // next_reopen_check didn't get there yet, "cruel" went into the old file descriptor

        // Now > next_reopen_check, "world" will reopen the file before logging
        thread::sleep(Duration::from_millis(1000));
        info!(logger, "world");
        thread::sleep(Duration::from_millis(50));
        assert!(log_path.exists());
        assert!(fs::read_to_string(log_path).unwrap().contains("INFO world"));
    }

    #[test]
    fn file_rotation_works() {
        let dir = tempdir();
        let logger = FileLoggerBuilder::new(dir.path().join("foo.log"))
            .rotate_size(128)
            .rotate_keep(2)
            .build()
            .unwrap();

        info!(logger, "hello");
        thread::sleep(Duration::from_millis(50));
        assert!(dir.path().join("foo.log").exists());
        assert!(!dir.path().join("foo.log.1").exists());

        info!(logger, "world");
        thread::sleep(Duration::from_millis(50));
        assert!(dir.path().join("foo.log").exists());
        assert!(dir.path().join("foo.log.1").exists());
        assert!(!dir.path().join("foo.log.2").exists());

        info!(logger, "vec(0): {:?}", vec![0; 128]);
        thread::sleep(Duration::from_millis(50));
        assert!(dir.path().join("foo.log").exists());
        assert!(dir.path().join("foo.log.1").exists());
        assert!(dir.path().join("foo.log.2").exists());
        assert!(!dir.path().join("foo.log.3").exists());

        info!(logger, "vec(1): {:?}", vec![0; 128]);
        thread::sleep(Duration::from_millis(50));
        assert!(dir.path().join("foo.log").exists());
        assert!(dir.path().join("foo.log.1").exists());
        assert!(dir.path().join("foo.log.2").exists());
        assert!(!dir.path().join("foo.log.3").exists());
    }

    #[test]
    fn file_gzip_rotation_works() {
        let dir = tempdir();
        let logger = FileLoggerBuilder::new(dir.path().join("foo.log"))
            .rotate_size(128)
            .rotate_keep(2)
            .rotate_compress(true)
            .build()
            .unwrap();

        info!(logger, "hello");
        thread::sleep(Duration::from_millis(50));
        assert!(dir.path().join("foo.log").exists());
        assert!(!dir.path().join("foo.log.1").exists());

        info!(logger, "world");
        thread::sleep(Duration::from_millis(50));
        assert!(dir.path().join("foo.log").exists());
        assert!(dir.path().join("foo.log.1.gz").exists());
        assert!(!dir.path().join("foo.log.2.gz").exists());

        info!(logger, "vec(0): {:?}", vec![0; 128]);
        thread::sleep(Duration::from_millis(50));
        assert!(dir.path().join("foo.log").exists());
        assert!(dir.path().join("foo.log.1.gz").exists());
        assert!(dir.path().join("foo.log.2.gz").exists());
        assert!(!dir.path().join("foo.log.3.gz").exists());

        info!(logger, "vec(1): {:?}", vec![0; 128]);
        thread::sleep(Duration::from_millis(50));
        assert!(dir.path().join("foo.log").exists());
        assert!(dir.path().join("foo.log.1.gz").exists());
        assert!(dir.path().join("foo.log.2.gz").exists());
        assert!(!dir.path().join("foo.log.3.gz").exists());
    }

    #[test]
    fn test_path_template_to_path() {
        let dir = tempdir();
        let path_template = dir
            .path()
            .join("foo_{timestamp}.log")
            .to_str()
            .ok_or(ErrorKind::Invalid)
            .unwrap()
            .to_string();
        let actual = path_template_to_path(
            &path_template,
            "%Y%m%d_%H%M",
            TimeZone::Utc, // Local is difficult to test, omitting :(
            Utc.from_utc_datetime(&NaiveDateTime::from_timestamp_opt(1537265991, 0).unwrap()),
        );
        let expected = dir.path().join("foo_20180918_1019.log");
        assert_eq!(expected, actual);
    }

    fn tempdir() -> TempDir {
        TempDirBuilder::new()
            .prefix("sloggers_test")
            .tempdir()
            .expect("Cannot create a temporary directory")
    }
}
