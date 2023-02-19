use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use itertools::{Itertools, MultiPeek};
use parsers::{number::read_number, string::read_string};
use utils::{peek_static, take_static};

pub type JsonObject = HashMap<String, JsonValue>;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Object(Box<JsonObject>),
    Array(Box<Vec<JsonValue>>),
    String(Box<String>),
    Number(f64),
    Boolean(bool),
    Null,
}

mod parsers;
mod utils;

enum ParseState {
    Value,
    Object,
    Array,
}

pub fn parse<'a>(json_buf: &'a [u8]) -> Result<JsonValue> {
    if json_buf.len() >= 0x20000000 {
        bail!(
            "File len {:x} longer than max allowed of 0x20000000",
            json_buf.len()
        );
    }

    let mut json = json_buf.into_iter().copied().multipeek();
    ignore_ws(&mut json);

    let val = parse_next(&mut json, ParseState::Value)?;

    return Ok(val);
}

fn parse_next<'a, I: Iterator<Item = u8>>(
    json: &'a mut MultiPeek<I>,
    state: ParseState,
) -> Result<JsonValue> {
    ignore_ws(json);
    if let Some(char) = json.peek() {
        match state {
            ParseState::Value => {
                if is_string(char) {
                    json.reset_peek();
                    return Ok(JsonValue::String(Box::new(read_string(json)?)));
                }
                if is_number(char) {
                    json.reset_peek();
                    return Ok(JsonValue::Number(read_number(json)?));
                }
                if is_array(char) {
                    _ = json.next().unwrap();
                    return parse_next(json, ParseState::Array);
                }
                if is_object(char) {
                    _ = json.next().unwrap();
                    return parse_next(json, ParseState::Object);
                }
                json.reset_peek();
                let next_4: [u8; 4] =
                    peek_static(json).map_or(Err(anyhow!("Expected next value")), |v| Ok(v))?;
                if &next_4 == b"true" {
                    take_static::<4, _, _>(json);
                    return Ok(JsonValue::Boolean(true));
                }
                if &next_4 == b"null" {
                    take_static::<4, _, _>(json);
                    return Ok(JsonValue::Null);
                }
                if &next_4 == b"fals" && json.peek() == Some(&b'e') {
                    take_static::<5, _, _>(json);
                    return Ok(JsonValue::Boolean(false));
                }
                return Err(anyhow!("Expected next value"));
            }
            ParseState::Array => {
                let mut contents = Vec::new();
                if b']' == *char {
                    return Ok(JsonValue::Array(Box::new(contents)));
                }
                loop {
                    ignore_ws(json);
                    contents.push(parse_next(json, ParseState::Value)?);
                    ignore_ws(json);

                    match json.next() {
                        Some(b']') => break,
                        Some(b',') => continue,
                        _ => bail!("Expected next value"),
                    }
                }
                contents.shrink_to_fit();
                return Ok(JsonValue::Array(Box::new(contents)));
            }
            ParseState::Object => {
                let mut contents = HashMap::new();
                if *char == b'}' {
                    return Ok(JsonValue::Object(Box::new(contents)));
                }
                loop {
                    ignore_ws(json);
                    let key = read_string(json)?;
                    ignore_ws(json);
                    if json.next() != Some(b':') {
                        bail!("Expected colon after key");
                    }
                    ignore_ws(json);
                    let value = parse_next(json, ParseState::Value)?;
                    contents.insert(key, value);
                    ignore_ws(json);
                    match json.next() {
                        Some(b'}') => break,
                        Some(b',') => continue,
                        _ => bail!("Expected next value"),
                    }
                }
                return Ok(JsonValue::Object(Box::new(contents)));
            }
        }
    }
    return Ok(JsonValue::Null);
}

fn ignore_ws<'a, I: Iterator<Item = u8>>(json: &'a mut MultiPeek<I>) {
    json.reset_peek();
    json.peeking_take_while(is_whitespace).for_each(|_| {});
    json.reset_peek();
}

fn is_whitespace(char: &u8) -> bool {
    *char == 0x0020 || *char == 0x000A || *char == 0x000D || *char == 0x0009
}

fn is_string(char: &u8) -> bool {
    *char == b'"'
}

fn is_object(char: &u8) -> bool {
    *char == b'{'
}

fn is_array(char: &u8) -> bool {
    *char == b'['
}

fn is_number(char: &u8) -> bool {
    (*char >= b'0' && *char <= b'9') || *char == b'-'
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(mem::size_of::<JsonValue>(), 16);
    }

    #[test]
    fn string() {
        // "string, \"string\", string—🎸🦕"
        let val = parse(
            "\"string, \\\"string\\\", string—🎸\\uD83E\\uDD95\\u3ED8\\u0003\\f\"".as_bytes(),
        )
        .unwrap();
        assert_eq!(
            val,
            JsonValue::String(Box::new(String::from(
                "string, \"string\", string—🎸🦕㻘\x03\x0C"
            )))
        )
    }

    #[test]
    fn json_array() {
        let mut string = "[5   ,\n\n".repeat(500);
        string.push_str("[\"algo\", 3.1415926535, 5.2e+50, \"\",null,true,false,[],[],[],[[[[[[[[[[[[[[]]]]]]]]]]]]]]]");
        string.push_str(&"]".repeat(500));
        let ret = parse(string.as_bytes()).unwrap();
        eprintln!("{:?}", ret);
    }

    #[test]
    fn json_atoms() {
        let string = "[null, true,false,null,  true, false]";
        let ret = parse(string.as_bytes()).unwrap();
        assert_eq!(
            ret,
            JsonValue::Array(Box::new(vec![
                JsonValue::Null,
                JsonValue::Boolean(true),
                JsonValue::Boolean(false),
                JsonValue::Null,
                JsonValue::Boolean(true),
                JsonValue::Boolean(false)
            ]))
        );
    }

    #[test]
    fn json_object() {
        let string = "{\n\t\t\"name\":\"Steve\"\n\t}";
        let ret = parse(string.as_bytes()).unwrap();
        match ret {
            JsonValue::Object(obj) => match obj.get("name") {
                Some(JsonValue::String(str)) => {
                    assert_eq!(str.as_str(), "Steve");
                }
                _ => panic!("Expect name field to be string"),
            },
            _ => panic!("Expected object"),
        }
    }
}
