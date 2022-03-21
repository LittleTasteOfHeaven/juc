// This file is part of "juc"
// All rights reserved
// Copyright (c) Junon, Antonin Hérault

use std::fmt;
use std::fs::File;
use std::io::Read;

use crate::junon::{
    compilation::{
        parsing::{
            tokens::*,
        },
    },
    logger::*,
};

pub struct Parser {
    parsed: Vec<Vec<Token>>,
    source: String,
    content: String,
}

impl fmt::Debug for Parser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "---\n")?;
        for line in &self.parsed {
            if line.len() < 1 {
                write!(f, "\n")?;
            } else {
                write!(f, "{:?}\n", line)?;
            }
        }
        Ok(())
    }
}

impl Parser {
    pub fn new(source: &String) -> Self {
        Self {
            parsed: vec!(),
            source: source.to_string(),
            content: String::new(),
        }
    }

    pub fn run(&mut self) {
        self.read_file_content();

        let mut token = String::new();
        let mut line: Vec<Token> = vec!();

        let mut was_double_char = false;
        let mut is_string = false;
        let mut string_content = String::new();

        for (i, c) in self.content.chars().enumerate() {
            // String creation
            
            // Get the first character because it's a String of one character
            if c == get_string_token(Token::StringDot).chars().nth(0).unwrap() { 
                if is_string { // ending of string
                    is_string = false;
                    line.push(get_token(&string_content));
                } else { // beginning of string
                    is_string = true;
                }
                continue;
            } 
            if is_string {
                string_content.push(c);
                continue; // don't care of others possibilities, we want raw 
                // characters in the String
            }
            
            // New line detected
            if c == '\n' {
                // Push the last token of the line
                Self::push_token(&mut token, &mut line);

                // Push the line into the parsed 2d list
                self.parsed.push(line.clone());
                line = vec!(); // reset line
                
                continue; // and then '\n' will be not pushed
            }

            // When it's a special character (not letter or number, not simple 
            // point).
            // SEE `tokens::should_be_cut()`
            if should_be_cut(&c) {
                Self::push_token(&mut token, &mut line);

                // And push the special character detected as a new token
                if c != ' ' && !was_double_char {
                    if i != self.content.len() - 1 && 
                        c == self.content.chars().nth(i + 1).unwrap()
                    {
                        line.push(get_token(&format!("{}{}", c, c)));
                        was_double_char = true;
                        continue;
                    }
    
                    line.push(get_token(&format!("{}", c)));
                }
                was_double_char = false;
                continue;
            }

            token.push(c); // it's still the same "word"/"token"
        }
    }

    /// Push the token into the line whenever it's not a null token.
    /// The given `token` name parameter is a String, and it is converted into
    /// a `Token` before pushing.
    fn push_token(token: &mut String, line: &mut Vec<Token>) {
        if *token != String::new() {
            line.push(get_token(&token));
            *token = String::new(); // reset
        }
    }

    /// Update the `content` attribute to the file's content by opening a new
    /// file stream (readable) on it.
    fn read_file_content(&mut self) {
        let mut stream = File::open(&self.source).unwrap(); // `unwrap()` is 
        // called because the source file was already checked before
        match stream.read_to_string(&mut self.content) {
            Err(_) => {
                let mut logger = Logger::new();

                logger.add_log(
                    Log::new(
                        LogLevel::Error,
                        "Unreadable file".to_string(),
                        "The given source file cannot be read".to_string()
                    )
                    .add_hint("It's probably corrupted or it's not a text file"
                        .to_string()),
                );

                logger.interpret();
            },
            Ok(_) => {}
        }
    }

    pub fn parsed(&self) -> &Vec<Vec<Token>> {
        &self.parsed
    }
}
