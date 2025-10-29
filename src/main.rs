use std::fs;

#[derive(Debug, PartialEq, Clone)]
enum Token {
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
    
    Comma,
    Dot,
    Colon,
    Semicolon,

    Assign,

    Plus,
    PlusAssign,
    Minus,
    MinusAssign,
    Star,
    StarAssign,
    Slash,
    SlashAssign,
    Percent,
    PercentAssign,

    Ampersand,
    AmpersandAssign,
    Pipe,
    PipeAssign,
    Caret,
    CaretAssign,
    LeftShift,
    LeftShiftAssign,
    RightShift,
    RightShiftAssign,

    And,
    Or,
    Bang,

    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,

    IntegerLiteral(i32),
    FloatLiteral(f32),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),

    If,
    Else,
    While,
    // For,
    Fn,
    Let,
    Return,
    Continue,
    Break,
}

#[derive(Debug)]
struct TokenFrame {
    line: usize,
    column: usize,
    length: usize,
}

impl TokenFrame {
    pub fn print_source_context(&self, source: &str) {
        let line_str = match source.lines().nth(self.line) {
            Some(line) => line,
            None => return,
        };

        let line_num = (self.line + 1).to_string();
        let padding_width = line_num.len();

        eprintln!("{:>width$} | {}", line_num, line_str, width = padding_width);

        let indicator_padding = " ".repeat(self.column);
        let indicator = "^".repeat(self.length.max(1));
        
        eprintln!("{} | {}{}", 
            " ".repeat(padding_width),
            indicator_padding,
            indicator
        );
    }
}

#[derive(Debug)]
enum LexerError {
    UnterminatedString(String, TokenFrame),
    InvalidNumber(String, TokenFrame),
    UnknownToken(String, TokenFrame),
}

impl LexerError {
    pub fn print_with_source(&self, source: &str) {
        let (message, frame) = match self {
            LexerError::UnterminatedString(s, f) => (format!("Unterminated string: \"{}\"", s), f),
            LexerError::InvalidNumber(s, f) => (format!("Invalid number: {}", s), f),
            LexerError::UnknownToken(s, f) => (format!("Unknown token: {}", s), f),
        };

        eprintln!("Error: {} at line {}, column {}",
            message,
            frame.line + 1,
            frame.column + 1
        );

        frame.print_source_context(source);
    }
}

struct Lexer {
    chars: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            position: 0,
            line: 0,
            column: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.chars.get(self.position).copied() {
            self.position += 1;

            if c == '\n' {
                self.line+=1;
                self.column = 0;
            } else if c != '\r' {
                self.column += 1;
            }

            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.consume();
            } else { break }
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<(Token, TokenFrame), LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.position;

        let next_char = match self.consume() {
            Some(c) => c,
            None => return None
        };

        enum BareError {
            InvalidNumber(String),
            UnterminatedString(String),
            UnknownToken(String),
        }

