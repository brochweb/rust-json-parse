use std::str::FromStr;

use itertools::MultiPeek;

use crate::{JsonResult, ParseError};

pub fn read_number<I: Iterator<Item = u8>>(json: &mut MultiPeek<I>) -> JsonResult<f64> {
    let mut num_buf: [u8; 320] = [0; 320];
    let mut num_len: usize = 0;
    while let Some(byte) = json.peek() {
        match *byte {
            b'0'..=b'9' | b'.' | b'e' | b'E' | b'-' | b'+' => {
                if num_len >= num_buf.len() {
                    return Err(ParseError::InvalidNumberLiteral);
                }
                num_buf[num_len] = json.next().unwrap();
                num_len += 1;
            }
            _ => break,
        }
    }
    let number = f64::from_str(std::str::from_utf8(&num_buf[0..num_len]).unwrap())
        .map_err(|_| ParseError::InvalidNumberLiteral)?;

    Ok(number)
}
