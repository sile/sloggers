use slog::Logger;
use slog_scope;
use slog_stdlog;

use Result;

pub fn set_stdlog_logger(logger: Logger) -> Result<()> {
    let _guard = slog_scope::set_global_logger(logger);
    track_err!(slog_stdlog::init())
}
