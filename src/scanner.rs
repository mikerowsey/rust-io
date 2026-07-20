use std::io::{self, Read};

/// Fast buffered input scanner.
///
/// Reads the entire input into memory and provides efficient parsing of
/// whitespace-delimited tokens.
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

    /// Reads the next unsigned 32-bit integer.
    #[must_use]
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    /// Reads the next `usize`.
    #[must_use]
    #[inline]
    pub fn next_usize(&mut self) -> usize {
        self.next_u64() as usize
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

    /// Reads the next signed 32-bit integer.
    #[must_use]
    #[inline]
    pub fn next_i32(&mut self) -> i32 {
        self.next_i64() as i32
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

    /// Reads the next token as a `String`.
    #[must_use]
    #[inline]
    pub fn next_string(&mut self) -> String {
        String::from_utf8_lossy(self.next_bytes()).into_owned()
    }

    /// Reads the next ASCII character.
    ///
    /// This method does not decode UTF-8 and is intended for ASCII input.
    #[must_use]
    #[inline]
    pub fn next_char(&mut self) -> char {
        self.skip_whitespace();
        let c = self.buf[self.idx] as char;
        self.idx += 1;
        c
    }

    /// Reads the next line without the trailing `\n`.
    #[must_use]
    pub fn next_line(&mut self) -> &[u8] {
        let start = self.idx;
        while self.idx < self.buf.len() && self.buf[self.idx] != b'\n' {
            self.idx += 1;
        }
        let mut end = self.idx;
        if end > start && self.buf[end - 1] == b'\r' {
            end -= 1;
        }
        if self.idx < self.buf.len() {
            self.idx += 1;
        }
        &self.buf[start..end]
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
        let mut s = Scanner::from_reader(Cursor::new(b"123 abc")).unwrap();
        assert_eq!(s.next_u64(), 123);
        assert_eq!(s.next_string(), "abc");
        assert!(s.is_empty());
    }

    #[test]
    fn parses_signed_and_unsigned() {
        let mut s = Scanner::from_bytes(b"-42 99");
        assert_eq!(s.next_i64(), -42);
        assert_eq!(s.next_u64(), 99);
    }

    #[test]
    fn parses_bytes_and_string() {
        let mut s = Scanner::from_bytes(b"hello world");
        assert_eq!(s.next_bytes(), b"hello");
        assert_eq!(s.next_string(), "world");
    }

    #[test]
    fn parses_ascii_char() {
        let mut s = Scanner::from_bytes(b" A");
        assert_eq!(s.next_char(), 'A');
    }

    #[test]
    fn parses_lines_and_crlf() {
        let mut s = Scanner::from_bytes(b"one\r\ntwo\n");
        assert_eq!(s.next_line(), b"one");
        assert_eq!(s.next_line(), b"two");
        assert!(s.is_empty());
    }

    #[test]
    fn default_from_bytes_empty() {
        let s = Scanner::from_bytes(b"");
        assert!(s.is_empty());
    }
}
