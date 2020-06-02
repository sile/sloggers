//! Mocks for the POSIX `syslog` API.
//! 
//! The mock `syslog` function here is a bit different from the real one. It
//! takes exactly three parameters, whereas the real one takes two or more.
//! This works for our purposes because this crate always calls it with exactly
//! three parameters anyway.

use libc::{c_char, c_int};
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::mem;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Condvar, Mutex, MutexGuard};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    OpenLog {
        ident: String,
        flags: c_int,
        facility: c_int,
    },
    CloseLog,
    SysLog {
        priority: c_int,
        message_f: String,
        message: String,
    },
    DropOwnedIdent(String),
}

static EVENTS: Lazy<Mutex<Vec<Event>>> = Lazy::new(|| Mutex::new(Vec::new()));
static EVENTS_CV: Lazy<Condvar> = Lazy::new(|| Condvar::new());
static TESTING: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub fn testing<T>(f: impl FnOnce() -> T) -> (T, Vec<Event>) {
    let _locked = TESTING.lock().unwrap();

    let result = catch_unwind(AssertUnwindSafe(f));
    let events = take_events();

    match result {
        Ok(ok) => (ok, events),
        Err(panicked) => resume_unwind(panicked),
    }
}

pub fn take_events() -> Vec<Event> {
    let mut events: MutexGuard<Vec<Event>> = EVENTS.lock().unwrap();
    mem::take(&mut *events)
}

pub fn push_event(event: Event) {
    let mut events: MutexGuard<Vec<Event>> = EVENTS.lock().unwrap();
    events.push(event);
    EVENTS_CV.notify_all();
}

pub fn wait_for_event_matching(matching: impl Fn(&Event) -> bool) {
    let mut events: MutexGuard<Vec<Event>> = EVENTS.lock().unwrap();

    while !events.iter().any(&matching) {
        events = EVENTS_CV.wait(events).unwrap();
    }
}

pub unsafe extern "C" fn openlog(ident: *const c_char, logopt: c_int, facility: c_int) {
    push_event(Event::OpenLog {
        ident: string_from_ptr(ident),
        flags: logopt,
        facility,
    });
}

pub unsafe extern "C" fn closelog() {
    push_event(Event::CloseLog);
}

pub unsafe extern "C" fn syslog(priority: c_int, message_f: *const c_char, message: *const c_char) {
    push_event(Event::SysLog {
        priority,
        message_f: string_from_ptr(message_f),
        message: string_from_ptr(message),
    });
}

pub unsafe fn string_from_ptr(ptr: *const c_char) -> String {
    String::from(CStr::from_ptr(ptr).to_string_lossy())
}
