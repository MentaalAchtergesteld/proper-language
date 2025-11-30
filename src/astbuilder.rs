use std::mem::discriminant;

use crate::common::*;
use crate::peekablecursor::PeekableCursor;
use crate::lexer::Token;
use crate::reporting::{Span, Spanned};

#[derive(Clone, Debug)]
pub struct FunctionDefinition {
    pub signature: FunctionSignature,
    pub body: Expression
}

#[derive(Clone, Debug)]
pub struct ImplBlock {
    pub trait_path: Option<Path>,
    pub type_path: Path,
    pub functions: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expression
}


#[derive(Clone, Debug)]
pub enum Statement {
    Let {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        value: Expression
    },

    FunctionDefinition(FunctionDefinition),
    StructDefinition(StructDefinition),
    EnumDefinition(EnumDefinition),
    TraitDefinition(TraitDefinition),
    ImplBlock(ImplBlock),
    ExpressionStatement(Expression),
}

type ExprNode = Spanned<Expression>;
type ExprResult = Result<ExprNode, Spanned<AstError>>;

#[derive(Clone, Debug)]
pub enum Expression {
    Literal(LiteralValue),
    Path(Path),

    If {
        condition: Box<ExprNode>,
        then_branch: Box<ExprNode>,
        else_branch: Option<Box<ExprNode>>
    },
    IfLet {
        pattern: Pattern,
        value: Box<ExprNode>,
        then_branch: Box<ExprNode>,
        else_branch: Option<Box<ExprNode>>
    },

    Match {
        value: Box<ExprNode>,
        arms: Vec<MatchArm>,
    },

    Block(Vec<Statement>),

    While {
        condition: Box<ExprNode>,
        body: Box<Expression>,
    },
    WhileLet {
        pattern: Pattern,
        value: Box<ExprNode>,
        body: Box<ExprNode>
    },

    For {
        pattern: Pattern,
        iterable: Box<ExprNode>,
        body: Box<ExprNode>,
    },

    Closure {
        params: Vec<Pattern>,
        return_type: Option<TypeAnnotation>,
        body: Box<ExprNode>,
    },

    Array(Vec<ExprNode>),
    Tuple(Vec<ExprNode>),

    StructLiteral {
        path: Path,
        fields: Vec<(String, ExprNode)>,
    },

    Assign {
        left: Box<ExprNode>,
        op: AssignmentOperator,
        right: Box<ExprNode>
    },

    Range {
        start: Box<ExprNode>,
        end: Box<ExprNode>
    },

    Binary {
        left: Box<ExprNode>,
        op: BinaryOperator,
        right: Box<ExprNode>
    },

    Unary {
        op: UnaryOperator,
        right: Box<ExprNode>
    },

    Call {
        callee: Box<ExprNode>,
        args: Vec<ExprNode>,
    },

    Index {
        callee: Box<ExprNode>,
        index: Box<ExprNode>
    },

    Field {
        callee: Box<ExprNode>,
        field: String
    },
    Try(Box<ExprNode>),

    Return(Option<Box<ExprNode>>),
    Break,
    Continue,
}

impl Token {
    pub fn prefix_binding_power(&self) -> Option<u8> {
        match self {
            Self::Minus | Self::Bang => Some(19),
            _ => None
        }
    }

    pub fn infix_binding_power(&self) -> Option<(u8, u8)> {
        match self {
            Token::Plus | Token::Minus => Some((11, 12)),
            Token::Star | Token::Slash => Some((13, 14)),
            _ => todo!()
        }
    }
}

#[derive(Debug)]
pub enum AstError {
    ExpectedToken(Token, Token),
    ExpectedPattern(Token),
    ExpectedExpression(Token),
    RepeatedRange,
    UnexpectedEOF,
}

pub struct AstBuilder<'a> {
    tokens: PeekableCursor<'a, Spanned<Token>>
}

impl<'a> AstBuilder<'a> {
    pub fn new(tokens: &'a [Spanned<Token>]) -> Self {
        Self { tokens: PeekableCursor::new(tokens) }
    }

    fn consume_token_or_err(&mut self) -> Result<&Spanned<Token>, Spanned<AstError>> {
        self.tokens.consume()
            .ok_or(Spanned::new(AstError::UnexpectedEOF, Span::default()))
    }

    fn peek_token_or_err(&self) -> Result<&Spanned<Token>, Spanned<AstError>> {
        self.tokens.peek()
            .ok_or(Spanned::new(AstError::UnexpectedEOF, Span::default()))
    }

