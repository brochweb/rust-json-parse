use std::string::String as StdString;
use std::vec::Vec as StdVec;

use bumpalo::collections::{String, Vec};
use bumpalo::Bump;
use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::{BumpWrapper, HashMap};
use parsers::{number::read_number, string::read_string};
use slice_iter::{CopyIter, SliceIter};

mod parsers;
mod slice_iter;

pub type JsonObject<'bump> =
    HashMap<String<'bump>, JsonValue<'bump>, DefaultHashBuilder, BumpWrapper<'bump>>;

pub type JsonResult<T> = Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub enum JsonValue<'bump> {
    Object(&'bump mut JsonObject<'bump>),
    Array(&'bump mut Vec<'bump, JsonValue<'bump>>),
    String(&'bump mut String<'bump>),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(thiserror::Error, Debug)]
pub enum JsonError {
    #[error("File too long: len {len} longer than maximum allowed of 500 MiB")]
    FileTooLong { len: usize },
    #[error("ParseError: {0}, with this json remaining: {1}")]
    ParseError(ParseError, StdString),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Expected closing bracket or comma, found {found:?}")]
    ExpectedEndOfArray { found: Option<char> },
    #[error("Expected closing brace or comma, found {found:?}")]
    ExpectedEndOfObject { found: Option<char> },
    #[error("Expected next value, found {found:?}")]
    ExpectedNextValue { found: Option<char> },
    #[error("Expect colon after object key, found {found:?}")]
    ExpectedColon { found: Option<char> },
    #[error("Invalid number literal")]
    InvalidNumberLiteral,
    #[error("Invalid string escape")]
    InvalidStringEscape,
    #[error("JSON string was not valid UTF-8: {string}")]
    InvalidUtf8 { string: StdString },
    #[error("String must start with a quote `\"`, found {found:?}")]
    StringQuote { found: Option<char> },
    #[error("Expected the next value, found the end of the file")]
    UnexpectedEndOfFile,
}

enum ParseState {
    Value,
    Object,
    Array,
}

pub struct JsonDocument<'a> {
    pub root: JsonValue<'a>,
    allocator: Bump,
}

impl<'a> JsonDocument<'a> {
    pub fn init() -> Self {
        Self {
            root: JsonValue::Null,
            allocator: Bump::new(),
        }
    }
    pub fn parse_slice(&'a mut self, slice: &[u8]) -> Result<&'a JsonValue<'a>, JsonError> {
        self.root = parse(slice, &self.allocator)?;
        Ok(&self.root)
    }
}

pub fn parse<'a, 'bump>(
    json_buf: &'a [u8],
    allocator: &'bump Bump,
) -> Result<JsonValue<'bump>, JsonError> {
    if json_buf.len() >= 0x20000000 {
        return Err(JsonError::FileTooLong {
            len: json_buf.len(),
        });
    }

    let mut json = SliceIter::new(json_buf);
    ignore_ws(&mut json);

    match parse_next(&mut json, allocator, ParseState::Value) {
        Ok(v) => return Ok(v),
        Err(e) => {
            let remaining = StdString::from_utf8_lossy(&json.collect::<StdVec<u8>>()).to_string();
            return Err(JsonError::ParseError(
                e,
                if remaining.len() >= 500 {
                    format!("{}…", &remaining[0..500])
                } else {
                    remaining
                },
            ));
        }
    };
}

