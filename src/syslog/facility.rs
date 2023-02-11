use crate::error::{Error, ErrorKind};
use libc::c_int;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::result::Result as StdResult;
use std::str::FromStr;

/// A syslog facility. Conversions are provided to and from `c_int`.
///
/// Available facilities depend on the target platform. All variants of this
/// `enum` are available on all platforms, and variants not present on the
/// target platform will be mapped to a reasonable alternative.
#[allow(missing_docs)]
#[derive(Default, Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
pub enum Facility {
    Auth,

    /// Log messages containing sensitive information.
    ///
    /// Available on: Linux, Emscripten, macOS, iOS, FreeBSD, DragonFly BSD,
    /// OpenBSD, NetBSD
    ///
    /// On other platforms: becomes `Auth`
    AuthPriv,

    /// Periodic task scheduling daemons like `cron`.
    ///
    /// Available on: Linux, Emscripten, macOS, iOS, FreeBSD, DragonFly BSD,
    /// OpenBSD, NetBSD, Solaris, illumos
    ///
    /// On other platforms: becomes `Daemon`
    Cron,

    Daemon,

    /// FTP server.
    ///
    /// Available on: Linux, Emscripten, macOS, iOS, FreeBSD, DragonFly BSD,
    /// OpenBSD, NetBSD
    ///
    /// On other platforms: becomes `Daemon`
    Ftp,

    Kern,

    /// macOS installer.
    ///
    /// Available on: macOS, iOS
    ///
    /// On other platforms: becomes `User`
    Install,

    /// `launchd`, the macOS process supervisor.
    ///
    /// Available on: macOS, iOS
    ///
    /// On other platforms: becomes `Daemon`
    Launchd,

    Local0,
    Local1,
    Local2,
    Local3,
    Local4,
    Local5,
    Local6,
    Local7,
    Lpr,
    Mail,

    /// Network Time Protocol daemon.
    ///
    /// Available on: FreeBSD, DragonFly BSD
    ///
    /// On other platforms: becomes `Daemon`
    Ntp,

    /// NeXT/early macOS `NetInfo` system.
    ///
    /// Available on: macOS, iOS
    ///
    /// On other platforms: becomes `Daemon`
    NetInfo,

    News,

    /// macOS Remote Access Service.
    ///
    /// Available on: macOS, iOS
    ///
    /// On other platforms: becomes `User`
    Ras,

    /// macOS remote authentication and authorization.
    ///
    /// Available on: macOS, iOS
    ///
    /// On other platforms: becomes `Daemon`
    RemoteAuth,

    /// Security subsystems.
    ///
    /// Available on: FreeBSD, DragonFly BSD
    ///
    /// On other platforms: becomes `Auth`
    Security,

    Syslog,
    #[default]
    User,
    Uucp,
}

impl Facility {
    /// Gets the name of this `Facility`, in lowercase.
    ///
    /// The `FromStr` implementation accepts the same names, but it is
    /// case-insensitive.
    pub fn name(self) -> &'static str {
        match self {
            Facility::Auth => "auth",
            Facility::AuthPriv => "authpriv",
            Facility::Cron => "cron",
            Facility::Daemon => "daemon",
            Facility::Ftp => "ftp",
            Facility::Kern => "kern",
            Facility::Install => "install",
            Facility::Launchd => "launchd",
            Facility::Local0 => "local0",
            Facility::Local1 => "local1",
            Facility::Local2 => "local2",
            Facility::Local3 => "local3",
            Facility::Local4 => "local4",
            Facility::Local5 => "local5",
            Facility::Local6 => "local6",
            Facility::Local7 => "local7",
            Facility::Lpr => "lpr",
            Facility::Mail => "mail",
            Facility::Ntp => "ntp",
            Facility::NetInfo => "netinfo",
            Facility::News => "news",
            Facility::Ras => "ras",
            Facility::RemoteAuth => "remoteauth",
            Facility::Security => "security",
            Facility::Syslog => "syslog",
            Facility::User => "user",
            Facility::Uucp => "uucp",
        }
    }
}

