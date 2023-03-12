use bumpalo::collections::{String, Vec};
use bumpalo::Bump;
use itertools::Itertools;

use crate::slice_iter::CopyIter;
use crate::{JsonResult, ParseError};

pub fn read_string<'a, 'b, I: CopyIter<'a, Item = u8>>(
    json: &mut I,
    alloc: &'b Bump,
) -> JsonResult<String<'b>> {
    let c = json.next();
    if Some(b'"') != c {
        return Err(ParseError::StringQuote {
            found: c.map(|v| v as char),
        });
    }
    let mut buf: Vec<u8> = Vec::new_in(alloc);
    loop {
        let chunk = json.take_while_chunked::<8, _, _>(
            |chunk| {
                chunk[0] != b'"'
                    && chunk[0] != b'\\'
                    && chunk[1] != b'"'
                    && chunk[1] != b'\\'
                    && chunk[2] != b'"'
                    && chunk[2] != b'\\'
                    && chunk[3] != b'"'
                    && chunk[3] != b'\\'
                    && chunk[4] != b'"'
                    && chunk[4] != b'\\'
                    && chunk[5] != b'"'
                    && chunk[5] != b'\\'
                    && chunk[6] != b'"'
                    && chunk[6] != b'\\'
                    && chunk[7] != b'"'
                    && chunk[7] != b'\\'
            },
            |itm| itm != b'"' && itm != b'\\',
        );
        buf.reserve_exact(chunk.len());
        let offset = buf.len();
        unsafe { buf.set_len(offset + chunk.len()) };
        buf[offset..(offset + chunk.len())].copy_from_slice(chunk);

        match json.next() {
            Some(b'"') => break,
            Some(b'\\') => {
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
                                &std::str::from_utf8(&code).map_err(|_| {
                                    ParseError::InvalidUtf8 {
                                        string: std::string::String::from_utf8_lossy(&code)
                                            .to_string(),
                                    }
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
                                &std::str::from_utf8(&second_code).map_err(|_| {
                                    ParseError::InvalidUtf8 {
                                        string: std::string::String::from_utf8_lossy(&second_code)
                                            .to_string(),
                                    }
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
            }
            Some(ch) => buf.push(ch),
            None => return Err(ParseError::UnexpectedEndOfFile),
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
