use lexer::{LexerError, Token};
use parser::Parser;

use crate::lexer::Lexer ;

mod cursor;
mod lexer;
mod parser;

fn main() -> Result<(), ()> {
    let input = r#"
        let x = 42;
        let y = 0xFF;
        let z = 0b1010;
        let pi = 3.14;
        let s = "hello";
        x += y * 2;
        if x > 10 {
            print(s);
        }

        let range = 2..5;
        let inclusive_range = 10..=232;
    "#;

    let chars = input.chars().collect::<Vec<char>>();
    let lexer = Lexer::new(&chars);

    let tokens = lexer.collect::<Result<Vec<Token>, LexerError>>()
        .map_err(|e| eprintln!("ERROR: couldn't tokenize input: {e:?}"))?;
    
    println!("Tokens: {tokens:?}");

    let parser = Parser::new(&tokens);

    Ok(())
}
