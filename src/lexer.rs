use crate::{peekablecursor::PeekableCursor, reporting::Spanned};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    OpenParen,    // (
    CloseParen,   // )
    OpenCurly,    // {
    CloseCurly,   // }
    OpenBracket,  // [
    CloseBracket, // ]

    Comma,         // ,
    Dot,           // .
    Semicolon,     // ;
    Underscore,    // _
    Colon,         // :
    PathSeperator, // ::
    Range,         // ..
    QuestionMark,  // ?
    Arrow,         // ->
    FatArrow,      // =>

    Assign,  // =
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %

    Ampersand, // &
    Pipe,      // |
    Caret,     // ^

    And,  // &&
    Or,   // ||
    Bang, // !

    Equal,    // ==
    NotEqual, // !=

    LessThan,    // <
    GreaterThan, // >

    IntegerLiteral(i32),
    FloatLiteral(f32),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),

    Let,
    Fn,
    Struct,
    Trait,
    Impl,
    Enum,
    If,
    Else,
    Match,
    While,
    For,
    In,
    Return,
    Break,
    Continue,

    Eof
}

#[derive(Debug)]
pub enum LexerError {
    InvalidNumber(String),
    UnterminatedString(String),
    UnknownToken(String),
}

pub struct Lexer<'a> {
    source: PeekableCursor<'a, char>,
    emitted_eof: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [char]) -> Self {
        Self { source: PeekableCursor::new(source), emitted_eof: false }
    }

    fn consume_and_return(&mut self, token: Token) -> Token {
        self.source.consume();
        token
    }

    fn parse_token(&mut self) -> Result<Spanned<Token>, Spanned<LexerError>> {
        self.source.consume_while(|c| c.is_whitespace());
        if self.source.is_at_end() {
            self.emitted_eof = true;
            return Ok(Spanned::new(Token::Eof, self.source.position..self.source.position))
        }

        let char = self.source.peek().unwrap();

        let start_position = self.source.position;
        let token = match char {
            '{' => self.consume_and_return(Token::OpenCurly), 
            '}' => self.consume_and_return(Token::CloseCurly), 
            '(' => self.consume_and_return(Token::OpenParen), 
            ')' => self.consume_and_return(Token::CloseParen), 
            '[' => self.consume_and_return(Token::OpenBracket), 
            ']' => self.consume_and_return(Token::CloseBracket), 

            ',' => self.consume_and_return(Token::Comma), 
            '.' => {
                self.source.consume();
                match self.source.peek() {
                    Some('.') => self.consume_and_return(Token::Range),
                    _ => Token::Dot
                }
            }, 
            ';' => self.consume_and_return(Token::Semicolon), 

            ':' => {
                self.source.consume();
                match self.source.peek() {
                    Some(':') => self.consume_and_return(Token::PathSeperator),
                    _   => Token::Colon
                }
            },

            '?' => self.consume_and_return(Token::QuestionMark),

            '=' => {
                self.source.consume();
                match self.source.peek() {
                    Some('>') => self.consume_and_return(Token::FatArrow),
                    Some('=') => self.consume_and_return(Token::Equal),
                    _         => Token::Assign
                }
            },
            '-' => {
                self.source.consume();
                match self.source.peek() {
                    Some('>') => self.consume_and_return(Token::Arrow),
                    _         => Token::Minus,
                }
            },
            '+' => self.consume_and_return(Token::Plus),
            '*' => self.consume_and_return(Token::Star),
            '/' => self.consume_and_return(Token::Slash),
            '%' => self.consume_and_return(Token::Percent),
            '&' => {
                self.source.consume();
                match self.source.peek() {
                    Some('&') => self.consume_and_return(Token::And),
                    _         => Token::Ampersand
                }
            },
            '|' => {
                self.source.consume();
                match self.source.peek() {
                    Some('|') => self.consume_and_return(Token::Or),
                    _         => Token::Pipe
                }
            },
            '^' => self.consume_and_return(Token::Caret),
            '!' => {
                self.source.consume();
                match self.source.peek() {
                    Some('=') => self.consume_and_return(Token::NotEqual),
                    _         => Token::Bang
                }
            },
            '>' => self.consume_and_return(Token::GreaterThan),
            '<' => self.consume_and_return(Token::LessThan),

            '0'..='9' => {
                let mut number = self.source.consume_while(|c| c.is_numeric()).iter().collect::<String>();
                if self.source.peek() == Some(&'.') && self.source.peek_n(1) != Some(&'.') {
                    self.source.consume();
                    let decimals = self.source.consume_while(|c| c.is_numeric()).iter().collect::<String>();
                    number.push_str(".");
                    number.push_str(&decimals);

                    match number.parse::<f32>() {
                        Ok(f) => Token::FloatLiteral(f),
                        Err(_) => return Err(Spanned::new(LexerError::InvalidNumber(number), start_position..self.source.position))
                    }
                } else {
                    match number.parse::<i32>() {
                        Ok(f) => Token::IntegerLiteral(f),
                        Err(_) => return Err(Spanned::new(LexerError::InvalidNumber(number), start_position..self.source.position))
                    }
                }
            },
            '"' => {
                self.source.consume();
                let string = self.source.consume_while(|c| c != &'"').iter().collect::<String>();
                if self.source.peek().is_none() {
                    return Err(Spanned::new(LexerError::UnterminatedString(string), start_position..self.source.position));
                }
                self.source.consume();
                Token::StringLiteral(string)
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                let identifier = self.source
                    .consume_while(|c| c.is_alphanumeric() || c == &'_')
                    .iter()
                    .collect::<String>();

                match identifier.as_str() {
                    "true"  => Token::BoolLiteral(true),
                    "false" => Token::BoolLiteral(false),

                    "let"      => Token::Let,
                    "fn"       => Token::Fn,
                    "struct"   => Token::Struct,
                    "trait"    => Token::Trait,
                    "impl"     => Token::Impl,
                    "enum"     => Token::Enum,
                    "if"       => Token::If,
                    "else"     => Token::Else,
                    "match"    => Token::Match,
                    "while"    => Token::While,
                    "for"      => Token::For,
                    "in"       => Token::In,
                    "return"   => Token::Return,
                    "break"    => Token::Break,
                    "continue" => Token::Continue,
                    "_"        => Token::Underscore,
                    _ => Token::Identifier(identifier),
                }
            }
            _ => {
                let c = self.source.consume().unwrap();
                return Err(Spanned::new(LexerError::UnknownToken(c.to_string()), start_position..self.source.position))
            },
        };

        Ok(Spanned::new(token, start_position..self.source.position))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Spanned<Token>, Spanned<LexerError>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.emitted_eof {
            None
        } else {
            Some(self.parse_token())
        }
    }
}
