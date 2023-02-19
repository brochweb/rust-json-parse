use anyhow::{anyhow, bail, Result};
use itertools::{Itertools, MultiPeek};

use crate::utils::take_static;

pub fn read_string<'a, I: Iterator<Item = u8>>(json: &'a mut MultiPeek<I>) -> Result<String> {
    if Some(b'"') != json.next() {
        bail!("String must start with \"");
    }
    let mut buf: Vec<u8> = Vec::new();
    loop {
        buf.extend(json.peeking_take_while(|c| *c != b'"' && *c != b'\\'));
        match json.next() {
            Some(b'"') => break,
            Some(b'\\') => {
                let escape = json
                    .next()
                    .map_or(Err(anyhow!("Invalid string escape")), |v| Ok(v))?;
                match escape {
                    b'"' | b'\\' | b'/' => buf.push(escape),
                    b'b' => buf.push(0x08), // Backspace char
                    b'f' => buf.push(0x0C), // Form-feed char
                    b'n' => buf.push(0x0A), // Newline char
                    b'r' => buf.push(0x0D), // Carriage return char
                    b't' => buf.push(0x09), // Tab char
                    b'u' => {
                        let code = take_static::<4, _, _>(json)
                            .map_or(Err(anyhow!("Invalid string escape")), |v| Ok(v))?;
                        let mut codepoint =
                            [u16::from_str_radix(&std::str::from_utf8(&code)?, 16)?, 0];
                        let mut utf16_len: usize = 1;
                        if Some(&b'\\') == json.peek() && Some(&b'u') == json.peek() {
                            _ = json.next().unwrap();
                            _ = json.next().unwrap();
                            let second_code = take_static::<4, _, _>(json)
                                .map_or(Err(anyhow!("Invalid string escape")), |v| Ok(v))?;
                            codepoint[1] =
                                u16::from_str_radix(&std::str::from_utf8(&second_code)?, 16)?;
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
                                        bail!("Invalid string escape");
                                    }
                                }
                            })
                            .try_collect()?;
                    }
                    _ => bail!("Invalid string escape"),
                }
            }
            Some(_) => unreachable!(),
            None => bail!("Expected end of string"),
        }
    }
    Ok(String::from_utf8(buf)?)
}
