// This file is part of "juc"
// Under the MIT License
// Copyright (c) Junon, Antonin Hérault

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;

use jup::{
    parser::Parser, 
    tokens::Token,
};

use checking;
use checking::data::CheckerData;

use crate::{
    caller::Caller,
    data::CompilerData,
    defaults,
    scope::Scope,
};

use logging::level::LogLevel;
use logging::log::Log;
use logging::logger::Logger;

use objects::{
    function::Function, 
    variable::Variable
};

/// Trait for a Compiler followed by all platform's compilers \
/// Some functions are already defined because they are cross-platform \
/// The general documentation is written here to avoid to write the same
/// documentation to each platform's compilers. But a specific compiler can
/// have its own documentation
pub trait Compiler: Caller {
    /// Starting point \
    /// Do some stuff useful
    fn init(&mut self);

    /// Starting point for each source file
    fn init_one(&mut self, source: &String) -> Result<(), Logger> {
        self.data().current_source = format!(
            "{}/{}.asm", 
            defaults::BUILD_FOLDER, 
            source
        );
        
        self.data().parser = Some(
            Parser::from_path(Path::new(source)).unwrap()
        );

        self.data().parser.as_mut()
            .unwrap()
            .run();

        // run all checkers for the current source file
        let checker_data = CheckerData {
            source: source.clone(),
            parsed: self.data().parser.as_ref().unwrap().parsed().clone(),
            logger: Logger::new(),
            line_i: 0,
            token_i: 0,
        };
        checking::run_checkers(checker_data)
    }

    /// Main function where each source file is transformed to an objet file
    fn run(&mut self) {
        self.init();

        // Returned logs
        let mut loggers: Vec<Logger> = vec!();

        for source in self.data().sources.clone() {
            // Module name it's the filename without the ".ju" extension
            self.data().current_scope = Scope::from(vec![
                format!("{}", source)
                    .split(defaults::EXTENSION_COMPLETE)
                    .collect::<String>()
            ]);

            match self.init_one(&source) {
                Ok(()) => {},
                Err(logger) => {
                    loggers.push(logger.clone());

                    // When it doesn't contain only warnings
                    if logger.get_result() != Ok(()) {
                        continue;
                    }
                }
            }
            self.call();
            self.finish_one(&source);
        }

        let mut should_be_stopped = false;
        for logger in loggers {
            // `should_be_stopped` is not updated as the function call return 
            // value because it could be `false`
            if logger.print_all(false) {
                should_be_stopped = true;
            }
        }

        if should_be_stopped {
            process::exit(1);
        }

        self.link();
        self.finish();
    }

    /// Methods caller according to the current token
    fn call(&mut self) {
        let parsed: Vec<Vec<Token>> = self.data().parser.as_ref()
            .unwrap()
            .parsed()
            .to_vec();

        for line in parsed.iter() {
            self.data().current_line = line.clone();

            let mut previous_token = Token::None;
            let mut break_line = false; // to break the loop from the closure

            for token in line.iter() {
                if break_line {
                    break;
                }

                self.data().current_token = token.clone();
                self.check_for_instruction(
                    line, 
                    &mut break_line, 
                    &token,
                    &mut previous_token
                );
                previous_token = token.clone();
            }
        }
    }

    /// Function "linked" with `call()` because it does the `Caller` calls
    fn check_for_instruction(
        &mut self, 
        line: &Vec<Token>, 
        break_line: &mut bool,
        token: &Token,
        previous_token: &mut Token
    ) {
        let mut line_iter_for_next_tokens = line.iter();
        line_iter_for_next_tokens.next();

        let next_tokens: Vec<Token> = line_iter_for_next_tokens
            .map(| x | x.clone() )
            .collect(); // as vector
            
        // NOTE "break" instructions means : stop reading the line
        match previous_token {
            Token::Assembly => {
                self.when_assembly_code(next_tokens);
                *break_line = true;
                return;
            }
            Token::Assign => self.when_assign(line.to_vec()),
            Token::Function => self.when_function(next_tokens),
            Token::Return => self.when_return(next_tokens),
            Token::Static => {
                self.when_static(next_tokens);
                *break_line = true;
                return;
            }
            Token::Variable => {
                self.when_variable(next_tokens);
                *break_line = true;
                return;
            }

            Token::Print => {
                self.when_print(next_tokens);
                *break_line = true;
                return;
            }
            Token::Exit => {
                self.when_exit(next_tokens);
                *break_line = true;
                return;
            }

            // First token of the line
            Token::None => {
                // Lonely token, execute it right now
                if line.len() == 1 {
                    *previous_token = token.clone();
                    
                    // Call again with same arguments
                    self.check_for_instruction(
                        line, 
                        break_line,
                        token,
                        previous_token
                    );
                }
            },
            _ => self.when_other(),
        }
    }

    /// Link all generated files to one output file (library or binary according
    /// to the selected one)
    fn link(&mut self);

    /// Exit point \
    /// Delete all temporary files and do linking
    fn finish(&mut self) {}

    /// Exit point for each source file
    fn finish_one(&mut self, source: &String);

    /// Data getter
    fn data(&mut self) -> &mut CompilerData;

    // --- ASM code generators

    /// Variable declaration, can 
    fn add_variable(&mut self, variable: Variable);
    fn add_static_variable(&mut self, variable: Variable);
    fn add_function(&mut self, function: Function);

    fn change_variable_value(&mut self, variable: &Variable);

    fn return_(&mut self, value: String);

    fn print(&mut self, to_print: String);
    fn exit(&mut self, value: String);
}
