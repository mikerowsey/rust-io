use std::io::{self, Read};

/// Fast scanner for whitespace-delimited input.
///
/// The scanner reads the entire input into memory and efficiently parses
/// whitespace-delimited tokens into typed values.
///
/// # Example
///
/// ```
/// use rust_io::Scanner;
///
/// let mut scan = Scanner::from_bytes(b"42 hello");
/// let n: u64 = scan.next();
/// let s: String = scan.next();
///
/// assert_eq!(n, 42);
/// assert_eq!(s, "hello");
/// ```
///
/// Designed for competitive programming and other token-based input.
#[must_use]
pub struct Scanner {
    buf: Vec<u8>,
    idx: usize,
}

/// A value that can be read from a [`Scanner`].
pub trait Readable: Sized {
    /// Reads `Self` from `scan`.
    fn read(scan: &mut Scanner) -> Self;
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

    /// Reads the next whitespace-delimited token as a typed value.
    #[must_use]
    #[inline]
    pub fn next<T: Readable>(&mut self) -> T {
        T::read(self)
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

    #[inline]
    fn parse_f64(token: &[u8]) -> f64 {
        std::str::from_utf8(token)
            .expect("invalid UTF-8")
            .parse()
            .expect("invalid floating-point value")
    }
}

macro_rules! impl_readable_unsigned {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Readable for $ty {
                #[inline]
                fn read(scan: &mut Scanner) -> Self {
                    Scanner::parse_unsigned::<$ty>(scan.next_bytes())
                }
            }
        )*
    };
}

macro_rules! impl_readable_signed {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Readable for $ty {
                #[inline]
                fn read(scan: &mut Scanner) -> Self {
                    Scanner::parse_signed::<$ty>(scan.next_bytes())
                }
            }
        )*
    };
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
impl_readable_unsigned!(u32, u64, u128, usize);
impl_readable_signed!(i32, i64, i128, isize);

impl Readable for f64 {
    #[inline]
    fn read(scan: &mut Scanner) -> Self {
        Scanner::parse_f64(scan.next_bytes())
    }
}

impl Readable for bool {
    #[inline]
    fn read(scan: &mut Scanner) -> Self {
        match scan.next_bytes() {
            b"true" | b"1" => true,
            b"false" | b"0" => false,
            _ => panic!("invalid boolean token"),
        }
    }
}

impl Readable for String {
    #[inline]
    fn read(scan: &mut Scanner) -> Self {
        String::from_utf8(scan.next_bytes().to_vec()).expect("invalid UTF-8")
    }
}

impl Readable for Vec<u8> {
    #[inline]
    fn read(scan: &mut Scanner) -> Self {
        scan.next_bytes().to_vec()
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

        let n: u64 = input.next();
        let token: Vec<u8> = input.next();

        assert_eq!(n, 123);
        assert_eq!(token, b"abc");
        assert!(input.is_empty());
    }

    #[test]
    fn parses_unsigned_integers() {
        let mut input = Scanner::from_bytes(b"42 99");

        let first: u64 = input.next();
        let second: u64 = input.next();

        assert_eq!(first, 42);
        assert_eq!(second, 99);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_smaller_unsigned_integers() {
        let mut input = Scanner::from_bytes(b"42 99 123");

        let first: u32 = input.next();
        let second: usize = input.next();
        let third: u128 = input.next();

        assert_eq!(first, 42);
        assert_eq!(second, 99);
        assert_eq!(third, 123);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_signed_integers() {
        let mut input = Scanner::from_bytes(b"-42 99 -7");

        let first: i64 = input.next();
        let second: i64 = input.next();
        let third: i64 = input.next();

        assert_eq!(first, -42);
        assert_eq!(second, 99);
        assert_eq!(third, -7);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_smaller_signed_integers() {
        let mut input = Scanner::from_bytes(b"-42 99 -7");

        let first: i32 = input.next();
        let second: isize = input.next();
        let third: i128 = input.next();

        assert_eq!(first, -42);
        assert_eq!(second, 99);
        assert_eq!(third, -7);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_signed_integer_boundaries() {
        let mut input = Scanner::from_bytes(b"9223372036854775807 -9223372036854775808");

        assert_eq!(input.next::<i64>(), i64::MAX);
        assert_eq!(input.next::<i64>(), i64::MIN);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_unsigned_integer_boundary() {
        let mut input = Scanner::from_bytes(b"18446744073709551615");

        assert_eq!(input.next::<u64>(), u64::MAX);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_smaller_unsigned_integer_boundaries() {
        let u32_max = u32::MAX.to_string();
        let usize_max = usize::MAX.to_string();
        let u128_max = u128::MAX.to_string();
        let input = format!("{u32_max} {usize_max} {u128_max}");
        let mut scanner = Scanner::from_bytes(input.as_bytes());

        assert_eq!(scanner.next::<u32>(), u32::MAX);
        assert_eq!(scanner.next::<usize>(), usize::MAX);
        assert_eq!(scanner.next::<u128>(), u128::MAX);
        assert!(scanner.is_empty());
    }

    #[test]
    #[should_panic]
    fn rejects_unsigned_integer_overflow() {
        let mut input = Scanner::from_bytes(b"18446744073709551616");

        let _ = input.next::<u64>();
    }

    #[test]
    #[should_panic]
    fn rejects_signed_integer_overflow() {
        let mut input = Scanner::from_bytes(b"-9223372036854775809");

        let _ = input.next::<i64>();
    }

    #[test]
    #[should_panic]
    fn rejects_invalid_unsigned_token() {
        let mut input = Scanner::from_bytes(b"abc");

        let _ = input.next::<u64>();
    }

    #[test]
    #[should_panic]
    fn rejects_invalid_signed_token() {
        let mut input = Scanner::from_bytes(b"+");

        let _ = input.next::<i64>();
    }

    #[test]
    fn parses_floats() {
        let mut input = Scanner::from_bytes(b"2.14 -2.5 1e3");

        assert_eq!(input.next::<f64>(), 2.14);
        assert_eq!(input.next::<f64>(), -2.5);
        assert_eq!(input.next::<f64>(), 1000.0);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_strings() {
        let mut input = Scanner::from_bytes(b"hello world");

        let first: String = input.next();
        let second: String = input.next();

        assert_eq!(first, "hello");
        assert_eq!(second, "world");
        assert!(input.is_empty());
    }

    #[test]
    fn parses_bools() {
        let mut input = Scanner::from_bytes(b"true false 1 0");

        assert!(input.next::<bool>());
        assert!(!input.next::<bool>());
        assert!(input.next::<bool>());
        assert!(!input.next::<bool>());
        assert!(input.is_empty());
    }

    #[test]
    fn parses_bytes() {
        let mut input = Scanner::from_bytes(b"hello world");

        assert_eq!(input.next::<Vec<u8>>(), b"hello");
        assert_eq!(input.next::<Vec<u8>>(), b"world");
        assert!(input.is_empty());
    }

    #[test]
    fn empty_input() {
        let input = Scanner::from_bytes(b"");

        assert!(input.is_empty());
    }
}
