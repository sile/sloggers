use crate::syslog::format::CustomMsgFormat;
use crate::syslog::{mock, Facility, SyslogBuilder};
use crate::types::{Severity, SourceLocation};
use crate::Build;
use slog::{debug, info};
use std::ffi::CStr;

#[test]
fn test_log() {
    let ((), events) = mock::testing(|| {
        {
            let tmp_logger = SyslogBuilder::new()
                .ident_str("hello")
                .log_ndelay()
                .log_odelay()
                .log_pid()
                .level(Severity::Debug)
                .source_location(SourceLocation::None)
                .build()
                .unwrap();

            debug!(tmp_logger, "Constructed a temporary logger.");

            // The logger will be dropped at this point, which should result in
            // a `closelog` call.
        }

        let logger = SyslogBuilder::new()
            .facility(Facility::Local0)
            .level(Severity::Debug)
            .ident_str("sloggers-example-app")
            .source_location(SourceLocation::None)
            .build()
            .unwrap();

        info!(logger, "Hello, world! This is a test message from `sloggers::syslog`."; "test" => "message");

        mock::wait_for_event_matching(|event| match event {
            mock::Event::SysLog { message, .. } => message.contains("This is a test message"),
            _ => false,
        });

        let logger2 = SyslogBuilder::new()
            .facility(Facility::Local1)
            .ident(CStr::from_bytes_with_nul(b"logger2\0").unwrap())
            .source_location(SourceLocation::None)
            .format(CustomMsgFormat(|_, _, _| Err(slog::Error::Other)))
            .build()
            .unwrap();

        info!(logger2, "Message from second logger while first still active."; "key" => "value");

        mock::wait_for_event_matching(|event| match event {
            mock::Event::SysLog { message, .. } => message == &slog::Error::Other.to_string(),
            _ => false,
        });
    });

    let expected_events = vec![
        mock::Event::OpenLog {
            facility: libc::LOG_USER,
            flags: libc::LOG_ODELAY | libc::LOG_PID,
            ident: "hello".to_string(),
        },
        mock::Event::SysLog {
            priority: libc::LOG_DEBUG,
            message_f: "%s".to_string(),
            message: "Constructed a temporary logger.".to_string(),
        },
        // This logger will `closelog` when dropped, because it has to in order
        // to free its `ident` string.
        mock::Event::CloseLog,
        mock::Event::DropOwnedIdent("hello".to_string()),
        mock::Event::OpenLog {
            facility: libc::LOG_LOCAL0,
            flags: 0,
            ident: "sloggers-example-app".to_string(),
        },
        mock::Event::SysLog {
            priority: libc::LOG_INFO,
            message_f: "%s".to_string(),
            message:
                "Hello, world! This is a test message from `sloggers::syslog`. [test=\"message\"]"
                    .to_string(),
        },
        mock::Event::OpenLog {
            facility: libc::LOG_LOCAL1,
            flags: 0,
            ident: "logger2".to_string(),
        },
        mock::Event::SysLog {
            priority: libc::LOG_INFO,
            message_f: "%s".to_string(),
            message: "Message from second logger while first still active.".to_string(),
        },
        mock::Event::SysLog {
            priority: libc::LOG_ERR,
            message_f: "Error fully formatting the previous log message: %s".to_string(),
            message: slog::Error::Other.to_string(),
        },
        mock::Event::DropOwnedIdent("sloggers-example-app".to_string()),
        // No `CloseLog` for `logger2` because it doesn't own its `ident`.
    ];

    assert!(
        events == expected_events,
        "events didn't match\ngot: {:#?}\nexpected: {:#?}",
        events,
        expected_events
    );
}
