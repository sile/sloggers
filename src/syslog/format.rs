//! Ways to format syslog messages with structured data.
//!
//! See [`MsgFormat`] for more details.
//!
//! [`MsgFormat`]: trait.MsgFormat.html

use serde::{Deserialize, Serialize};
use slog::{OwnedKVList, Record, KV};
use std::cell::Cell;
use std::fmt::{self, Debug, Display};
use std::sync::Arc;

/// A way to format syslog messages with structured data.
///
/// Syslog does not support structured log data. If Slog key-value pairs are to
/// be included with log messages, they must be included as part of the
/// message. Implementations of this trait determine if and how this will be
/// done.
pub trait MsgFormat: Sync + Send + Debug {
    /// Formats a log message and its key-value pairs into the given `Formatter`.
    ///
    /// Note that this method returns `slog::Result`, not `std::fmt::Result`.
    /// The caller of this method is responsible for handling the error,
    /// likely by storing it elsewhere and picking it up later. See the
    /// implementation of the `to_string` method for an example of how to do
    /// this.
    fn fmt(&self, f: &mut fmt::Formatter, record: &Record, values: &OwnedKVList) -> slog::Result;

    /// Formats a log message and its key-value pairs into a new `String`.
    fn to_string(&self, record: &Record, values: &OwnedKVList) -> slog::Result<String> {
        // This helper structure provides a convenient way to implement
        // `Display` with a closure.
        struct ClosureAsDisplay<F: Fn(&mut fmt::Formatter) -> fmt::Result>(F);
        impl<F: Fn(&mut fmt::Formatter) -> fmt::Result> Display for ClosureAsDisplay<F> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0(f)
            }
        }

        // If there is an error calling `self.fmt`, it will be stored here. We
        // have to use `Cell` because the `Display::fmt` method doesn't get a
        // mutable reference to `self`.
        let result: Cell<Option<slog::Error>> = Cell::new(None);

        // Construct our `Display` implementation…
        let s = ClosureAsDisplay(|f| {
            // Do the formatting.
            if let Err(e) = MsgFormat::fmt(self, f, record, values) {
                // If there's an error, smuggle it out.
                result.set(Some(e));
            }
            // Pretend to succeed, even if there was an error, or else
            // `to_string` will panic.
            Ok(())
        })
        // …and use it to generate a `String`.
        .to_string();

        // Extract the error, if any. It should be waiting inside `result`.
        if let Some(e) = result.take() {
            Err(e)
        } else {
            Ok(s)
        }
    }
}

impl<T: MsgFormat + ?Sized> MsgFormat for &T {
    fn fmt(&self, f: &mut fmt::Formatter, record: &Record, values: &OwnedKVList) -> slog::Result {
        MsgFormat::fmt(&**self, f, record, values)
    }
}

impl<T: MsgFormat + ?Sized> MsgFormat for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter, record: &Record, values: &OwnedKVList) -> slog::Result {
        MsgFormat::fmt(&**self, f, record, values)
    }
}

impl<T: MsgFormat + ?Sized> MsgFormat for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter, record: &Record, values: &OwnedKVList) -> slog::Result {
        MsgFormat::fmt(&**self, f, record, values)
    }
}

/// An implementation of [`MsgFormat`] that discards the key-value pairs and
/// logs only the [`msg`] part of a log [`Record`].
///
/// [`msg`]: https://docs.rs/slog/2/slog/struct.Record.html#method.msg
/// [`MsgFormat`]: trait.MsgFormat.html
/// [`Record`]: https://docs.rs/slog/2/slog/struct.Record.html
#[derive(Clone, Copy, Debug, Default)]
pub struct BasicMsgFormat;
impl MsgFormat for BasicMsgFormat {
    fn fmt(&self, f: &mut fmt::Formatter, record: &Record, _: &OwnedKVList) -> slog::Result {
        write!(f, "{}", record.msg())?;
        Ok(())
    }
}

