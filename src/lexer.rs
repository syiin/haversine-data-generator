use std::fs::File;
use std::io::{self, BufReader, BufRead, Read};
use std::string::String;

#[derive(Clone)]
pub enum Token {
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Comma,
    Colon,
    StringContent(String),
    Number(f64),
    // Boolean(bool),
    // Null,
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
            // Token::Boolean(b) => b.to_string(),
            // Token::Null => "null".to_string(),
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
                handle_digit_termination(&mut output, &mut digit, &mut in_digit);
                output.push(Token::CloseBrace);
            }
            b'[' => output.push(Token::OpenBracket),
            b']' => {
                handle_digit_termination(&mut output, &mut digit, &mut in_digit);
                output.push(Token::CloseBracket);
            }
            b':' => output.push(Token::Colon),
            b',' => {
                handle_digit_termination(&mut output, &mut digit, &mut in_digit);
                output.push(Token::Comma);
            },
            b'"' => {
                if in_string {
                    output.push(Token::StringContent(string.clone()));
                    string.clear();
                    in_string = false;
                } else {
                    in_string = true;
                }
            }
            b'0'..=b'9' | b'-' | b'.' | b'e' | b'E' | b'+' => {
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

pub fn handle_digit_termination(output: &mut Vec<Token>, digit: &mut String, in_digit: &mut bool){
    if !digit.is_empty() {
        if let Ok(n) = digit.parse() {
            output.push(Token::Number(n));
            digit.clear();
        } else {
            println!("Error parsing number: {}", digit);
            digit.clear();
        }
    }
    *in_digit = false;
}
