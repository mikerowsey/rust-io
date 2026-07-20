use std::{
    fmt,
    io::{self, Write},
};

/// Buffered output that accumulates bytes in memory.
///
/// Values are written through the [`Writable`] trait. The completed buffer can
/// be extracted with [`Output::into_bytes`] or written to any
/// [`std::io::Write`] implementation with [`Output::write_to`].
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

    /// Writes a value into the buffer.
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

    /// Writes iterator items separated by spaces.
    #[inline]
    pub fn write_iter<I, T>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
        T: Writable,
    {
        self.write_iter_delimited(iter, b' ');
    }

    /// Writes iterator items separated by `delimiter`.
    #[inline]
    pub fn write_iter_delimited<I, T>(&mut self, iter: I, delimiter: u8)
    where
        I: IntoIterator<Item = T>,
        T: Writable,
    {
        let mut iter = iter.into_iter();

        if let Some(first) = iter.next() {
            self.write(first);

            for value in iter {
                self.write_byte(delimiter);
                self.write(value);
            }
        }
    }

    /// Writes iterator items separated by spaces, followed by a newline.
    #[inline]
    pub fn writeln_iter<I, T>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
        T: Writable,
    {
        self.write_iter(iter);
        self.endl();
    }

    /// Writes slice items separated by spaces.
    #[inline]
    pub fn write_slice<T: Writable + Copy>(&mut self, slice: &[T]) {
        self.write_iter(slice.iter().copied());
    }

    /// Appends a newline byte.
    #[inline]
    pub fn endl(&mut self) {
        self.write_byte(b'\n');
    }

    /// Writes the entire buffered output to `writer`.
    ///
    /// # Errors
    ///
    /// Returns any error produced by the destination writer.
    pub fn write_to<W: Write>(self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.buf)
    }

    /// Consumes the output and returns its underlying byte buffer.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    /// Returns the number of bytes currently buffered.
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns the current capacity of the internal byte buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Returns `true` when the buffer contains no bytes.
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

impl Writable for bool {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_bytes(if self { b"true" } else { b"false" });
    }
}

macro_rules! impl_writable_unsigned {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl Writable for $ty {
                #[inline]
                fn write_to(self, out: &mut Output) {
                    out.write_u64(self as u64);
                }
            }
        )+
    };
}

impl_writable_unsigned!(u8, u16, u32, u64, usize);

macro_rules! impl_writable_signed {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl Writable for $ty {
                #[inline]
                fn write_to(self, out: &mut Output) {
                    out.write_i64(self as i64);
                }
            }
        )+
    };
}

impl_writable_signed!(i8, i16, i32, i64, isize);

impl Writable for f32 {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_f64(self as f64);
    }
}

impl Writable for f64 {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_f64(self);
    }
}

impl Writable for &str {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_bytes(self.as_bytes());
    }
}

impl Writable for String {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_bytes(self.as_bytes());
    }
}

impl Writable for &String {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_bytes(self.as_bytes());
    }
}

impl Writable for &[u8] {
    #[inline]
    fn write_to(self, out: &mut Output) {
        out.write_bytes(self);
    }
}

