use std::fs::soft_link;

use crate::peekablecursor::PeekableCursor;
use crate::lexer::Token;

pub enum LiteralValue {
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Empty,
}

pub enum BinaryOperator {
    Add, Subtract, Multiply, Divide, Modulo,
    Equal, NotEqual, GreaterThan, GreaterEqual, LessThan, LessEqual,
    And, Or,
    BitwiseAnd, BitwiseOr, BitwiseXor, LeftShift, RightShift
}

pub enum UnaryOperator {
    Negate,
    Not
}

pub enum TypeAnnotation {
    Path(Path),
    Array(Box<TypeAnnotation>),
    Tuple(Vec<TypeAnnotation>),
    Function {
        params: Vec<TypeAnnotation>,
        return_type: Box<TypeAnnotation>
    }
}

pub struct PathSegment {
    pub ident: String,
    pub generic_args: Option<Vec<TypeAnnotation>>
}

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

pub struct Parameter {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

pub struct FunctionDefinition {
    pub signature: FunctionSignature,
    pub block: Expression
}

pub struct Field {
    pub name: String,
    pub type_annotation: TypeAnnotation
}

pub struct StructDefinition {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
}

pub enum EnumPayload {
    Tuple(Vec<TypeAnnotation>),
    Struct(Vec<Field>)
}

pub struct EnumVariant {
    name: String,
    payload: Option<EnumPayload>
}

pub struct EnumDefinition {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
}

pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeAnnotation>,
}

pub struct FunctionSignature {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
}

pub struct TraitDefinition {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub functions: Vec<FunctionSignature>,
}

pub struct ImplBlock {
    pub trait_path: Option<Path>,
    pub type_path: Path,
    pub functions: Vec<Statement>,
}

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

pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expression
}

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
}

pub enum Expression {
    Literal(LiteralValue),
    Path(Path),

    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>
    },

    Match {
        expression: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>
    },

    Block {
        statements: Vec<Statement>,
        result: Option<Box<Expression>>
    },

    While {
        condition: Box<Expression>,
        body: Box<Expression>,
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
    ExpectedToken(Token),
    ExpectedPattern(Token),
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
            Err(AstError::ExpectedToken(token.clone()))
        }
    }

    fn expect_integer(&mut self) -> Result<i32, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        match token {
            Token::IntegerLiteral(v) => Ok(*v),
            _ => Err(AstError::ExpectedToken(Token::IntegerLiteral(0)))
        }
    }

    fn expect_float(&mut self) -> Result<f32, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        match token {
            Token::FloatLiteral(v) => Ok(*v),
            _ => Err(AstError::ExpectedToken(Token::FloatLiteral(0.)))
        }
    }

    fn expect_bool(&mut self) -> Result<bool, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        match token {
            Token::BoolLiteral(v) => Ok(*v),
            _ => Err(AstError::ExpectedToken(Token::BoolLiteral(false)))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, AstError> {
        let token = self.tokens.consume().ok_or(AstError::UnexpectedEOF)?;
        match token {
            Token::Identifier(v) => Ok(v.clone()),
            _ => Err(AstError::ExpectedToken(Token::StringLiteral(String::new())))
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
                let path = self.parse_path()?;

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
        let path = self.parse_path()?;
        let mut type_ann = TypeAnnotation::Path(path);

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

    fn parse_path(&mut self) -> Result<Path, AstError> {
        let mut segments = Vec::new();
        
        let first_ident = self.expect_identifier()?;

        let first_generics = if self.tokens.peek() == Some(&Token::LessThan) {
            Some(self.parse_generic_args()?)
        } else {
            None
        };
        segments.push(PathSegment { ident: first_ident, generic_args: first_generics });

        while self.tokens.peek() == Some(&Token::PathSeperator) {
            self.tokens.consume();

            let ident = self.expect_identifier()?;
            let generic_args = if self.tokens.peek() == Some(&Token::LessThan) {
                Some(self.parse_generic_args()?)
            } else {
                None
            };
            segments.push(PathSegment { ident, generic_args });
        }
        Ok(Path { segments })
    }

    fn parse_block(&mut self) -> Result<Expression, AstError> {}

    fn parse_field_block(&mut self) -> Result<Vec<Field>, AstError> {
        let mut fields = Vec::new();

        self.expect(Token::OpenCurly)?;
        while self.tokens.peek() != Some(&Token::CloseCurly) {
            let name = self.expect_identifier()?;
            self.expect(Token::Colon)?;
            let type_annotation = self.parse_type_annotation()?;

            fields.push(Field { name, type_annotation });
            if self.tokens.peek() != Some(&Token::Comma) { break }
            self.expect(Token::Comma);
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
            self.expect(Token::Comma);
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


    fn parse_expression(&mut self) -> Result<Expression, AstError> {}

    fn parse_let_statement(&mut self) -> Result<Statement, AstError> {
        self.expect(Token::Let)?;

        let pattern = self.parse_pattern()?;

        let type_annotation = if self.tokens.peek() == Some(&Token::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let value = self.parse_expression()?;
        self.expect(Token::Semicolon);

        Ok(Statement::Let { pattern, type_annotation, value })
    }

    fn parse_fn_definition(&mut self) -> Result<Statement, AstError> {
        let signature = self.parse_function_signature()?;
        let block = self.parse_block()?;

        Ok(Statement::FunctionDefinition(FunctionDefinition {
            signature,
            block
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
                    self.expect(Token::OpenParen);
                    while self.tokens.peek() != Some(&Token::CloseParen) {
                        let type_annotation = self.parse_type_annotation()?;
                        annotations.push(type_annotation); 

                        if self.tokens.peek() != Some(&Token::Comma) { break }
                        self.expect(Token::Comma);
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
            self.expect(Token::Comma);
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

        let first_path = self.parse_path()?;

        let (trait_path, type_path) = if self.tokens.peek() == Some(&Token::For) {
            self.expect(Token::For)?;
            let type_path = self.parse_path()?;
            (Some(first_path), type_path)
        } else {
            (None, first_path)
        };

        self.expect(Token::OpenCurly)?;
        let mut functions = Vec::new();
        while self.tokens.peek() != Some(&Token::CloseCurly) {
            let function = self.parse_fn_definition()?;
            functions.push(function);
            if self.tokens.peek() != Some(&Token::Comma) { break }
            self.expect(Token::Comma);
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
        self.expect(Token::Semicolon)?;
        Ok(Statement::ExpressionStatement(expression))
    }

    fn parse_statement(&mut self) -> Result<Statement, AstError> {
        match self.tokens.peek() {
            Some(Token::Let) => self.parse_let_statement(),
            Some(Token::Fn) => self.parse_fn_definition(),
            Some(Token::Struct) => self.parse_struct_definition(),
            Some(Token::Enum) => self.parse_enum_definition(),
            Some(Token::Trait) => self.parse_trait_definition(),
            Some(Token::Impl) => self.parse_impl_definition(),
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
}
