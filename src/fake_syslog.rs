#![cfg(not(unix))]

use serde::{de::Error, Deserialize, Deserializer, Serialize};

/// Fake syslog configuration type, for platforms where syslog is not
/// supported. Cannot be constructed.
#[derive(Clone, Debug, Serialize)]
pub enum SyslogNotSupported {}

impl<'de> Deserialize<'de> for SyslogNotSupported {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(D::Error::custom("syslog is not supported on this platform"))
    }
}