        let token_result = match next_char {
            '(' => Ok(Token::OpenParen),
            ')' => Ok(Token::CloseParen),
            '{' => Ok(Token::OpenCurly),
            '}' => Ok(Token::CloseCurly),
            '[' => Ok(Token::OpenBracket),
            ']' => Ok(Token::CloseBracket),

            ',' => Ok(Token::Comma),
            '.' => Ok(Token::Dot),
            ':' => Ok(Token::Colon),
            ';' => Ok(Token::Semicolon),

            '=' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::Equal)
                },
                _ => Ok(Token::Assign)
            },
            '+' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::PlusAssign)
                },
                _ => Ok(Token::Plus)
            },
            '-' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::MinusAssign)
                },
                _ => Ok(Token::Minus)
            },
            '*' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::StarAssign)
                },
                _ => Ok(Token::Star)
            },
            '/' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::SlashAssign)
                },
                _ => Ok(Token::Slash)
            },
            '%' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::PercentAssign)
                },
                _ => Ok(Token::Percent)
            },


            '&' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::AmpersandAssign)
                },
                Some('&') => {
                    self.consume();
                    Ok(Token::And)
                }
                _ => Ok(Token::Ampersand)
            },
            '|' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::PipeAssign)
                },
                Some('|') => {
                    self.consume();
                    Ok(Token::Or)
                }
                _ => Ok(Token::Pipe)
            },
            '^' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::CaretAssign)
                },
                _ => Ok(Token::Caret)
            },
            '>' => match self.peek() {
                Some('>') => {
                    self.consume();
                    match self.peek() {
                        Some('=') => {
                            self.consume();
                            Ok(Token::RightShiftAssign)
                        },
                        _ => Ok(Token::RightShift)
                    }
                },
                Some('=') => {
                    self.consume();

                    Ok(Token::GreaterThanEqual)
                }
                _ => Ok(Token::GreaterThan)
            },
            '<' => match self.peek() {
                Some('<') => {
                    self.consume();
                    match self.peek() {
                        Some('=') => {
                            self.consume();
                            Ok(Token::LeftShiftAssign)
                        },
                        _ => Ok(Token::LeftShift)
                    }
                },
                Some('=') => {
                    self.consume();
                    Ok(Token::LessThanEqual)
                }
                _ => Ok(Token::LessThan)
            },
            '!' => match self.peek() {
                Some('=') => {
                    self.consume();
                    Ok(Token::NotEqual)
                },
                _ => Ok(Token::Bang)
            },
            next_char if next_char.is_digit(10) => {
                let mut number_str = String::from(next_char);
                let mut seen_dot = false;

                while let Some(c) = self.peek() {
                    match c {
                        c if c.is_digit(10) => number_str.push(self.consume().unwrap()),
                        '.' => {
                            number_str.push(self.consume().unwrap());
                            seen_dot = true;
                        },
                        _ => break,
                    }
                }

                if seen_dot {
                    match number_str.parse::<f32>() {
                        Ok(n) => Ok(Token::FloatLiteral(n)),
                        Err(_) => Err(BareError::InvalidNumber(number_str))
                    }
                } else {
                    match number_str.parse::<i32>() {
                        Ok(n) => Ok(Token::IntegerLiteral(n)),
                        Err(_) => Err(BareError::InvalidNumber(number_str))
                    }
                }
            },
            '"' => {
                let mut string_str = String::new();
                let mut closed = false;
                while let Some(_) = self.peek() {
                    let c = self.consume().unwrap();
                    if c == '\n' { break };
                    if c == '"' { closed = true; break };
                    string_str.push(c);
                }

                if closed {
                    Ok(Token::StringLiteral(string_str))
                } else {
                    Err(BareError::UnterminatedString(string_str))
                }

            }
            next_char if next_char.is_alphabetic() || next_char == '_' => {
                let mut identifier_str = String::from(next_char);
                while let Some(c) = self.peek() {
                    if c.is_whitespace() { break};
                    if !c.is_alphanumeric() && c != '_' { break };
                    identifier_str.push(self.consume().unwrap());
                }

                match identifier_str.as_str() {
                    "true"     => Ok(Token::BoolLiteral(true)),
                    "false"    => Ok(Token::BoolLiteral(false)),
                    "if"       => Ok(Token::If),
                    "else"     => Ok(Token::Else),
                    "while"    => Ok(Token::While),
                    // "for"      => Ok(Token::For),
                    "fn"       => Ok(Token::Fn),
                    "let"      => Ok(Token::Let),
                    "return"   => Ok(Token::Return),
                    "continue" => Ok(Token::Continue),
                    "break"    => Ok(Token::Break),
                    _          => Ok(Token::Identifier(identifier_str))
                }
            },
            _ => Err(BareError::UnknownToken(next_char.to_string())),
        };

        let end_pos = self.position;
        let frame = TokenFrame {
            line: start_line,
            column: start_column,
            length: end_pos - start_pos
        };

        match token_result {
            Ok(token) => Some(Ok((token, frame))),
            Err(bare_err) => {
                let rich_err = match bare_err {
                    BareError::InvalidNumber(s) => LexerError::InvalidNumber(s, frame),
                    BareError::UnterminatedString(s) => LexerError::UnterminatedString(s, frame),
                    BareError::UnknownToken(s) => LexerError::UnknownToken(s, frame),
                };
                Some(Err(rich_err))
            }
        }
    }
}

#[derive(Debug)]
enum ASTBuilderError {
    UnexpectedToken(Token),
    ExpectedToken(Token),
    UnexpectedEOF,
}

#[derive(Debug)]
enum BinaryOperator {
    Or,
    And,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug)]