/// A [`MsgFormat`] implementation that calls a closure to perform the
/// formatting.
///
/// This is meant to provide a convenient way to implement a custom
/// `MsgFormat`.
///
/// # Example
///
/// ```
/// use sloggers::Build;
/// use sloggers::syslog::SyslogBuilder;
/// use sloggers::syslog::format::CustomMsgFormat;
///
/// let logger = SyslogBuilder::new()
///     .format(CustomMsgFormat(|f, record, _| {
///         write!(f, "here's a message: {}", record.msg())?;
///         Ok(())
///     }))
///     .build()
///     .unwrap();
/// ```
///
/// Note the use of the `?` operator. The closure is expected to return
/// `Result<(), slog::Error>`, not the `Result<(), std::fmt::Error>` that
/// `write!` returns. `slog::Error` does have a conversion from
/// `std::fmt::Error`, which the `?` operator will automatically perform.
///
/// [`MsgFormat`]: trait.MsgFormat.html
pub struct CustomMsgFormat<
    T: Fn(&mut fmt::Formatter, &Record, &OwnedKVList) -> slog::Result + Send + Sync,
>(pub T);
impl<T: Fn(&mut fmt::Formatter, &Record, &OwnedKVList) -> slog::Result + Send + Sync> MsgFormat
    for CustomMsgFormat<T>
{
    fn fmt(&self, f: &mut fmt::Formatter, record: &Record, values: &OwnedKVList) -> slog::Result {
        self.0(f, record, values)
    }
}
impl<T: Fn(&mut fmt::Formatter, &Record, &OwnedKVList) -> slog::Result + Send + Sync> Debug
    for CustomMsgFormat<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomMsgFormat").finish()
    }
}

/// Copies input to output, but escapes characters as prescribed by RFC 5424 for PARAM-VALUEs.
struct Rfc5424LikeValueEscaper<W: fmt::Write>(W);

impl<W: fmt::Write> fmt::Write for Rfc5424LikeValueEscaper<W> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while let Some(index) = s.find(|c| c == '\\' || c == '"' || c == ']') {
            if index != 0 {
                self.0.write_str(&s[..index])?;
            }

            // All three delimiters are ASCII characters, so this won't have bogus results.
            self.write_char(s.as_bytes()[index] as char)?;

            if s.len() >= index {
                s = &s[(index + 1)..];
            } else {
                s = "";
                break;
            }
        }

        if !s.is_empty() {
            self.0.write_str(s)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        match c {
            '\\' => self.0.write_str(r"\\"),
            '"' => self.0.write_str("\\\""),
            ']' => self.0.write_str("\\]"),
            _ => write!(self.0, "{}", c),
        }
    }
}

#[test]
fn test_rfc_5424_like_value_escaper() {
    use std::iter;

    fn case(input: &str, expected_output: &str) {
        let mut e = Rfc5424LikeValueEscaper(String::new());
        fmt::Write::write_str(&mut e, input).unwrap();
        assert_eq!(e.0, expected_output);
    }

    // Test that each character is properly escaped.
    for c in &['\\', '"', ']'] {
        let ec = format!("\\{}", c);

        {
            let input = format!("{}", c);
            case(&*input, &*ec);
        }

        for at_start_count in 0..=2 {
            for at_mid_count in 0..=2 {
                for at_end_count in 0..=2 {
                    // First, we assemble the input and expected output strings.
                    let mut input = String::new();
                    let mut expected_output = String::new();

                    // Place the symbol(s) at the beginning of the strings.
                    input.extend(iter::repeat(c).take(at_start_count));
                    expected_output.extend(iter::repeat(&*ec).take(at_start_count));

                    // First plain text.
                    input.push_str("foo");
                    expected_output.push_str("foo");

                    // Middle symbol(s).
                    input.extend(iter::repeat(c).take(at_mid_count));
                    expected_output.extend(iter::repeat(&*ec).take(at_mid_count));

                    // Second plain text.
                    input.push_str("bar");
                    expected_output.push_str("bar");

                    // End symbol(s).
                    input.extend(iter::repeat(c).take(at_end_count));
                    expected_output.extend(iter::repeat(&*ec).take(at_end_count));

                    // Finally, test this combination.
                    case(&*input, &*expected_output);
                }
            }
        }
    }

    case("", "");
    case("foo", "foo");
    case("[foo]", "[foo\\]");
    case("\\\"]", "\\\\\\\"\\]"); // \"] ⇒ \\\"\]
}

