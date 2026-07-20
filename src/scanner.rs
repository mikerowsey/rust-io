use std::io::{self, Read};

#[must_use]
pub struct Scanner {
    buf: Vec<u8>,
    idx: usize,
}

impl Scanner {
    pub fn new() -> Self {
        Self::from_reader(io::stdin())
    }

    pub fn from_reader<R: Read>(mut reader: R) -> Self {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).unwrap();

        Self { buf, idx: 0 }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            buf: bytes.to_vec(),
            idx: 0,
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.skip_whitespace();
        self.parse_u64()
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    #[inline]
    pub fn next_usize(&mut self) -> usize {
        self.next_u64() as usize
    }

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

    #[inline]
    pub fn next_i32(&mut self) -> i32 {
        self.next_i64() as i32
    }

    #[inline]
    pub fn next_bytes(&mut self) -> &[u8] {
        self.skip_whitespace();

        let start = self.idx;

        while self.idx < self.buf.len() && !self.buf[self.idx].is_ascii_whitespace() {
            self.idx += 1;
        }

        &self.buf[start..self.idx]
    }

    #[inline]
    pub fn next_string(&mut self) -> String {
        String::from_utf8_lossy(self.next_bytes()).into_owned()
    }

    #[inline]
    pub fn next_char(&mut self) -> char {
        self.skip_whitespace();
        let c = self.buf[self.idx] as char;
        self.idx += 1;
        c
    }

    pub fn next_line(&mut self) -> &[u8] {
        let start = self.idx;

        while self.idx < self.buf.len() && self.buf[self.idx] != b'\n' {
            self.idx += 1;
        }

        let line = &self.buf[start..self.idx];

        // Skip the newline if present
        if self.idx < self.buf.len() && self.buf[self.idx] == b'\n' {
            self.idx += 1;
        }

        line
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
            value = value * 10 + (self.buf[self.idx] - b'0') as u64;
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
