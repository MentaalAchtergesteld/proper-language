use crate::lexer::Lexer ;

mod lexer;

fn main() {
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
    "#;

    let chars: Vec<char> = input.chars().collect();
    let lexer = Lexer::new(&chars);

    for token in lexer {
        println!("{:?}", token.map_err(|e| eprintln!("ERROR: Couldn't parse token: {e:?}")).unwrap())
    } 
}
