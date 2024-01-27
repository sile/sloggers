use super::format::MsgFormat;
use super::SyslogBuilder;
use libc::{c_char, c_int};
use once_cell::sync::Lazy;
use slog::{Drain, Level, OwnedKVList, Record};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::ptr;
use std::result::Result as StdResult;
use std::sync::{Arc, Mutex, MutexGuard};

#[cfg(test)]
use super::mock::{self, closelog, openlog, syslog};
#[cfg(not(test))]
use libc::{closelog, openlog, syslog};

/// Keeps track of which `ident` string was most recently passed to `openlog`.
///
/// The mutex is to be locked while calling `openlog` or `closelog`. It
/// contains a possibly-null pointer to the `ident` string most recently passed
/// to `openlog`, if that pointer came from `CStr::as_ptr`.
///
/// The pointer is stored as a `usize` because pointers are `!Send`. It is only
/// used for comparison, never dereferenced.
///
/// # Purpose and rationale
///
/// The POSIX `openlog` function accepts a pointer to a C string. Though POSIX
/// does not specify the expected lifetime of the string, all known
/// implementations either
///
/// 1. keep the pointer in a global variable, or
/// 2. copy the string into an internal buffer, which is kept in a global
///    variable.
///
/// When running with an implementation in the second category, the string may
/// be safely freed right away. When running with an implementation in the
/// first category, however, the string must not be freed until either
/// `closelog` is called or `openlog` is called with a *different, non-null*
/// `ident`.
///
/// This mutex keeps track of which `ident` was most recently passed, making it
/// possible to decide whether `closelog` needs to be called before a given
/// `ident` string is dropped.
///
/// (Note: In the original 4.4BSD source code, the pointer is kept in a global
/// variable, but `closelog` does *not* clear the pointer. In this case, it is
/// only safe to free the string after `openlog` has been called with a
/// different, non-null `ident`. Fortunately, all present-day implementations
/// of `closelog` either clear the pointer or don't retain it at all.)
#[allow(clippy::mutex_atomic)]
static LAST_UNIQUE_IDENT: Lazy<Mutex<usize>> =
    Lazy::new(|| Mutex::new(ptr::null::<c_char>() as usize));

pub(super) struct SyslogDrain {
    /// The `ident` string, if it is owned by this `SyslogDrain`.
    ///
    /// This is kept so that the string can be freed (and `closelog` called, if
    /// necessary) when this `SyslogDrain` is dropped.
    unique_ident: Option<Box<CStr>>,

    /// The format for log messages.
    format: Arc<dyn MsgFormat>,
}

impl SyslogDrain {
    pub fn new(builder: &SyslogBuilder) -> Self {
        // `ident` is the pointer that will be passed to `openlog`, maybe null.
        //
        // `unique_ident` is the same pointer, wrapped in `Some` and `NonNull`,
        // but only if the `ident` string provided by the application is owned.
        // Otherwise it's `None`, indicating that `ident` either is null or
        // points to a `&'static` string.
        let (ident, unique_ident): (*const c_char, Option<Box<CStr>>) = match builder.ident.clone()
        {
            Some(Cow::Owned(ident_s)) => {
                let unique_ident = ident_s.into_boxed_c_str();

                // Calling `NonNull:new_unchecked` is correct because
                // `CString::into_raw` never returns a null pointer.
                (unique_ident.as_ptr(), Some(unique_ident))
            }
            Some(Cow::Borrowed(ident_s)) => (ident_s.as_ptr(), None),
            None => (ptr::null(), None),
        };

        {
            // `openlog` and `closelog` are only called while holding the mutex
            // around `last_unique_ident`.
            let mut last_unique_ident: MutexGuard<usize> = LAST_UNIQUE_IDENT.lock().unwrap();

            // Here, we call `openlog`. This has to happen *before* freeing the
            // previous `ident` string, if applicable.
            unsafe {
                openlog(ident, builder.option, builder.facility.into());
            }

            // If `openlog` is called with a null `ident` pointer, then the
            // `ident` string passed to it previously will remain in use. But
            // if the `ident` pointer is not null, then `last_unique_ident`
            // needs updating.
            if !ident.is_null() {
                *last_unique_ident = match &unique_ident {
                    // If the `ident` string is owned, store the pointer to it.
                    Some(s) => s.as_ptr(),

                    // If the `ident` string is not owned, set the stored
                    // pointer to null.
                    None => ptr::null::<c_char>(),
                } as usize;
            }
        }

        SyslogDrain {
            unique_ident,
            format: builder.format.clone(),
        }
    }
}