    fn peek_inner(&self) -> Option<&Token> {
        match self.tokens.peek() {
            Some(s) => Some(&s.value),
            None => None
        }
    }

    fn peek_n_inner(&self, n: usize) -> Option<&Token> {
        match self.tokens.peek_n(n) {
            Some(s) => Some(&s.value),
            None => None
        }
    }

    fn check_next(&self, expected: &Token) -> bool {
        if let Some(t) = self.tokens.peek() {
            discriminant(&t.value) == discriminant(expected)
        } else { false }
    }

    fn check_n(&self, expected: &Token, n: usize) -> bool {
        if let Some(t) = self.tokens.peek_n(n) {
            discriminant(&t.value) == discriminant(expected)
        } else { false }
    }

    fn check_and_consume(&mut self, expected: &Token) -> bool {
        if self.check_next(expected) {
            self.tokens.consume();
            true
        } else { false }
    }

    fn expect(&mut self, expected: &Token) -> Result<&Spanned<Token>, Spanned<AstError>> {
        let token = self.consume_token_or_err()?;

        if discriminant(&token.value) == discriminant(&expected) {
            Ok(token)
        } else {
            Err(Spanned::new(
                AstError::ExpectedToken(expected.clone(), token.value.clone()),
                token.span,
            ))
        }
    }

    fn expect_integer(&mut self) -> Result<Spanned<i32>, Spanned<AstError>> {
        let token = self.consume_token_or_err()?;
        match token.value {
            Token::IntegerLiteral(v) => Ok(Spanned::new(v, token.span)),
            _ => Err(Spanned::new(
                AstError::ExpectedToken(Token::IntegerLiteral(0), token.value.clone()),
                token.span
            ))
        }
    }

    fn expect_float(&mut self) -> Result<Spanned<f32>, Spanned<AstError>> {
        let token = self.consume_token_or_err()?;
        match token.value {
            Token::FloatLiteral(v) => Ok(Spanned::new(v, token.span)),
            _ => Err(Spanned::new(
                AstError::ExpectedToken(Token::FloatLiteral(0.0), token.value.clone()),
                token.span
            ))
        }
    }

