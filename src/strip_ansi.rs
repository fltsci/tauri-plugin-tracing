//! ANSI escape code stripping for clean file output.

use std::io::Write;
use std::sync::{Mutex, MutexGuard};

/// A writer wrapper that strips ANSI escape codes from all output.
///
/// This is used for file logging to ensure clean output even when stdout
/// layers use ANSI colors. Due to how `tracing_subscriber` shares span field
/// formatting between layers, ANSI codes from one layer can leak into others.
/// This wrapper strips those codes at write time.
///
/// Uses a zero-copy fast path when no ANSI codes are present. Thread-safe
/// via internal Mutex.
///
/// # Example
///
/// ```rust,no_run
/// use tauri_plugin_tracing::StripAnsiWriter;
/// use tauri_plugin_tracing::tracing_appender::non_blocking;
/// use tauri_plugin_tracing::tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
///
/// let file_appender = tracing_appender::rolling::daily("/tmp/logs", "app.log");
/// let (non_blocking, _guard) = non_blocking(file_appender);
///
/// tracing_subscriber::registry()
///     .with(fmt::layer())  // stdout with ANSI
///     .with(fmt::layer().with_writer(StripAnsiWriter::new(non_blocking)).with_ansi(false))
///     .init();
/// ```
pub struct StripAnsiWriter<W> {
    pub(crate) inner: Mutex<W>,
}

impl<W> StripAnsiWriter<W> {
    /// Creates a new `StripAnsiWriter` that wraps the given writer.
    pub fn new(inner: W) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }
}

/// Strips ANSI escape codes from input and writes to output.
/// Returns the number of bytes from input that were processed.
pub(crate) fn strip_ansi_and_write<W: Write>(writer: &mut W, buf: &[u8]) -> std::io::Result<usize> {
    let input_len = buf.len();

    // Fast path: use memchr to check for ESC byte. If none, write directly.
    let Some(first_esc) = memchr::memchr(0x1b, buf) else {
        writer.write_all(buf)?;
        return Ok(input_len);
    };

    // Slow path: ANSI codes present, need to strip them
    // Pre-allocate with capacity for worst case (all kept)
    let mut output = Vec::with_capacity(input_len);

    // Copy everything before first ESC
    output.extend_from_slice(&buf[..first_esc]);
    let mut i = first_esc;

    while i < buf.len() {
        if buf[i] == 0x1b && i + 1 < buf.len() && buf[i + 1] == b'[' {
            // Found ESC[, skip the SGR sequence
            i += 2;
            while i < buf.len() {
                let c = buf[i];
                i += 1;
                if c == b'm' {
                    break;
                }
                if !c.is_ascii_digit() && c != b';' {
                    break;
                }
            }
        } else {
            output.push(buf[i]);
            i += 1;
        }
    }

    writer.write_all(&output)?;
    Ok(input_len)
}

/// A writer handle returned by the [`MakeWriter`](tracing_subscriber::fmt::MakeWriter) implementation.
///
/// This type implements [`std::io::Write`] and strips ANSI codes during writes.
pub struct StripAnsiWriterGuard<'a, W> {
    guard: MutexGuard<'a, W>,
}

impl<W: Write> Write for StripAnsiWriterGuard<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        strip_ansi_and_write(&mut *self.guard, buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.guard.flush()
    }
}

// Implement MakeWriter so this can be used with fmt::layer().with_writer()
impl<'a, W: Write + 'a> tracing_subscriber::fmt::MakeWriter<'a> for StripAnsiWriter<W> {
    type Writer = StripAnsiWriterGuard<'a, W>;

    fn make_writer(&'a self) -> Self::Writer {
        StripAnsiWriterGuard {
            guard: self.inner.lock().unwrap_or_else(|e| e.into_inner()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::fmt::MakeWriter;

    #[test]
    fn strip_ansi_fast_path_no_escape() {
        let mut output = Vec::new();
        let input = b"Hello, world!";
        let written = strip_ansi_and_write(&mut output, input).unwrap();
        assert_eq!(written, input.len());
        assert_eq!(output, input);
    }

    #[test]
    fn strip_ansi_removes_sgr_sequence() {
        let mut output = Vec::new();
        let input = b"\x1b[32mgreen\x1b[0m";
        let written = strip_ansi_and_write(&mut output, input).unwrap();
        assert_eq!(written, input.len());
        assert_eq!(output, b"green");
    }

    #[test]
    fn strip_ansi_removes_multiple_sequences() {
        let mut output = Vec::new();
        let input = b"\x1b[1m\x1b[31mBold Red\x1b[0m Normal";
        let written = strip_ansi_and_write(&mut output, input).unwrap();
        assert_eq!(written, input.len());
        assert_eq!(output, b"Bold Red Normal");
    }

    #[test]
    fn strip_ansi_handles_complex_sgr() {
        let mut output = Vec::new();
        // SGR with multiple parameters: ESC[1;31;42m
        let input = b"\x1b[1;31;42mStyled\x1b[0m";
        let written = strip_ansi_and_write(&mut output, input).unwrap();
        assert_eq!(written, input.len());
        assert_eq!(output, b"Styled");
    }

    #[test]
    fn strip_ansi_preserves_non_sgr_escape() {
        let mut output = Vec::new();
        // ESC not followed by [ should be preserved
        let input = b"Hello\x1bWorld";
        let written = strip_ansi_and_write(&mut output, input).unwrap();
        assert_eq!(written, input.len());
        assert_eq!(output, b"Hello\x1bWorld");
    }

    #[test]
    fn strip_ansi_handles_escape_at_end() {
        let mut output = Vec::new();
        let input = b"Hello\x1b";
        let written = strip_ansi_and_write(&mut output, input).unwrap();
        assert_eq!(written, input.len());
        assert_eq!(output, b"Hello\x1b");
    }

    #[test]
    fn strip_ansi_handles_incomplete_sequence() {
        let mut output = Vec::new();
        // ESC[ without terminator
        let input = b"Hello\x1b[31";
        let written = strip_ansi_and_write(&mut output, input).unwrap();
        assert_eq!(written, input.len());
        // Incomplete sequence is stripped up to where parsing stops
        assert_eq!(output, b"Hello");
    }

    #[test]
    fn strip_ansi_writer_works() {
        let inner = Vec::new();
        let writer = StripAnsiWriter::new(inner);
        {
            let mut guard = writer.make_writer();
            guard.write_all(b"\x1b[32mtest\x1b[0m").unwrap();
        }
        let result = writer.inner.lock().unwrap();
        assert_eq!(&*result, b"test");
    }

    #[test]
    fn strip_ansi_empty_input() {
        let mut output = Vec::new();
        let input = b"";
        let written = strip_ansi_and_write(&mut output, input).unwrap();
        assert_eq!(written, 0);
        assert_eq!(output, b"");
    }

    #[test]
    fn strip_ansi_only_escape_sequences() {
        let mut output = Vec::new();
        let input = b"\x1b[31m\x1b[0m";
        let written = strip_ansi_and_write(&mut output, input).unwrap();
        assert_eq!(written, input.len());
        assert_eq!(output, b"");
    }
}
