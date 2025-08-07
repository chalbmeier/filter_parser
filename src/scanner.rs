use std::fmt;

use crate::error::ParsingError;
use crate::token_type::TokenType::{self, *};

/// The `Scanner` loops through the source code, identifying tokens and returning them as
/// Vec<Token>.
pub struct Scanner<'a> {
    source: &'a str,
    chars: std::str::CharIndices<'a>, // iterator over chars of source
    tokens: &'a mut Vec<Token>, // stores scanned tokens
    start: Option<(usize, char)>,  // start char of lexeme
    current: Option<(usize, char)>, // current char (byte index, char)
    next: Option<(usize, char)>, // next char  (byte index, char)
    line: usize,  // current line in source
    column: usize, // current column in source
    column_start: usize, // start column of lexeme
    errors: &'a mut Vec<ParsingError>,
    pub had_error: bool,
}

impl<'a> Scanner<'a> {

    pub fn new(source: &'a str, tokens: &'a mut Vec<Token>, errors: &'a mut Vec<ParsingError>) -> Self {
        let mut chars = source.char_indices();
        let current = chars.next();
        let start = current; 
        let next = chars.clone().next();

        Scanner {
            source: source,
            chars: chars, 
            tokens: tokens, //Vec::new(),
            start: start,
            current: current,
            next: next,
            line: 1,
            column: 1,
            column_start: 1,
            errors: errors,
            had_error: false,
        }
    }

    pub fn scan(&mut self) -> Result<(), ParsingError> {

        while !self.at_end() {
            self.start = self.current;
            if let Err(e) = self.scan_token() {
               self.errors.push(e); 
               self.had_error = true;
            }; 
        }
        let _ = self.add_token(EOF, None);
        Ok(()) 
    }

    fn scan_token(&mut self) -> Result<(), ParsingError> {

        let c = match self.advance() {
            Some(ch) => ch,
            None => return Ok(()),
        };

        //println!("{:?}", c);

        let result = match c {
            '(' => self.add_token(LeftParen, None),
            ')' => self.add_token(RightParen, None),
            '[' => self.add_token(LeftBracket, None),
            ']' => self.add_token(RightBracket, None),
            '{' => self.add_token(LeftBrace, None),
            '}' => self.add_token(RightBrace, None),
            ',' => self.add_token(Comma, None),
            // allow '.' only within numbers 20.30?
            '.' => return Err(ParsingError::Report { message: "'.' only allowed as decimal separator".to_string(), line: self.line, column: self.column - 1}),
            ':' => self.add_token(Colon, None),
            ';' => self.add_token(SemiColon, None),
            '&' => self.add_token(And, None),
            '|' => self.add_token(Or, None),
            //'-' => self.add_token(Minus, None), '-' is consumed in number_or_identifier as part
            //of number. Unary operators currently not supported.
            '!' => self.match_and_add_token('=', BangEqual, Bang), // Bang really required?
            '=' => self.match_and_add_token('=', EqualEqual, Equal),
            '<' => self.match_and_add_token('=', LessEqual, Less),
            '>' => self.match_and_add_token('=', GreaterEqual, Greater),
            ' ' | '\r' | '\t'  => Ok(()),  // ignore whitespace
            '\n' =>  { self.line += 1; self.column = 1; self.column_start = 1;  Ok(()) }, 
            _ if (c.is_numeric() || c == '-') => self.number_or_identifier(c),
            _ if Self::is_alpha(c) => self.identifier(),
            _ => return Err(ParsingError::Report { message: "Unexpected character".to_string(), line: self.line, column: self.column - 1}),
        }; 
        result
    }

    fn identifier(&mut self) -> Result<(), ParsingError> {
        // Peek and advance as long as current char is alphanumeric
        while let Some(c) = self.peek() {
            if Self::is_alphanumeric(c) {
                self.advance();
            } else {
                break;
            }
        }

        self.add_token(Identifier, None)
    }

