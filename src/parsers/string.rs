use bumpalo::collections::{String, Vec};
use bumpalo::Bump;
use itertools::{Itertools, MultiPeek};

use crate::utils::take_static;
use crate::{JsonResult, ParseError};

pub fn read_string<'a, 'b, I: Iterator<Item = u8>>(
    json: &'a mut MultiPeek<I>,
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
        buf.extend(json.peeking_take_while(|c| *c != b'"' && *c != b'\\'));
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
                        let code = take_static::<4, _, _>(json)
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
                        if Some(&b'\\') == json.peek() && Some(&b'u') == json.peek() {
                            _ = json.next().unwrap();
                            _ = json.next().unwrap();
                            let second_code = take_static::<4, _, _>(json)
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
            Some(_) => unreachable!(),
            None => return Err(ParseError::UnexpectedEndOfFile),
        }
    }
    Ok(String::from_utf8(buf).map_err(|e| ParseError::InvalidUtf8 {
        string: std::string::String::from_utf8_lossy(e.as_bytes()).to_string(),
    })?)
}
