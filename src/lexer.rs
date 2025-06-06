use crate::cursor::Cursor;

#[derive(Debug)]
pub enum Token {
    // Literals
    
    Hex(i32),
    Bin(i32),
    Int(i32),
    Float(f32),
    String(String),
    Boolean(bool),
    
    // Identifiers & Keywords
    
    Identifier(String),

    KeywordIf,
    KeywordElse,
    KeywordWhile,
    KeywordFor,
    KeywordFn,
    KeywordLet,
    KeywordReturn,
    KeywordMatch,
    KeywordBreak,
    KeywordContinue,

    // Arithmetic Operators
    
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Comparison Operators
    
    Equal,
    NotEqual,
    Lesser,
    LesserEqual,
    Greater,
    GreaterEqual,

    // Logical Operators

    And,
    Or,
    Bang,

    // Bitwise Operators

    Ampersand,
    Pipe,
    Caret,
    LeftShift,
    RightShift,
    LeftShiftAssign,
    RightShiftAssign,
    AmpersandAssign,
    PipeAssign,
    CaretAssign,

    // Assignment Operators
    
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,

    // Punctuation
    
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBRacket,
    Comma,
    Semicolon,
    Colon,
    Dot,

    EOF
}


#[derive(Debug)]
pub enum LexerError {
    UnexpectedEndOfFile,
    InvalidString,
    InvalidNumber,
    UnknownToken(char)
}

pub struct Lexer<'a> {
    content: Cursor<'a, char>, 
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self {
            content: Cursor::new(content)
        }
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.content.chop_while(|c| c.is_whitespace());
        
        let Some(c) = self.content.peek(0) else {
            return Ok(Token::EOF)
        };

        match c {
            '+' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::PlusAssign) },
                _ => { self.content.chop(1); Ok(Token::Plus) },
            },
            '-' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::MinusAssign) },
                _ => { self.content.chop(1); Ok(Token::Minus) },
            },
            '*' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::StarAssign) },
                _ => { self.content.chop(1); Ok(Token::Star) },

            },
            '/' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::SlashAssign) },
                _ => { self.content.chop(1); Ok(Token::Slash) },
            },
            '%' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::PercentAssign) },
                _ => { self.content.chop(1); Ok(Token::Percent) },
            },

            '=' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::Equal) },
                _ => { self.content.chop(1); Ok(Token::Assign) },
            },
            '<' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::LesserEqual) },
                Some('<') => match self.content.peek(1) {
                    Some('=') => { self.content.chop(3); Ok(Token::LeftShiftAssign) },
                    _ => { self.content.chop(2); Ok(Token::LeftShift) }
                }
                _ => { self.content.chop(1); Ok(Token::Lesser) },
            },
            '>' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::GreaterEqual) },
                Some('>') => match self.content.peek(1) {
                    Some('=') => { self.content.chop(3); Ok(Token::RightShiftAssign) },
                    _ => { self.content.chop(2); Ok(Token::RightShift) }
                }
                _ => { self.content.chop(1); Ok(Token::Greater) },
            },
            '&' => match self.content.peek(1) {
                Some('&') => { self.content.chop(2); Ok(Token::And) },
                _ => { self.content.chop(1); Ok(Token::Ampersand) },
            },
            '|' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::Or) },
                _ => { self.content.chop(1); Ok(Token::Pipe) },
            },
            '!' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::NotEqual) },
                _ => { self.content.chop(1); Ok(Token::Bang) },
            },
            '^' => match self.content.peek(1) {
                Some('=') => { self.content.chop(2); Ok(Token::CaretAssign) },
                _ => { self.content.chop(1); Ok(Token::Caret) },
            },
            
            '"' => {
                self.content.chop(1);
                let string_chars = self.content.chop_while(|c| c != &'"').ok_or(LexerError::InvalidString)?;
                let string = string_chars.iter().collect::<String>();
                if self.content.peek(0) == Some(&'"') {
                    self.content.chop(1);
                    Ok(Token::String(string))
                } else {
                   Err(LexerError::InvalidString) 
                }
            }

            '0'..='9' => match self.content.peek(1) {
                Some('x') | Some('X') => {
                    self.content.chop(2);
                    let hex_digits = self.content.chop_while(|c| c.is_digit(16)).ok_or(LexerError::InvalidNumber)?;
                    let hex_str = hex_digits.iter().collect::<String>();
                    let hex_number = i32::from_str_radix(&hex_str, 16).or(Err(LexerError::InvalidNumber))?;
                    Ok(Token::Hex(hex_number))
                }
                Some('b') | Some('B') => {
                    self.content.chop(2);
                    let bin_digits = self.content.chop_while(|c| c.is_digit(2)).ok_or(LexerError::InvalidNumber)?;
                    let bin_str = bin_digits.iter().collect::<String>();
                    let bin_number = i32::from_str_radix(&bin_str, 2).or(Err(LexerError::InvalidNumber))?;
                    Ok(Token::Bin(bin_number))
                },
                _ =>  {
                    let digits = self.content.chop_while(|c| c.is_digit(10) || c == &'.').ok_or(LexerError::InvalidNumber)?;
                    let str = digits.iter().collect::<String>();
                    let is_float = str.contains('.');

                    if is_float {
                        let parsed = str.parse::<f32>();
                        if let Ok(float) = parsed {
                            Ok(Token::Float(float))
                        } else {
                            Err(LexerError::InvalidNumber) 
                        }
                    } else {
                        let parsed = str.parse::<i32>();
                        if let Ok(integer) = parsed {
                            Ok(Token::Int(integer))
                        } else {
                            Err(LexerError::InvalidNumber) 
                        }
                    }
                }
            }

            '(' => { self.content.chop(1); Ok(Token::LParen) },
            ')' => { self.content.chop(1); Ok(Token::RParen) },
            '{' => { self.content.chop(1); Ok(Token::LBrace) },
            '}' => { self.content.chop(1); Ok(Token::RBrace) },
            '[' => { self.content.chop(1); Ok(Token::LBracket) },
            ']' => { self.content.chop(1); Ok(Token::RBRacket) },

            ',' => { self.content.chop(1); Ok(Token::Comma) },
            ';' => { self.content.chop(1); Ok(Token::Semicolon) },
            ':' => { self.content.chop(1); Ok(Token::Colon) },
            '.' => { self.content.chop(1); Ok(Token::Dot) },
            
            'a'..='z' | 'A'..='Z' => {
               let ident_chars = self.content.chop_while(|c| c.is_alphanumeric() || c == &'_').ok_or(LexerError::UnexpectedEndOfFile)?;
               let identifier = ident_chars.iter().collect::<String>();

               match identifier.as_str() {
                   "if" => Ok(Token::KeywordIf),
                    "else" => Ok(Token::KeywordElse),
                    "fn" => Ok(Token::KeywordFn),
                    "let" => Ok(Token::KeywordLet),
                    "match" => Ok(Token::KeywordMatch),
                    "return" => Ok(Token::KeywordReturn),
                    "for" => Ok(Token::KeywordFor),
                    "while" => Ok(Token::KeywordWhile),
                    "continue" => Ok(Token::KeywordContinue),
                    "break" => Ok(Token::KeywordBreak),
                    "true" => Ok(Token::Boolean(true)),
                    "false" => Ok(Token::Boolean(false)),
                _ => Ok(Token::Identifier(identifier)), 
               }
            }
           _ => Err(LexerError::UnknownToken(*c)) 
        }
    }
}

impl Iterator for Lexer<'_> {
   type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.content.is_empty() {
            None
        } else {
            Some(self.next_token())
        }
    }
}
