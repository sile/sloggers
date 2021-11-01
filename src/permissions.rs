//! Cross platform functions to restrict file permissions when using the file logger.
#[cfg(unix)]
use std::fs::File;
use std::io;
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use winapi::um::winnt::{FILE_GENERIC_READ, FILE_GENERIC_WRITE, STANDARD_RIGHTS_ALL};

/// This is the security identifier in Windows for the owner of a file. See:
/// - https://docs.microsoft.com/en-us/troubleshoot/windows-server/identity/security-identifiers-in-windows#well-known-sids-all-versions-of-windows
#[cfg(windows)]
const OWNER_SID_STR: &str = "S-1-3-4";
/// We don't need any of the `AceFlags` listed here:
/// - https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-ace_header
#[cfg(windows)]
const OWNER_ACL_ENTRY_FLAGS: u8 = 0;
/// Generic Rights:
///  - https://docs.microsoft.com/en-us/windows/win32/fileio/file-security-and-access-rights
/// Individual Read/Write/Execute Permissions (referenced in generic rights link):
///  - https://docs.microsoft.com/en-us/windows/win32/wmisdk/file-and-directory-access-rights-constants
/// STANDARD_RIGHTS_ALL
///  - https://docs.microsoft.com/en-us/windows/win32/secauthz/access-mask
#[cfg(windows)]
const OWNER_ACL_ENTRY_MASK: u32 = FILE_GENERIC_READ | FILE_GENERIC_WRITE | STANDARD_RIGHTS_ALL;

/// Function to set the umask of the log files to `600`.
///
/// This ensures the log files are not world-readable.
#[cfg(unix)]
pub fn restrict_file_permissions(file: File) -> io::Result<File> {
    use std::os::unix::fs::PermissionsExt;
    let mut perm = file.metadata()?.permissions();
    perm.set_mode(0o600);
    file.set_permissions(perm)?;

    Ok(file)
}

/// Function to set the access control lists (ACLs) of the log files to only include the owner.
/// This is equivalent to a umask of `600` on `unix` systems.
///
/// This ensures the log fiels are not world-readable.
#[cfg(windows)]
pub fn restrict_file_permissions<P: AsRef<Path>>(path: P) -> io::Result<()> {
    use winapi::um::winnt::PSID;
    use windows_acl::acl::{AceType, ACL};
    use windows_acl::helper::sid_to_string;

    let path_str = path.as_ref().to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "Unable to open file path.".to_string(),
        )
    })?;

    let mut acl = ACL::from_file_path(path_str, false).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Unable to retrieve ACL: {:?}", e),
        )
    })?;

    let owner_sid = windows_acl::helper::string_to_sid(OWNER_SID_STR).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Unable to convert SID: {:?}", e),
        )
    })?;

    let entries = acl.all().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Unable to enumerate ACL entries: {:?}", e),
        )
    })?;

    // Add single entry for file owner.
    acl.add_entry(
        owner_sid.as_ptr() as PSID,
        AceType::AccessAllow,
        OWNER_ACL_ENTRY_FLAGS,
        OWNER_ACL_ENTRY_MASK,
    )
    .map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to add ACL entry for SID {} error={}",
                OWNER_SID_STR, e
            ),
        )
    })?;
    // Remove all AccessAllow entries from the file that aren't the owner_sid.
    for entry in &entries {
        if let Some(ref entry_sid) = entry.sid {
            let entry_sid_str = sid_to_string(entry_sid.as_ptr() as PSID)
                .unwrap_or_else(|_| "BadFormat".to_string());
            if entry_sid_str != OWNER_SID_STR {
                acl.remove(entry_sid.as_ptr() as PSID, Some(AceType::AccessAllow), None)
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to remove ACL entry for SID {}", entry_sid_str),
                        )
                    })?;
            }
        }
    }
    Ok(())
}
