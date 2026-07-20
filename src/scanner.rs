use std::io::{self, Read};

/// Fast scanner for whitespace-delimited input.
///
/// The scanner reads the entire input into memory and efficiently parses
/// primitive values and byte slices.
///
/// Designed for competitive programming and other token-based input.
#[must_use]
pub struct Scanner {
    buf: Vec<u8>,
    idx: usize,
}

impl Scanner {
    /// Creates a scanner that reads from standard input.
    pub fn new() -> Self {
        Self::from_reader(io::stdin()).expect("failed to read from stdin")
    }

    /// Creates a scanner by reading all bytes from the given reader.
    ///
    /// # Errors
    ///
    /// Returns any I/O error encountered while reading.
    pub fn from_reader<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut buf = Vec::with_capacity(1 << 20);
        reader.read_to_end(&mut buf)?;
        Ok(Self { buf, idx: 0 })
    }

    /// Creates a scanner from an in-memory byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            buf: bytes.to_vec(),
            idx: 0,
        }
    }

    /// Reads the next unsigned 64-bit integer.
    #[must_use]
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.skip_whitespace();
        self.parse_u64()
    }

    /// Reads the next signed 64-bit integer.
    #[must_use]
    #[inline]
    pub fn next_i64(&mut self) -> i64 {
        self.skip_whitespace();

        let negative = self.idx < self.buf.len() && self.buf[self.idx] == b'-';
        if negative {
            self.idx += 1;
        }

        let value = self.parse_u64() as i64;

        if negative { -value } else { value }
    }

    /// Reads the next 64-bit floating-point value.
    ///
    /// # Panics
    ///
    /// Panics if the next token is not a valid `f64`.
    #[must_use]
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        std::str::from_utf8(self.next_bytes())
            .expect("invalid UTF-8")
            .parse()
            .expect("invalid floating-point value")
    }

    /// Reads the next whitespace-delimited token as UTF-8.
    #[must_use]
    #[inline]
    pub fn next_str(&mut self) -> &str {
        std::str::from_utf8(self.next_bytes()).expect("invalid UTF-8")
    }

    /// Reads the next whitespace-delimited token as raw bytes.
    #[must_use]
    #[inline]
    pub fn next_bytes(&mut self) -> &[u8] {
        self.skip_whitespace();

        let start = self.idx;

        while self.idx < self.buf.len() && !self.buf[self.idx].is_ascii_whitespace() {
            self.idx += 1;
        }

        &self.buf[start..self.idx]
    }

    /// Returns `true` if no unread input remains.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.idx >= self.buf.len()
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        while self.idx < self.buf.len() && self.buf[self.idx].is_ascii_whitespace() {
            self.idx += 1;
        }
    }

    #[inline]
    fn parse_u64(&mut self) -> u64 {
        let mut value = 0u64;

        while self.idx < self.buf.len() && self.buf[self.idx].is_ascii_digit() {
            value *= 10;
            value += (self.buf[self.idx] - b'0') as u64;
            self.idx += 1;
        }

        value
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn from_reader_reads_input() {
        let mut input = Scanner::from_reader(Cursor::new(b"123 abc")).unwrap();

        assert_eq!(input.next_u64(), 123);
        assert_eq!(input.next_bytes(), b"abc");
        assert!(input.is_empty());
    }

    #[test]
    fn parses_unsigned_integers() {
        let mut input = Scanner::from_bytes(b"42 99");

        assert_eq!(input.next_u64(), 42);
        assert_eq!(input.next_u64(), 99);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_signed_integers() {
        let mut input = Scanner::from_bytes(b"-42 99 -7");

        assert_eq!(input.next_i64(), -42);
        assert_eq!(input.next_i64(), 99);
        assert_eq!(input.next_i64(), -7);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_floats() {
        let mut input = Scanner::from_bytes(b"2.14 -2.5 1e3");

        assert_eq!(input.next_f64(), 2.14);
        assert_eq!(input.next_f64(), -2.5);
        assert_eq!(input.next_f64(), 1000.0);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_strings() {
        let mut input = Scanner::from_bytes(b"hello world");

        assert_eq!(input.next_str(), "hello");
        assert_eq!(input.next_str(), "world");
        assert!(input.is_empty());
    }

    #[test]
    fn parses_bytes() {
        let mut input = Scanner::from_bytes(b"hello world");

        assert_eq!(input.next_bytes(), b"hello");
        assert_eq!(input.next_bytes(), b"world");
        assert!(input.is_empty());
    }

    #[test]
    fn empty_input() {
        let input = Scanner::from_bytes(b"");

        assert!(input.is_empty());
    }
}
