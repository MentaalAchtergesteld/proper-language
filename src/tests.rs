use std::{fs, panic, path::Path};

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

#[test]
fn test_impl_block() {
    use crate::astbuilder::*;
    let ast = parse("impl Point { fn get_position() -> (int, int) { (self.x, self.y) } }");
    match &ast[0] {
        Statement::ImplBlock(i) => {
            assert!(i.trait_path.is_none());
            assert_eq!(i.type_path.get_first_name().unwrap(), "Point");
            assert_eq!(i.functions.len(), 1);
        },
        _ => panic!("Expected ImplBlock")
    }

    let ast = parse("impl Position for Point { fn get_position() -> (int, int) { (self.x, self.y) } }");
    match &ast[0] {
        Statement::ImplBlock(i) => {
            assert!(i.trait_path.is_some());
            let trait_path = i.trait_path.clone().unwrap();
            assert_eq!(trait_path.get_first_name().unwrap(), "Position");
            assert_eq!(i.functions.len(), 1);
        },
        _ => panic!("Expected ImplBlock")
    }
}

#[test]
fn test_return() {
    use crate::astbuilder::*;

    // Return without keyword
    let ast = parse("10 + 200");
    match &ast[0] {
        Statement::Return(expr) => {
            let (left, op, right) = if let Expression::Binary { left, op, right } = expr {
                (left, op, right)
            } else { panic!("Expected BinaryExpression") };

            assert!(matches!(&**left, Expression::Literal(LiteralValue::Integer(10))));
            assert!(matches!(op, BinaryOperator::Add));
            assert!(matches!(&**right, Expression::Literal(LiteralValue::Integer(200))));
        },
        _ => panic!("Expected Return")
    }

    let ast = parse("return 500 * 30");
    match &ast[0] {
        Statement::Return(expr) => {
            let (left, op, right) = if let Expression::Binary { left, op, right } = expr {
                (left, op, right)
            } else { panic!("Expected BinaryExpression") };

            assert!(matches!(&**left, Expression::Literal(LiteralValue::Integer(500))));
            assert!(matches!(op, BinaryOperator::Multiply));
            assert!(matches!(&**right, Expression::Literal(LiteralValue::Integer(30))));
        },
        _ => panic!("Expected Return")
    }
}

#[test]
fn test_operator_precedence() {
    use crate::astbuilder::*;

    let ast = parse("1 + 2 * 3;");
    let expr = match &ast[0] {
        Statement::ExpressionStatement(expr) => expr,
        _ => panic!("Expected ExpressionStatement")
    };

    let (left, op, right) = match expr {
        Expression::Binary { left, op, right } => (left, op, right),
        _ => panic!("Expected BinaryExpression")
    };
    assert!(matches!(op, BinaryOperator::Add));
    assert!(matches!(**left, Expression::Literal(LiteralValue::Integer(1))));

    let (left, op, right) = match &**right {
        Expression::Binary { left, op, right } => (left, op, right),
        _ => panic!("Expected BinaryExpression")
    };
    assert!(matches!(op, BinaryOperator::Multiply));
    assert!(matches!(**left, Expression::Literal(LiteralValue::Integer(2))));
    assert!(matches!(**right, Expression::Literal(LiteralValue::Integer(3))));

    let ast = parse("(1 + 2) * 3;");
    let expr = match &ast[0] {
        Statement::ExpressionStatement(expr) => expr,
        _ => panic!("Expected ExpressionStatement")
    };

    let op = match expr {
        Expression::Binary { op, .. } => op,
        _ => panic!("Expected BinaryExpression")
    };
    assert!(matches!(op, BinaryOperator::Multiply));
}

#[test]
fn test_postfix_chains() {
    use crate::astbuilder::*;

    let ast = parse("a.b[0]();");

    let expr = match &ast[0] {
        Statement::ExpressionStatement(expr) => expr,
        _ => panic!("Expected ExpressionStatement")
    };

    let call_callee = match expr {
        Expression::Call { callee, .. } => callee,
        _ => panic!("Expected CallExpression")
    };

    let index_callee = match &**call_callee {
        Expression::Index { callee, .. } => callee,
        _ => panic!("Expected IndexExpression")
    };

    let field = match &**index_callee {
        Expression::Field { field, .. } => field,
        _ => panic!("Expected FieldExpression")
    };
    assert_eq!(field, "b");
}

#[test]
fn test_control_flow() {
    use crate::astbuilder::*;

    let ast = parse("if x { 1 } else { 2 };");

    match &ast[0] {
        Statement::ExpressionStatement(Expression::If { condition, then_branch, else_branch }) => {
            assert!(matches!(**condition, Expression::Path(..)));
            assert!(matches!(**then_branch, Expression::Block { .. }));
            assert!(else_branch.is_some());
        },
        _ => panic!("Expected IfExpression")
    }

    let ast = parse("while true { break; };");
    match &ast[0] {
        Statement::ExpressionStatement(Expression::While { condition, body }) => {
            assert!(matches!(**condition, Expression::Literal(LiteralValue::Bool(true))));

            match &**body {
                Expression::Block { statements, .. } => assert!(matches!(statements[0], Statement::Break)),
                _ => panic!("Expected BlockExpression")
            };
        },
        _ => panic!("Expected WhileExpression")
    }
}

