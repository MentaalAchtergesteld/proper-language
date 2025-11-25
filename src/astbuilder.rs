use crate::peekablecursor::PeekableCursor;
use crate::lexer::Token;

#[derive(Clone, Debug)]
pub enum LiteralValue {
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Empty,
}

#[derive(Clone, Debug)]
pub enum BinaryOperator {
    Add, Subtract, Multiply, Divide, Modulo,
    Equal, NotEqual, GreaterThan, GreaterEqual, LessThan, LessEqual,
    And, Or,
    BitwiseAnd, BitwiseOr, BitwiseXor, LeftShift, RightShift
}

#[derive(Clone, Debug)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LeftShiftAssign,
    RightShiftAssign,
}

#[derive(Clone, Debug)]
pub enum UnaryOperator {
    Negate,
    Not
}

#[derive(Clone, Debug)]
pub enum TypeAnnotation {
    Path(Path),
    Array(Box<TypeAnnotation>),
    Tuple(Vec<TypeAnnotation>),
    Function {
        params: Vec<TypeAnnotation>,
        return_type: Box<TypeAnnotation>
    }
}

#[derive(Clone, Debug)]
pub struct PathSegment {
    pub ident: String,
    pub generic_args: Option<Vec<TypeAnnotation>>
}

#[derive(Clone, Debug)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

impl Path {
    pub fn is_simple(&self) -> bool {
        if self.segments.len() > 1 { return false }

        match self.segments.first() {
            Some(segment) => segment.generic_args.is_none(),
            None => true
        }
    }

    pub fn get_first_name(&self) -> Option<String> {
        self.segments.first().map(|s| s.ident.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Clone, Debug)]
pub struct FunctionDefinition {
    pub signature: FunctionSignature,
    pub body: Expression
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub type_annotation: TypeAnnotation
}

#[derive(Clone, Debug)]
pub struct StructDefinition {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug)]
pub enum EnumPayload {
    Tuple(Vec<TypeAnnotation>),
    Struct(Vec<Field>)
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    name: String,
    payload: Option<EnumPayload>
}

#[derive(Clone, Debug)]
pub struct EnumDefinition {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Clone, Debug)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeAnnotation>,
}

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
}

#[derive(Clone, Debug)]
pub struct TraitDefinition {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub functions: Vec<FunctionSignature>,
}

#[derive(Clone, Debug)]
pub struct ImplBlock {
    pub trait_path: Option<Path>,
    pub type_path: Path,
    pub functions: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Literal(LiteralValue),
    Path(Path),
    Identifier(String),
    Wildcard,
    Tuple {
        path: Option<Path>,
        patterns: Vec<Pattern>,
    },

    Struct {
        path: Path,
        fields: Vec<String>
    }
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expression
}


#[derive(Clone, Debug)]
pub enum Statement {
    Let {
        pattern: Pattern,
        type_annotation: Option<TypeAnnotation>,
        value: Expression
    },

    FunctionDefinition(FunctionDefinition),
    StructDefinition(StructDefinition),
    EnumDefinition(EnumDefinition),
    TraitDefinition(TraitDefinition),
    ImplBlock(ImplBlock),
    ExpressionStatement(Expression),
    Return(Expression),
    Break,
    Continue,
}

#[derive(Clone, Debug)]
pub enum Expression {
    Literal(LiteralValue),
    Path(Path),

    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>
    },
    IfLet {
        pattern: Pattern,
        value: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>
    },

    Match {
        value: Box<Expression>,
        arms: Vec<MatchArm>,
    },

    Block {
        statements: Vec<Statement>,
        result: Option<Box<Expression>>
    },

    While {
        condition: Box<Expression>,
        body: Box<Expression>,
    },
    WhileLet {
        pattern: Pattern,
        value: Box<Expression>,
        body: Box<Expression>
    },

    For {
        pattern: Pattern,
        iterable: Box<Expression>,
        body: Box<Expression>,
    },

    Closure {
        params: Vec<Pattern>,
        return_type: Option<TypeAnnotation>,
        body: Box<Expression>,
    },

    Array(Vec<Expression>),
    Tuple(Vec<Expression>),

    StructLiteral {
        path: Path,
        fields: Vec<(String, Expression)>,
    },

    Assign {
        left: Box<Expression>,
        op: AssignmentOperator,
        right: Box<Expression>
    },

    Range {
        start: Box<Expression>,
        end: Box<Expression>
    },

    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>
    },

    Unary {
        op: UnaryOperator,
        right: Box<Expression>
    },

    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },

    Index {
        callee: Box<Expression>,
        index: Box<Expression>
    },

    Field {
        callee: Box<Expression>,
        field: String
    },

    Try(Box<Expression>)
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
    tokens: PeekableCursor<'a, Token>
}

