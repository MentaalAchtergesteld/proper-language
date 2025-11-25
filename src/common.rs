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

