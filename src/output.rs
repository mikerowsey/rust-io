use std::{
    fmt,
    io::{self, Write},
};

/// Fast buffered output.
///
/// Values are written through the [`Writable`] trait into an internal byte
/// buffer. The completed buffer can be extracted with [`Output::into_bytes`]
/// or written to any [`std::io::Write`] implementation with
/// [`Output::write_to`].
#[must_use]
pub struct Output {
    buf: Vec<u8>,
}

/// A value that can write itself into an [`Output`] buffer.
pub trait Writable {
    /// Writes `self` into `out`.
    fn write_to(self, out: &mut Output);
}

impl Output {
    /// Creates an empty output buffer with a default capacity of 1 MiB.
    pub fn new() -> Self {
        Self::with_capacity(1 << 20)
    }

    /// Creates an empty output buffer with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    /// Writes a value.
    #[inline]
    pub fn write<T: Writable>(&mut self, value: T) {
        value.write_to(self);
    }

    /// Writes a value followed by a newline.
    #[inline]
    pub fn writeln<T: Writable>(&mut self, value: T) {
        self.write(value);
        self.endl();
    }

    /// Appends a newline.
    #[inline]
    pub fn endl(&mut self) {
        self.write_byte(b'\n');
    }

    /// Writes the buffered output to `writer`.
    ///
    /// # Errors
    ///
    /// Returns any error produced by the destination writer.
    pub fn write_to<W: Write>(self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.buf)
    }

    /// Consumes the output and returns the underlying byte buffer.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    /// Returns `true` if the output buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    #[inline]
    fn write_byte(&mut self, byte: u8) {
        self.buf.push(byte);
    }

    #[inline]
    fn write_u64(&mut self, mut value: u64) {
        if value == 0 {
            self.write_byte(b'0');
            return;
        }

        let start = self.buf.len();

        while value > 0 {
            self.write_byte(b'0' + (value % 10) as u8);
            value /= 10;
        }

        self.buf[start..].reverse();
    }

    #[inline]
    fn write_i64(&mut self, value: i64) {
        if value < 0 {
            self.write_byte(b'-');
            self.write_u64(value.unsigned_abs());
        } else {
            self.write_u64(value as u64);
        }
    }

    #[inline]
    fn write_f64(&mut self, value: f64) {
        self.write_bytes(value.to_string().as_bytes());
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Write for Output {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}

impl Writable for u64 {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_u64(self);
    }
}

impl Writable for i64 {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_i64(self);
    }
}

impl Writable for f64 {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_f64(self);
    }
}

impl Writable for bool {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_bytes(if self { b"true" } else { b"false" });
    }
}

impl Writable for &[u8] {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_bytes(self);
    }
}

impl<const N: usize> Writable for &[u8; N] {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_bytes(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    fn output_string(out: Output) -> String {
        String::from_utf8(out.into_bytes()).unwrap()
    }

    #[test]
    fn new_output_is_empty() {
        assert!(Output::new().is_empty());
    }

    #[test]
    fn default_output_is_empty() {
        assert!(Output::default().is_empty());
    }

    #[test]
    fn writes_u64() {
        let mut out = Output::with_capacity(0);
        out.write(0_u64);
        assert_eq!(output_string(out), "0");
    }

    #[test]
    fn writes_i64() {
        let mut out = Output::with_capacity(0);
        out.write(42_i64);
        assert_eq!(output_string(out), "42");
    }

    #[test]
    fn writes_f64() {
        let mut out = Output::with_capacity(0);
        out.write(1.5_f64);
        assert_eq!(output_string(out), "1.5");
    }

    #[test]
    fn writes_bytes() {
        let mut out = Output::with_capacity(0);
        out.write(b"hello world");
        assert_eq!(output_string(out), "hello world");
    }

    #[test]
    fn writes_bool() {
        let mut out = Output::with_capacity(0);
        out.write(true);
        assert_eq!(output_string(out), "true");
    }

    #[test]
    fn writeln_appends_newline() {
        let mut out = Output::with_capacity(0);

        out.writeln(b"hello");

        assert_eq!(output_string(out), "hello\n");
    }

    #[test]
    fn endl_appends_newline() {
        let mut out = Output::with_capacity(0);

        out.endl();

        assert_eq!(out.into_bytes(), b"\n");
    }

    #[test]
    fn write_to_writes_into_any_io_writer() {
        let mut out = Output::with_capacity(0);
        out.write(b"hello");

        let mut dest = Vec::new();
        out.write_to(&mut dest).unwrap();

        assert_eq!(dest, b"hello");
    }

    #[test]
    fn implements_fmt_write() {
        let mut out = Output::with_capacity(0);

        write!(&mut out, "{} + {} = {}", 2, 3, 5).unwrap();

        assert_eq!(output_string(out), "2 + 3 = 5");
    }
}