impl<'a> AstBuilder<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens: PeekableCursor::new(tokens) }
    }

    fn expect(&mut self, expected: Token) -> Result<&Token, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        if *token == expected {
            Ok(token)
        } else {
            Err(AstError::ExpectedToken(expected.clone(), token.clone()))
        }
    }

    fn expect_integer(&mut self) -> Result<i32, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        match token {
            Token::IntegerLiteral(v) => Ok(*v),
            _ => Err(AstError::ExpectedToken(Token::IntegerLiteral(0), token.clone()))
        }
    }

    fn expect_float(&mut self) -> Result<f32, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        match token {
            Token::FloatLiteral(v) => Ok(*v),
            _ => Err(AstError::ExpectedToken(Token::FloatLiteral(0.), token.clone()))
        }
    }

    fn expect_bool(&mut self) -> Result<bool, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        match token {
            Token::BoolLiteral(v) => Ok(*v),
            _ => Err(AstError::ExpectedToken(Token::BoolLiteral(false), token.clone()))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        match token {
            Token::Identifier(v) => Ok(v.clone()),
            _ => Err(AstError::ExpectedToken(Token::Identifier(String::new()), token.clone()))
        }
    }

    fn expect_string(&mut self) -> Result<String, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        match token {
            Token::StringLiteral(v) => Ok(v.clone()),
            _ => Err(AstError::ExpectedToken(Token::StringLiteral(String::new()), token.clone()))
        }
    }
    
    fn parse_struct_pattern_fields(&mut self) -> Result<Vec<String>, AstError> {
        self.expect(Token::OpenCurly)?;
        let mut fields = Vec::new();
        while self.tokens.peek() != Some(&Token::CloseCurly) {
            let field = self.expect_identifier()?;
            fields.push(field);

            if self.tokens.peek() != Some(&Token::Comma) { break }
            self.expect(Token::Comma)?;
        }
        self.expect(Token::CloseCurly)?;
        Ok(fields)
    }

    fn parse_comma_list<T>(&mut self, end_token: Token, parse_item: impl Fn(&mut Self) -> Result<T, AstError>) -> Result<Vec<T>, AstError> {
        let mut items = Vec::new();

        while self.tokens.peek() != Some(&end_token) {
            items.push(parse_item(self)?);
            if self.tokens.peek() != Some(&Token::Comma) { break }
            self.tokens.consume();
        }
        self.expect(end_token)?;

        Ok(items)
    }

    fn parse_pattern_list(&mut self) -> Result<Vec<Pattern>, AstError> {
        self.expect(Token::OpenParen)?;
        let mut patterns = Vec::new();
        while self.tokens.peek() != Some(&Token::CloseParen) {
            let pattern = self.parse_pattern()?;
            patterns.push(pattern);

            if self.tokens.peek() != Some(&Token::Comma) { break}
            self.expect(Token::Comma)?;
        }
        self.expect(Token::CloseParen)?;
        Ok(patterns)
    }

    fn parse_pattern(&mut self) -> Result<Pattern, AstError> {
        match self.tokens.peek() {
            Some(Token::IntegerLiteral(_)) |
            Some(Token::FloatLiteral(_)) |
            Some(Token::StringLiteral(_)) |
            Some(Token::BoolLiteral(_)) => {
                let token = self.tokens.consume().unwrap();
                let literal_value = match token {
                    Token::IntegerLiteral(v) => LiteralValue::Integer(*v),
                    Token::FloatLiteral(v) => LiteralValue::Float(*v),
                    Token::StringLiteral(v) => LiteralValue::String(v.clone()),
                    Token::BoolLiteral(v) => LiteralValue::Bool(*v),
                    _ => unreachable!()
                };
                Ok(Pattern::Literal(literal_value))
            },
            Some(Token::Underscore) => {
                self.tokens.consume();
                Ok(Pattern::Wildcard)
            },
            Some(Token::Identifier(_)) => {
                let path = self.parse_expr_path()?;

                match self.tokens.peek() {
                    Some(Token::OpenCurly) => {
                        let fields = self.parse_struct_pattern_fields()?;
                        Ok(Pattern::Struct { path, fields })
                    },
                    Some(Token::OpenParen) => {
                        let patterns = self.parse_pattern_list()?;
                        Ok(Pattern::Tuple { path: Some(path), patterns })
                    },
                    _ => {
                        if path.is_simple() {
                            Ok(Pattern::Identifier(path.get_first_name().unwrap()))
                        } else {
                            Ok(Pattern::Path(path))
                        }
                    }
                }
            },
            Some(Token::OpenParen) => {
                self.tokens.consume();
                let patterns = self.parse_pattern_list()?;
                Ok(Pattern::Tuple { path: None, patterns })
            },
            _ => {
                let token = self.tokens.consume().unwrap_or(&Token::Eof);
                Err(AstError::ExpectedPattern(token.clone()))
            }
        }
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, AstError> {
        let mut type_ann = match self.tokens.peek() {
            Some(&Token::OpenParen) => {
                self.tokens.consume();
                let types = self.parse_comma_list(
                    Token::CloseParen,
                    |this| this.parse_type_annotation()
                )?;
                TypeAnnotation::Tuple(types)
            },
            Some(&Token::Fn) => {
                self.tokens.consume();
                let params = self.parse_comma_list(
                    Token::CloseParen,
                    |this| this.parse_type_annotation()
                )?;

                let return_type = if self.tokens.peek() == Some(&Token::Arrow) {
                    self.tokens.consume();
                    Box::new(self.parse_type_annotation()?)
                } else {
                    Box::new(TypeAnnotation::Tuple(Vec::new()))
                };

                TypeAnnotation::Function { params, return_type }
            },
            _ => TypeAnnotation::Path(self.parse_type_path()?)
        };

        while self.tokens.peek() == Some(&Token::OpenBracket) {
            self.tokens.consume();
            self.expect(Token::CloseBracket)?;

            type_ann = TypeAnnotation::Array(Box::new(type_ann));
        }

        Ok(type_ann)
    }

    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, AstError> {
        if self.tokens.peek() != Some(&Token::LessThan) { return Ok(Vec::new()) }
        self.expect(Token::LessThan)?;

        let mut params = Vec::new();

        while self.tokens.peek() != Some(&Token::GreaterThan) {
            let name = self.expect_identifier()?;
            let mut bounds = Vec::new();
            if self.tokens.peek() == Some(&Token::Colon) {
                self.tokens.consume();

                loop {
                    bounds.push(self.parse_type_annotation()?);
                    if self.tokens.peek() != Some(&Token::Plus) {
                        break;
                    }
                    self.tokens.consume();
                }
            }
            params.push(GenericParam { name, bounds });

            if self.tokens.peek() != Some(&Token::Comma) { break }
            self.expect(Token::Comma)?;
        }
        self.expect(Token::GreaterThan)?;
        Ok(params)

    }

    fn parse_generic_args(&mut self) -> Result<Vec<TypeAnnotation>, AstError> {
        self.expect(Token::LessThan)?;
        let mut args = Vec::new();
        while self.tokens.peek() != Some(&Token::GreaterThan) {
            args.push(self.parse_type_annotation()?); 
            if self.tokens.peek() != Some(&Token::Comma) { break }
            self.expect(Token::Comma)?;
        }
        self.expect(Token::GreaterThan)?;
        Ok(args)
    }

    fn parse_path_internal(&mut self, allow_implicit_generics: bool) -> Result<Path, AstError> {
        let mut segments = Vec::new();

        loop {
            let ident = self.expect_identifier()?;
            let mut generic_args = None;

            if self.tokens.peek() == Some(&Token::PathSeperator)
            && self.tokens.peek_n(1) == Some(&Token::LessThan) {
                self.tokens.consume();
                generic_args = Some(self.parse_generic_args()?);
            } else if allow_implicit_generics && self.tokens.peek() == Some(&Token::LessThan) {
                generic_args = Some(self.parse_generic_args()?);
            }

            segments.push(PathSegment { ident, generic_args });

            if self.tokens.peek() == Some(&Token::PathSeperator) {
                self.tokens.consume();
            }  else { break; }
        }

        Ok(Path { segments })
    }

    fn parse_type_path(&mut self) -> Result<Path, AstError> { self.parse_path_internal(true) }
    fn parse_expr_path(&mut self) -> Result<Path, AstError> { self.parse_path_internal(false) }

    fn parse_block(&mut self) -> Result<Expression, AstError> {
        let mut statements = Vec::new();
        self.expect(Token::OpenCurly)?;
        while self.tokens.peek() != Some(&Token::CloseCurly) {
            let statement = self.parse_statement()?;
            statements.push(statement);
        }
        self.expect(Token::CloseCurly)?;

        let result = match statements.last() {
            Some(Statement::Return(value)) => Some(Box::new(value.clone())),
            _ => None
        };

        Ok(Expression::Block { statements, result })
    }

    fn parse_field_block(&mut self) -> Result<Vec<Field>, AstError> {
        let mut fields = Vec::new();

        self.expect(Token::OpenCurly)?;
        while self.tokens.peek() != Some(&Token::CloseCurly) {
            let name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let type_annotation = self.parse_type_annotation()?;

            fields.push(Field { name, type_annotation });
            if self.tokens.peek() != Some(&Token::Comma) { break }
            self.expect(Token::Comma)?;
        }
        self.expect(Token::CloseCurly)?;
        Ok(fields)
    }

    fn parse_function_signature(&mut self) -> Result<FunctionSignature, AstError> {
        self.expect(Token::Fn)?;
        let name = self.expect_identifier()?;
        let generics = self.parse_generic_params()?;

        let mut params = Vec::new();

        self.expect(Token::OpenParen)?;
        while self.tokens.peek() != Some(&Token::CloseParen) {
            let name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let type_annotation = self.parse_type_annotation()?;
            params.push(Parameter { name, type_annotation });
            if self.tokens.peek() != Some(&Token::Comma) { break; }
            self.expect(Token::Comma)?;
        }
        self.expect(Token::CloseParen)?;

        let return_type = match self.tokens.peek() {
            Some(&Token::Arrow) => {
                self.tokens.consume();
                Some(self.parse_type_annotation()?)
            },
            _ => None
        };

        Ok(FunctionSignature { name, generics, params, return_type })
    }

    fn parse_binary(
        &mut self,
        next_level: fn(&mut Self) -> Result<Expression, AstError>,
        mut map_token_to_op: impl FnMut(&PeekableCursor<Token>) -> Option<(BinaryOperator, usize)>,
    ) -> Result<Expression, AstError> {
        let mut left = next_level(self)?;

        while let Some(_) = self.tokens.peek() {
            let (op, to_consume) = match map_token_to_op(&self.tokens) {
                Some(t) => t,
                None => break
            };

            self.tokens.consume_n(to_consume);

            let right = next_level(self)?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right)
            }
        }
        Ok(left)
    }

    fn parse_if(&mut self) -> Result<Expression, AstError> {
        self.expect(Token::If)?;

        let mut expr = if self.tokens.peek() == Some(&Token::Let) {
            self.tokens.consume();
            let pattern = self.parse_pattern()?;
            self.expect(Token::Assign)?;
            let value = Box::new(self.parse_expression()?);

            let then_branch = Box::new(self.parse_expression()?);
            Expression::IfLet { pattern, value, then_branch, else_branch: None }
        } else {
            let condition = Box::new(self.parse_expression()?);
            let then_branch = Box::new(self.parse_expression()?);

            Expression::If { condition, then_branch, else_branch: None }
        };

        let found_else_branch = if self.tokens.peek() == Some(&Token::Else) {
            self.tokens.consume();
            Some(Box::new(self.parse_expression()?))
        } else { None };

        match &mut expr {
            Expression::If { else_branch, .. } => *else_branch = found_else_branch,
            Expression::IfLet { else_branch, .. } => *else_branch = found_else_branch,
            _ => unreachable!()
        };

        Ok(expr)
    }

    fn parse_match(&mut self) -> Result<Expression, AstError> {
        self.expect(Token::Match)?;

        let value = Box::new(self.parse_expression()?);

        let mut arms = Vec::new();

        self.expect(Token::OpenCurly)?;
        while self.tokens.peek() != Some(&Token::CloseCurly) {
            let pattern = self.parse_pattern()?;
            self.expect(Token::FatArrow)?;
            let body = self.parse_expression()?;

            arms.push(MatchArm { pattern, body });
            
            if self.tokens.peek() != Some(&Token::Comma) { break }
            self.tokens.consume();
        }
        self.expect(Token::CloseCurly)?;

        Ok(Expression::Match { value, arms })
    }

    fn parse_while(&mut self) -> Result<Expression, AstError> {
        self.expect(Token::While)?;

        if self.tokens.peek() == Some(&Token::Let) {
            self.tokens.consume();
            let pattern = self.parse_pattern()?;
            self.expect(Token::Assign)?;
            let value = Box::new(self.parse_expression()?);
            let body = Box::new(self.parse_expression()?);

            Ok(Expression::WhileLet { pattern, value, body })
        } else {
            let condition = Box::new(self.parse_expression()?);
            let body = Box::new(self.parse_expression()?);

            Ok(Expression::While { condition, body })
        }
    }

    fn parse_for(&mut self) -> Result<Expression, AstError> {
        self.expect(Token::For)?;

        let pattern = self.parse_pattern()?;
        self.expect(Token::In)?;

        let iterable = Box::new(self.parse_expression()?);

        let body = Box::new(self.parse_expression()?);

        Ok(Expression::For { pattern, iterable, body })
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, AstError> {
        let token = self.tokens.peek().ok_or(AstError::UnexpectedEOF)?;

        match token {
            Token::IntegerLiteral(_) => Ok(Expression::Literal(LiteralValue::Integer(self.expect_integer()?))),
            Token::FloatLiteral(_) => Ok(Expression::Literal(LiteralValue::Float(self.expect_float()?))),
            Token::BoolLiteral(_) => Ok(Expression::Literal(LiteralValue::Bool(self.expect_bool()?))),
            Token::StringLiteral(_) => Ok(Expression::Literal(LiteralValue::String(self.expect_string()?))),
            Token::Identifier(_) => {
                let path = self.parse_expr_path()?;

                if self.tokens.peek() != Some(&Token::OpenCurly) { return Ok(Expression::Path(path)) }

                let is_struct = match (self.tokens.peek_n(1), self.tokens.peek_n(2)) {
                    (Some(Token::Identifier(_)), Some(Token::Colon)) => true,
                    (Some(Token::CloseCurly), _) => true,
                    _ => false,
                };

                if !is_struct { return Ok(Expression::Path(path)) }

                self.tokens.consume();

                let mut fields = Vec::new();
                while self.tokens.peek() != Some(&Token::CloseCurly) {
                    let name = self.expect_identifier()?;
                    self.expect(Token::Colon)?;
                    let expr = self.parse_expression()?;
                    fields.push((name, expr));

                    if self.tokens.peek() != Some(&Token::Comma) { break }
                    self.expect(Token::Comma)?;
                }
                self.expect(Token::CloseCurly)?;


                Ok(Expression::StructLiteral { path, fields })
            },
            Token::OpenParen => {
                self.tokens.consume();

                if self.tokens.peek() == Some(&Token::CloseParen) {
                    self.tokens.consume();
                    return Ok(Expression::Tuple(Vec::new()))
                }

                let first_expr = self.parse_expression()?;

                if self.tokens.peek() == Some(&Token::Comma) {
                    self.tokens.consume();

                    let mut elements = vec![first_expr];

                    while self.tokens.peek() != Some(&Token::CloseParen) {
                        let expr = self.parse_expression()?;
                        elements.push(expr);

                        if self.tokens.peek() != Some(&Token::Comma) { break }
                        self.expect(Token::Comma)?;
                    }
                    self.expect(Token::CloseParen)?;

                    Ok(Expression::Tuple(elements))
                } else {
                    self.expect(Token::CloseParen)?;
                    Ok(first_expr)
                }
            },
            Token::OpenBracket => {
                self.tokens.consume();

                let mut elements = Vec::new();
                while self.tokens.peek() != Some(&Token::CloseBracket) {
                    let expr = self.parse_expression()?;
                    elements.push(expr);

                    if self.tokens.peek() != Some(&Token::Comma) { break }
                    self.expect(Token::Comma)?;
                }
                self.expect(Token::CloseBracket)?;

                Ok(Expression::Array(elements))
            },

            Token::If => self.parse_if(),
            Token::Match => self.parse_match(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::OpenCurly => self.parse_block(),
            _ => Err(AstError::ExpectedExpression(token.clone()))
        }
    }

    fn parse_postfix_expression(&mut self) -> Result<Expression, AstError> {
        let mut expr = self.parse_primary_expression()?;

        while let Some(token) = self.tokens.peek() {
            match token {
                Token::OpenParen => {
                    self.tokens.consume();

                    let mut args = Vec::new();
                    while self.tokens.peek() != Some(&Token::CloseParen) {
                        let expr = self.parse_expression()?;
                        args.push(expr);
                        if self.tokens.peek() != Some(&Token::Comma) { break; }
                        self.expect(Token::Comma)?;
                    }
                    self.expect(Token::CloseParen)?;

                    expr = Expression::Call {
                        callee: Box::new(expr),
                        args
                    };
                },
                Token::OpenBracket => {
                    self.tokens.consume();
                    let index = self.parse_expression()?;
                    self.expect(Token::CloseBracket)?;

                    expr = Expression::Index {
                        callee: Box::new(expr),
                        index: Box::new(index)
                    };
                },
                Token::Dot => {
                    self.tokens.consume();
                    let field = self.expect_identifier()?;

                    expr = Expression::Field {
                        callee: Box::new(expr),
                        field,
                    };
                },
                Token::QuestionMark => {
                    self.tokens.consume();
                    expr = Expression::Try(Box::new(expr));
                },
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, AstError> {
        let op = match self.tokens.peek() {
            Some(Token::Minus) => Some(UnaryOperator::Negate),
            Some(Token::Bang) => Some(UnaryOperator::Not),
            _ => None
        };

        if let Some(op) = op {
            self.tokens.consume();
            let right = Box::new(self.parse_unary_expression()?);
            Ok(Expression::Unary { op, right })
        } else {
            self.parse_postfix_expression()
        }
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression, AstError> {
        self.parse_binary(Self::parse_unary_expression, |t| match t.peek() {
            Some(Token::Star) => {
                if t.peek_n(1) == Some(&Token::Assign) { return None };
                Some((BinaryOperator::Multiply, 1))
            },
            Some(Token::Slash) => {
                if t.peek_n(1) == Some(&Token::Assign) { return None };
                Some((BinaryOperator::Divide, 1))
            },
            Some(Token::Percent) => {
                if t.peek_n(1) == Some(&Token::Assign) { return None };
                Some((BinaryOperator::Modulo, 1))
            },
            _ => None,
        })
    }

    fn parse_additive_expression(&mut self) -> Result<Expression, AstError> {
        self.parse_binary(Self::parse_multiplicative_expression, |t| match t.peek() {
            Some(Token::Plus) => {
                if t.peek_n(1) == Some(&Token::Assign) { return None };
                Some((BinaryOperator::Add, 1))
            },
            Some(Token::Minus) => {
                if t.peek_n(1) == Some(&Token::Assign) { return None };
                Some((BinaryOperator::Subtract, 1))
            },
            _ => None,
        })
    }

    fn parse_shift_and_relational_expression(&mut self) -> Result<Expression, AstError> {
        self.parse_binary(Self::parse_additive_expression, |t| match t.peek() {
            Some(Token::GreaterThan) => {
                match t.peek_n(1) {
                    Some(&Token::GreaterThan) => {
                        if t.peek_n(1) == Some(&Token::Assign) { return None };
                        Some((BinaryOperator::RightShift, 2))
                    }
                    Some(&Token::Equal) => Some((BinaryOperator::GreaterEqual, 2)),
                    _ => Some((BinaryOperator::GreaterThan, 1))
                }
            },
            Some(Token::LessThan) => {
                match t.peek_n(1) {
                    Some(&Token::LessThan) => {
                        if t.peek_n(1) == Some(&Token::Assign) { return None };
                        Some((BinaryOperator::LeftShift, 2))
                    }, 
                    Some(&Token::Equal) => Some((BinaryOperator::LessEqual, 2)), 
                    _ => Some((BinaryOperator::LessThan, 1))
                }
            },
            _ => None
        })
    }

    fn parse_equality_epxression(&mut self) -> Result<Expression, AstError> {
        self.parse_binary(Self::parse_shift_and_relational_expression, |t| match t.peek() {
            Some(Token::Equal) => Some((BinaryOperator::Equal, 1)),
            Some(Token::NotEqual) => Some((BinaryOperator::NotEqual, 1)),
            _ => None,
        })
    }

    fn parse_bitwise_and_expression(&mut self) -> Result<Expression, AstError> {
        self.parse_binary(Self::parse_equality_epxression, |t| match t.peek() {
            Some(Token::Ampersand) => {
                if t.peek_n(1) == Some(&Token::Assign) { return None };
                Some((BinaryOperator::BitwiseAnd, 1))
            },
            _ => None,
        })
    }

    fn parse_bitwise_xor_expression(&mut self) -> Result<Expression, AstError> {
        self.parse_binary(Self::parse_bitwise_and_expression, |t| match t.peek() {
            Some(Token::Caret) => {
                if t.peek_n(1) == Some(&Token::Assign) { return None };
                Some((BinaryOperator::BitwiseXor, 1))
            },
            _ => None,
        })
    }

    fn parse_bitwise_or_expression(&mut self) -> Result<Expression, AstError> {
        self.parse_binary(Self::parse_bitwise_xor_expression, |t| match t.peek() {
            Some(Token::Pipe) => {
                if t.peek_n(1) == Some(&Token::Assign) { return None };
                Some((BinaryOperator::BitwiseOr, 1))
            },
            _ => None,
        })
    }

    fn parse_logical_and_expression(&mut self) -> Result<Expression, AstError> {
        self.parse_binary(Self::parse_bitwise_or_expression, |t| match t.peek() {
            Some(Token::And) => Some((BinaryOperator::And, 1)),
            _ => None,
        })
    }

    fn parse_logical_or_expression(&mut self) -> Result<Expression, AstError> {
        self.parse_binary(Self::parse_logical_and_expression, |t| match t.peek() {
            Some(Token::Or) => Some((BinaryOperator::Or, 1)),
            _ => None,
        })
    }

    fn parse_range(&mut self) -> Result<Expression, AstError> {
        let start = self.parse_logical_or_expression()?;
        if self.tokens.peek() == Some(&Token::Range) {
            let start = Box::new(start);
            self.tokens.consume();
            let end = Box::new(self.parse_logical_or_expression()?);
            Ok(Expression::Range { start, end })
        } else {
            Ok(start)
        }
    }

    fn parse_assignment(&mut self) -> Result<Expression, AstError> {
        let left = self.parse_range()?;

        let token = self.tokens.peek();
        let op = match token {
            Some(Token::Assign) => { self.tokens.consume(); AssignmentOperator::Assign },
            Some(_) if self.tokens.peek_n(1) == Some(&Token::Assign) => {
                let op = match self.tokens.peek() {
                    Some(Token::Plus)      => { self.tokens.consume(); AssignmentOperator::AddAssign },
                    Some(Token::Minus)     => { self.tokens.consume(); AssignmentOperator::SubAssign },
                    Some(Token::Star)      => { self.tokens.consume(); AssignmentOperator::MulAssign },
                    Some(Token::Slash)     => { self.tokens.consume(); AssignmentOperator::DivAssign },
                    Some(Token::Percent)   => { self.tokens.consume(); AssignmentOperator::ModAssign },
                    Some(Token::Ampersand) => { self.tokens.consume(); AssignmentOperator::AndAssign},
                    Some(Token::Pipe)      => { self.tokens.consume(); AssignmentOperator::OrAssign },
                    Some(Token::Caret)     => { self.tokens.consume(); AssignmentOperator::XorAssign },
                    Some(Token::LessThan) if self.tokens.peek_n(1) == Some(&Token::LessThan) => {
                        self.tokens.consume_n(2);
                        AssignmentOperator::LeftShiftAssign
                    },
                    Some(Token::GreaterThan) if self.tokens.peek_n(1) == Some(&Token::GreaterThan) => {
                        self.tokens.consume_n(2);
                        AssignmentOperator::RightShiftAssign
                    },
                    _ => return Ok(left),
                };
                self.tokens.consume();
                op
            },
            _ => return Ok(left),
        };

        let right = Box::new(self.parse_assignment()?);
        Ok(Expression::Assign { left: Box::new(left), op, right })
    }

    fn parse_expression(&mut self) -> Result<Expression, AstError> {
        self.parse_assignment()
    }

    fn parse_let_statement(&mut self) -> Result<Statement, AstError> {
        self.expect(Token::Let)?;

        let pattern = self.parse_pattern()?;

        let type_annotation = if self.tokens.peek() == Some(&Token::Colon) {
            self.tokens.consume();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.expect(Token::Assign)?;

        let value = self.parse_expression()?;
        self.expect(Token::Semicolon)?;

        Ok(Statement::Let { pattern, type_annotation, value })
    }

    fn parse_fn_definition(&mut self) -> Result<Statement, AstError> {
        let signature = self.parse_function_signature()?;
        let body = self.parse_expression()?;

        Ok(Statement::FunctionDefinition(FunctionDefinition {
            signature,
            body 
        }))
    }

    fn parse_struct_definition(&mut self) -> Result<Statement, AstError> {
        self.expect(Token::Struct)?;

        let name = self.expect_identifier()?;
        let generics = self.parse_generic_params()?;
        let fields = self.parse_field_block()?; 

        Ok(Statement::StructDefinition(StructDefinition {
            name,
            generics,
            fields
        }))
    }

    fn parse_enum_definition(&mut self) -> Result<Statement, AstError> {
        self.expect(Token::Enum)?;

        let name = self.expect_identifier()?;
        let generics = self.parse_generic_params()?;

        let mut variants = Vec::new();

        self.expect(Token::OpenCurly)?;
        while self.tokens.peek() != Some(&Token::CloseCurly) {
            let name = self.expect_identifier()?;

            let payload = match self.tokens.peek() {
                Some(Token::OpenParen) => {
                    let mut annotations = Vec::new();
                    self.expect(Token::OpenParen)?;
                    while self.tokens.peek() != Some(&Token::CloseParen) {
                        let type_annotation = self.parse_type_annotation()?;
                        annotations.push(type_annotation); 

                        if self.tokens.peek() != Some(&Token::Comma) { break }
                        self.expect(Token::Comma)?;
                    }
                    self.expect(Token::CloseParen)?;
                    
                    Some(EnumPayload::Tuple(annotations))
                },
                Some(Token::OpenCurly) => {
                    let fields = self.parse_field_block()?;
                    Some(EnumPayload::Struct(fields))
                },
                _ => None,
            };

            variants.push(EnumVariant { name, payload });

            if self.tokens.peek() != Some(&Token::Comma) { break }
            self.expect(Token::Comma)?;
        }
        self.expect(Token::CloseCurly)?;

        Ok(Statement::EnumDefinition(EnumDefinition {
            name,
            generics,
            variants
        }))
    }

    fn parse_trait_definition(&mut self) -> Result<Statement, AstError> {
        self.expect(Token::Trait)?;

        let name = self.expect_identifier()?;
        let generics = self.parse_generic_params()?;

        let mut functions = Vec::new();

        self.expect(Token::OpenCurly)?;
        while self.tokens.peek() != Some(&Token::CloseCurly) {
            let signature = self.parse_function_signature()?;
            self.expect(Token::Semicolon)?;
            functions.push(signature);
        }
        self.expect(Token::CloseCurly)?;

        Ok(Statement::TraitDefinition(TraitDefinition {
            name,
            generics,
            functions
        }))
    }

    fn parse_impl_definition(&mut self) -> Result<Statement, AstError> {
        self.expect(Token::Impl)?;

        let first_path = self.parse_type_path()?;

        let (trait_path, type_path) = if self.tokens.peek() == Some(&Token::For) {
            self.expect(Token::For)?;
            let type_path = self.parse_type_path()?;
            (Some(first_path), type_path)
        } else {
            (None, first_path)
        };

        self.expect(Token::OpenCurly)?;
        let mut functions = Vec::new();
        while self.tokens.peek() != Some(&Token::CloseCurly) {
            let function = self.parse_fn_definition()?;
            functions.push(function);
        }
        self.expect(Token::CloseCurly)?;

        Ok(Statement::ImplBlock(ImplBlock {
            trait_path,
            type_path,
            functions
        }))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, AstError> {
        let expression = self.parse_expression()?;
        if self.tokens.peek() == Some(&Token::Semicolon) {
            self.tokens.consume();
            Ok(Statement::ExpressionStatement(expression))
        } else {
            Ok(Statement::Return(expression))
        }
    }

    fn parse_return_statement(&mut self) -> Result<Statement, AstError> {
        self.expect(Token::Return)?;
        let expression = self.parse_expression()?;
        if self.tokens.peek() == Some(&Token::Semicolon) { self.tokens.consume(); }
        Ok(Statement::Return(expression))
    }

    fn parse_break_statement(&mut self) -> Result<Statement, AstError> {
        self.expect(Token::Break)?;
        self.expect(Token::Semicolon)?;
        Ok(Statement::Break)
    }

    fn parse_continue_statement(&mut self) -> Result<Statement, AstError> {
        self.expect(Token::Continue)?;
        self.expect(Token::Semicolon)?;
        Ok(Statement::Continue)
    }

    fn parse_statement(&mut self) -> Result<Statement, AstError> {
        match self.tokens.peek() {
            Some(Token::Let) => self.parse_let_statement(),
            Some(Token::Fn) => self.parse_fn_definition(),
            Some(Token::Struct) => self.parse_struct_definition(),
            Some(Token::Enum) => self.parse_enum_definition(),
            Some(Token::Trait) => self.parse_trait_definition(),
            Some(Token::Impl) => self.parse_impl_definition(),
            Some(Token::Return) => self.parse_return_statement(),
            Some(Token::Break) => self.parse_break_statement(),
            Some(Token::Continue) => self.parse_continue_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_program(&mut self) -> Result<Vec<Statement>, AstError> {
        let mut program = Vec::new();

        while self.tokens.peek() != Some(&Token::Eof) {
            let statement = self.parse_statement()?;
            program.push(statement);
        }

        Ok(program)
    }

    pub fn build(&mut self) -> Result<Vec<Statement>, AstError> {
        self.parse_program()
    }
}
