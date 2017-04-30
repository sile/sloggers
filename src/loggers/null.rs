use slog::{Logger, Discard, OwnedKV, SendSyncRefUnwindSafeKV};

#[derive(Debug)]
pub struct NullLoggerBuilder;
impl NullLoggerBuilder {
    pub fn new() -> NullLoggerBuilder {
        NullLoggerBuilder
    }
    pub fn finish<T>(&self, values: OwnedKV<T>) -> Logger
        where T: SendSyncRefUnwindSafeKV + 'static
    {
        Logger::root(Discard, values)
    }
}