fn parse_next<'a, 'bump, I: CopyIter<'a, Item = u8>>(
    json: &mut I,
    alloc: &'bump Bump,
    state: ParseState,
) -> JsonResult<JsonValue<'bump>> {
    ignore_ws(json);
    if let Some(char) = json.peek_copy() {
        match state {
            ParseState::Value => {
                if is_string(char) {
                    return Ok(JsonValue::String(alloc.alloc(read_string(json, alloc)?)));
                }
                if is_number(char) {
                    return Ok(JsonValue::Number(read_number(json)?));
                }
                if is_array(char) {
                    json.ignore_next();
                    return parse_next(json, alloc, ParseState::Array);
                }
                if is_object(char) {
                    json.ignore_next();
                    return parse_next(json, alloc, ParseState::Object);
                }
                let next_4: [u8; 4] = json
                    .peek_many()
                    .map_or(Err(ParseError::UnexpectedEndOfFile), |v| Ok(v))?;
                if &next_4 == b"true" {
                    json.ignore_many(4);
                    return Ok(JsonValue::Boolean(true));
                }
                if &next_4 == b"null" {
                    json.ignore_many(4);
                    return Ok(JsonValue::Null);
                }
                if json.peek_many_ref(5) == Some(b"false") {
                    json.ignore_many(5);
                    return Ok(JsonValue::Boolean(false));
                }
                return Err(ParseError::ExpectedNextValue {
                    found: json.next().map(|i| i as char),
                });
            }
            ParseState::Array => {
                let mut contents = Vec::new_in(alloc);
                if b']' == char {
                    _ = json.next().unwrap();
                    return Ok(JsonValue::Array(alloc.alloc(contents)));
                }
                loop {
                    ignore_ws(json);
                    contents.push(parse_next(json, alloc, ParseState::Value)?);
                    ignore_ws(json);

                    match json.next() {
                        Some(b']') => break,
                        Some(b',') => continue,
                        v => {
                            return Err(ParseError::ExpectedEndOfArray {
                                found: v.map(|v| v as char),
                            })
                        }
                    }
                }
                contents.shrink_to_fit();
                return Ok(JsonValue::Array(alloc.alloc(contents)));
            }
            ParseState::Object => {
                let mut contents: HashMap<String<'bump>, JsonValue<'bump>, _, BumpWrapper> =
                    HashMap::new_in(BumpWrapper(alloc));
                if char == b'}' {
                    _ = json.next().unwrap();
                    return Ok(JsonValue::Object(alloc.alloc(contents)));
                }
                loop {
                    ignore_ws(json);
                    let key = read_string(json, alloc)?;
                    ignore_ws(json);
                    let c = json.next();
                    if c != Some(b':') {
                        return Err(ParseError::ExpectedColon {
                            found: c.map(|i| i as char),
                        });
                    }
                    ignore_ws(json);
                    let value = parse_next(json, alloc, ParseState::Value)?;
                    contents.insert(key, value);
                    ignore_ws(json);
                    match json.next() {
                        Some(b'}') => break,
                        Some(b',') => continue,
                        v => {
                            return Err(ParseError::ExpectedEndOfObject {
                                found: v.map(|i| i as char),
                            })
                        }
                    }
                }
                return Ok(JsonValue::Object(alloc.alloc(contents)));
            }
        }
    }
    return Ok(JsonValue::Null);
}

fn ignore_ws<'a, I: CopyIter<'a, Item = u8>>(json: &mut I) {
    while json.peek_copy().map_or(false, is_whitespace) {
        json.ignore_next();
    }
}

fn is_whitespace(char: u8) -> bool {
    char == 0x0020 || char == 0x000A || char == 0x000D || char == 0x0009
}

fn is_string(char: u8) -> bool {
    char == b'"'
}

fn is_object(char: u8) -> bool {
    char == b'{'
}

fn is_array(char: u8) -> bool {
    char == b'['
}

fn is_number(char: u8) -> bool {
    (char >= b'0' && char <= b'9') || char == b'-'
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
        let bump = Bump::new();
        let val = parse(
            "\"string, \\\"string\\\", string—🎸\\uD83E\\uDD95\\u3ED8\\u0003\\f\"".as_bytes(),
            &bump,
        )
        .unwrap();
        assert_eq!(
            val,
            JsonValue::String(bump.alloc(String::from_str_in(
                "string, \"string\", string—🎸🦕㻘\x03\x0C",
                &bump
            )))
        )
    }

    #[test]
    fn json_array() {
        let mut string = "[5   ,\n\n".repeat(100);
        string.push_str("[\"algo\", 3.1415926535, 5.2e+50, \"\",null,true,false,[],[],[],[[[[[[[[[[[[[[]]]]]]]]]]]]]]]");
        string.push_str(&"]".repeat(100));
        let bump = Bump::new();
        let ret = parse(string.as_bytes(), &bump).unwrap();
        eprintln!("{:?}", ret);
    }

    #[test]
    fn json_atoms() {
        let string = "[null, true,false,null,  true, false]";
        let bump = Bump::new();
        let ret = parse(string.as_bytes(), &bump).unwrap();
        assert_eq!(
            ret,
            JsonValue::Array(bump.alloc(bumpalo::vec![
                in &bump;
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
        let bump = Bump::new();
        let ret = parse(string.as_bytes(), &bump).unwrap();
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
