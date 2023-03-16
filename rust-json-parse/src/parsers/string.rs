use std::simd::u8x16;

use bumpalo::collections::{String, Vec};
use bumpalo::Bump;
use itertools::Itertools;

use crate::slice_iter::{CopyIter, SliceIter};
use crate::{JsonResult, ParseError};

pub fn read_string<'a, 'b>(
    json: &mut SliceIter<'a, u8>,
    alloc: &'b Bump,
) -> JsonResult<String<'b>> {
    let c = json.next();
    if Some(b'"') != c {
        return Err(ParseError::StringQuote {
            found: c.map(|v| v as char),
        });
    }
    let mut buf: Vec<u8> = Vec::new_in(alloc);
    // if 12 or under, probably just short key

    'parsing_block: {
        let conditions = [u8x16::splat(b'"'), u8x16::splat(b'\\')];
        loop {
            let chunk = json.take_while_ne_simd(conditions, |ch| ch != b'"' && ch != b'\\');
            buf.reserve_exact(chunk.len());
            let offset = buf.len();
            unsafe {
                buf.as_mut_ptr()
                    .add(offset)
                    .copy_from_nonoverlapping(chunk.as_ptr(), chunk.len());
                buf.set_len(offset + chunk.len());
            };

            for _ in 0..16 {
                match json.next() {
                    Some(b'"') => break 'parsing_block,
                    Some(b'\\') => escape(json, &mut buf)?,
                    Some(ch) => buf.push(ch),
                    None => return Err(ParseError::UnexpectedEndOfFile),
                }
            }
        }
    }
    buf.shrink_to_fit();
    // Ok(String::from_utf8(buf).map_err(|e| ParseError::InvalidUtf8 {
    //     string: std::string::String::from_utf8_lossy(e.as_bytes()).to_string(),
    // })?)
    if simdutf8::basic::from_utf8(&buf).is_ok() {
        Ok(unsafe { String::from_utf8_unchecked(buf) })
    } else {
        Err(ParseError::InvalidUtf8 {
            string: std::string::String::from_utf8_lossy(&buf).to_string(),
        })
    }
}

#[inline]
fn escape<'a, I: CopyIter<'a, Item = u8>>(json: &mut I, buf: &mut Vec<u8>) -> JsonResult<()> {
    let escape = json
        .next()
        .map_or(Err(ParseError::InvalidStringEscape), |v| Ok(v))?;
    match escape {
        b'"' | b'\\' | b'/' => buf.push(escape),
        b'b' => buf.push(0x08), // Backspace char
        b'f' => buf.push(0x0C), // Form-feed char
        b'n' => buf.push(0x0A), // Newline char
        b'r' => buf.push(0x0D), // Carriage return char
        b't' => buf.push(0x09), // Tab char
        b'u' => {
            let code = json
                .take_many::<4>()
                .map_or(Err(ParseError::InvalidStringEscape), |v| Ok(v))?;
            let mut codepoint = [
                u16::from_str_radix(
                    &std::str::from_utf8(&code).map_err(|_| ParseError::InvalidUtf8 {
                        string: std::string::String::from_utf8_lossy(&code).to_string(),
                    })?,
                    16,
                )
                .map_err(|_| ParseError::InvalidStringEscape)?,
                0,
            ];
            let mut utf16_len: usize = 1;
            if json.peek_many_ref(2) == Some(b"\\u") {
                json.ignore_many(2);
                let second_code = json
                    .take_many::<4>()
                    .map_or(Err(ParseError::InvalidStringEscape), |v| Ok(v))?;
                codepoint[1] = u16::from_str_radix(
                    &std::str::from_utf8(&second_code).map_err(|_| ParseError::InvalidUtf8 {
                        string: std::string::String::from_utf8_lossy(&second_code).to_string(),
                    })?,
                    16,
                )
                .map_err(|_| ParseError::InvalidStringEscape)?;
                utf16_len += 1;
            }
            std::char::decode_utf16(codepoint[0..utf16_len].into_iter().copied())
                .map(|r| match r {
                    Ok(char) => {
                        let mut dst: [u8; 4] = [0; 4];
                        buf.extend_from_slice(char.encode_utf8(&mut dst).as_bytes());
                        Ok(())
                    }
                    Err(byte) => {
                        if byte.unpaired_surrogate() <= 0x1F {
                            buf.push(byte.unpaired_surrogate() as u8);
                            Ok(())
                        } else {
                            return Err(ParseError::InvalidStringEscape);
                        }
                    }
                })
                .try_collect()?;
        }
        _ => return Err(ParseError::InvalidStringEscape),
    }
    Ok(())
}
