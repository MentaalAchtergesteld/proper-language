use crate::{astbuilder, lexer};

fn tokenize(source: &str) -> Vec<lexer::Token> {
    use crate::lexer::Lexer;
    let chars = source.chars().collect::<Vec<char>>();
    Lexer::new(&chars)
        .collect::<Result<Vec<_>, _>>()
        .expect("Lexer failed")
}

fn build_ast(tokens: &[lexer::Token]) -> Vec<astbuilder::Statement> {
    use crate::astbuilder::AstBuilder;
    AstBuilder::new(tokens).build().expect("AstBuilder failed")
}

fn parse(source: &str) -> Vec<astbuilder::Statement> {
    let tokens = tokenize(source);
    let ast = build_ast(&tokens);
    ast
}

#[test]
fn test_all_tokens() {
    use crate::lexer::Token;
    let source = r#"
        ()
        {}
        []

        ,
        .
        ;
        _
        :
        ::
        ..
        ?
        ->
        =>

        =
        +
        -
        *
        /
        %

        &
        |
        ^

        &&
        ||
        !

        ==
        !=
        
        <
        >

        69
        4.20
        "Hello, World!"
        true false
        identifier
        
        let
        fn
        struct
        trait
        impl
        enum
        if
        else
        match
        while
        for
        in
        return
        break
        continue
    "#;

    let expected = vec![
        Token::OpenParen,
        Token::CloseParen,
        Token::OpenCurly,
        Token::CloseCurly,
        Token::OpenBracket,
        Token::CloseBracket,

        Token::Comma,
        Token::Dot,
        Token::Semicolon,
        Token::Underscore,
        Token::Colon,
        Token::PathSeperator,
        Token::Range,
        Token::QuestionMark,
        Token::Arrow,
        Token::FatArrow,

        Token::Assign,
        Token::Plus,
        Token::Minus,
        Token::Star,
        Token::Slash,
        Token::Percent,

        Token::Ampersand,
        Token::Pipe,
        Token::Caret,

        Token::And,
        Token::Or,
        Token::Bang,

        Token::Equal,
        Token::NotEqual,

        Token::LessThan,
        Token::GreaterThan,

        Token::IntegerLiteral(69),
        Token::FloatLiteral(4.20),
        Token::StringLiteral("Hello, World!".to_string()),
        Token::BoolLiteral(true),
        Token::BoolLiteral(false),
        Token::Identifier("identifier".to_string()),

        Token::Let,
        Token::Fn,
        Token::Struct,
        Token::Trait,
        Token::Impl,
        Token::Enum,
        Token::If,
        Token::Else,
        Token::Match,
        Token::While,
        Token::For,
        Token::In,
        Token::Return,
        Token::Break,
        Token::Continue,

        Token::Eof, 
    ];

    let tokens = tokenize(source);

    for (i, token) in expected.iter().enumerate() {
        assert_eq!(&tokens[i], token);
    }
}

#[test]
fn test_let_statement() {
    use crate::astbuilder::*;

    let ast = parse("let test: int = 10;");
    match &ast[0] {
        Statement::Let { pattern, type_annotation, value } => {
            assert!(matches!(pattern, Pattern::Identifier(name) if name == "test"));
            assert!(type_annotation.is_some());
            assert!(matches!(value, Expression::Literal(LiteralValue::Integer(10))));
        },
        _ => panic!("Expected Let statement")
    }
    
    let ast = parse("let x = true;");
    match &ast[0] {
        Statement::Let { type_annotation, .. } => {
            assert!(type_annotation.is_none());
        },
        _ => panic!("Expected Let")
    }
}

#[test]
fn test_fn_definition() {
    use crate::astbuilder::*;

    let ast = parse("fn add(a: int, b: int) -> int { a + b }");
    match &ast[0] {
        Statement::FunctionDefinition(func) => {
            assert_eq!(func.signature.name, "add");
            assert_eq!(func.signature.params.len(), 2);

            assert_eq!(func.signature.params[0].name, "a");
            assert_eq!(func.signature.params[1].name, "b");

            assert!(func.signature.return_type.is_some());
            assert!(matches!(func.body, Expression::Block { .. }));
        },
        _ => panic!("Expected FunctionDefinition"),
    }
}

#[test]
fn test_struct_definition() {
    use crate::astbuilder::*;

    let ast = parse("struct Point { x: int, y: int }");
    match &ast[0] {
        Statement::StructDefinition(s) => {
            assert_eq!(s.name, "Point");
            assert_eq!(s.fields.len(), 2);

            assert_eq!(s.fields[0].name, "x");
            assert_eq!(s.fields[1].name, "y");
        },
        _ => panic!("Expected StructDefinition")
    }
}

#[test]
fn test_enum_definition() {
    use crate::astbuilder::*;

    let ast = parse("enum Option<T> { Some(T), None }");
    match &ast[0] {
        Statement::EnumDefinition(e) => {
            assert_eq!(e.name, "Option");
            assert_eq!(e.generics.len(), 1);
            assert_eq!(e.variants.len(), 2);
        },
        _ => panic!("Expected EnumDefinition")
    }

}

#[test]
fn test_trait_definition() {
    use crate::astbuilder::*;

    let ast = parse("trait Position { fn get_position() -> (int, int); }");
    match &ast[0] {
        Statement::TraitDefinition(t) => {
            assert_eq!(t.name, "Position");
            assert_eq!(t.functions.len(), 1);
            assert_eq!(t.functions[0].name, "get_position");
        },
        _ => panic!("Expected TraitDefinition")
    }
}
