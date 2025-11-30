use std::fs;

use crate::{desugarer::Desugarer, reporting::Spanned};
pub use crate::{astbuilder::AstBuilder, lexer::{Lexer, LexerError, Token}};

mod reporting;
mod common;
mod peekablecursor;
mod lexer;
mod astbuilder;
mod hir;
mod desugarer;
mod typechecker;

#[cfg(test)]
mod tests;

fn main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<String>>();

    let filepath = args.get(1).ok_or(())
        .map_err(|_| eprintln!("ERROR: Correct usage: {} <filename>", &args[0]))?;

    let source_code = fs::read_to_string(filepath)
        .map_err(|e| eprintln!("ERROR: Couldn't read file '{filepath}': {e}"))?;

    let chars = source_code.chars().collect::<Vec<char>>();

    let tokens = Lexer::new(&chars).collect::<Result<Vec<Spanned<Token>>, Spanned<LexerError>>>()
        .map_err(|e| eprintln!("ERROR: couldn't tokenize source: {:?} at {:?}", e.value, &chars[e.span.start..e.span.end]))?;


    println!("{tokens:?}");

    let ast_tree = AstBuilder::new(&tokens).build()
        .map_err(|e| eprintln!("ERROR: couldn't build AST tree: {e:?}"))?;

    println!("{ast_tree:?}");

    let mut desugarer = Desugarer::new();
    let desugared = ast_tree.iter().map(|s| desugarer.desugar_statement(s.clone())).collect::<Vec<hir::Statement>>();
    println!("{desugared:?}");

    Ok(())
}
