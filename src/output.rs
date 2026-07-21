use std::{
    fmt,
    io::{self, Write},
};

/// Fast-buffered output.
///
/// Values are pushed through the [`Writable`] trait into an internal byte
/// buffer. The completed buffer can be extracted with [`OutputBuffer::into_bytes`]
/// or written to any [`Write`] implementation with
/// [`OutputBuffer::write_to`].
#[must_use]
pub struct OutputBuffer {
    buf: Vec<u8>,
    delimiter: Option<Vec<u8>>,
    needs_delimiter: bool,
}

/// A value that can write itself into an [`OutputBuffer`].
pub trait Writable {
    /// Writes `self` into `out`.
    fn write_to(self, out: &mut OutputBuffer);
}

impl OutputBuffer {
    /// Creates an empty output buffer with a default capacity of 1 MiB.
    pub fn new() -> Self {
        Self::with_capacity(1 << 20)
    }

    /// Creates an empty output buffer with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            delimiter: None,
            needs_delimiter: false,
        }
    }

    /// Pushes a value.
    #[inline]
    pub fn push<T: Writable>(&mut self, value: T) {
        value.write_to(self);
    }

    /// Pushes raw bytes.
    #[inline]
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.write_delimiter();
        self.append_bytes(bytes);
        self.needs_delimiter = true;
    }

    /// Pushes UTF-8 text.
    #[inline]
    pub fn push_str(&mut self, value: &str) {
        self.push_bytes(value.as_bytes());
    }

    /// Sets a delimiter that is written before each pushed token after the first.
    ///
    /// Use [`OutputBuffer::endl`] to terminate a line without writing a delimiter
    /// before the newline.
    pub fn set_delimiter<D: AsRef<[u8]>>(&mut self, delimiter: D) {
        self.delimiter = Some(delimiter.as_ref().to_vec());
    }

    /// Clears the current delimiter.
    pub fn clear_delimiter(&mut self) {
        self.delimiter = None;
        self.needs_delimiter = false;
    }

    /// Pushes a newline.
    #[inline]
    pub fn endl(&mut self) {
        self.write_byte(b'\n');
        self.needs_delimiter = false;
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
    fn write_u128(&mut self, mut value: u128) {
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
    fn write_i128(&mut self, value: i128) {
        if value < 0 {
            self.write_byte(b'-');
            self.write_u128(value.unsigned_abs());
        } else {
            self.write_u128(value as u128);
        }
    }

    #[inline]
    fn write_f64(&mut self, value: f64) {
        self.push_str(&value.to_string());
    }

    #[inline]
    fn write_delimiter(&mut self) {
        if self.needs_delimiter {
            if let Some(delimiter) = &self.delimiter {
                self.buf.extend_from_slice(delimiter);
            }
        }
    }

    #[inline]
    fn append_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }
}

impl Default for OutputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Write for OutputBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.append_bytes(s.as_bytes());
        Ok(())
    }
}

impl Writable for u64 {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.write_u64(self);
    }
}

impl Writable for u32 {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.write_u64(self as u64);
    }
}

impl Writable for u128 {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.write_u128(self);
    }
}

impl Writable for usize {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.write_u128(self as u128);
    }
}

impl Writable for i64 {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.write_i128(self as i128);
    }
}

impl Writable for i32 {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.write_i128(self as i128);
    }
}

impl Writable for i128 {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.write_i128(self);
    }
}

impl Writable for isize {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.write_i128(self as i128);
    }
}

impl Writable for f64 {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.write_f64(self);
    }
}

impl Writable for bool {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.push_bytes(if self { b"true" } else { b"false" });
    }
}

impl Writable for &[u8] {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.push_bytes(self);
    }
}

impl<const N: usize> Writable for &[u8; N] {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.push_bytes(self);
    }
}

impl Writable for &str {
    #[inline]
    fn write_to(self, out: &mut OutputBuffer) {
        out.push_str(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    fn output_string(out: OutputBuffer) -> String {
        String::from_utf8(out.into_bytes()).unwrap()
    }

    #[test]
    fn new_output_is_empty() {
        assert!(OutputBuffer::new().is_empty());
    }

    #[test]
    fn default_output_is_empty() {
        assert!(OutputBuffer::default().is_empty());
    }

    #[test]
    fn pushes_u64() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push(0_u64);
        assert_eq!(output_string(out), "0");
    }

    #[test]
    fn pushes_i64() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push(42_i64);
        assert_eq!(output_string(out), "42");
    }

    #[test]
    fn pushes_smaller_signed_integers() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push(42_i32);
        out.push(b" ");
        out.push(99_isize);
        assert_eq!(output_string(out), "42 99");
    }

    #[test]
    fn pushes_f64() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push(1.5_f64);
        assert_eq!(output_string(out), "1.5");
    }

    #[test]
    fn pushes_larger_integers() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push(123_u128);
        out.push(b" ");
        out.push(456_usize);
        out.push(b" ");
        out.push(i128::MIN);
        assert_eq!(
            output_string(out),
            format!("123 456 {}", i128::MIN)
        );
    }

    #[test]
    fn pushes_bytes() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push_bytes(b"hello world");
        assert_eq!(output_string(out), "hello world");
    }

    #[test]
    fn pushes_bool() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push(true);
        assert_eq!(output_string(out), "true");
    }

    #[test]
    fn write_to_writes_into_any_io_writer() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push(b"hello");

        let mut dest = Vec::new();
        out.write_to(&mut dest).unwrap();

        assert_eq!(dest, b"hello");
    }

    #[test]
    fn implements_fmt_write() {
        let mut out = OutputBuffer::with_capacity(0);

        write!(&mut out, "{} + {} = {}", 2, 3, 5).unwrap();

        assert_eq!(output_string(out), "2 + 3 = 5");
    }

    #[test]
    fn pushes_str() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push_str("hello");
        assert_eq!(output_string(out), "hello");
    }

    #[test]
    fn pushes_newline_with_endl() {
        let mut out = OutputBuffer::with_capacity(0);
        out.push_str("hello");
        out.endl();
        out.push_str("world");
        assert_eq!(output_string(out), "hello\nworld");
    }

    #[test]
    fn writes_delimiter_between_tokens() {
        let mut out = OutputBuffer::with_capacity(0);
        out.set_delimiter(", ");
        out.push("a");
        out.push("b");
        out.push("c");
        assert_eq!(output_string(out), "a, b, c");
    }

    #[test]
    fn delimiter_resets_after_endl() {
        let mut out = OutputBuffer::with_capacity(0);
        out.set_delimiter(" ");
        out.push("last");
        out.endl();
        out.push("real");
        out.push("token");
        assert_eq!(output_string(out), "last\nreal token");
    }
}
