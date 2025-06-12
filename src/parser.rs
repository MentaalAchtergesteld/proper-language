use std::collections::HashMap;

use crate::{cursor::Cursor, lexer::Token};

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Equal,
    NotEqual,
    Lesser,
    LesserEqual,
    Greater,
    GreaterEqual,

    And,
    Or,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Invert,
    Negative,
    Positive
}

#[derive(Debug)]
pub enum RangeType {
    Exclusive,
    Inclusive
}

#[derive(Debug)]
pub enum Literal {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    String(String)
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),

    Array(Vec<Expression>),
    Object(HashMap<String, Expression>),

    BinaryOperation {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: BinaryOperator
    },
    UnaryOperator {
        expression: Box<Expression>,
        operator: UnaryOperator 
    },
    FunctionCall {
        function: Box<Expression>,
        args: Vec<Expression>
    },
    ArrayIndex {
        target: Box<Expression>,
        index: Box<Expression>
    },
    Field {
        target: Box<Expression>,
        name: String
    },
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        range_type: RangeType
    }
}

#[derive(Debug)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Wildcard
}

#[derive(Debug)]
pub enum MatchBody {
    Expression(Expression),
    Block(Vec<Statement>)
}

#[derive(Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchBody
} 

#[derive(Debug)]
pub enum ElseBranch {
    Else(Vec<Statement>),
    If(Box<Statement>)
}

#[derive(Debug)]
pub enum Statement {
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<ElseBranch>
    },
    While {
        condition: Expression,
        block: Vec<Statement>
    },
    For {
        iterator: String,
        iterable: Expression,
        block: Vec<Statement>
    },
    Break,
    Continue,
    FunctionDefinition {
        name: String,
        parameters: Vec<String>,
        block: Vec<Statement>
    },
    Return(Option<Expression>),
    Let {
        name: String,
        value: Option<Expression>
    },
    Assign {
        target: Expression,
        operator: Option<BinaryOperator>,
        value: Expression,
    },
    Match {
        expression: Expression,
        arms: Vec<MatchArm>
    },
    Expression(Expression)
}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token),
    ExpectedToken(Token),
    UnexpectedEndOfFile
}