impl Display for Facility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl From<Facility> for c_int {
    fn from(facility: Facility) -> Self {
        match facility {
            Facility::Auth => libc::LOG_AUTH,
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "emscripten",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
                target_env = "uclibc"
            ))]
            Facility::AuthPriv => libc::LOG_AUTHPRIV,
            #[cfg(not(any(
                target_os = "linux",
                target_os = "android",
                target_os = "emscripten",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
                target_env = "uclibc"
            )))]
            Facility::AuthPriv => libc::LOG_AUTH,
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "emscripten",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "solaris",
                target_os = "illumos",
                target_env = "uclibc"
            ))]
            Facility::Cron => libc::LOG_CRON,
            #[cfg(not(any(
                target_os = "linux",
                target_os = "android",
                target_os = "emscripten",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "solaris",
                target_os = "illumos",
                target_env = "uclibc"
            )))]
            Facility::Cron => libc::LOG_DAEMON,
            Facility::Daemon => libc::LOG_DAEMON,
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "emscripten",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
                target_env = "uclibc"
            ))]
            Facility::Ftp => libc::LOG_FTP,
            #[cfg(not(any(
                target_os = "linux",
                target_os = "android",
                target_os = "emscripten",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
                target_env = "uclibc"
            )))]
            Facility::Ftp => libc::LOG_DAEMON,
            Facility::Kern => libc::LOG_KERN,
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            Facility::Install => libc::LOG_INSTALL,
            #[cfg(not(any(target_os = "macos", target_os = "ios")))]
            Facility::Install => libc::LOG_USER,
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            Facility::Launchd => libc::LOG_LAUNCHD,
            #[cfg(not(any(target_os = "macos", target_os = "ios")))]
            Facility::Launchd => libc::LOG_DAEMON,
            Facility::Local0 => libc::LOG_LOCAL0,
            Facility::Local1 => libc::LOG_LOCAL1,
            Facility::Local2 => libc::LOG_LOCAL2,
            Facility::Local3 => libc::LOG_LOCAL3,
            Facility::Local4 => libc::LOG_LOCAL4,
            Facility::Local5 => libc::LOG_LOCAL5,
            Facility::Local6 => libc::LOG_LOCAL6,
            Facility::Local7 => libc::LOG_LOCAL7,
            Facility::Lpr => libc::LOG_LPR,
            Facility::Mail => libc::LOG_MAIL,
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            Facility::Ntp => libc::LOG_NTP,
            #[cfg(not(any(target_os = "freebsd", target_os = "dragonfly")))]
            Facility::Ntp => libc::LOG_DAEMON,
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            Facility::NetInfo => libc::LOG_NETINFO,
            #[cfg(not(any(target_os = "macos", target_os = "ios")))]
            Facility::NetInfo => libc::LOG_DAEMON,
            Facility::News => libc::LOG_NEWS,
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            Facility::Ras => libc::LOG_RAS,
            #[cfg(not(any(target_os = "macos", target_os = "ios")))]
            Facility::Ras => libc::LOG_USER,
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            Facility::RemoteAuth => libc::LOG_REMOTEAUTH,
            #[cfg(not(any(target_os = "macos", target_os = "ios")))]
            Facility::RemoteAuth => libc::LOG_DAEMON,
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            Facility::Security => libc::LOG_SECURITY,
            #[cfg(not(any(target_os = "freebsd", target_os = "dragonfly")))]
            Facility::Security => libc::LOG_AUTH,
            Facility::Syslog => libc::LOG_SYSLOG,
            Facility::User => libc::LOG_USER,
            Facility::Uucp => libc::LOG_UUCP,
        }
    }
}

impl FromStr for Facility {
    type Err = UnknownFacilityError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        let s = s.to_ascii_lowercase();