enum UnaryOperator {
    Not,
    Negate,
}

#[derive(Debug)]
enum LiteralValue {
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool)
}

#[derive(Debug)]
enum Expression {
    BinaryExpression {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: BinaryOperator,
    },
    UnaryExpression {
        operator: UnaryOperator,
        right: Box<Expression>,
    },
    Literal(LiteralValue),
    Grouping(Box<Expression>),
    Variable(String),
    Assign {
        variable: Box<Expression>,
        value: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>
    },
    Index {
        callee: Box<Expression>,
        index: Box<Expression>
    },
    Field {
        callee: Box<Expression>,
        name: String
    },
    Array(Vec<Expression>),
    Struct(Vec<(String, Expression)>)
}

#[derive(Debug)]
enum Statement {
    If {
        condition: Expression,
        block: Vec<Statement>,
        else_branch: Option<Vec<Statement>>
    },
    While {
        condition: Expression,
        block: Vec<Statement>
    },
    FnDefinition {
        name: String,
        params: Vec<String>,
        block: Vec<Statement>
    },
    Let {
        name: String,
        value: Option<Expression>
    },
    Return {
        value: Option<Expression>
    },
    Continue,
    Break,
    Expression {
        value: Expression
    }
}

struct ASTBuilder {
    tokens: Vec<Token>,
    frames: Vec<TokenFrame>,
    position: usize
}