    fn expect_bool(&mut self) -> Result<Spanned<bool>, Spanned<AstError>> {
        let token = self.consume_token_or_err()?;
        match token.value {
            Token::BoolLiteral(v) => Ok(Spanned::new(v, token.span)),
            _ => Err(Spanned::new(
                AstError::ExpectedToken(Token::BoolLiteral(false), token.value.clone()),
                token.span
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<Spanned<String>, Spanned<AstError>> {
        let token = self.consume_token_or_err()?;
        match &token.value {
            Token::Identifier(v) => Ok(Spanned::new(v.to_string(), token.span)),
            _ => Err(Spanned::new(
                AstError::ExpectedToken(Token::Identifier(String::new()), token.value.clone()),
                token.span
            ))
        }
    }

    fn expect_string(&mut self) -> Result<Spanned<String>, Spanned<AstError>> {
        let token = self.consume_token_or_err()?;
        match &token.value {
            Token::StringLiteral(v) => Ok(Spanned::new(v.to_string(), token.span)),
            _ => Err(Spanned::new(
                AstError::ExpectedToken(Token::StringLiteral(String::new()), token.value.clone()),
                token.span
            ))
        }
    }

    fn parse_seperated_list<T>(
        &mut self,
        terminator: Token,
        seperator: Token,
        parse_item: impl Fn(&mut Self) -> Result<T, Spanned<AstError>>
    ) -> Result<Vec<T>, Spanned<AstError>> {
        let mut items = Vec::new();

        while self.peek_inner() != Some(&terminator) {
            items.push(parse_item(self)?);

            if self.peek_inner() != Some(&seperator) { break }
            self.expect(&seperator)?;
            
            if self.peek_inner() == Some(&terminator) { break }
        }
        self.expect(&terminator)?;
        Ok(items)
    } 

    fn parse_grouping_or_tuple(&mut self) -> ExprResult {
        let open = self.expect(&Token::OpenParen)?.span;

        if self.check_next(&Token::CloseParen) {
            let close = self.tokens.consume().unwrap();
            return Ok(Spanned::new(Expression::Tuple(vec![]), open.combine(&close.span)))
        }

        let first = self.parse_expression(0)?;

        if !self.check_next(&Token::Comma) {
            let close = self.expect(&Token::CloseParen)?;
            return Ok(Spanned::new(first.value, open.combine(&close.span)))
        }

        self.tokens.consume();

        let mut items = vec![first];
        while !self.check_next(&Token::CloseParen) {
            items.push(self.parse_expression(0)?);
            if !self.check_next(&Token::Comma) { break }
            self.expect(&Token::Comma)?;
        } 

        let close = self.expect(&Token::CloseParen)?;

        Ok(Spanned::new(Expression::Tuple(items), open.combine(&close.span)))
    }

    fn parse_array_literal(&mut self) -> ExprResult {
        let open = self.expect(&Token::OpenBracket)?.span;
        let array = self.parse_seperated_list(Token::CloseBracket, Token::Comma, |p| p.parse_expression(0))?;
        Ok(Spanned::new(
            Expression::Array(array),
            open.extend(self.tokens.position)
        ))
    }

    fn parse_block_expression(&mut self) -> ExprResult {
        let open = self.expect(&Token::OpenCurly)?.span;
        let mut stmts = Vec::new();
        while !self.check_next(&Token::CloseCurly) {
            todo!()
        }
        let close = self.expect(&Token::CloseCurly)?.span;

        Ok(Spanned::new(
            Expression::Block(stmts),
            open.combine(&close)
        ))
    }

    fn parse_function_type(&mut self) -> Result<Spanned<TypeAnnotation>, Spanned<AstError>> {
        let start = self.tokens.consume().unwrap().span;

        self.expect(&Token::OpenParen)?;

        let params = self.parse_seperated_list(
            Token::CloseParen,
            Token::Comma,
            |p| p.parse_type_annotation()
        )?;

        let return_type = if self.check_and_consume(&Token::Arrow) {
            Some(Box::new(self.parse_type_annotation()?))
        } else { None };

        let end_span = match &return_type {
            Some(rt) => rt.span,
            None => params.last().map(|p| p.span).unwrap_or(start)
        };

        Ok(Spanned::new(
            TypeAnnotation::Function { params, return_type },
            start.combine(&end_span)
        ))
    }

    fn parse_type_annotation(&mut self) -> Result<Spanned<TypeAnnotation>, Spanned<AstError>> {
        let token = self.peek_token_or_err()?;

        match token.value {
            Token::OpenBracket => {
                let start = self.tokens.consume().unwrap().span;

                let inner_type = self.parse_type_annotation()?;

                let end = self.expect(&Token::CloseBracket)?;

                let span = start.combine(&end.span);
                Ok(Spanned::new(
                    TypeAnnotation::Array(Box::new(inner_type)),
                    span
                ))
            },
            Token::OpenParen => {
                let start = self.tokens.consume().unwrap().span;
                
                let types = self.parse_seperated_list(
                    Token::CloseParen,
                    Token::Comma,
                    |p| p.parse_type_annotation()
                )?;

                let end_span = types.last().map(|t| t.span).unwrap_or(start);

                Ok(Spanned::new(
                    TypeAnnotation::Tuple(types),
                    start.combine(&end_span),
                ))
            },
            Token::Fn => self.parse_function_type(),
            _ => {
                let path = self.parse_type_path()?;
                let span = path.span;
                Ok(Spanned::new(TypeAnnotation::Path(path), span))
            }
        }
    }

    fn parse_generic_args(&mut self) -> Result<Spanned<Vec<SpannedType>>, Spanned<AstError>> {
        let start = self.expect(&Token::LessThan)?.span;

        let args = self.parse_seperated_list(
            Token::GreaterThan,
            Token::Comma,
            |p| p.parse_type_annotation()
        )?;

        let end = self.expect(&Token::GreaterThan)?.span;

        let span = start.combine(&end);
        Ok(Spanned::new(args, span))
    }

    fn parse_path(&mut self, allow_implicit_generics: bool) -> Result<Spanned<Path>, Spanned<AstError>> {
        let span = Span::from(self.tokens.position..self.tokens.position);
        let mut path = Path::new();
        loop {
            let ident = self.expect_identifier()?;

            let generics = if self.check_n(&Token::PathSeperator, 1) && self.check_n(&Token::LessThan, 2) {
                self.tokens.consume();
                Some(self.parse_generic_args()?)
            } else if allow_implicit_generics && self.check_next(&Token::LessThan) {
                Some(self.parse_generic_args()?)
            } else { None };

            
            span.extend(self.tokens.position);
            path.push(PathSegment::new(ident, generics));

            if !self.check_and_consume(&Token::PathSeperator) { break }
        }

        Ok(Spanned::new(path, span))
    }
    
    fn parse_type_path(&mut self) -> Result<Spanned<Path>, Spanned<AstError>> {
        self.parse_path(true)
    }

    fn parse_expr_path(&mut self) -> Result<Spanned<Path>, Spanned<AstError>> {
        self.parse_path(false)
    }

    fn parse_struct_literal(&mut self, path: Spanned<Path>) -> ExprResult {
        self.expect(&Token::OpenCurly)?;

        let mut fields = Vec::new();
        while !self.check_next(&Token::CloseCurly) {
            let name = self.expect_identifier()?;
            self.expect(&Token::Colon)?;
            let expr = self.parse_expression(0)?;

            fields.push((name.value, expr));

            if !self.check_next(&Token::Comma) { break }
            self.expect(&Token::Comma)?;
        }

        let close = self.expect(&Token::CloseCurly)?.span;
        Ok(Spanned::new(
            Expression::StructLiteral { path: path.value, fields },
            path.span.combine(&close)
        ))
    }

    fn parse_path_or_struct(&mut self) -> ExprResult {
        let path = self.parse_expr_path()?;

        if self.check_next(&Token::OpenCurly) {
            self.parse_struct_literal(path)
        } else {
            Ok(Spanned::new(Expression::Path(path.value), path.span))
        }
    }

    fn parse_pattern(&mut self) -> Result<Spanned<Pattern>, Spanned<AstError>> {
       let token = self.peek_token_or_err()?;

        match token.value {
            Token::Underscore => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Pattern::Wildcard, t.span))
            },
            Token::IntegerLiteral(v) => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Expression::Literal(LiteralValue::Integer(v)), t.span))
            },
            Token::FloatLiteral(v) => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Pattern::Literal(LiteralValue::Float(v)), t.span))
            },
            Token::BoolLiteral(v) => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Pattern::Literal(LiteralValue::Bool(v)), t.span))
            },
            Token::StringLiteral(v) => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Pattern::Literal(LiteralValue::String(v)), t.span))
            },
            Token::OpenParen => {
                let start = self.tokens.consume().unwrap().span;

                let patterns = self.parse_seperated_list(
                    Token::CloseParen,
                    Token::Comma,
                    |p| p.parse_pattern()
                )?;
                let span = start.extend(self.tokens.position);

                Ok(Spanned::new(
                    Pattern::Tuple { path: None, patterns },
                    span
                ))

            }
        }
    }

    fn parse_if_let_expression(&mut self) -> ExprResult {
        let open = self.expect(&Token::If)?.span;
        self.expect(&Token::Let)?;

        let pattern = self.parse_pattern()?;
        self.expect(&Token::Assign)?;

        let value = Box::new(self.parse_expression(0)?);
        let then_branch = Box::new(self.parse_block_expression()?);

        let else_branch = if self.check_next(&Token::Else) {
            Some(Box::new(self.parse_expression(0)?))
        } else { None };

        Ok(Spanned::new(
            Expression::IfLet { pattern, value, then_branch, else_branch },
            open.extend(self.tokens.position)
        ))
    }

    fn parse_if_expression(&mut self) -> ExprResult {
        if self.check_n(&Token::Let, 2) { return self.parse_if_let_expression() }
        let open = self.expect(&Token::If)?.span;

        let condition = Box::new(self.parse_expression(0)?);
        let then_branch = Box::new(self.parse_expression(0)?);

        let else_branch = if self.check_next(&Token::Else) {
            Some(Box::new(self.parse_expression(0)?))
        } else { None };

        Ok(Spanned::new(Expression::If { condition, then_branch, else_branch }, open.extend(self.tokens.position)))
    }
    fn parse_match_expression(&mut self) -> ExprResult { todo!() }
    fn parse_while_expression(&mut self) -> ExprResult { todo!() }
    fn parse_for_expression(&mut self) -> ExprResult { todo!() }

    fn parse_primary(&mut self) -> ExprResult {
        let token = self.peek_token_or_err()?.clone();

        match token.value {
            Token::IntegerLiteral(v) => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Expression::Literal(LiteralValue::Integer(v)), t.span))
            },
            Token::FloatLiteral(v) => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Expression::Literal(LiteralValue::Float(v)), t.span))
            },
            Token::BoolLiteral(v) => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Expression::Literal(LiteralValue::Bool(v)), t.span))
            },
            Token::StringLiteral(v) => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Expression::Literal(LiteralValue::String(v)), t.span))
            },

            Token::OpenParen => self.parse_grouping_or_tuple(),
            Token::OpenBracket => self.parse_array_literal(),
            Token::OpenCurly => self.parse_block_expression(),
            Token::Identifier(_) => self.parse_path_or_struct(),

            Token::If => self.parse_if_expression(),
            Token::Match => self.parse_match_expression(),
            Token::While => self.parse_while_expression(),
            Token::For => self.parse_for_expression(),

            Token::Return => {
                let start_span = self.tokens.consume().unwrap().span;

                let (val, end_span) = if !self.check_next(&Token::Semicolon) && !self.check_next(&Token::CloseCurly) {
                    let expr = self.parse_expression(0)?;
                    let span = expr.span;
                    (Some(Box::new(expr)), span)
                } else {
                    (None, start_span)
                };

                Ok(Spanned::new(Expression::Return(val), start_span.combine(&end_span)))
            },

            Token::Break => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Expression::Break, t.span))
            },
            Token::Continue => {
                let t = self.tokens.consume().unwrap();
                Ok(Spanned::new(Expression::Continue, t.span))
            },
            
            _ => Err(Spanned::new(
                AstError::ExpectedExpression(token.value.clone()),
                token.span
            ))
        }
    }

    fn parse_expression(&mut self, min_bp: u8) -> ExprResult {
        let prefix_bp = self.peek_token_or_err()
            .map(|t| t.value.prefix_binding_power())?;

        let mut lhs = if let Some(r_bp) = prefix_bp {
            let (op, op_span) = {
                let op_token = self.tokens.consume().unwrap();
                let op = UnaryOperator::from_token(&op_token.value).unwrap();
                let span = op_token.span;
                (op, span)
            };

            let right = self.parse_expression(r_bp)?;
            let span = op_span.combine(&right.span);
            Spanned::new(Expression::Unary { op, right: Box::new(right) }, span)
        } else {
            self.parse_primary()?
        };

        while let Some(token) = self.tokens.peek() {
            let (l_bp, r_bp) = match token.value.infix_binding_power() {
                Some(bp) => bp,
                None => break
            };

            if l_bp < min_bp { break }

            let op_token = self.tokens.consume().unwrap().clone();

            lhs = match op_token.value {
                Token::OpenParen => {
                    let args = self.parse_seperated_list(Token::CloseParen, Token::Comma, |p| p.parse_expression(0))?;
                    Spanned::new(Expression::Call {
                        callee: Box::new(lhs),
                        args
                    }, op_token.span.start..self.tokens.position)
                },
                Token::OpenBracket => {
                    let index = self.parse_expression(0)?;
                    self.expect(&Token::CloseBracket)?;

                    let span = lhs.span.combine(&index.span);
                    Spanned::new(Expression::Index {
                        callee: Box::new(lhs),
                        index: Box::new(index)
                    }, span)
                },
                Token::Dot => {
                    let field = self.expect_identifier()?;

                    let span = lhs.span.combine(&field.span);
                    Spanned::new(Expression::Field {
                        callee: Box::new(lhs),
                        field: field.value
                    }, span)
                },
                Token::QuestionMark => {
                    let span = lhs.span;
                    Spanned::new(Expression::Try(Box::new(lhs)), span)
                },
                _ => {
                    let right = self.parse_expression(r_bp)?;
                    let op = match op_token.value {
                        Token::Plus => BinaryOperator::Add,
                        Token::Minus => BinaryOperator::Subtract,
                        Token::Star => BinaryOperator::Multiply,
                        Token::Slash => BinaryOperator::Divide,
                        Token::Percent => BinaryOperator::Modulo,
                        Token::Equal => BinaryOperator::Equal,
                        Token::NotEqual => BinaryOperator::NotEqual,
                        Token::And => BinaryOperator::And,
                        Token::Or => BinaryOperator::Or,
                        Token::Ampersand => BinaryOperator::BitwiseAnd,
                        Token::Pipe => BinaryOperator::BitwiseOr,
                        Token::Caret => BinaryOperator::BitwiseXor,
                        Token::GreaterThan => match self.peek_inner() {
                            Some(Token::GreaterThan) => { self.tokens.consume(); BinaryOperator::RightShift },
                            Some(Token::Assign)      => { self.tokens.consume(); BinaryOperator::GreaterEqual },
                            _ => BinaryOperator::GreaterThan,
                        },
                        Token::LessThan => match self.peek_inner() {
                            Some(Token::LessThan) => { self.tokens.consume(); BinaryOperator::LeftShift },
                            Some(Token::Assign)   => { self.tokens.consume(); BinaryOperator::LessEqual },
                            _ => BinaryOperator::LessThan,
                        },
                        _ => unreachable!()
                    };

                    let span = lhs.span.combine(&right.span);
                    Spanned::new(Expression::Binary {
                        left: Box::new(lhs),
                        op,
                        right: Box::new(right)
                    }, span)
                }
            };
        }

        Ok(lhs)
    }
}
