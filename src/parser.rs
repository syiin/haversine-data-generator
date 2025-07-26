use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::string::String;
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
