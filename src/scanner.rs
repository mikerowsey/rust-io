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
        let stdin = io::stdin();
        Self::from_reader(stdin.lock()).expect("failed to read from stdin")
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
        Self::parse_unsigned::<u64>(self.next_bytes())
    }

    /// Reads the next unsigned 32-bit integer.
    #[must_use]
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        Self::parse_unsigned::<u32>(self.next_bytes())
    }

    /// Reads the next unsigned 128-bit integer.
    #[must_use]
    #[inline]
    pub fn next_u128(&mut self) -> u128 {
        Self::parse_unsigned::<u128>(self.next_bytes())
    }

    /// Reads the next pointer-sized unsigned integer.
    #[must_use]
    #[inline]
    pub fn next_usize(&mut self) -> usize {
        Self::parse_unsigned::<usize>(self.next_bytes())
    }

    /// Reads the next signed 64-bit integer.
    #[must_use]
    #[inline]
    pub fn next_i64(&mut self) -> i64 {
        Self::parse_signed::<i64>(self.next_bytes())
    }

    /// Reads the next signed 32-bit integer.
    #[must_use]
    #[inline]
    pub fn next_i32(&mut self) -> i32 {
        Self::parse_signed::<i32>(self.next_bytes())
    }

    /// Reads the next signed 128-bit integer.
    #[must_use]
    #[inline]
    pub fn next_i128(&mut self) -> i128 {
        Self::parse_signed::<i128>(self.next_bytes())
    }

    /// Reads the next pointer-sized signed integer.
    #[must_use]
    #[inline]
    pub fn next_isize(&mut self) -> isize {
        Self::parse_signed::<isize>(self.next_bytes())
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

    /// Reads the next boolean value.
    ///
    /// Accepts `true` / `false`, and also `1` / `0`.
    #[must_use]
    #[inline]
    pub fn next_bool(&mut self) -> bool {
        match self.next_bytes() {
            b"true" | b"1" => true,
            b"false" | b"0" => false,
            _ => panic!("invalid boolean token"),
        }
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
    fn parse_unsigned<T>(token: &[u8]) -> T
    where
        T: FromU128,
    {
        if token.is_empty() {
            panic!("expected unsigned integer token");
        }

        let mut value = 0u128;

        for &byte in token {
            if !byte.is_ascii_digit() {
                panic!("invalid unsigned integer token");
            }

            let digit = (byte - b'0') as u128;
            value = value
                .checked_mul(10)
                .and_then(|current| current.checked_add(digit))
                .expect("unsigned integer overflow");
        }

        T::from_u128(value)
    }

    #[inline]
    fn parse_signed<T>(token: &[u8]) -> T
    where
        T: FromI128,
    {
        if token.is_empty() {
            panic!("expected signed integer token");
        }

        let (negative, digits) = match token.split_first() {
            Some((b'-', rest)) => (true, rest),
            Some((b'+', rest)) => (false, rest),
            _ => (false, token),
        };

        let magnitude = Self::parse_u128_digits(digits);
        T::from_parts(negative, magnitude)
    }

    #[inline]
    fn parse_u128_digits(token: &[u8]) -> u128 {
        if token.is_empty() {
            panic!("expected unsigned integer token");
        }

        let mut value = 0u128;

        for &byte in token {
            if !byte.is_ascii_digit() {
                panic!("invalid unsigned integer token");
            }

            let digit = (byte - b'0') as u128;
            value = value
                .checked_mul(10)
                .and_then(|current| current.checked_add(digit))
                .expect("unsigned integer overflow");
        }

        value
    }
}

trait FromU128: Sized {
    fn from_u128(value: u128) -> Self;
}

trait FromI128: Sized {
    fn from_parts(negative: bool, magnitude: u128) -> Self;
}

macro_rules! impl_from_u128 {
    ($($ty:ty),* $(,)?) => {
        $(
            impl FromU128 for $ty {
                #[inline]
                fn from_u128(value: u128) -> Self {
                    if value <= <$ty>::MAX as u128 {
                        value as $ty
                    } else {
                        panic!("unsigned integer overflow");
                    }
                }
            }
        )*
    };
}

macro_rules! impl_from_i128 {
    ($($ty:ty),* $(,)?) => {
        $(
            impl FromI128 for $ty {
                #[inline]
                fn from_parts(negative: bool, magnitude: u128) -> Self {
                    let max = <$ty>::MAX as u128;
                    let min_magnitude = max + 1;

                    if negative {
                        if magnitude == min_magnitude {
                            <$ty>::MIN
                        } else if magnitude <= max {
                            -(magnitude as $ty)
                        } else {
                            panic!("signed integer overflow");
                        }
                    } else if magnitude <= max {
                        magnitude as $ty
                    } else {
                        panic!("signed integer overflow");
                    }
                }
            }
        )*
    };
}

impl_from_u128!(u32, u64, u128, usize);
impl_from_i128!(i32, i64, i128, isize);

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
    fn parses_smaller_unsigned_integers() {
        let mut input = Scanner::from_bytes(b"42 99 123");

        assert_eq!(input.next_u32(), 42);
        assert_eq!(input.next_usize(), 99);
        assert_eq!(input.next_u128(), 123);
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
    fn parses_smaller_signed_integers() {
        let mut input = Scanner::from_bytes(b"-42 99 -7");

        assert_eq!(input.next_i32(), -42);
        assert_eq!(input.next_isize(), 99);
        assert_eq!(input.next_i128(), -7);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_signed_integer_boundaries() {
        let mut input = Scanner::from_bytes(b"9223372036854775807 -9223372036854775808");

        assert_eq!(input.next_i64(), i64::MAX);
        assert_eq!(input.next_i64(), i64::MIN);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_unsigned_integer_boundary() {
        let mut input = Scanner::from_bytes(b"18446744073709551615");

        assert_eq!(input.next_u64(), u64::MAX);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_smaller_unsigned_integer_boundaries() {
        let u32_max = u32::MAX.to_string();
        let usize_max = usize::MAX.to_string();
        let u128_max = u128::MAX.to_string();
        let input = format!("{u32_max} {usize_max} {u128_max}");
        let mut scanner = Scanner::from_bytes(input.as_bytes());

        assert_eq!(scanner.next_u32(), u32::MAX);
        assert_eq!(scanner.next_usize(), usize::MAX);
        assert_eq!(scanner.next_u128(), u128::MAX);
        assert!(scanner.is_empty());
    }

    #[test]
    #[should_panic]
    fn rejects_unsigned_integer_overflow() {
        let mut input = Scanner::from_bytes(b"18446744073709551616");

        let _ = input.next_u64();
    }

    #[test]
    #[should_panic]
    fn rejects_signed_integer_overflow() {
        let mut input = Scanner::from_bytes(b"-9223372036854775809");

        let _ = input.next_i64();
    }

    #[test]
    #[should_panic]
    fn rejects_invalid_unsigned_token() {
        let mut input = Scanner::from_bytes(b"abc");

        let _ = input.next_u64();
    }

    #[test]
    #[should_panic]
    fn rejects_invalid_signed_token() {
        let mut input = Scanner::from_bytes(b"+");

        let _ = input.next_i64();
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
    fn parses_bools() {
        let mut input = Scanner::from_bytes(b"true false 1 0");

        assert!(input.next_bool());
        assert!(!input.next_bool());
        assert!(input.next_bool());
        assert!(!input.next_bool());
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
