use anyhow::{anyhow, bail, Result};
use itertools::{Itertools, MultiPeek};

use crate::utils::take_static;

pub fn read_string<'a, I: Iterator<Item = char>>(json: &'a mut MultiPeek<I>) -> Result<String> {
    if Some('"') != json.next() {
        bail!("String must start with \"");
    }
    let mut out = String::new();
    loop {
        out.extend(json.peeking_take_while(|c| *c != '"' && *c != '\\'));
        match json.next() {
            Some('"') => break,
            Some('\\') => {
                let escape = json
                    .next()
                    .map_or(Err(anyhow!("Invalid string escape")), |v| Ok(v))?;
                match escape {
                    '"' | '\\' | '/' => out.push(escape),
                    'b' => out.push(0x08 as char), // Backspace char
                    'f' => out.push(0x0C as char), // Form-feed char
                    'n' => out.push(0x0A as char), // Newline char
                    'r' => out.push(0x0D as char), // Carriage return char
                    't' => out.push(0x09 as char), // Tab char
                    'u' => {
                        let code = take_static::<4, _, _>(json)
                            .map_or(Err(anyhow!("Invalid string escape")), |v| Ok(v))?;
                        let mut codepoint = [
                            u16::from_str_radix(&code.iter().collect::<String>(), 16)?,
                            0,
                        ];
                        let mut utf16_len: usize = 1;
                        if Some(&'\\') == json.peek() && Some(&'u') == json.peek() {
                            _ = json.next().unwrap();
                            _ = json.next().unwrap();
                            let second_code = take_static::<4, _, _>(json)
                                .map_or(Err(anyhow!("Invalid string escape")), |v| Ok(v))?;
                            codepoint[1] =
                                u16::from_str_radix(&second_code.iter().collect::<String>(), 16)?;
                            utf16_len += 1;
                        }
                        std::char::decode_utf16(codepoint[0..utf16_len].into_iter().copied())
                            .map(|r| match r {
                                Ok(char) => {
                                    let mut dst: [u8; 4] = [0; 4];
                                    out.push_str(char.encode_utf8(&mut dst));
                                    Ok(())
                                }
                                Err(byte) => {
                                    if byte.unpaired_surrogate() <= 0x1F {
                                        out.push(
                                            char::from_u32(byte.unpaired_surrogate() as u32)
                                                .unwrap(),
                                        );
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
    Ok(out)
}
