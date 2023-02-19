use std::str::FromStr;

use anyhow::Result;
use itertools::MultiPeek;

pub fn read_number<I: Iterator<Item = char>>(json: &mut MultiPeek<I>) -> Result<f64> {
    let mut num_string = String::new();
    while let Some(byte) = json.peek() {
        match *byte {
            '0'..='9' | '.' | 'e' | 'E' | '-' | '+' => {
                num_string.push(json.next().unwrap());
            }
            _ => break,
        }
    }
    let number = f64::from_str(&num_string)?;

    Ok(number)
}
