use crate::TokenFrame;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
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
pub enum LexerError {
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

pub struct Lexer {
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
