use crate::peekablecursor::PeekableCursor;

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

    fn parse_token(&mut self) -> Result<Token, LexerError> {
        self.source.consume_while(|c| c.is_whitespace());
        if self.source.is_at_end() { self.emitted_eof = true; return Ok(Token::Eof) }

        let char = self.source.peek().unwrap();

        match char {
            '{' => Ok(self.consume_and_return(Token::OpenCurly)), 
            '}' => Ok(self.consume_and_return(Token::CloseCurly)), 
            '(' => Ok(self.consume_and_return(Token::OpenParen)), 
            ')' => Ok(self.consume_and_return(Token::CloseParen)), 
            '[' => Ok(self.consume_and_return(Token::OpenBracket)), 
            ']' => Ok(self.consume_and_return(Token::CloseBracket)), 

            ',' => Ok(self.consume_and_return(Token::Comma)), 
            '.' => {
                self.source.consume();
                match self.source.peek() {
                    Some('.') => Ok(self.consume_and_return(Token::Range)),
                    _ => Ok(Token::Dot)
                }
            }, 
            ';' => Ok(self.consume_and_return(Token::Semicolon)), 

            ':' => {
                self.source.consume();
                match self.source.peek() {
                    Some(':') => Ok(self.consume_and_return(Token::PathSeperator)),
                    _   => Ok(Token::Colon)
                }
            },

            '?' => Ok(self.consume_and_return(Token::QuestionMark)),

            '=' => {
                self.source.consume();
                match self.source.peek() {
                    Some('>') => Ok(self.consume_and_return(Token::FatArrow)),
                    Some('=') => Ok(self.consume_and_return(Token::Equal)),
                    _         => Ok(Token::Assign)
                }
            },
            '-' => {
                self.source.consume();
                match self.source.peek() {
                    Some('>') => Ok(self.consume_and_return(Token::Arrow)),
                    _         => Ok(Token::Minus),
                }
            },
            '+' => Ok(self.consume_and_return(Token::Plus)),
            '*' => Ok(self.consume_and_return(Token::Star)),
            '/' => Ok(self.consume_and_return(Token::Slash)),
            '%' => Ok(self.consume_and_return(Token::Percent)),
            '&' => {
                self.source.consume();
                match self.source.peek() {
                    Some('&') => Ok(self.consume_and_return(Token::And)),
                    _         => Ok(Token::Ampersand)
                }
            },
            '|' => {
                self.source.consume();
                match self.source.peek() {
                    Some('|') => Ok(self.consume_and_return(Token::Or)),
                    _         => Ok(Token::Pipe)
                }
            },
            '^' => Ok(self.consume_and_return(Token::Caret)),
            '!' => {
                self.source.consume();
                match self.source.peek() {
                    Some('=') => Ok(self.consume_and_return(Token::NotEqual)),
                    _         => Ok(Token::Bang)
                }
            },
            '>' => Ok(self.consume_and_return(Token::GreaterThan)),
            '<' => Ok(self.consume_and_return(Token::LessThan)),

            '0'..='9' => {
                let mut number = self.source.consume_while(|c| c.is_numeric()).iter().collect::<String>();
                if let Some('.') = self.source.peek() {
                    self.source.consume();
                    let decimals = self.source.consume_while(|c| c.is_numeric()).iter().collect::<String>();
                    number.push_str(".");
                    number.push_str(&decimals);

                    number.parse::<f32>()
                        .map(|f| Token::FloatLiteral(f))
                        .map_err(|_| LexerError::InvalidNumber(number))
                } else {
                    number.parse::<i32>()
                        .map(|f| Token::IntegerLiteral(f))
                        .map_err(|_| LexerError::InvalidNumber(number))
                }
            },
            '"' => {
                self.source.consume();
                let string = self.source.consume_while(|c| c != &'"').iter().collect::<String>();
                if self.source.peek().is_none() {
                    return Err(LexerError::UnterminatedString(string));
                }
                self.source.consume();
                Ok(Token::StringLiteral(string))
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                let identifier = self.source
                    .consume_while(|c| c.is_alphanumeric() || c == &'_')
                    .iter()
                    .collect::<String>();

                match identifier.as_str() {
                    "true" => Ok(Token::BoolLiteral(true)),
                    "false" => Ok(Token::BoolLiteral(false)),

                    "let"      => Ok(Token::Let),
                    "fn"       => Ok(Token::Fn),
                    "struct"   => Ok(Token::Struct),
                    "trait"    => Ok(Token::Trait),
                    "impl"     => Ok(Token::Impl),
                    "enum"     => Ok(Token::Enum),
                    "if"       => Ok(Token::If),
                    "else"     => Ok(Token::Else),
                    "match"    => Ok(Token::Match),
                    "while"    => Ok(Token::While),
                    "for"      => Ok(Token::For),
                    "in"       => Ok(Token::In),
                    "return"   => Ok(Token::Return),
                    "break"    => Ok(Token::Break),
                    "continue" => Ok(Token::Continue),
                    "_"        => Ok(Token::Underscore),
                    _ => Ok(Token::Identifier(identifier)),
                }
            }
            _ => {
                let c = self.source.consume().unwrap();
                Err(LexerError::UnknownToken(c.to_string()))
            },
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.emitted_eof {
            None
        } else {
            Some(self.parse_token())
        }
    }
}
