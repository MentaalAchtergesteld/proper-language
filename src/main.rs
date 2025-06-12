use lexer::{LexerError, Token};
use parser::Parser;

use crate::lexer::Lexer ;

mod cursor;
mod lexer;
mod parser;
mod compiler;

fn main() -> Result<(), ()> {
    const TEST_SRC: &str = r#"
        // Variabele declaraties en assignments
        let x = 42;
        let y = 3.14;
        let s = "hello";
        let b = true;
        let o;                    // zonder initializer

        x = x + 1;
        y += 2.0;
        s = s + " world";
        b = !b;

        // If / else if / else
        if x < 10 {
            x = x * 2;
        } else if x < 100 {
            x -= 5;
        } else {
            x = 0;
        }

        // While en For
        while b && x != 0 {
            x = x - 1;
            if x == 5 { break }
            continue
        }

        for i in 0..=10 {
            let arr = [1, 2, 3, i];
            let obj = { foo: i, bar: "baz", };
            obj.foo;
            arr[2];
            print(arr[i].field());
        }

        // Functie definitie en calls
        fn add(a, b) {
            return a + b;
        }
        let z = add(x, y);

        // Match statement
        match z {
            0 => println("zero"),
            1 => { println("one or two"); },
            _ => println("many"),
        }
    "#;

    let chars = TEST_SRC.chars().collect::<Vec<char>>();

    let lexer = Lexer::new(&chars);
    let tokens = lexer.collect::<Result<Vec<Token>, LexerError>>()
        .map_err(|e| eprintln!("ERROR: couldn't tokenize input: {e:?}"))?
        .iter()
        .filter(|token| !matches!(token, &Token::Comment(_)))
        .map(|token| token.clone())
        .collect::<Vec<Token>>();
    
    println!("Tokens: {tokens:?}");

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse_program()
        .map_err(|e| eprintln!("ERROR: couldn't parse tokens to AST: {e:?}"))?;
    
    println!("");
    println!("AST: {ast:?}");

    Ok(())
}
