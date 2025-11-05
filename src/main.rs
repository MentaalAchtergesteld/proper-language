use std::{fs::{self, File}, io::BufWriter};

use crate::{astbuilder::ASTBuilder, compiler::{disassemble_program, Compiler}, lexer::{Lexer, LexerError, Token}, vm::VM};

mod lexer;
mod astbuilder;
mod compiler;
mod vm;

#[derive(Debug, Clone, Copy)]
pub struct TokenFrame {
    line: usize,
    column: usize,
    length: usize,
}

impl TokenFrame {
    pub fn print_source_context(&self, source: &str) {
        let line_str = match source.lines().nth(self.line) {
            Some(line) => line.replace("\t", " "),
            None => return,
        };

        let line_num = (self.line + 1).to_string();
        let padding_width = line_num.len();

        eprintln!("{:>width$} | {}", line_num, line_str, width = padding_width);

        let indicator_padding = " ".repeat(self.column);
        let indicator = "^".repeat(self.length.max(1));
        
        eprintln!("{} | {}{}", 
            " ".repeat(padding_width),
            indicator_padding,
            indicator
        );
    }
}

fn main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<String>>();

    let filepath = args.get(1).ok_or(())
        .map_err(|_| eprintln!("ERROR: Correct usage: {} <filename>", &args[0]))?;

    let source_code = fs::read_to_string(filepath)
        .map_err(|e| eprintln!("ERROR: Couldn't read file '{filepath}': {e}"))?;

    let (tokens, token_frames) = Lexer::new(&source_code).collect::<Result<(Vec<Token>, Vec<TokenFrame>), LexerError>>()
        .map_err(|e| e.print_with_source(&source_code))?;

    let tree = ASTBuilder::new(tokens, token_frames).build()
        .map_err(|e| e.print_with_source(&source_code))?;

    let main_proto = Compiler::new().compile(tree)
        .map_err(|e| eprintln!("ERROR: couldn't compile AST: {e:?}"))?;

    let file = File::create("./output.asm")
        .map_err(|e| eprintln!("ERROR: couldn't create output file: {e}"))?;
    let mut writer = BufWriter::new(file);
    disassemble_program(&main_proto, &mut writer)
        .map_err(|e| eprintln!("ERROR: couldn't write disassembled program: {e}"))?;

    VM::new(main_proto).run();

    Ok(())
}
