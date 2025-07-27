use std::collections::HashMap;
use crate::lexer::Token;

pub enum StateItem {
    InObject,
    InArray,
    ExpectingKey,
    ExpectingColon,
    ExpectingValue,
    ExpectingCommaOrEnd,
}

pub enum JsonPrimitive {
    String(String),
    Number(f64),
}

pub enum JsonValue {
    Primitive(JsonPrimitive),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

pub fn parse_tokens(tokens: &[Token]){
    let mut state_stack: Vec<StateItem> = Vec::new();
    let mut parser_stack: Vec<JsonValue> = Vec::new();
    let mut key_stack: Vec<String> = Vec::new();

    state_stack.push(StateItem::ExpectingValue);

    for token in tokens {
        let current_state = state_stack.last().unwrap();
        match (current_state, &token) {
            (StateItem::ExpectingValue, Token::OpenBrace) => {
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
                state_stack.push(StateItem::ExpectingValue);
            },
            (StateItem::ExpectingValue, Token::StringContent(s)) => {
                state_stack.pop();
                state_stack.push(StateItem::ExpectingCommaOrEnd);
                if let Some(key) = key_stack.pop(){
                    if let Some(JsonValue::Object(map)) = parser_stack.last_mut() {
                        let value = JsonValue::Primitive(JsonPrimitive::String(s.clone()));
                        map.insert(key, value);
                    }
                }
            },
            (StateItem::ExpectingValue, Token::Number(n)) => {
                state_stack.pop();
                state_stack.push(StateItem::ExpectingCommaOrEnd);
                if let Some(key) = key_stack.pop(){
                    if let Some(JsonValue::Object(map)) = parser_stack.last_mut() {
                        let value = JsonValue::Primitive(JsonPrimitive::Number(n.clone()));
                        map.insert(key, value);
                    }
                }
            },
            (StateItem::ExpectingCommaOrEnd, Token::Comma) => {
                state_stack.pop();
                state_stack.push(StateItem::ExpectingKey);
            },
            (StateItem::ExpectingCommaOrEnd, Token::CloseBrace) => {
                state_stack.pop();
                state_stack.push(StateItem::ExpectingValue);
            }
            _ => { continue },
        }
    }
}

// fn token_to_state(token: &Token) -> StateItem {
//     match token {
//         Token::OpenBrace => StateItem::StartObject,
//         Token::CloseBrace => StateItem::EndObject,
//         Token::OpenBracket => StateItem::StartArray,
//         Token::CloseBracket => StateItem::EndArray,
//         Token::Comma => StateItem::ArrayValue,
//         Token::Colon => StateItem::Key,
//         Token::StringContent(_) => StateItem::PrimitiveValue,
//         Token::Number(_) => StateItem::PrimitiveValue,
//     }
// }
