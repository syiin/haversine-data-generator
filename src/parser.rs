use std::collections::HashMap;
use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum StateItem {
    InObject,
    InArray,
    ExpectingKey,
    ExpectingColon,
    ExpectingValue,
    ExpectingValueInObject,
    ExpectingValueInArray,
    ExpectingCommaOrEndObject,
    ExpectingCommaOrEndArray,
}


#[derive(Debug, Clone)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

pub fn parse_tokens(tokens: &[Token]) -> Option<JsonValue> {
    let mut state_stack: Vec<StateItem> = Vec::new();
    let mut parser_stack: Vec<JsonValue> = Vec::new();
    let mut key_stack: Vec<String> = Vec::new();

    state_stack.push(StateItem::ExpectingValue);

    for token in tokens {
        let current_state = state_stack.last().unwrap();
        match (current_state, &token) {
            (StateItem::ExpectingValue | StateItem::ExpectingValueInObject | StateItem::ExpectingValueInArray, Token::OpenBrace) => {
                state_stack.pop();
                state_stack.push(StateItem::InObject);
                state_stack.push(StateItem::ExpectingKey);

                parser_stack.push(JsonValue::Object(HashMap::new()));
            },
            (StateItem::ExpectingKey, Token::StringContent(s)) => {
                state_stack.pop();
                state_stack.push(StateItem::ExpectingColon);
                key_stack.push(s.clone());
            },
            (StateItem::ExpectingColon, Token::Colon) => {
                state_stack.pop();
                state_stack.push(StateItem::ExpectingCommaOrEndObject);
                state_stack.push(StateItem::ExpectingValueInObject);
            },
            (StateItem::ExpectingValueInObject | StateItem::ExpectingValueInArray, Token::StringContent(s)) => {
                state_stack.pop();
                if let Some(key) = key_stack.pop(){
                    if let Some(JsonValue::Object(map)) = parser_stack.last_mut() {
                        let value = JsonValue::String(s.clone());
                        map.insert(key, value);
                    }
                }
            },
            (StateItem::ExpectingValueInObject, Token::Number(n)) => {
                state_stack.pop();
                if let Some(key) = key_stack.pop(){
                    if let Some(JsonValue::Object(map)) = parser_stack.last_mut() {
                        let value = JsonValue::Number(n.clone());
                        map.insert(key, value);
                    }
                }
            },
            (StateItem::ExpectingCommaOrEndObject, Token::Comma) => {
                state_stack.pop();
                state_stack.push(StateItem::ExpectingKey);
            },
            (StateItem::ExpectingCommaOrEndObject, Token::CloseBrace) => {
                state_stack.pop(); // ExpectingCommaOrEndObject
                state_stack.pop(); // InObject

                if let Some(finished_obj) = parser_stack.pop() {
                    // Check the parent container (the new top of the stack) to decide what to do.
                    if let Some(parent) = parser_stack.last_mut() {
                        // Peek at parent
                        match parent {
                            JsonValue::Array(arr) => {
                                arr.push(finished_obj);
                            },
                            JsonValue::Object(map) => {
                                if let Some(key) = key_stack.pop() {
                                    map.insert(key, finished_obj);
                                }
                            },
                            _ => { panic!("Error: Expected array or object"); }
                        }
                    } else {
                        parser_stack.push(finished_obj); // Is root object
                    }
                }
            },
            (StateItem::ExpectingValue | StateItem::ExpectingValueInObject, Token::OpenBracket) => {
                state_stack.pop();
                state_stack.push(StateItem::InArray);
                state_stack.push(StateItem::ExpectingCommaOrEndArray);
                state_stack.push(StateItem::ExpectingValueInArray);
                parser_stack.push(JsonValue::Array(Vec::new()));
            },
            (StateItem::ExpectingCommaOrEndArray, Token::Comma) => {
                state_stack.pop();
                state_stack.push(StateItem::ExpectingCommaOrEndArray);
                state_stack.push(StateItem::ExpectingValueInArray);
            },
            (StateItem::ExpectingCommaOrEndArray, Token::CloseBracket) => {
                state_stack.pop(); // ExpectingCommaOrEndArray
                state_stack.pop(); // InArray

                if let Some(finished_arr) = parser_stack.pop() {
                    if let Some(parent) = parser_stack.last_mut() {
                        // Peek at parent
                        match parent {
                            JsonValue::Array(arr) => {
                                arr.push(finished_arr);
                            },
                            JsonValue::Object(map) => {
                                if let Some(key) = key_stack.pop() {
                                    map.insert(key, finished_arr);
                                }
                            },
                            _ => { panic!("Error: Expected array or object"); }
                        }
                    } else {
                        parser_stack.push(finished_arr); // is root
                    }
                }
            },
            _ => continue,
        }
    }

    assert!(state_stack.len() == 0);
    assert!(parser_stack.len() == 1);

    return parser_stack.last().cloned();
}

pub fn format_json(json: &JsonValue) -> String {
    let mut output = String::new();
    match json {
        JsonValue::String(s) => {
            output = format!("\"{}\"", s);
            output
        },
        JsonValue::Number(n) => {
            output = n.to_string();
            output
        },
        JsonValue::Object(map) => {
            output.push('{');
            for (i, (key, value)) in map.iter().enumerate() {
                if i != 0 {
                    output.push(',');
                }
                output.push_str(&format!("\"{}\":{}", key, format_json(value)));
            }
            output.push('}');
            output
        },
        JsonValue::Array(arr) => {
            output.push('[');
            for (i, value) in arr.iter().enumerate() {
                if i != 0 {
                    output.push(',');
                }
                output.push_str(&format_json(value));
            }
            output.push(']');
            output
        },
    }
}