impl Writable for char {
    #[inline]
    fn write_to(self, out: &mut Output) {
        let mut buf = [0; 4];
        out.write_bytes(self.encode_utf8(&mut buf).as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output_string(out: Output) -> String {
        String::from_utf8(out.into_bytes()).unwrap()
    }

    #[test]
    fn new_output_is_empty() {
        let out = Output::new();

        assert!(out.is_empty());
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn default_output_is_empty() {
        let out = Output::default();

        assert!(out.is_empty());
    }

    #[test]
    fn with_capacity_reserves_requested_capacity() {
        let out = Output::with_capacity(32);

        assert!(out.capacity() >= 32);
    }

    #[test]
    fn writes_unsigned_integers() {
        let mut out = Output::with_capacity(0);

        out.write(0_u8);
        out.write(' ');
        out.write(u16::MAX);
        out.write(' ');
        out.write(u32::MAX);
        out.write(' ');
        out.write(u64::MAX);
        out.write(' ');
        out.write(usize::MAX);

        assert_eq!(
            output_string(out),
            format!("0 {} {} {} {}", u16::MAX, u32::MAX, u64::MAX, usize::MAX)
        );
    }

    #[test]
    fn writes_signed_integers() {
        let mut out = Output::with_capacity(0);

        out.write(i8::MIN);
        out.write(' ');
        out.write(i16::MIN);
        out.write(' ');
        out.write(i32::MIN);
        out.write(' ');
        out.write(i64::MIN);
        out.write(' ');
        out.write(isize::MIN);

        assert_eq!(
            output_string(out),
            format!(
                "{} {} {} {} {}",
                i8::MIN,
                i16::MIN,
                i32::MIN,
                i64::MIN,
                isize::MIN
            )
        );
    }

    #[test]
    fn writes_floats() {
        let mut out = Output::with_capacity(0);

        out.write(1.5_f32);
        out.write(' ');
        out.write(-2.25_f64);

        assert_eq!(output_string(out), "1.5 -2.25");
    }

    #[test]
    fn writes_text_and_characters() {
        let mut out = Output::with_capacity(0);
        let owned = String::from("world");

        out.write("hello");
        out.write(' ');
        out.write(&owned);
        out.write(' ');
        out.write('λ');

        assert_eq!(output_string(out), "hello world λ");
    }

    #[test]
    fn writes_owned_string() {
        let mut out = Output::with_capacity(0);

        out.write(String::from("owned"));

        assert_eq!(output_string(out), "owned");
    }

    #[test]
    fn writes_boolean_values() {
        let mut out = Output::with_capacity(0);

        out.write(true);
        out.write(' ');
        out.write(false);

        assert_eq!(output_string(out), "true false");
    }

    #[test]
    fn writeln_appends_newline() {
        let mut out = Output::with_capacity(0);

        out.writeln("hello");

        assert_eq!(output_string(out), "hello\n");
    }

    #[test]
    fn endl_appends_newline() {
        let mut out = Output::with_capacity(0);

        out.endl();

        assert_eq!(out.into_bytes(), b"\n");
    }

    #[test]
    fn write_iter_uses_spaces() {
        let mut out = Output::with_capacity(0);

        out.write_iter([1, 2, 3]);

        assert_eq!(output_string(out), "1 2 3");
    }

    #[test]
    fn write_iter_delimited_uses_requested_delimiter() {
        let mut out = Output::with_capacity(0);

        out.write_iter_delimited([1, 2, 3], b',');

        assert_eq!(output_string(out), "1,2,3");
    }

    #[test]
    fn empty_iterator_writes_nothing() {
        let mut out = Output::with_capacity(0);

        out.write_iter(std::iter::empty::<i32>());

        assert!(out.is_empty());
    }

    #[test]
    fn writeln_iter_appends_newline() {
        let mut out = Output::with_capacity(0);

        out.writeln_iter([1, 2, 3]);

        assert_eq!(output_string(out), "1 2 3\n");
    }

    #[test]
    fn write_slice_writes_copied_values() {
        let mut out = Output::with_capacity(0);
        let values = [4, 5, 6];

        out.write_slice(&values);

        assert_eq!(output_string(out), "4 5 6");
    }

    #[test]
    fn writes_raw_bytes() {
        let mut out = Output::with_capacity(0);

        out.write(&b"abc"[..]);

        assert_eq!(out.into_bytes(), b"abc");
    }

    #[test]
    fn writes_references_to_primitives() {
        let mut out = Output::with_capacity(0);
        let value = 42_u64;
        let flag = true;
        let ch = 'x';

        out.write(value);
        out.write(' ');
        out.write(flag);
        out.write(' ');
        out.write(ch);

        assert_eq!(output_string(out), "42 true x");
    }

    #[test]
    fn len_tracks_buffered_bytes() {
        let mut out = Output::with_capacity(0);

        out.write("hello");

        assert_eq!(out.len(), 5);
        assert!(!out.is_empty());
    }

    #[test]
    fn write_to_writes_into_any_io_writer() {
        let mut out = Output::with_capacity(0);
        let mut destination = Vec::new();

        out.write("hello");
        out.write_to(&mut destination).unwrap();

        assert_eq!(destination, b"hello");
    }

    #[test]
    fn implements_fmt_write() {
        use std::fmt::Write as _;

        let mut out = Output::with_capacity(0);

        write!(&mut out, "{} + {} = {}", 2, 3, 5).unwrap();

        assert_eq!(output_string(out), "2 + 3 = 5");
    }
}
