use std::fs::File;
use std::io::{self, BufReader, BufRead, Read};
use std::path::Path;
use std::collections::HashMap;
use std::string::String;

pub enum Token {
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Comma,
    Colon,
    StringContent(String),
    Number(f64),
    Boolean(bool),
    Null,
}

impl Token {
    pub fn format(&self) -> String {
        match self {
            Token::OpenBrace => "{".to_string(),
            Token::CloseBrace => "}".to_string(),
            Token::OpenBracket => "[".to_string(),
            Token::CloseBracket => "]".to_string(),
            Token::Comma => ",".to_string(),
            Token::Colon => ":".to_string(),
            Token::StringContent(s) => s.to_string(),
            Token::Number(n) => n.to_string(),
            Token::Boolean(b) => b.to_string(),
            Token::Null => "null".to_string(),
        }
    }
}

pub fn parse_file(file: File) -> Vec<Token> {
    let mut output = Vec::new();
    let mut string = String::new();
    let mut in_string = false;

    let mut digit = String::new();
    let mut in_digit = false;

    let reader = BufReader::new(file);
    for byte_result in reader.bytes() {
        let byte = match byte_result {
            Ok(byte) => byte,
            Err(err) => {
                println!("Error: {}", err);
                continue;
            }
        };

        match byte {
            b'{' => output.push(Token::OpenBrace),
            b'}' => {
                if !digit.is_empty() {
                    output.push(Token::Number(digit.parse().unwrap()));
                    digit.clear();
                }
                in_digit = false;
                output.push(Token::CloseBrace);
            }
            b'[' => output.push(Token::OpenBracket),
            b']' => {
                if !digit.is_empty() {
                    output.push(Token::Number(digit.parse().unwrap()));
                    digit.clear();
                }
                in_digit = false;
                output.push(Token::CloseBracket);
            }
            b':' => output.push(Token::Colon),
            b',' => {
                if !digit.is_empty() {
                    if let Ok(n) = digit.parse() {
                        output.push(Token::Number(n));
                        digit.clear();
                    } else {
                        println!("Error parsing number: {}", digit);
                        continue;
                    }
                }
                in_digit = false;
                output.push(Token::Comma)
            },
            b'"' => {
                if in_string {
                    string.push(byte as char);
                    let tmp_string = string.clone();
                    in_string = false;
                    output.push(Token::StringContent(tmp_string));
                    string.clear();
                } else {
                    string.push(byte as char);
                    in_string = true;
                }
            }
            b'0'..=b'9' | b'-' | b'.' => {
                if in_string { 
                    string.push(byte as char);
                    continue;
                }
                digit.push(byte as char);
                in_digit = true;
            }
            _ => {
                if in_string {
                    string.push(byte as char);
                }
                continue;
            },
        }
    }
    return output;
}

enum StackItem {
    Token,
    JsonItem,
}


pub enum JsonValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
    Null,
}

pub struct JsonItem {
    pub key: String,
    pub value: JsonValue,
}

impl JsonItem {
    pub fn new(key: String, value: JsonValue) -> Self {
        JsonItem {
            key, value
        }
    }
}

pub fn parse_json(file_path: String){
    let mut stack: Vec<u8> = Vec::new();
    let mut open_quotes = false;
    let file = File::open(file_path).unwrap();
    for byte_result in file.bytes() {
        match byte_result {
            Ok(byte) => {
                if byte == b'{' {
                    stack.push(byte);
                } 
                if byte == b'"' {
                    open_quotes = !open_quotes;
                    if open_quotes {
                        stack.push(byte);
                    } else {
                        collate_string(&mut stack);
                    }
                }
                else {
                    continue;
                }
            },
            Err(err) => {
                println!("Error: {}", err);
            }
        }

    }
}

fn collate_string(stack: &mut Vec<u8>) -> JsonValue {
    let mut string = String::new();
    let mut current_byte = stack.pop().unwrap();
    while current_byte != b'"' {
        current_byte = stack.pop().unwrap();
        string.push(current_byte as char);
    }
    JsonValue::String(string)
}
