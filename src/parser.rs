use std::fs::File;
use std::io::{self, BufReader, BufRead, Read};
use std::path::Path;
use std::collections::HashMap;
use std::string::String;

enum Token {
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

struct Tokeniser {
    pos: usize,
    reader: BufReader<File>,
    tokens: Vec<Token>,
}

impl Tokeniser {
    pub fn new(file_path: String) -> Self {
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        Tokeniser {
            pos: 0,
            reader,
            tokens: Vec::new(),
        }
    }

    pub fn parse(file: File) -> Vec<Token> {
        let mut output = Vec::new();
        let mut string = String::new();
        let mut in_string = false;

        let mut digit = String::new();
        let mut in_digit = false;

        for byte_result in file.bytes() {
            let byte = byte_result.unwrap();
            if in_string {
                string.push(byte as char);
                continue;
            }
            if in_digit {
                digit.push(byte as char);
                continue;
            }
            match byte {
                b'{' => {
                    output.push(Token::OpenBrace);
                }
                b'}' => {
                    in_digit = false;
                    output.push(Token::CloseBrace);
                }
                b'[' => {
                    output.push(Token::OpenBracket);
                }
                b']' => {
                    in_digit = false;
                    output.push(Token::CloseBracket);
                }
                b':' => {
                    output.push(Token::Colon)
                }
                b',' => {
                    in_digit = false;
                    output.push(Token::Comma)
                }
                b'"' => {
                    if in_string {
                        let tmp_string = string.clone();
                        in_string = false;
                        output.push(Token::StringContent(tmp_string));
                        string.clear();
                    } else {
                        in_string = false;
                    }
                }
                _ => {
                    continue;
                }
            }
        }
        return output;
    }
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