        match &*s {
            "auth" => Ok(Facility::Auth),
            "authpriv" => Ok(Facility::AuthPriv),
            "cron" => Ok(Facility::Cron),
            "daemon" => Ok(Facility::Daemon),
            "ftp" => Ok(Facility::Ftp),
            "kern" => Ok(Facility::Kern),
            "install" => Ok(Facility::Install),
            "launchd" => Ok(Facility::Launchd),
            "local0" => Ok(Facility::Local0),
            "local1" => Ok(Facility::Local1),
            "local2" => Ok(Facility::Local2),
            "local3" => Ok(Facility::Local3),
            "local4" => Ok(Facility::Local4),
            "local5" => Ok(Facility::Local5),
            "local6" => Ok(Facility::Local6),
            "local7" => Ok(Facility::Local7),
            "lpr" => Ok(Facility::Lpr),
            "mail" => Ok(Facility::Mail),
            "ntp" => Ok(Facility::Ntp),
            "netinfo" => Ok(Facility::NetInfo),
            "news" => Ok(Facility::News),
            "ras" => Ok(Facility::Ras),
            "remoteauth" => Ok(Facility::RemoteAuth),
            "security" => Ok(Facility::Security),
            "syslog" => Ok(Facility::Syslog),
            "user" => Ok(Facility::User),
            "uucp" => Ok(Facility::Uucp),
            _ => Err(UnknownFacilityError { name: s }),
        }
    }
}

impl TryFrom<c_int> for Facility {
    type Error = Error;

    fn try_from(value: c_int) -> StdResult<Self, Self::Error> {
        match value {
            libc::LOG_AUTH => Ok(Facility::Auth),
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "emscripten",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
                target_env = "uclibc"
            ))]
            libc::LOG_AUTHPRIV => Ok(Facility::AuthPriv),
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "emscripten",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "solaris",
                target_os = "illumos",
                target_env = "uclibc"
            ))]
            libc::LOG_CRON => Ok(Facility::Cron),
            libc::LOG_DAEMON => Ok(Facility::Daemon),
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "emscripten",
                target_os = "macos",
                target_os = "ios",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
                target_env = "uclibc"
            ))]
            libc::LOG_FTP => Ok(Facility::Ftp),
            libc::LOG_KERN => Ok(Facility::Kern),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            libc::LOG_INSTALL => Ok(Facility::Install),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            libc::LOG_LAUNCHD => Ok(Facility::Launchd),
            libc::LOG_LOCAL0 => Ok(Facility::Local0),
            libc::LOG_LOCAL1 => Ok(Facility::Local1),
            libc::LOG_LOCAL2 => Ok(Facility::Local2),
            libc::LOG_LOCAL3 => Ok(Facility::Local3),
            libc::LOG_LOCAL4 => Ok(Facility::Local4),
            libc::LOG_LOCAL5 => Ok(Facility::Local5),
            libc::LOG_LOCAL6 => Ok(Facility::Local6),
            libc::LOG_LOCAL7 => Ok(Facility::Local7),
            libc::LOG_LPR => Ok(Facility::Lpr),
            libc::LOG_MAIL => Ok(Facility::Mail),
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            libc::LOG_NTP => Ok(Facility::Ntp),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            libc::LOG_NETINFO => Ok(Facility::NetInfo),
            libc::LOG_NEWS => Ok(Facility::News),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            libc::LOG_RAS => Ok(Facility::Ras),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            libc::LOG_REMOTEAUTH => Ok(Facility::RemoteAuth),
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            libc::LOG_SECURITY => Ok(Facility::Security),
            libc::LOG_SYSLOG => Ok(Facility::Syslog),
            libc::LOG_USER => Ok(Facility::User),
            libc::LOG_UUCP => Ok(Facility::Uucp),
            _ => Err(ErrorKind::Invalid.into()),
        }
    }
}

/// Indicates that `<Facility as FromStr>::from_str` was called with an unknown
/// facility name.
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[non_exhaustive]
pub struct UnknownFacilityError {
    name: String,
}

impl UnknownFacilityError {
    /// The unrecognized facility name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for UnknownFacilityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unrecognized syslog facility name `{}`", self.name)
    }
}

impl StdError for UnknownFacilityError {}

#[test]
fn test_facility_from_str() {
    assert_eq!(Facility::from_str("daemon"), Ok(Facility::Daemon));
    assert_eq!(
        Facility::from_str("foobar"),
        Err(UnknownFacilityError {
            name: "foobar".to_string()
        })
    );
    assert_eq!(
        Facility::from_str("foobar").unwrap_err().to_string(),
        "unrecognized syslog facility name `foobar`"
    );
}
