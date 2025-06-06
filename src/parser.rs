use std::collections::HashMap;

use crate::{cursor::Cursor, lexer::Token};

enum BinaryOperator {
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

enum UnaryOperator {
    Invert,
    Negative,
    Positive
}

enum RangeType {
    Exclusive,
    Inclusive
}

enum Literal {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    String(String)
}

enum Expression {
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

pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Wildcard
}

pub enum MatchBody {
    Expression(Expression),
    Block(Vec<Statement>)
}

pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchBody
} 

pub enum ElseBranch {
    Else(Vec<Statement>),
    If(Box<Statement>)
}

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
    Match {
        expression: Expression,
        arms: Vec<MatchArm>
    },
    Expression(Expression)
}

enum ParserError {
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

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {

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
        let identifier = self.expect_identifier()?;

        let iterator = match identifier {
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

    fn parse_return(&mut self) -> Result<Statement, ParserError> {}
    fn parse_match(&mut self) -> Result<Statement, ParserError> {}
    fn parse_let(&mut self) -> Result<Statement, ParserError> {}
    fn parse_break(&mut self) -> Result<Statement, ParserError> {}
    fn parse_continue(&mut self) -> Result<Statement, ParserError> {}
    fn parse_expression_statement(&mut self) -> Result<Statement, ParserError> {}

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParserError> {}

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, ParserError> {

    }
}
