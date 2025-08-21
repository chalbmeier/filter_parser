/// A recursive descent parser for SOEP-style filter syntax.
/// 
/// Grammar of filter syntax:
///
/// grouping -> "(" or_group ")" | "[" or_group "]" | "{" or_group "}" 
/// or_group ->  and_group ( "|" and_group )*
/// and_group ->  primary ( "&" primary )*
/// primary -> filter | grouping
/// filter -> ( set ( "=" | "==" | "!=" | ">" | ">=" | "<" | "<=" ) ( set | NUMBER | range | list ) ) 
/// set -> (( NUMBER | IDENTIFIER ) ";")? IDENTIFIER
/// range -> NUMBER : NUMBER
/// list -> NUMBER ("," NUMBER)+
///
/// Examples: "q01;elb0001=2", "elb0001=2:4", "q01;elb0001>=q02;elb0432", (q01;elb0001=1 &
/// q02;elb0002=1)" 

use crate::error::ParsingError;
use crate::expr::Expr;
use crate::scanner::Token;
use crate::token_type::TokenType::{self, *};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    errors: &'a mut Vec<ParsingError>,
    pub had_error: bool,
    synchronized: bool,
}

impl<'a> Parser<'a> {
    
    pub fn new(tokens: &'a Vec<Token>, errors: &'a mut Vec<ParsingError>, scanner_had_error: bool) -> Self {
        Parser {
            tokens: tokens,
            current: 0,
            errors: errors,
            had_error: scanner_had_error,
            synchronized: false,
        }
    }

    /// Parses all tokens to return a syntax tree. Encounterd errors are stored in `self.errors`.
    pub fn parse(&mut self) -> Result<Expr, ParsingError> {
       let mut result = None;
        while !self.at_end() {
          if let Ok(expr) = self.or_group() {
              if !self.at_end() & !self.synchronized {
                  // Case:  Missing '&' or '|'. Ex.: 'q01;elb001=1 q02;elb002=2'
                 return Err(self.error("Expected '&' or '|'".to_string(), true)) 
                // Case: Success
                } else {
                    result = Some(expr);    
                }
            // Error -> try to synchronize parser state
            } else {
               self.synchronize();
               result = None;
            }
        }

        let error = ParsingError::Report { message: "Parsing Error".to_string(), line: 1, column: 1 };
        if self.had_error {
           return Err(error) 
        } else {
            return result.ok_or(error)
        }
    }
    
    /// Advances parser to '&' or '|' after error
    fn synchronize(&mut self) {
        self.synchronized = true;
        while !self.at_end() {
            match self.peek().variant {
                Or => { self.advance(); return },
                And => { self.advance(); return },
                _ => {},
            }

            self.advance();
        }
    }

    /// Matches productions: grouping -> "(" or_group ")" | "[" or_group "]" | "{" or_group "}" 
    /// Ex.: "(q01;hl001=1 | q02;hl002=2)"
    fn grouping(&mut self) -> Result<Expr, ParsingError> {

        // Match '(' 
        if let Ok(expr) = self.consume(LeftParen, RightParen, ')') {
            return Ok(expr)
        // '['
        } else if let Ok(expr) = self.consume(LeftBracket, RightBracket, ']') { 
            return Ok(expr)
        // '{'
        } else if let Ok(expr) = self.consume(LeftBrace, RightBrace, '}') {
            return Ok(expr)
        // No parentheses
        } else {
            let msg = format!("Did not expect '{}'", self.peek().lexeme);
            Err(self.error(msg, !self.synchronized)) // suppress error for users if parser in
                                                     // synchronized state
        }
    }
    