    fn number_or_identifier(&mut self, c_start: char) -> Result<(), ParsingError> {
        
        // Rule out case of minus without number
        if c_start == '-'  {
            if !matches!(self.peek(), Some(c) if c.is_numeric()) {
                return Err(ParsingError::Report { message: "Expected number".to_string(), line: self.line, column: self.column });
            }
        }

        // Try to match number
        // Match integer part of decimal
        while matches!(self.peek(), Some(c) if c.is_numeric()) {
            self.advance();
        }

        // Fractional part
        if let (Some('.'), Some(c_next)) = (self.peek(), self.peek_next()) {
            if c_next.is_numeric() {
                self.advance(); // consume the '.'
                 while matches!(self.peek(), Some(c) if c.is_numeric()) {
                    self.advance();
                }
            }
        }

        // Try to match identifier
        // Numbers + alphabetic chars before ';' -> identifier
        let mut is_identifier = false;
        while matches!(self.peek(), Some(c) if Self::is_alpha(c)) {
            self.advance();
            is_identifier = true;
        }

        if is_identifier {
            self.add_token(Identifier, None)
        } else {
            self.add_token(Number, None)
        }
    }

    /// Extracts the string slice source[self.start..self.current]. 
    fn extract_substring(&self) -> Result<&str, ParsingError> {

        // Get index of first char of lexeme
        let (start_idx, _) = self.start.ok_or(ParsingError::Report {
            message: "Indexing into source failed.".to_string(), line: self.line, column: self.column}
        )?;
        
        // Get index after last char of lexeme
        let end_idx = if self.at_end() {
            self.source.len()
        } else {
            self.current
                .map(|(idx, _)| idx)
                .ok_or(ParsingError::Report {message: "Indexing into source failed.".to_string(), line: self.line, column: self.column})?
        };

        Ok(&self.source[start_idx..end_idx])
     }

    fn is_alpha(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn is_alphanumeric(c: char) -> bool {
        c.is_alphanumeric() || c == '_' 
    }

    fn match_and_add_token(&mut self, expected: char, type1: TokenType, type2: TokenType) -> Result<(), ParsingError> {
        let token = if self.match_char(expected) { type1 } else { type2 };
        self.add_token(token, None)
    }

    fn match_char(&mut self, expected: char) -> bool {
        if let Some(c) = self.peek() {
            if c == expected {
                self.advance_iter();
                return true
            }
        }
        false
    }

    fn at_end(&self) -> bool {
        self.current.is_none()
    }

    /// Advance iterator and indices
    fn advance_iter(&mut self) {
        self.current = self.next; // advance current
        self.chars.next(); // advance iterator
        self.next = self.chars.clone().next(); // advance next
        self.column += 1;
    }

    /// Return current char and advance to next.
    fn advance(&mut self) -> Option<char> {
        let c = self.current.map(|(_, c)| c);
        self.advance_iter();
        c
    }

    /// Return current char without advancing.
    fn peek(&self) -> Option<char> {
        self.current.map(|(_, c)| c)
    }

    /// Return next char without advancing.
    fn peek_next(&self) -> Option<char> {
        self.next.map(|(_, c)| c)
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) -> Result<(), ParsingError> {
        let lexeme = if token_type != EOF {
            self.extract_substring()?.to_string()
        } else {
            "".to_string()
        };
        let token = Token {
            variant: token_type,
            lexeme: lexeme, 
            literal: literal,
            line: self.line,
            column: self.column_start,
        };
        self.tokens.push(token);
        self.column_start = self.column;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub variant: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>, 
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} {}", self.variant, self.lexeme)?;
        if let Some(literal) =  &self.literal {
            write!(f, " {}", literal)?;
        }
        Ok(())
    }
}


#[derive(Debug, Clone, strum_macros::Display)]
pub enum Literal {
    Number(f64),
    Str(String),
}
