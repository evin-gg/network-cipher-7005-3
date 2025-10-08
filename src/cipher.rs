#![allow(dead_code)]
use std::str;

pub fn split_payload(input: &[u8]) -> String {

    let input_str = str::from_utf8(input).unwrap();
    let mut parts = input_str.splitn(2, '|');
    let key = parts.next().unwrap();
    let text = parts.next().unwrap();

    println!("Key: {}", key);
    println!("Text: {}", text); 

    let encoded_message = shift_vig(text, key);

    return encoded_message;
}

pub fn shift_vig(text: &str, key: &str) -> String {
    let mut result = String::new();

    
    let text_char_count = count_chars(text);
    let mut keystream: String = String::new();

    // push the key to the container until overflow or equal
    while keystream.chars().count() < text_char_count {
        keystream.push_str(key);
    }

    // convert to vector for iterative container and use take() to truncate
    let keystream_vec: Vec<char> = keystream.chars().take(text_char_count).collect();

    // index tracking
    let mut index = 0;

    for c in text.chars() {
        if c.is_ascii_lowercase() {

            // normalize to 0 - 25
            let key_val = keystream_vec[index] as i32 - 'a' as i32;
            let shifted_char = c as i32 - 'a' as i32 + key_val;
            result.push(lower_char_adjustment(shifted_char));
            index += 1
        }

        else if c.is_ascii_uppercase() {

            // normalize to 0 - 25 accounting for uppercase
            let key_val = (keystream_vec[index].to_ascii_uppercase() as u8 - 'A' as u8) as i32;
            let shifted_char = c as i32 - 'A' as i32 + key_val;
            result.push(upper_char_adjustment(shifted_char));
            index += 1
        }

        else {
            result.push(c);
        }
    }

    return result;
}

//counts chars in a string
pub fn count_chars(text: &str) -> usize {
    let mut count = 0;
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            count += 1;
        }
    }
    return count;
}

pub fn lower_char_adjustment(shifted_char: i32) -> char {

    // account for overflow
    let adjusted = shifted_char % 26;
    std::char::from_u32((adjusted + 'a' as i32) as u32).unwrap()
}

pub fn upper_char_adjustment(shifted_char: i32) -> char {

    // account for overflow
    let adjusted = shifted_char % 26;
    std::char::from_u32((adjusted + 'A' as i32) as u32).unwrap()
}
