#[derive(Debug, Clone, PartialEq, strum_macros::Display)]
pub enum TokenType {
    LeftParen, RightParen, // ()
    LeftBracket, RightBracket, // []
    LeftBrace, RightBrace, // {}
    Comma, Colon, SemiColon,
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    And, Or,
    Minus,
    Number,
    Identifier,
    EOF,
}
