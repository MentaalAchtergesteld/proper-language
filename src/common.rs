use crate::{reporting::Spanned, Token};

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
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    LeftShift,
    RightShift,
}

#[derive(Clone, Debug)]
pub enum UnaryOperator {
    Negate,
    Not
}

impl UnaryOperator {
    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Minus => Some(Self::Negate),
            Token::Bang => Some(Self::Not),
            _ => None,
        }
    }
}

pub type SpannedType = Spanned<TypeAnnotation>;
#[derive(Clone, Debug)]
pub enum TypeAnnotation {
    Path(Spanned<Path>),
    Array(Box<SpannedType>),
    Tuple(Vec<SpannedType>),
    Function {
        params: Vec<SpannedType>,
        return_type: Option<Box<SpannedType>>
    }
}

#[derive(Clone, Debug)]
pub struct PathSegment {
    pub ident: Spanned<String>,
    pub generics: Option<Spanned<Vec<SpannedType>>>
}

impl PathSegment {
    pub fn new(ident: Spanned<String>, generics: Option<Spanned<Vec<SpannedType>>>) -> Self {
        Self {
            ident: ident.into(),
            generics 
        }
    }

    pub fn ident(ident: Spanned<String>) -> Self {
        Self {
            ident: ident.into(),
            generics: None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Path {
    pub segments: Vec<PathSegment>
}

impl Path {
    pub fn new() -> Self { Path { segments: Vec::new() } }

    pub fn push(&mut self, segment: PathSegment) {
        self.segments.push(segment);
    }

    pub fn get_first_name(&self) -> Option<&str> {
        self.segments.first().map(|s| s.ident.as_str())
    }

    pub fn is_simple(&self) -> bool {
        self.segments.len() == 1 && self.segments[0].generics.is_none()
    }
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub type_annotation: TypeAnnotation
}


#[derive(Clone, Debug)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeAnnotation>,
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
    pub name: String,
    pub payload: Option<EnumPayload>
}

#[derive(Clone, Debug)]
pub struct EnumDefinition {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Literal(LiteralValue),
    Path(Path),
    Identifier(String),
    Wildcard,
    Tuple {
        path: Option<Path>,
        patterns: Vec<Spanned<Pattern>>,
    },

    Struct {
        path: Path,
        fields: Vec<String>
    }
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