impl Drop for SyslogDrain {
    fn drop(&mut self) {
        // Check if this `SyslogDrain` was created with an owned `ident`
        // string.
        if let Some(my_ident) = self.unique_ident.take() {
            // If so, then we need to check if that string is the one that
            // was most recently passed to `openlog`.
            let mut last_unique_ident: MutexGuard<usize> = match LAST_UNIQUE_IDENT.lock() {
                Ok(locked) => locked,

                // If the mutex was poisoned, then we'll just let the
                // string leak.
                //
                // There's no point in panicking here, and if there was a
                // panic after `openlog` but before the pointer in the
                // mutex was updated, then trying to free the pointed-to
                // string may result in undefined behavior from a double
                // free.
                //
                // Thankfully, Rust's standard mutex implementation
                // supports poisoning. Some alternative mutex
                // implementations, such as in the `parking_lot` crate,
                // don't support poisoning and would expose us to the
                // aforementioned undefined behavior.
                //
                // It would be nice if we could un-poison a poisoned mutex,
                // though. We have a perfectly good recovery strategy for
                // that situation (resetting its pointer to null), but no way
                // to use it.
                Err(_) => {
                    Box::leak(my_ident);
                    return;
                }
            };

            if my_ident.as_ptr() as usize == *last_unique_ident {
                // Yes, the most recently used string was ours. We need to
                // call `closelog` before our string is dropped.
                //
                // Note that this isn't completely free of races. It's still
                // possible for some other code to call `openlog` independently
                // of this module, after our `openlog` call. In that case, this
                // `closelog` call will incorrectly close *that* logging handle
                // instead of the one belonging to this `SyslogDrain`.
                //
                // Behavior in that case is still well-defined. Subsequent
                // calls to `syslog` will implicitly reopen the logging handle
                // anyway. The only problem is that the `openlog` options
                // (facility, program name, etc) will all be reset. For this
                // reason, it is a bad idea for a library to call `openlog` (or
                // construct a `SyslogDrain`!) except when instructed to do so
                // by the main program.
                unsafe {
                    closelog();
                }

                // Also, be sure to reset the pointer stored in the mutex.
                // Although it is never dereferenced, letting it dangle may
                // cause the above `if` to test true when it shouldn't, which
                // would result in `closelog` being called when it shouldn't.
                *last_unique_ident = ptr::null::<c_char>() as usize;
            }

            // When testing, before dropping the owned string, copy it into
            // a mock event. We'll still drop it, though, in order to test for
            // double-free bugs.
            #[cfg(test)]
            mock::push_event(mock::Event::DropOwnedIdent(String::from(
                my_ident.to_string_lossy(),
            )));

            // Now that `closelog` has been called, it's safe for our string to
            // be dropped, which will happen here.
        }
    }
}

impl Drain for SyslogDrain {
    type Ok = ();
    type Err = slog::Never;

    fn log(&self, record: &Record, values: &OwnedKVList) -> StdResult<Self::Ok, Self::Err> {
        // Format the message. If formatting fails, use an effectively null
        // format (which shouldn't ever fail), and separately log the error.
        let (msg, fmt_err) = match MsgFormat::to_string(self.format.as_ref(), record, values) {
            Ok(msg) => (msg, None),
            Err(fmt_err) => (record.msg().to_string(), Some(fmt_err.to_string())),
        };

        // Convert both strings to C strings.
        let msg = to_cstring_lossy(msg);
        let fmt_err = fmt_err.map(to_cstring_lossy);

        // Figure out the priority.
        let priority: c_int = match record.level() {
            Level::Critical => libc::LOG_CRIT,
            Level::Error => libc::LOG_ERR,
            Level::Warning => libc::LOG_WARNING,
            Level::Debug | Level::Trace => libc::LOG_DEBUG,

            // `slog::Level` isn't non-exhaustive, so adding any more levels
            // would be a breaking change. That is highly unlikely to ever
            // happen. Still, we'll handle the possibility here, just in case.
            _ => libc::LOG_INFO,
        };

        // All set. Submit the log message.
        unsafe {
            syslog(
                priority,
                CStr::from_bytes_with_nul_unchecked(b"%s\0").as_ptr(),
                msg.as_ptr(),
            );
        }

        // If there was a formatting error, log that too.
        if let Some(fmt_err) = fmt_err {
            unsafe {
                syslog(
                    libc::LOG_ERR,
                    CStr::from_bytes_with_nul_unchecked(
                        b"Error fully formatting the previous log message: %s\0",
                    )
                    .as_ptr(),
                    fmt_err.as_ptr(),
                );
            }
        }

        // Done.
        Ok(())
    }
}

/// Converts a `String` to a `CString`, stripping null bytes in the middle.
///
/// A null byte is added at the end if there isn't one already.
///
/// The difference between this and `CString::new` is that that method will
/// fail if there are any null bytes instead of stripping them.
fn to_cstring_lossy(s: String) -> CString {
    // Get the bytes of the string.
    let mut s: Vec<u8> = s.into();

    // Strip any null bytes from the string.
    s.retain(|b| *b != 0);

    // This is sound because we just stripped all the null bytes from the
    // input. Note that `CString::from_vec_unchecked` does add a null byte to
    // the end of the input.
    unsafe { CString::from_vec_unchecked(s) }
}
