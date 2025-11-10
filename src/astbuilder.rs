use std::panic::RefUnwindSafe;

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

pub struct Parameter {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

pub struct FunctionDefinition {
    pub signature: FunctionSignature,
    pub body: Expression
}


pub struct Field {
    pub name: String,
    pub type_annotation: TypeAnnotation
}

pub struct StructDefinition {
    pub name: String,
    pub generics: Option<Vec<String>>,
    pub fields: Vec<Field>,
}

pub enum EnumPayload {
    Tuple(Vec<TypeAnnotation>),
    Struct(Vec<Field>)
}

pub struct EnumVariant {
    pub name: String,
    pub payload: Option<EnumPayload>
}

pub struct EnumDefinition {
    pub name: String,
    pub generics: Option<Vec<String>>,
    pub variants: Vec<EnumVariant>,
}

pub struct FunctionSignature {
    pub name: String,
    pub generics: Option<Vec<String>>,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
}

pub struct TraitDefinition {
    pub name: String,
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
    UnexpectedEOF,
}

pub struct AstBuilder<'a> {
    tokens: PeekableCursor<'a, Token>
}

impl<'a> AstBuilder<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens: PeekableCursor::new(tokens) }
    }

    pub fn parse_statement(&mut self) -> Result<Statement, AstError> {
        let next = self.tokens.peek().ok_or(AstError::UnexpectedEOF)?;
        match next {
            Token::Let => self.parse_let(),
            Token::Fn => self.parse_fn_def(),
            Token::Struct => self.parse_struct_def(),
            Token::Enum => self.parse_enum_def(),
            Toke
        }
    }

    pub fn build(&mut self) -> Result<Expression, AstError> {
        let mut statements = Vec::new();
        while !self.tokens.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(Expression::Block { statements, result: None })
    }
}
