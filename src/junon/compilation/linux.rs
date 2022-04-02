// This file is part of "juc"
// All rights reserved
// Copyright (c) Junon, Antonin Hérault

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::junon::{
    args::Args,
    compilation::{
        base,
        data::CompilerData,
        defaults::linux_defaults::*,
        defaults::*,
        objects::{
            function::Function, 
            type_, type_::Type, 
            variable::Variable
        },
        caller::Caller,
    },
    platform,
};

pub struct LinuxCompiler {
    data: CompilerData,
    section_text: Vec<String>,
    section_data: Vec<String>,
}

impl LinuxCompiler {
    pub fn new(data: CompilerData) -> Self {
        Self {
            data,
            section_text: vec![],
            section_data: vec![],
        }
    }
}

impl Caller for LinuxCompiler {}

/// SEE Functions' documentation from `base::Compiler` because they are not
/// written here a new time
impl base::Compiler for LinuxCompiler {
    fn init(&mut self) {
        if self.data().is_library {
            return;
        }

        let to_write: String = format!(
            "section .text\n\
                \tglobal {}\n\
            extern {}\n\
            {}:\n\
                \tcall {}",
            START_FUNCTION, ENTRY_POINT, START_FUNCTION, ENTRY_POINT,
        );

        let path: String = format!("{}/{}", BUILD_FOLDER, START_FILE);
        let path = Path::new(&path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        File::create(path)
            .unwrap() // already checked before
            .write_all(to_write.as_bytes())
            .unwrap();

        platform::exec(
            ASSEMBLER.to_string(),
            /* arguments */
            &[
                format!("{}/{}", BUILD_FOLDER, START_FILE),
                "-f".to_string(),
                "elf64".to_string(),
                "-o".to_string(),
                format!("{}/{}.o", BUILD_FOLDER, START_FILE),
            ],
        );
    }

    fn link(&mut self) {
        let mut bin_filename: String = OUTPUT_FILE.to_string();
        Args::when_flag('o', &self.data().options, |bin_filename_: String| {
            bin_filename = bin_filename_;
        });

        let mut args = vec!["-o".to_string(), bin_filename.to_string()];
        if self.data().is_library {
            args.push("-shared".to_string());
        } else {
            // When it's a library, the start file is not created
            args.push(format!("{}/{}.o", BUILD_FOLDER, START_FILE));
        }

        for source in &self.data().sources {
            args.push(format!("{}/{}.o", BUILD_FOLDER, source));
        }

        platform::exec(LINKER.to_string(), &args);
    }

    fn finish(&mut self) {}

    fn finish_one(&mut self, source: &String) {
        self.write_asm(format!(
            "section .text\n{}",
            self.section_text
                .iter()
                .map(|x| format!("\t{}\n", x)) // function id
                .collect::<String>()
        ));
        self.write_asm(format!(
            "section .data\n{}",
            self.section_data
                .iter()
                .map(|x| format!("\t{}\n", x)) // variable id
                .collect::<String>()
        ));

        // Reset for the next file
        self.section_text = vec![];
        self.section_data = vec![];

        platform::exec(
            ASSEMBLER.to_string(),
            // Arguments
            &[
                format!("{}/{}.asm", BUILD_FOLDER, source),
                // Compiling to elf64 object file type
                "-f".to_string(),
                "elf64".to_string(),
                // The output is the same name than the source file but with
                // an ".o" extension
                "-o".to_string(),
                format!("{}/{}.o", BUILD_FOLDER, source),
            ],
        );
    }

    fn data(&mut self) -> &mut CompilerData {
        &mut self.data
    }

    // --- ASM code generators

    fn add_variable(&mut self, variable: Variable) {
        self.data().variable_stack.insert(
            variable.id().to_string(), 
            variable.clone()
        );

        let init_value: String = variable.current_value().clone();
        let to_write: String = format!(
            "\tmov [rbp - {}], dword {} ; {}", 
            type_::type_size_to_asm(variable.type_().clone()),
            init_value,
            variable.id()
        );
        self.write_asm(to_write);
    }

    fn add_static_variable(&mut self, variable: Variable) {
        let mut init_value: String = variable.current_value().clone();

        // Auto terminate strings by NULL character
        if *variable.type_() == Type::Str && init_value != "0".to_string() {
            init_value = format!("`{}`", init_value);
            init_value += ", 0";
        }

        self.section_data.push(format!(
            "{}: {} {}",
            variable.id(),
            type_::type_to_asm(variable.type_().clone()),
            init_value
        ));
    }

    fn add_function(&mut self, function: Function) {
        self.section_text.push(format!("global {}", function.id()));

        let to_write: String = format!(
            "{}:\n\
            \tpush rbp",
            function.id(),
        );
        self.write_asm(to_write);
    }

    fn change_variable_value(&mut self, variable: &Variable) {
        let variable_size = 4;

        let to_write: String = format!(
            "\tmov [rbp - {}], dword {} ; {}", 
            variable_size,
            variable.current_value(),
            variable.id()
        );
        self.write_asm(to_write);
    }

    fn return_(&mut self, value: String) {
        let to_write: Vec<String> = vec!(
            "nop".to_string(),
            "pop rbp".to_string(),
            format!("mov rax, {}", value),
            "ret".to_string(),
        );

        self.write_asm(
            to_write.iter()
                .map(| x | format!("\t{}\n", x))
                .collect::<String>()
        );
    }
}
