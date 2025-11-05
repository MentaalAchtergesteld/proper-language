use crate::{lexer::Token, TokenFrame};

#[derive(Debug)]
pub enum ASTBuilderError {
    UnexpectedToken(Token, TokenFrame),
    ExpectedToken(Token, TokenFrame),
    UnexpectedEOF(TokenFrame),
}

impl ASTBuilderError {
    pub fn print_with_source(&self, source: &str) {
        let (message, frame) = match self {
            Self::UnexpectedToken(s, f) => (format!("Unexpected token: {:?}", s), f),
            Self::ExpectedToken(s, f) => (format!("Expected {:?}", s), f),
            Self::UnexpectedEOF(f) => (format!("Unexpected end of file"), f),
        };

        eprintln!("Error: {} at line {}, column {}",
            message,
            frame.line + 1,
            frame.column + 1
        );

        frame.print_source_context(source);
    }
}

#[derive(Debug)]
pub enum BinaryOperator {
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
pub enum UnaryOperator {
    Not,
    Negate,
}

#[derive(Debug)]
pub enum LiteralValue {
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool)
}

#[derive(Debug)]
pub enum Expression {
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
pub enum Statement {
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
        value: Expression
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

pub struct ASTBuilder {
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
        let frame = self.frames[self.position];
        let next = self.consume().ok_or(ASTBuilderError::UnexpectedEOF(frame))?;
        if *next == token {
            Ok(next)
        } else {
            Err(ASTBuilderError::ExpectedToken(token, frame))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ASTBuilderError> {
        let frame = self.frames[self.position];
        let next = self.consume().ok_or(ASTBuilderError::UnexpectedEOF(frame))?;
        match next {
            Token::Identifier(name) => Ok(name.clone()),
            _ => Err(ASTBuilderError::ExpectedToken(Token::Identifier(String::new()), frame))
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
        let pos = self.position;

        match self.peek() {
            Some(Token::OpenParen) => {
                self.consume();
                let expr = self.parse_expression()?;
                self.expect_token(Token::CloseParen)?;
                Ok(expr)
            },
            
            Some(Token::OpenBracket) => self.parse_array(),
            Some(Token::OpenCurly) => self.parse_struct(),

            Some(Token::IntegerLiteral(il)) => {
                let val = *il;
                self.consume();
                Ok(Expression::Literal(LiteralValue::Integer(val)))
            },
            Some(Token::FloatLiteral(fl)) => {
                let val = *fl;
                self.consume();
                Ok(Expression::Literal(LiteralValue::Float(val)))
            },
            Some(Token::StringLiteral(sl)) => {
                let val = sl.clone();
                self.consume();
                Ok(Expression::Literal(LiteralValue::String(val)))
            },
            Some(Token::BoolLiteral(bl)) => {
                let val = *bl;
                self.consume();
                Ok(Expression::Literal(LiteralValue::Bool(val)))
            },
            Some(Token::Identifier(ident)) => {
                let val = ident.clone();
                self.consume();
                Ok(Expression::Variable(val))
            }
            Some(token) => Err(ASTBuilderError::UnexpectedToken(
                token.clone(), 
                self.frames[pos].clone()
            )),
            
            None => {
                let frame = if pos > 0 {
                    self.frames[pos - 1].clone()
                } else {
                    TokenFrame { line: 0, column: 0, length: 0 }
                };
                Err(ASTBuilderError::UnexpectedEOF(frame))
            }
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
        self.expect_token(Token::Assign)?;
        let value = self.parse_expression()?;
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
        let expr = self.parse_expression()?;

        let stmt = if let Some(Token::Assign) = self.peek() {
            self.consume();
            let value = self.parse_expression()?;
            Statement::Expression {
                value: Expression::Assign { variable: Box::new(expr), value: Box::new(value) }
            }
        } else {
            Statement::Expression { value: expr }
        };
        self.expect_token(Token::Semicolon)?;
        Ok(stmt)
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
            None                  => Err(ASTBuilderError::UnexpectedEOF(self.frames[self.position])),
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
