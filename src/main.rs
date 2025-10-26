use std::{fs, iter::Peekable, str::Chars};

#[derive(Debug)]
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
    For,
    Fn,
    Let,
    Return,
    Continue,
    Break,

    EndOfFile,
}

#[derive(Debug)]
enum LexerError {
    UnterminatedString(String),
    InvalidNumber(String),
    UnknownToken(String),
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer { chars: source.chars().peekable() }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else { break }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();    

        let next_char = match self.chars.next() {
            Some(c) => c,
            None => return None
        };

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

            '=' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::Equal)
                },
                _ => Ok(Token::Assign)
            },
            '+' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::PlusAssign)
                },
                _ => Ok(Token::Plus)
            },
            '-' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::MinusAssign)
                },
                _ => Ok(Token::Minus)
            },
            '*' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::StarAssign)
                },
                _ => Ok(Token::Star)
            },
            '/' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::SlashAssign)
                },
                _ => Ok(Token::Slash)
            },
            '%' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::PercentAssign)
                },
                _ => Ok(Token::Percent)
            },


            '&' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::AmpersandAssign)
                },
                Some('&') => {
                    self.chars.next();
                    Ok(Token::And)
                }
                _ => Ok(Token::Ampersand)
            },
            '|' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::PipeAssign)
                },
                Some('|') => {
                    self.chars.next();
                    Ok(Token::Or)
                }
                _ => Ok(Token::Pipe)
            },
            '^' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::CaretAssign)
                },
                _ => Ok(Token::Caret)
            },
            '>' => match self.chars.peek() {
                Some('>') => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some('=') => {
                            self.chars.next();
                            Ok(Token::RightShiftAssign)
                        },
                        _ => Ok(Token::RightShift)
                    }
                },
                Some('=') => {
                    self.chars.next();

                    Ok(Token::GreaterThanEqual)
                }
                _ => Ok(Token::GreaterThan)
            },
            '<' => match self.chars.peek() {
                Some('<') => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some('=') => {
                            self.chars.next();
                            Ok(Token::RightShiftAssign)
                        },
                        _ => Ok(Token::RightShift)
                    }
                },
                Some('=') => {
                    self.chars.next();
                    Ok(Token::LessThanEqual)
                }
                _ => Ok(Token::LessThan)
            },
            '!' => match self.chars.peek() {
                Some('=') => {
                    self.chars.next();
                    Ok(Token::NotEqual)
                },
                _ => Ok(Token::Bang)
            },
            next_char if next_char.is_digit(10) => {
                let mut number_str = String::from(next_char);
                let mut seen_dot = false;

                while let Some(&c) = self.chars.peek() {
                    match c {
                        c if c.is_digit(10) => number_str.push(self.chars.next().unwrap()),
                        '.' => {
                            number_str.push(self.chars.next().unwrap());
                            seen_dot = true;
                        },
                        _ => break,
                    }
                }

                if seen_dot {
                    match number_str.parse::<f32>() {
                        Ok(n) => Ok(Token::FloatLiteral(n)),
                        Err(_) => Err(LexerError::InvalidNumber(number_str))
                    }
                } else {
                    match number_str.parse::<i32>() {
                        Ok(n) => Ok(Token::IntegerLiteral(n)),
                        Err(_) => Err(LexerError::InvalidNumber(number_str))
                    }
                }
            },
            '"' => {
                let mut string_str = String::new();
                let mut closed = false;
                while let Some(&_) = self.chars.peek() {
                    let c = self.chars.next().unwrap();
                    if c == '"' { closed = true; break };
                    string_str.push(c);
                }

                if closed {
                    Ok(Token::StringLiteral(string_str))
                } else {
                    Err(LexerError::UnterminatedString(string_str))
                }

            }
            _ => {
                let mut identifier_str = String::from(next_char);
                while let Some(&c) = self.chars.peek() {
                    if c.is_whitespace() { break };
                    if !c.is_alphanumeric() && c != '_' { break };
                    identifier_str.push(self.chars.next().unwrap());
                }

                match identifier_str.as_str() {
                    "true"     => Ok(Token::BoolLiteral(true)),
                    "false"    => Ok(Token::BoolLiteral(false)),
                    "if"       => Ok(Token::If),
                    "else"     => Ok(Token::Else),
                    "while"    => Ok(Token::While),
                    "for"      => Ok(Token::For),
                    "fn"       => Ok(Token::Fn),
                    "let"      => Ok(Token::Let),
                    "return"   => Ok(Token::Return),
                    "continue" => Ok(Token::Continue),
                    "break"    => Ok(Token::Break),
                    _          => Ok(Token::Identifier(identifier_str))
                }
            }
        };

        Some(token_result)
    }
}

fn main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        let program = &args[0];
        eprintln!("ERROR: Correct usage: {program} <filename>");
        return Err(());
    }

    let filepath = &args[1];

    let source_code = fs::read_to_string(filepath)
        .map_err(|e| eprintln!("ERROR: Couldn't read file '{filepath}': {e}"))?;


    for token in Lexer::new(&source_code) {
        println!("Token: {token:?}");
    }

    Ok(())
}
