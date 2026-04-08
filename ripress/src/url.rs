use std::{borrow::Cow, string::FromUtf8Error};

pub fn decode(data: &str) -> Result<Cow<'_, str>, FromUtf8Error> {
    match decode_binary(data.as_bytes()) {
        Cow::Borrowed(_) => Ok(Cow::Borrowed(data)),
        Cow::Owned(s) => Ok(Cow::Owned(String::from_utf8(s)?)),
    }
}

struct NeverRealloc<'a, T>(pub &'a mut Vec<T>);

impl<T> NeverRealloc<'_, T> {
    #[inline]
    pub fn push(&mut self, val: T) {
        // these branches only exist to remove redundant reallocation code
        // (the capacity is always sufficient)
        if self.0.len() != self.0.capacity() {
            self.0.push(val);
        }
    }
    #[inline]
    pub fn extend_from_slice(&mut self, val: &[T])
    where
        T: Clone,
    {
        if self.0.capacity() - self.0.len() >= val.len() {
            self.0.extend_from_slice(val);
        }
    }
}

pub(crate) fn from_hex_digit(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    }
}

/// Decode percent-encoded string as binary data, in any encoding.
///
/// Unencoded `+` is preserved literally, and _not_ changed to a space.
pub fn decode_binary(data: &[u8]) -> Cow<'_, [u8]> {
    let offset = data.iter().take_while(|&&c| c != b'%').count();
    if offset >= data.len() {
        return Cow::Borrowed(data);
    }

    let mut decoded: Vec<u8> = Vec::with_capacity(data.len());
    let mut out = NeverRealloc(&mut decoded);

    let (ascii, mut data) = data.split_at(offset);
    out.extend_from_slice(ascii);

    loop {
        let mut parts = data.splitn(2, |&c| c == b'%');
        // first the decoded non-% part
        let non_escaped_part = parts.next().unwrap();
        let rest = parts.next();
        if rest.is_none() && out.0.is_empty() {
            // if empty there were no '%' in the string
            return data.into();
        }
        out.extend_from_slice(non_escaped_part);

        // then decode one %xx
        match rest {
            Some(rest) => match rest.get(0..2) {
                Some(&[first, second]) => match from_hex_digit(first) {
                    Some(first_val) => match from_hex_digit(second) {
                        Some(second_val) => {
                            out.push((first_val << 4) | second_val);
                            data = &rest[2..];
                        }
                        None => {
                            out.extend_from_slice(&[b'%', first]);
                            data = &rest[1..];
                        }
                    },
                    None => {
                        out.push(b'%');
                        data = rest;
                    }
                },
                _ => {
                    // too short
                    out.push(b'%');
                    out.extend_from_slice(rest);
                    break;
                }
            },
            None => break,
        }
    }
    Cow::Owned(decoded)
}

#[inline(always)]
pub(crate) fn encode(data: &str) -> Cow<'_, str> {
    encode_binary(data.as_bytes())
}

/// Percent-encodes every byte except alphanumerics and `-`, `_`, `.`, `~`.
#[inline]
pub(crate) fn encode_binary(data: &[u8]) -> Cow<'_, str> {
    let mut escaped = String::with_capacity(data.len() | 15);
    let unmodified = append_string(data, &mut escaped, true);
    if unmodified {
        return Cow::Borrowed(unsafe {
            // encode_into has checked it's ASCII
            str::from_utf8_unchecked(data)
        });
    }
    Cow::Owned(escaped)
}

fn append_string(data: &[u8], escaped: &mut String, may_skip: bool) -> bool {
    encode_into(data, may_skip, |s| {
        escaped.push_str(s);
        Ok::<_, std::convert::Infallible>(())
    })
    .unwrap()
}

fn encode_into<E>(
    mut data: &[u8],
    may_skip_write: bool,
    mut push_str: impl FnMut(&str) -> Result<(), E>,
) -> Result<bool, E> {
    let mut pushed = false;
    loop {
        // Fast path to skip over safe chars at the beginning of the remaining string
        let ascii_len = data.iter()
            .take_while(|&&c| matches!(c, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' |  b'-' | b'.' | b'_' | b'~')).count();

        let (safe, rest) = if ascii_len >= data.len() {
            if !pushed && may_skip_write {
                return Ok(true);
            }
            (data, &[][..]) // redundant to optimize out a panic in split_at
        } else {
            data.split_at(ascii_len)
        };
        pushed = true;
        if !safe.is_empty() {
            push_str(unsafe { str::from_utf8_unchecked(safe) })?;
        }
        if rest.is_empty() {
            break;
        }

        match rest.split_first() {
            Some((byte, rest)) => {
                let enc = &[b'%', to_hex_digit(byte >> 4), to_hex_digit(byte & 15)];
                push_str(unsafe { str::from_utf8_unchecked(enc) })?;
                data = rest;
            }
            None => break,
        };
    }
    Ok(false)
}

#[inline]
fn to_hex_digit(digit: u8) -> u8 {
    debug_assert!(digit < 16, "to_hex_digit only accepts 0-15");
    match digit {
        0..=9 => b'0' + digit,
        10..=15 => b'A' - 10 + digit,
        _ => unreachable!(),
    }
}