/// An implementation of [`MsgFormat`] that formats the key-value pairs of a
/// log [`Record`] similarly to [RFC 5424].
///
/// # Not really RFC 5424
///
/// This does not actually generate conformant RFC 5424 STRUCTURED-DATA. The
/// differences are:
///
/// * All key-value pairs are placed into a single SD-ELEMENT.
/// * The SD-ELEMENT does not contain an SD-ID, only SD-PARAMs.
/// * PARAM-NAMEs are encoded in UTF-8, not ASCII.
/// * Forbidden characters in PARAM-NAMEs are not filtered out, nor is an error
///   raised if a key contains such characters.
///
/// # Example output
///
/// Given a log message `Hello, world!`, where the key `key1` has the value
/// `value1` and `key2` has the value `value2`, the formatted message will be
/// `Hello, world! [key1="value1" key2="value2"]` (possibly with `key2` first
/// instead of `key1`).
///
/// [`MsgFormat`]: trait.MsgFormat.html
/// [`Record`]: https://docs.rs/slog/2/slog/struct.Record.html
/// [RFC 5424]: https://tools.ietf.org/html/rfc5424
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultMsgFormat;
impl MsgFormat for DefaultMsgFormat {
    fn fmt(&self, f: &mut fmt::Formatter, record: &Record, values: &OwnedKVList) -> slog::Result {
        struct SerializerImpl<'a, 'b> {
            f: &'a mut fmt::Formatter<'b>,
            is_first_kv: bool,
        }

        impl<'a, 'b> SerializerImpl<'a, 'b> {
            fn new(f: &'a mut fmt::Formatter<'b>) -> Self {
                Self {
                    f,
                    is_first_kv: true,
                }
            }

            fn finish(&mut self) -> slog::Result {
                if !self.is_first_kv {
                    write!(self.f, "]")?;
                }
                Ok(())
            }
        }

        impl<'a, 'b> slog::Serializer for SerializerImpl<'a, 'b> {
            fn emit_arguments(&mut self, key: slog::Key, val: &fmt::Arguments) -> slog::Result {
                use fmt::Write;

                self.f
                    .write_str(if self.is_first_kv { " [" } else { " " })?;
                self.is_first_kv = false;

                // Write the key unaltered, but escape the value.
                //
                // RFC 5424 does not allow space, ']', '"', or '\' to
                // appear in PARAM-NAMEs, and does not allow such
                // characters to be escaped.
                write!(self.f, "{}=\"", key)?;
                write!(Rfc5424LikeValueEscaper(&mut self.f), "{}", val)?;
                self.f.write_char('"')?;
                Ok(())
            }
        }

        write!(f, "{}", record.msg())?;

        {
            let mut serializer = SerializerImpl::new(f);

            values.serialize(record, &mut serializer)?;
            record.kv().serialize(record, &mut serializer)?;
            serializer.finish()?;
        }

        Ok(())
    }
}

/// Makes sure the example output for `DefaultMsgFormat` is what it actually
/// generates.
#[test]
fn test_default_msg_format() {
    use slog::Level;

    let result = DefaultMsgFormat
        .to_string(
            &record!(
                Level::Info,
                "",
                &format_args!("Hello, world!"),
                b!("key1" => "value1")
            ),
            &o!("key2" => "value2").into(),
        )
        .expect("formatting failed");

    assert!(
        // The KVs' order is not well-defined, so they might get reversed.
        result == "Hello, world! [key1=\"value1\" key2=\"value2\"]"
            || result == "Hello, world! [key2=\"value2\" key1=\"value1\"]"
    );
}

/// Enumeration of built-in `MsgFormat`s, for use with serde.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum MsgFormatConfig {
    /// [`DefaultMsgFormat`](struct.DefaultMsgFormat.html).
    Default,

    /// [`BasicMsgFormat`](struct.BasicMsgFormat.html).
    Basic,
}

impl Default for MsgFormatConfig {
    fn default() -> Self {
        MsgFormatConfig::Default
    }
}

impl From<MsgFormatConfig> for Arc<dyn MsgFormat> {
    fn from(conf: MsgFormatConfig) -> Self {
        Self::from(&conf)
    }
}

impl From<&MsgFormatConfig> for Arc<dyn MsgFormat> {
    fn from(conf: &MsgFormatConfig) -> Self {
        match *conf {
            MsgFormatConfig::Default => Arc::new(DefaultMsgFormat),
            MsgFormatConfig::Basic => Arc::new(BasicMsgFormat),
        }
    }
}
