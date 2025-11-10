use std::fs;

use crate::{astbuilder::AstBuilder, lexer::{Lexer, LexerError, Token}};

mod peekablecursor;
mod astbuilder;
mod lexer;

fn main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<String>>();

    let filepath = args.get(1).ok_or(())
        .map_err(|_| eprintln!("ERROR: Correct usage: {} <filename>", &args[0]))?;

    let source_code = fs::read_to_string(filepath)
        .map_err(|e| eprintln!("ERROR: Couldn't read file '{filepath}': {e}"))?;

    let chars = source_code.chars().collect::<Vec<char>>();

    let tokens = Lexer::new(&chars).collect::<Result<Vec<Token>, LexerError>>()
        .map_err(|e| eprintln!("ERROR: couldn't tokenize source: {e:?}"))?;

    println!("{tokens:?}");

    let ast_tree = AstBuilder::new(&tokens).build()
        .map_err(|e| eprintln!("ERROR: couldn't build AST tree: {e:?}"))?;

    println!("{ast_tree:?}");


    Ok(())
}
