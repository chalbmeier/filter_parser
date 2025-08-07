use crate::scanner::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Grouping {
        expr: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Term {
        expr: Box<Expr>,
    },
   Filter {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    }, 
   Set {
        question: Option<Token>,
        item: Token,
    }, 
   Element,
   Range {
        left: Token,
        right: Token,
    },
    List {
        value: Token,
        next: Box<Expr>,
    },
    EndOfList,
    Literal {
        value: Token, 
    },
}