impl ASTBuilder {
    pub fn new(tokens: Vec<Token>, frames: Vec<TokenFrame>) -> ASTBuilder {
        ASTBuilder {
            tokens,
            frames,
            position: 0
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn consume(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    fn expect_token(&mut self, token: Token) -> Result<&Token, ASTBuilderError> {
        let next = self.consume().ok_or(ASTBuilderError::UnexpectedEOF)?;
        if *next == token {
            Ok(next)
        } else {
            Err(ASTBuilderError::ExpectedToken(token))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ASTBuilderError> {
        let next = self.consume().ok_or(ASTBuilderError::UnexpectedEOF)?;
        match next {
            Token::Identifier(name) => Ok(name.clone()),
            _ => Err(ASTBuilderError::ExpectedToken(Token::Identifier(String::new())))
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn parse_binary(
        &mut self, 
        next_level: fn(&mut Self) -> Result<Expression, ASTBuilderError>,
        map_token_to_op: impl Fn(&Token) -> Option<BinaryOperator>,
    ) -> Result<Expression, ASTBuilderError> {
        let mut left = next_level(self)?; 

        while let Some(token) = self.peek() {
            let operator = match map_token_to_op(token) {
                Some(t) => t,
                None => break
            };
            self.consume();

            let right = next_level(self)?;
            left = Expression::BinaryExpression {
                left: Box::new(left),
                right: Box::new(right),
                operator
            };
        }
        Ok(left)
    }

    fn parse_array(&mut self) -> Result<Expression, ASTBuilderError> {
        self.expect_token(Token::OpenBracket)?;
        let mut values = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::CloseBracket { break };

            let value = self.parse_expression()?;
            values.push(value);

            if self.peek() != Some(&Token::Comma) { break };
            self.consume();
        } 
        self.expect_token(Token::CloseBracket)?;

        Ok(Expression::Array(values))
    }

    fn parse_struct(&mut self) -> Result<Expression, ASTBuilderError> {
        self.expect_token(Token::OpenCurly)?;
        let mut values = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::CloseCurly { break };

            let name = self.expect_identifier()?;
            self.expect_token(Token::Colon)?;
            let value = self.parse_expression()?;

            values.push((name, value));

            if self.peek() != Some(&Token::Comma) { break };
            self.consume();
        }
        self.expect_token(Token::CloseCurly)?;

        Ok(Expression::Struct(values))
    }

    fn parse_atom(&mut self) -> Result<Expression, ASTBuilderError> {
        match self.peek() {
            Some(Token::OpenParen) => {
                let expr = self.parse_expression()?;
                self.expect_token(Token::CloseParen)?;
                Ok(expr)
            },
            Some(Token::OpenBracket) => self.parse_array(),
            Some(Token::OpenCurly) => self.parse_struct(),
            Some(Token::IntegerLiteral(il)) => Ok(Expression::Literal(LiteralValue::Integer(*il))),
            Some(Token::FloatLiteral(fl)) => Ok(Expression::Literal(LiteralValue::Float(*fl))),
            Some(Token::StringLiteral(sl)) => Ok(Expression::Literal(LiteralValue::String(sl.clone()))),
            Some(Token::BoolLiteral(bl)) => Ok(Expression::Literal(LiteralValue::Bool(*bl))),
            Some(token) => Err(ASTBuilderError::UnexpectedToken(token.clone())),
            _ => Err(ASTBuilderError::UnexpectedEOF),
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, ASTBuilderError> {
        let mut left = self.parse_atom()?;
        while let Some(token) = self.peek() {
            match *token {
                Token::OpenBracket => {
                    self.expect_token(Token::OpenBracket)?;
                    let expr = self.parse_expression()?;

                    left = Expression::Index { callee: Box::new(left), index: Box::new(expr) };
                    self.expect_token(Token::CloseBracket)?;
                },
                Token::OpenParen => {
                    self.expect_token(Token::OpenParen)?;
                    let mut args = Vec::new();
                    while let Some(token) = self.peek() {
                        if *token == Token::CloseParen { break };

                        let expr = self.parse_expression()?;
                        args.push(expr);

                        if self.peek() != Some(&Token::Comma) { break };
                        self.consume();
                    }
                    self.expect_token(Token::CloseParen)?;

                    left = Expression::Call { callee: Box::new(left), args }
                }
                Token::Dot => {
                    while let Some(token) = self.peek() {
                        if *token != Token::Dot { break };
                        self.consume();
                        let name = self.expect_identifier()?;
                        left = Expression::Field { callee: Box::new(left), name }
                    }
                },
                _ => break,
            }
        } 
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ASTBuilderError> {
        let operator = match self.peek() {
            Some(Token::Bang) => Some(UnaryOperator::Not),
            Some(Token::Minus) => Some(UnaryOperator::Negate),
            Some(Token::Plus) => {
                self.consume();
                return self.parse_unary();
            },
            _ => None
        };

        if let Some(operator) = operator {
            self.consume();

            let right = Box::new(self.parse_unary()?);

            Ok(Expression::UnaryExpression {
                operator, right
            })
        } else {
            self.parse_primary()
        }
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, ASTBuilderError> {
        self.parse_binary(Self::parse_unary, |t| match t {
                Token::Star => Some(BinaryOperator::Multiply),
                Token::Slash => Some(BinaryOperator::Divide),
                Token::Percent => Some(BinaryOperator::Modulo),
                _ => None
        })
    }

    fn parse_additive(&mut self) -> Result<Expression, ASTBuilderError> {
        self.parse_binary(Self::parse_multiplicative, |t| match t {
                Token::Plus => Some(BinaryOperator::Add),
                Token::Minus => Some(BinaryOperator::Subtract),
                _ => None
        })
    }

    fn parse_relational(&mut self) -> Result<Expression, ASTBuilderError> {
        self.parse_binary(Self::parse_additive, |t| match t {
                Token::LessThan => Some(BinaryOperator::LessThan),
                Token::LessThanEqual => Some(BinaryOperator::LessThanEqual),
                Token::GreaterThan => Some(BinaryOperator::GreaterThan),
                Token::GreaterThanEqual => Some(BinaryOperator::GreaterThanEqual),
                _ => None
        })
    }

    fn parse_equality(&mut self) -> Result<Expression, ASTBuilderError> {
        self.parse_binary(Self::parse_relational, |t| match t {
                Token::Equal => Some(BinaryOperator::Equal),
                Token::NotEqual => Some(BinaryOperator::NotEqual),
                _ => None
        })
    }

    fn parse_logical(&mut self) -> Result<Expression, ASTBuilderError> {
        self.parse_binary(Self::parse_equality, |t| match t {
            Token::Or => Some(BinaryOperator::Or),
            Token::And => Some(BinaryOperator::And),
            _ => None,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, ASTBuilderError> {
        self.parse_logical()
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ASTBuilderError> {
        self.expect_token(Token::If)?;

        let condition = self.parse_expression()?;
        let block = self.parse_block()?;

        let else_branch = match self.peek() {
            Some(Token::Else) => {
                self.consume();
                match self.peek() {
                    Some(Token::If) => Some(vec![self.parse_if_statement()?]),
                    _ => Some(self.parse_block()?)
                }
            },
            _ => None,
        };

        Ok(Statement::If {
            condition,
            block,
            else_branch
        }) 
    }

    fn parse_while_statement(&mut self) -> Result<Statement, ASTBuilderError> {
        self.expect_token(Token::While)?;

        let condition = self.parse_expression()?;
        let block = self.parse_block()?;

        Ok(Statement::While { condition, block })
    }

    // fn parse_for_statement(&mut self)        -> Result<Statement, ASTBuilderError> {}

    fn parse_fn_statement(&mut self) -> Result<Statement, ASTBuilderError> {
        self.expect_token(Token::Fn)?;

        let name = self.expect_identifier()?;

        self.expect_token(Token::OpenParen)?;
        let mut params = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::CloseParen { break };

            let param = self.expect_identifier()?;
            params.push(param);

            if self.peek() != Some(&Token::Comma) { break };
            self.consume();
        }
        self.expect_token(Token::CloseParen)?;

        let block = self.parse_block()?;

        Ok(Statement::FnDefinition { name, params, block })
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ASTBuilderError> {
        self.expect_token(Token::Let)?;

        let name = self.expect_identifier()?;

        let value = match self.peek() {
            Some(Token::Assign) => {
                self.consume();
                Some(self.parse_expression()?)
            },
            _ => None,
        };
        self.expect_token(Token::Semicolon)?;

        Ok(Statement::Let { name, value })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ASTBuilderError> {
        self.expect_token(Token::Return)?;
        let value = match self.peek() {
            Some(Token::Semicolon) => None,
            _ => Some(self.parse_expression()?),
        };
        self.expect_token(Token::Semicolon)?;
        Ok(Statement::Return { value })
    }

    fn parse_continue_statement(&mut self) -> Result<Statement, ASTBuilderError> {
        self.expect_token(Token::Continue)?;
        self.expect_token(Token::Semicolon)?;
        Ok(Statement::Continue)
    }

    fn parse_break_statement(&mut self)      -> Result<Statement, ASTBuilderError> {
        self.expect_token(Token::Break)?;
        self.expect_token(Token::Semicolon)?;
        Ok(Statement::Break)
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ASTBuilderError> {
        let value = self.parse_expression()?;
        self.expect_token(Token::Semicolon)?;
        Ok(Statement::Expression { value })
    }

    fn parse_statement(&mut self) -> Result<Statement, ASTBuilderError> {
        match self.peek() {
            Some(Token::If)       => self.parse_if_statement(),
            Some(Token::While)    => self.parse_while_statement(),
            // Some(Token::For)      => self.parse_for_statement(),
            Some(Token::Fn)       => self.parse_fn_statement(),
            Some(Token::Let)      => self.parse_let_statement(),
            Some(Token::Return)   => self.parse_return_statement(),
            Some(Token::Continue) => self.parse_continue_statement(),
            Some(Token::Break)    => self.parse_break_statement(),
            Some(_)               => self.parse_expression_statement(),
            None                  => Err(ASTBuilderError::UnexpectedEOF),
        } 
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ASTBuilderError> {
        let mut statements = Vec::new();
        self.expect_token(Token::OpenCurly)?;

        while let Some(token) = self.peek() {
            if *token == Token::CloseCurly { break; }
            statements.push(self.parse_statement()?);
        }
        self.expect_token(Token::CloseCurly)?;

        Ok(statements)
    }

    pub fn build(&mut self) -> Result<Vec<Statement>, ASTBuilderError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }
}

fn main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<String>>();

    let filepath = args.get(1).ok_or(())
        .map_err(|_| eprintln!("ERROR: Correct usage: {} <filename>", &args[0]))?;


    let source_code = fs::read_to_string(filepath)
        .map_err(|e| eprintln!("ERROR: Couldn't read file '{filepath}': {e}"))?;

    let (tokens, token_frames) = Lexer::new(&source_code).collect::<Result<(Vec<Token>, Vec<TokenFrame>), LexerError>>()
        .map_err(|e| e.print_with_source(&source_code))?;

    let tree = ASTBuilder::new(tokens, token_frames).build()
        .map_err(|e| e.print_with_source(&source_code))?;

    println!("{tree:?}");

    Ok(())
}