#[test]
fn test_match_expression() {
    use crate::astbuilder::*;

    let ast = parse("match x { 1 => true, _ => false };");

    match &ast[0] {
        Statement::ExpressionStatement(Expression::Match { value, arms }) => {
            assert!(matches!(**value, Expression::Path(..)));
            assert_eq!(arms.len(), 2);

            let arm1 = &arms[0];
            assert!(matches!(arm1.pattern, Pattern::Literal(LiteralValue::Integer(1))));
            assert!(matches!(arm1.body, Expression::Literal(LiteralValue::Bool(true))));

            let arm2 = &arms[1];
            assert!(matches!(arm2.pattern, Pattern::Wildcard));
        },
        _ => panic!("Expected MatchExpression")
    }
}

#[test]
fn test_generics_parsing() {
    use crate::astbuilder::*;

    let ast = parse("let a: Vec<Vec<int>> = [];");
    match &ast[0] {
        Statement::Let { type_annotation, .. } => {
            assert!(type_annotation.is_some());
        },
        _ => panic!("Expected LetStatement"),
    }

    let ast = parse("1 >> 2;");
    match &ast[0] {
        Statement::ExpressionStatement(expr) => match expr {
            Expression::Binary { op, .. } => {
                assert!(matches!(op, BinaryOperator::RightShift));
            }
            _ => panic!("Expected BinaryExpression"),
        }
        _ => panic!("Expected ExpressionStatement"),
    }
}


#[test]
fn test_turbofish() {
    use crate::astbuilder::*;

    let ast = parse("size_of::<int>();");
    match &ast[0] {
        Statement::ExpressionStatement(Expression::Call { callee, .. }) => {
            match &**callee {
                Expression::Path(path) => {
                    let segment = path.segments.last().unwrap();
                    assert_eq!(segment.ident, "size_of");
                    assert!(segment.generic_args.is_some());
                }
                _ => panic!("Expected PathExpression"),
            }
        }
        _ => panic!("Expected CallExpression"),
    }

    let ast = parse("let p = Point::<int> { x: 10 };");
    match &ast[0] {
        Statement::Let { value, .. } => {
             match value {
                 Expression::StructLiteral { path, .. } => {
                     let segment = path.segments.last().unwrap();
                     assert_eq!(segment.ident, "Point");
                     assert!(segment.generic_args.is_some());
                 }
                 _ => panic!("Expected StructLiteral"),
             }
        }
        _ => panic!("Expected LetStatement"),
    }
}

#[test]
fn test_if_let_while_let() {
    use crate::astbuilder::*;

    let ast = parse("if let Some(x) = opt { x };");
    match &ast[0] {
        Statement::ExpressionStatement(Expression::IfLet { pattern, value, .. }) => {
            assert!(matches!(pattern, Pattern::Tuple { .. })); 
            assert!(matches!(**value, Expression::Path(..)));
        }
        _ => panic!("Expected IfLetStatement"),
    }

    let ast = parse("while let Some(x) = iter.next() { print(x); };");
    match &ast[0] {
        Statement::ExpressionStatement(Expression::WhileLet { .. }) => {}
        _ => panic!("Expected WhileLetStatement"),
    }
}

#[test]
fn test_for_loop() {
    use crate::astbuilder::*;

    let ast = parse("for i in 0..10 { };");
    match &ast[0] {
        Statement::ExpressionStatement(Expression::For { pattern, iterable, .. }) => {
            assert!(matches!(pattern, Pattern::Identifier(..)));
            assert!(matches!(**iterable, Expression::Range { .. }));
        }
        _ => panic!("Expected ForExpression"),
    }
}

#[test]
fn test_unary_operators() {
    use crate::astbuilder::*;

    let ast = parse("let x = -10 + !true;");
    match &ast[0] {
        Statement::Let { value, .. } => {
            match value {
                Expression::Binary { left, right, .. } => {
                    match &**left {
                        Expression::Unary { op, right: val, .. } => {
                            assert!(matches!(op, UnaryOperator::Negate));
                            assert!(matches!(**val, Expression::Literal(LiteralValue::Integer(10))));
                        }
                        _ => panic!("Expected UnaryExpression"),
                    }
                    match &**right {
                        Expression::Unary { op, right: val, .. } => {
                            assert!(matches!(op, UnaryOperator::Not));
                            assert!(matches!(**val, Expression::Literal(LiteralValue::Bool(true))));
                        }
                        _ => panic!("Expected UnaryExpression"),
                    }
                }
                _ => panic!("Expected BinaryExpression"),
            }
        }
        _ => panic!("Expected LetStatement"),
    }
}

#[test]
fn test_parse_all_features_file() {
    use astbuilder::*;
    use lexer::*;
    
    let path = std::path::Path::new("./tests/test.pr");
    let source = fs::read_to_string(path).expect("Can't open test file");

    println!("Parsing file: {:?}", path);

    let char_vec: Vec<char> = source.chars().collect();
    let lexer = Lexer::new(&char_vec);
    
    let tokens: Vec<_> = lexer.collect::<Result<_, _>>()
        .expect("Lexer failed");

    let mut parser = AstBuilder::new(&tokens);
    let ast = parser.build();

    match ast {
        Ok(statements) => {
            println!("Succesfully parsed. Statement count: {}", statements.len());
        },
        Err(e) => {
            panic!("Parser failed: {:?}", e);
        }
    }
}
