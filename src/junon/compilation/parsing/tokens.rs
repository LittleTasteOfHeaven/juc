// This file is part of "juc"
// All rights reserved
// Copyright (c) Junon, Antonin Hérault

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Token {
    AssemblyCode,
    Function,
    Return,
    Static,
    StringDot,
    Variable,

    // Something that is not a token
    RawString(Box<str>),
}

/// Get a `Token` enum object from the name as String
/// SEE `token_to_string()` (reversed function)
pub fn string_to_token(token_name: &String) -> Token {
    match token_name.as_str() {
        "@" => Token::AssemblyCode,
        "func" => Token::Function,
        "ret" => Token::Return,
        "static" => Token::Static,
        "\"" => Token::StringDot,
        "variable" => Token::Variable,
        _ => Token::RawString(token_name.clone().into_boxed_str()),
    }
}

/// Get the name as String of a `Token` enum object
/// SEE `string_to_token()` (reversed function)
pub fn token_to_string(token: Token) -> String {
    match token {
        Token::AssemblyCode => "@",
        Token::Function => "func",
        Token::Return => "ret",
        Token::Static => "static",
        Token::StringDot => "\"",
        Token::Variable => "variable",
        Token::RawString(ref val) => &*val,
    }.to_string()
}

/// If the character is special (it means that it's not a letter from the Latin 
/// alphabet or if it's not a number), it return "true": the character should be
/// cut by the parser in a new case (should be not placed with the previous 
/// character/word)
pub fn should_be_cut(c: &char) -> bool {
    if (*c >= 'A' && *c <= 'Z') || (*c >= 'a' && *c <= 'z') {
        false
    } else if (*c >= '0' && *c <= '9') || (*c == '.') {
        false
    } else {
        true
    }
}