    /// Consumes pairs of parentheses by matching the productions:
    /// grouping -> "(" or_group ")" 
    /// Ex.: "(q01;hl001=1 | q02;hl002=2)"
    fn consume(&mut self, left_paren: TokenType, right_paren: TokenType, expect: char) -> Result<Expr, ParsingError> {
        if self.match_token(&[left_paren]) {
            let expr = self.or_group()?;
            if self.match_token(&[right_paren]) {
                return Ok(Expr::Grouping { expr: Box::new(expr) })
            } else {
                let msg = format!("Expected '{}'", expect);
                return Err(self.error(msg, true))
            }
        } else {
            return Err(self.error("Not a parenthesized expression".to_string(), false))
        }
    }

    /// Matches production: or_group ->  and_group ( "|" and_group )*
    fn or_group(&mut self) -> Result<Expr, ParsingError> {
        let mut left = self.and_group()?;
        while self.match_token(&[Or]) {
            let operator = self.previous(); 
            if let Ok(right) = self.and_group() {
                left = Expr::Logical { left: Box::new(left), operator: operator, right: Box::new(right) };
            } else {
                return Err(self.error("Expected filter expression".to_string(), true));
            }
        }
        Ok(left)
    }

    /// Matches production: and_group -> primary ( "&" primary )* 
    fn and_group(&mut self) -> Result<Expr, ParsingError> {
        let mut left = self.primary()?;
        while self.match_token(&[And]) {
            let operator = self.previous();
            if let Ok(right) = self.and_group() {
                left = Expr::Logical { left: Box::new(left), operator: operator, right: Box::new(right) }; 
            } else {
                return Err(self.error("Expected filter expression".to_string(), true));
            }
        }
       Ok(left) 
    }

    /// Matches production: primary -> filter | grouping
    fn primary(&mut self) -> Result<Expr, ParsingError> {
        if let Ok(filter) = self.filter() {
            return Ok(filter)
        } else if let Ok(grouping) = self.grouping() {
            return Ok(grouping)
        } else {
            return Err(self.error("Expected filter or one of '(', '[', '{'".to_string(), !self.synchronized))
        }
    }

    /// Matches the production: filter -> ( set ( "=" | "==" | "!=" | ">" | ">=" | "<" | "<=" ) ( set | NUMBER | range | list ) ) 
    /// Ex.: 'q02;elb0003>1' or 'elb0002=1' or '02;elb0002!=elb0001'
    fn filter(&mut self) -> Result<Expr, ParsingError> {

        // Match left hand side, ex: q04;elb0003
        let set = self.set()?;

        // Match operator
        let operator = if self.match_token(&[Equal, Equal, EqualEqual, BangEqual, Greater, GreaterEqual, Less, LessEqual]) {
            self.previous()
        } else {
            return Err(self.error("Expected one of '=', '==', '!=', '>', '>=', '<', '<='".to_string(), true))
        };
        // Match right hand side
        // Match range
        if let Ok(expr) = self.range() {
            return Ok(Expr::Filter { left: Box::new(set), operator: operator, right: Box::new(expr) })

        // Match list
        } else if self.check_next(&[Comma]) {
            if let Ok(expr) = self.list() {
                return Ok(Expr::Filter { left: Box::new(set), operator: operator, right: Box::new(expr) })
            } else {
                return Err(self.error("list() failed in filter()".to_string(), false))
            }
        // Match set
        } else if self.check(&Identifier) || self.check_next(&[SemiColon]) {
            if let Ok(expr) = self.set() {
                return Ok(Expr::Filter { left: Box::new(set), operator: operator, right: Box::new(expr) })
            } else {
                return Err(self.error("set() failed in filter()".to_string(), false))
            }
        // Match number
        } else if self.match_token(&[Number]) {
           let number = self.previous();
           return Ok(Expr::Filter { left: Box::new(set), operator: operator, right: Box::new(Expr::Literal {value: number }) })
        } else {
            return Err(self.error("Expected number, list of numbers, range, or item".to_string(), true))
        }
    }