pub struct Parser<'a> {
    tokens: Cursor<'a, Token> 
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: Cursor::new(tokens)
        }
    }

    fn expect_token(&mut self, token: Token) -> Result<Token, ParserError> {
        let found_token = self.tokens.chop(1).ok_or(ParserError::UnexpectedEndOfFile)?[0].clone();
        
        if found_token == token {
            Ok(found_token)
        } else {
            Err(ParserError::ExpectedToken(token))
        }
    }

    fn expect_identifier(&mut self) -> Result<Expression, ParserError> {
        let next_token = self.tokens.chop(1).ok_or(ParserError::UnexpectedEndOfFile)?[0].clone();
        match next_token { 
            Token::Identifier(identifier) => Ok(Expression::Identifier(identifier)),
            _ => Err(ParserError::ExpectedToken(Token::Identifier("".into()))),
        }
    }

    fn parse_array(&mut self) -> Result<Expression, ParserError> {
        self.expect_token(Token::LBracket)?;
        let mut elements = Vec::new();

        while self.tokens.peek(0) != Some(&Token::RBRacket) {
            elements.push(self.parse_expression()?);

            match self.tokens.peek(0) {
                Some(&Token::Comma) => self.tokens.chop(1),
                Some(&Token::RBRacket) => break,
                Some(other) => return Err(ParserError::UnexpectedToken(other.clone())),
                _ => return Err(ParserError::UnexpectedEndOfFile)
            };
        }
        self.expect_token(Token::RBRacket)?;

        Ok(Expression::Array(elements))
    }

    fn parse_object(&mut self) -> Result<Expression, ParserError> {
        self.expect_token(Token::LBrace)?;
        let mut values = HashMap::new();

        while self.tokens.peek(0) != Some(&Token::RBrace) {
            let key = match self.expect_identifier()? {
                Expression::Identifier(name) => name,
                _ => unreachable!()
            };

            self.expect_token(Token::Colon)?;

            let value = self.parse_expression()?;
            values.insert(key, value); 

            match self.tokens.peek(0) {
                Some(&Token::Comma) => self.tokens.chop(1),
                Some(&Token::RBrace) => break,
                Some(other) => return Err(ParserError::UnexpectedToken(other.clone())),
                _ => return Err(ParserError::UnexpectedEndOfFile)
            };
        }

        self.expect_token(Token::RBrace)?;

        Ok(Expression::Object(values))
    }

    fn parse_atom(&mut self) -> Result<Expression, ParserError> {
        match self.tokens.peek(0).cloned() {
            Some(Token::Int(i)) => { self.tokens.chop(1); Ok(Expression::Literal(Literal::Integer(i))) },
            Some(Token::Float(f)) => { self.tokens.chop(1); Ok(Expression::Literal(Literal::Float(f))) },
            Some(Token::Boolean(b)) => { self.tokens.chop(1); Ok(Expression::Literal(Literal::Boolean(b))) },
            Some(Token::String(s)) => { self.tokens.chop(1); Ok(Expression::Literal(Literal::String(s))) },
            Some(Token::Identifier(ident)) => { self.tokens.chop(1); Ok(Expression::Identifier(ident)) },
            Some(Token::LParen) => {
                self.tokens.chop(1);
                let expression = self.parse_expression()?;
                self.expect_token(Token::RParen)?;
                Ok(expression)
            },
            Some(Token::LBracket) => self.parse_array(),
            Some(Token::LBrace) => self.parse_object(),
            Some(other) => Err(ParserError::UnexpectedToken(other.clone())),
            _ => Err(ParserError::UnexpectedEndOfFile)
        }
    }

    fn parse_postfix(&mut self, value: Expression) -> Result<Expression, ParserError> {
        match self.tokens.peek(0) {
            Some(&Token::LParen) => {
                self.tokens.chop(1);

                let mut args = Vec::new();

                while self.tokens.peek(0) != Some(&Token::RParen) {
                    args.push(self.parse_expression()?);

                    match self.tokens.peek(0) {
                        Some(&Token::Comma) => self.tokens.chop(1),
                        Some(&Token::RParen) => break,
                        Some(other) => return Err(ParserError::UnexpectedToken(other.clone())),
                        _ => return Err(ParserError::UnexpectedEndOfFile)
                    };
                }

                self.expect_token(Token::RParen)?;

                Ok(Expression::FunctionCall { function: Box::new(value), args })
            },
            Some(&Token::LBracket) => {
                self.tokens.chop(1);

                let index = Box::new(self.parse_expression()?);
                self.expect_token(Token::RBRacket)?;

                Ok(Expression::ArrayIndex { target: Box::new(value), index })
            },
            Some(&Token::Dot) => {
                self.tokens.chop(1);

                let name = match self.expect_identifier()? {
                    Expression::Identifier(name) => name,
                    _ => unreachable!()
                };

                Ok(Expression::Field { target: Box::new(value), name })
            },
            Some(other) => Err(ParserError::UnexpectedToken(other.clone())),
            _ => Err(ParserError::UnexpectedEndOfFile)
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_atom()?;

        while let Some(_token @ (&Token::LParen | &Token::LBracket | &Token::Dot)) = self.tokens.peek(0) {
            expression = self.parse_postfix(expression)?;
        }

        Ok(expression)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, ParserError> {
        let operator = match self.tokens.peek(0) {
            Some(&Token::Bang) => { self.tokens.chop(1); Some(UnaryOperator::Invert) },
            Some(&Token::Minus) => { self.tokens.chop(1); Some(UnaryOperator::Negative) }, 
            Some(&Token::Plus) => { self.tokens.chop(1); Some(UnaryOperator::Positive) }, 
            _ => None
        };

        let expression = self.parse_primary_expression()?;

        if let Some(operator) = operator {
            Ok(Expression::UnaryOperator { expression: Box::new(expression), operator })
        } else {
            Ok(expression)
        }
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression, ParserError> {
        let left = Box::new(self.parse_unary_expression()?);

        match self.tokens.peek(0) {
            Some(&Token::Star) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_unary_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::Mul })
            },
            Some(&Token::Slash) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_unary_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::Div })
            },
            _ => Ok(*left)
        }

    }

    fn parse_additive_expression(&mut self) -> Result<Expression, ParserError> {
        let left = Box::new(self.parse_multiplicative_expression()?);

        match self.tokens.peek(0) {
            Some(&Token::Plus) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_multiplicative_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::Add })
            },
            Some(&Token::Minus) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_multiplicative_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::Sub })
            },
            _ => Ok(*left)
        }

    }

    fn parse_relational_expression(&mut self) -> Result<Expression, ParserError> {
        let left = Box::new(self.parse_additive_expression()?);

        match self.tokens.peek(0) {
            Some(&Token::Lesser) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_additive_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::Lesser })
            },
            Some(&Token::LesserEqual) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_additive_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::LesserEqual})
            },
            Some(&Token::Greater) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_additive_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::Greater})
            },
            Some(&Token::GreaterEqual) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_additive_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::GreaterEqual})
            },
            _ => Ok(*left)
        }

    }

    fn parse_equality_expression(&mut self) -> Result<Expression, ParserError> {
        let left = Box::new(self.parse_relational_expression()?);

        match self.tokens.peek(0) {
            Some(&Token::Equal) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_relational_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::Equal })
            },
            Some(&Token::NotEqual) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_relational_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::NotEqual })
            },
            _ => Ok(*left)
        }

    }

    fn parse_logical_and_expression(&mut self) -> Result<Expression, ParserError> {
        let left = Box::new(self.parse_equality_expression()?);

        match self.tokens.peek(0) {
            Some(&Token::And) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_equality_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::And })
            },
            _ => Ok(*left)
        }
    }

    fn parse_logical_or_expression(&mut self) -> Result<Expression, ParserError> {
        let left = Box::new(self.parse_logical_and_expression()?);

        match self.tokens.peek(0) {
            Some(&Token::Or) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_logical_and_expression()?);

                Ok(Expression::BinaryOperation { left, right, operator: BinaryOperator::Or })
            },
            _ => Ok(*left)
        }
    }

    fn parse_range_expression(&mut self) -> Result<Expression, ParserError> {
        let left = Box::new(self.parse_logical_or_expression()?);

        match self.tokens.peek(0) {
            Some(&Token::Range) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_logical_or_expression()?);

                Ok(Expression::Range { start: left, end: right, range_type: RangeType::Exclusive })
            },
            Some(&Token::RangeInclusive) => {
                self.tokens.chop(1);
                let right = Box::new(self.parse_logical_or_expression()?);

                Ok(Expression::Range { start: left, end: right, range_type: RangeType::Inclusive })
            },
            _ => Ok(*left),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_range_expression()
    }

    fn parse_if(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::KeywordIf)?;

        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;

        let else_branch = if self.tokens.peek(0) == Some(&Token::KeywordElse) {
            self.tokens.chop(1);
            if self.tokens.peek(0) == Some(&Token::KeywordIf) {
                Some(ElseBranch::If(Box::new(self.parse_if()?)))
            } else {
                Some(ElseBranch::Else(self.parse_block()?))
            }
        } else { None };

        Ok(Statement::If { condition, then_branch, else_branch })
    }

    fn parse_while(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::KeywordWhile)?;

        let condition = self.parse_expression()?;
        let block = self.parse_block()?;

        Ok(Statement::While { condition, block })
    }

    fn parse_for(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::KeywordFor)?;

        let iterator = match self.expect_identifier()? {
            Expression::Identifier(ident) => ident,
            _ => unreachable!(),
        };

        self.expect_token(Token::KeywordIn)?;
        let iterable = self.parse_expression()?; 

        let block = self.parse_block()?;

        Ok(Statement::For { iterator, iterable, block })
    }

    fn parse_param_list(&mut self) -> Result<Vec<String>, ParserError> {
        let mut params = Vec::new();
        loop {
            match self.expect_identifier()? {
                Expression::Identifier(ident) => params.push(ident),
                _ => unreachable!()
            }

            match self.tokens.peek(0) {
                Some(&Token::Comma) => { self.tokens.chop(1) }
                _ => break
            };
        }

        Ok(params)
    }

    fn parse_function(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::KeywordFn)?;
        let identifier = self.expect_identifier()?;

        let name = match identifier {
            Expression::Identifier(ident) => ident,
            _ => unreachable!()
        };

        self.expect_token(Token::LParen)?;
        let parameters = self.parse_param_list()?;
        self.expect_token(Token::RParen)?;

        let block = self.parse_block()?;

        Ok(Statement::FunctionDefinition { name, parameters, block })
    }

    fn parse_return(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::KeywordReturn)?;

        let value = if self.tokens.peek(0) == Some(&Token::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.expect_token(Token::Semicolon)?;

        Ok(Statement::Return(value))
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParserError> {
        let pattern = match self.tokens.peek(0).cloned() {
            Some(Token::Identifier(name)) => { self.tokens.chop(1); Pattern::Identifier(name) },
            Some(Token::Underscore) => { self.tokens.chop(1); Pattern::Wildcard },
            Some(Token::Int(i)) => { self.tokens.chop(1); Pattern::Literal(Literal::Integer(i)) },
            Some(Token::Float(f)) => { self.tokens.chop(1); Pattern::Literal(Literal::Float(f)) },
            Some(Token::Boolean(b)) => { self.tokens.chop(1); Pattern::Literal(Literal::Boolean(b)) },
            Some(Token::String(s)) => { self.tokens.chop(1); Pattern::Literal(Literal::String(s)) },
            Some(other) => return Err(ParserError::UnexpectedToken(other.clone())),
            _ => return Err(ParserError::UnexpectedEndOfFile)

        };

        self.expect_token(Token::MatchArrow)?;

        let body = if self.tokens.peek(0) == Some(&Token::LBrace) {
            MatchBody::Block(self.parse_block()?)
        } else {
            MatchBody::Expression(self.parse_expression()?)
        };

        Ok(MatchArm { pattern, body })

    }

    fn parse_match(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::KeywordMatch)?;

        let expression = self.parse_expression()?;

        self.expect_token(Token::LBrace)?;

        let mut arms = Vec::new();

        while self.tokens.peek(0) != Some(&Token::RBrace) {
            arms.push(self.parse_match_arm()?);

            self.expect_token(Token::Comma)?;
        }

        self.expect_token(Token::RBrace)?;

        Ok(Statement::Match { expression, arms })
    }

    fn parse_let(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::KeywordLet)?;

        let name = match self.expect_identifier()? {
            Expression::Identifier(name) => name,
            _ => unreachable!()
        };

        let value = if self.tokens.peek(0) == Some(&Token::Assign) {
            self.expect_token(Token::Assign)?;

            let expression = self.parse_expression()?;
            Some(expression)
        } else {
            None
        };

        self.expect_token(Token::Semicolon)?;
        
        Ok(Statement::Let { name, value })
    }

    fn parse_break(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::KeywordBreak)?;
        Ok(Statement::Break)
    }

    fn parse_continue(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::KeywordContinue)?;
        Ok(Statement::Continue)
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParserError> {
        let left = self.parse_expression()?;

        let statement = match self.tokens.peek(0) {
            Some(&Token::Assign) => {
                self.tokens.chop(1);
                let right = self.parse_expression()?;
                Ok(Statement::Assign { target: left, operator: None, value: right })
            },
            Some(&Token::PlusAssign) => {
                self.tokens.chop(1);
                let right = self.parse_expression()?;
                Ok(Statement::Assign { target: left, operator: Some(BinaryOperator::Add), value: right })
            },
            Some(&Token::MinusAssign) => {
                self.tokens.chop(1);
                let right = self.parse_expression()?;
                Ok(Statement::Assign { target: left, operator: Some(BinaryOperator::Sub), value: right })
            },
            Some(&Token::StarAssign) => {
                self.tokens.chop(1);
                let right = self.parse_expression()?;
                Ok(Statement::Assign { target: left, operator: Some(BinaryOperator::Mul), value: right })
            },
            Some(&Token::SlashAssign) => {
                self.tokens.chop(1);
                let right = self.parse_expression()?;
                Ok(Statement::Assign { target: left, operator: Some(BinaryOperator::Div), value: right })
            },
            Some(&Token::Semicolon) =>  Ok(Statement::Expression(left)),
            Some(_) => return Err(ParserError::ExpectedToken(Token::Semicolon)),
            _ => return Err(ParserError::UnexpectedEndOfFile)
        };

        self.expect_token(Token::Semicolon)?;
        statement
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.tokens.peek(0) {
            Some(Token::KeywordIf) => self.parse_if(),
            Some(Token::KeywordWhile) => self.parse_while(),
            Some(Token::KeywordFor) => self.parse_for(),
            Some(Token::KeywordFn) => self.parse_function(),
            Some(Token::KeywordReturn) => self.parse_return(),
            Some(Token::KeywordMatch) => self.parse_match(),
            Some(Token::KeywordLet) => self.parse_let(),
            Some(Token::KeywordBreak) => self.parse_break(),
            Some(Token::KeywordContinue) => self.parse_continue(),
            Some(_) => self.parse_expression_statement(),
            None => Err(ParserError::UnexpectedEndOfFile),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParserError> {
        self.expect_token(Token::LBrace)?;
        
        let mut statements = Vec::new();
        while self.tokens.peek(0) != Some(&Token::RBrace) {
           statements.push(self.parse_statement()?); 
        }

        self.expect_token(Token::RBrace)?;
        Ok(statements) 
    }

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, ParserError> {
        let mut statements = Vec::new();

        while self.tokens.peek(0) != Some(&Token::EOF) {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }
}
