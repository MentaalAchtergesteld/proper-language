use crate::common::*;

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub signature: FunctionSignature,
    pub body: Expression
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub trait_path: Option<Path>,
    pub type_path: Path,
    pub functions: Vec<Statement>
}

#[derive(Debug, Clone)]
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
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expression
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(LiteralValue),
    Path(Path),
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
    },
    Match {
        value: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    Block(Vec<Statement>),
    Closure {
        params: Vec<Pattern>,
        return_type: Option<TypeAnnotation>,
        body: Box<Expression>,
    },
    Array(Vec<Expression>),
    Tuple(Vec<Expression>),
    StructLiteral {
        path: Path,
        fields: Vec<(String, Expression)>
    },
    Assign {
        target: Box<Expression>,
        value: Box<Expression>
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
    Loop(Box<Expression>),
    Return(Box<Expression>),
    Break,
    Continue
}