    /// Matches production: range -> NUMBER ":" NUMBER
    /// Ex.: "1:5"
    fn range(&mut self) -> Result<Expr, ParsingError> {
        // Check next token before consuming anything
        if self.check_next(&[Colon]) {
            
            // Match left number
            if self.match_token(&[Number]) {
                let number_left = self.previous(); 
                self.advance(); // consume ':'

                // Match right numbner
                if self.match_token(&[Number]) {
                    let number_right = self.previous();
                    return Ok(Expr::Range{ left: number_left, right: number_right })
                } else {
                    return Err(self.error("Expected number".to_string(), true))
                } 
            } else {
                return Err(self.error("Expected number".to_string(), true))
            }
        } else {
            Err(self.error("No ':' in range()".to_string(), false)) // Non-reporting error
        }
    }
         
    /// Matches production: list -> NUMBER ( "," NUMBER )*
    /// Ex.: "2,4,10"
    /// Caution: Function also matches a single number ( list -> NUMBER )
    fn list(&mut self) -> Result<Expr, ParsingError> {
       if self.match_token(&[Number]) {
            let value = self.previous();
            if self.match_token(&[Comma]) {
                let list = self.list()?;
                return Ok( Expr::List { value: value, next: Box::new(list) })
            } else {
                return Ok( Expr::List { value: value, next: Box::new(Expr::EndOfList ) })
            }
        } else {
            Err(self.error("Expected number".to_string(), true))
        }
    }

    /// Matches the production:  set → (( NUMBER | IDENTIFIER )+ ";")? IDENTIFIER
    /// Ex: 'q01;elb0001' or 'elb0001'
    fn set(&mut self) -> Result<Expr, ParsingError> {
        // Case with ';', ex.: q01;elb001
        if self.check_next(&[SemiColon]) {
            
            // match question
            if self.match_token(&[Identifier, Number]) {
                let question = self.previous();
                self.advance(); // consume ';'
                                
                // match item
                if self.match_token(&[Identifier]) {
                    let item = self.previous();
                    return Ok(Expr::Set { question: Some(question), item: item });
                } else {
                    return Err(self.error("Expected item identifier".to_string(), true));
                }
            } else {
                return Err(self.error("Expected question identifier".to_string(), true));
            }

        // Case without ';', ex.: elb001
        } else {
            // match item
            if self.match_token(&[Identifier]) {
                let item = self.previous();
                return Ok(Expr::Set { question: None, item: item });
            } else {
                // allowed to fail because primary() matches grouping() after filter(). Rewrite in
                // update
                return Err(self.error("Expected item identifier".to_string(), false));
            }
        }
    }

    fn match_token(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
       if self.at_end() {
           false 
        } else {
            self.peek().variant == *token_type 
        }
    }

    #[allow(dead_code)]
    fn check_token(&self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                return true
            }
        }
        false
    }

    fn check_next(&self, token_types: &[TokenType]) -> bool {
        if self.next_is_end() {
            false
        } else {
            for token_type in token_types {
                if self.tokens[self.current + 1].variant == *token_type {
                    return true
                }
            }
            false
        }
    }

    fn advance(&mut self) {
        if !self.at_end() {
            self.current += 1;
        }
        //self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn at_end(&self) -> bool {
        self.tokens[self.current].variant == EOF
    }

    fn next_is_end(&self) -> bool {
        let at_end = self.at_end();
        if !at_end {
            return self.tokens[self.current + 1].variant == EOF
        }
        at_end
    }

    /// Creates a new ParsingError variant.
    /// ParsingError::Report is meant to be reported to the user, while ParsingErrorInternal is
    /// not.
    fn error(&mut self, message: String, fatal: bool) -> ParsingError {
        if fatal { self.had_error = true; }
        let token = self.peek();
        let (line, column) = (token.line, token.column);

        let error = if fatal {
            ParsingError::Report { message, line, column }
        } else {
            ParsingError::Internal { message, line, column}
        };
        self.errors.push(error.clone());
        error
    }
}
