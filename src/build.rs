use slog::Logger;

use Result;

pub trait Build {
    fn build(&self) -> Result<Logger>;
}
