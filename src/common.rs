use crate::Token;

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

impl UnaryOperator {
    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Minus => Some(Self::Negate),
            Token::Bang => Some(Self::Not),
            _ => None,
        }
    }
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

impl PathSegment {
    pub fn ident(identifier: &str) -> Self {
        Self { ident: identifier.to_string(), generic_args: None } 
    }
}

#[derive(Clone, Debug)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

impl Path {
    pub fn new() -> Self { Path { segments: Vec::new() } }

    pub fn push(mut self, segment: PathSegment) -> Self {
        self.segments.push(segment);        
        self
    }

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

impl From<Vec<PathSegment>> for Path {
    fn from(segments: Vec<PathSegment>) -> Self {
        Path { segments }
    }
}

impl From<PathSegment> for Path {
    fn from(segment: PathSegment) -> Self {
        Path { segments: vec![segment] }
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
        patterns: Vec<Pattern>,
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

