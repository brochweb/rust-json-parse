use std::str::FromStr;

use anyhow::{bail, Result};
use itertools::MultiPeek;

pub fn read_number<I: Iterator<Item = u8>>(json: &mut MultiPeek<I>) -> Result<f64> {
    let mut num_buf: [u8; 320] = [0; 320];
    let mut num_len: usize = 0;
    while let Some(byte) = json.peek() {
        match *byte {
            b'0'..=b'9' | b'.' | b'e' | b'E' | b'-' | b'+' => {
                if num_len >= num_buf.len() {
                    bail!("Invalid number literal");
                }
                num_buf[num_len] = json.next().unwrap();
                num_len += 1;
            }
            _ => break,
        }
    }
    eprintln!("Number buf: {:x?}\nNumber length: {}\n", num_buf, num_len);
    let number = f64::from_str(std::str::from_utf8(&num_buf[0..num_len]).unwrap())?;

    Ok(number)
}
