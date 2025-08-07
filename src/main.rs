use std::env;

use filter_parser::expr::Expr;
use filter_parser::error::{self, ParsingError};
use filter_parser::parser::Parser;
use filter_parser::scanner::{Scanner, Token};


fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        let expr = run(&args[1], true);
        match expr {
            Ok(expr) => println!("{:?}", expr),
            Err(_) => {}
        }
    }
}

pub fn run(source: &str, print_error: bool) -> Result<Expr, ParsingError> {
    let mut errors = Vec::<ParsingError>::new();
    let mut tokens = Vec::<Token>::new();

    let mut scanner = Scanner::new(source, &mut tokens, &mut errors);
    if let Err(_) = scanner.scan() {};
    let had_error = scanner.had_error;

    let mut parser = Parser::new(&tokens, &mut errors, had_error);
    let expr = parser.parse();

    if print_error { error::print_errors(source, &errors); }

    expr
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input() {
        let cases = vec![
            "q01;elb0001=1",
            "q01;elb0001=-1",
            "q01;elb0001=1.324",
            "01;elb0001=10",
            "1a;elb0001=10",
            "1ö;hl0012=2",
            "1a;elb0001>10",
            "1a;elb0001<10",
            "1a;elb0001<=10",
            "1a;elb0001==10",
            "-1;elb0001==10",
            "-1a;elb0001==10",
            "q01;elb001 = 2:3",
            "q01;elb001 = -2:-3",
            "q01;hl0001 = 1, 2, 4",
            "q01;hl0001 = 1,2,4",
            "q01;hl0001 = -1,-2,4",
            "q02;hl0012 = hl0001",
            "q02;hl0012 = q01;hl001",
            "q01;hl0001=1 & q02;hl0012=3",
            "q01;hl0001=1&q02;hl0012=3",
            "q01;hl0001=1 & q02;hl0012=3 & q03;hl041=4",
            "q01;hl0001=1 | q02;hl0012=3 | q03;hl041=4",
            "q01;hl0001=1 & q02;hl0012=3 | q03;hl041=4",
            "q01;hl0001=1 & (q02;hl0012=3 | q03;hl041=4)",
            "q01;hl0001=1 | (q02;hl0012=3 | q03;hl041=4)",
            "(q01;hl0001=1)",
            "(q01;hl0001=1 & q02;hl0012=3)",
            "[q01;hl0001=1 & q02;hl0012=3]",
            "{q01;hl0001=1 & q02;hl0012=3}",
            "(q01;hl0001=1 & q02;hl0012=3) | q03;hl003=4",
            "(q01;hl0001=1 & q02;hl0012=3 & q02;hl0013=3) | q03;hl003=4",
        ];

        for case in cases {
           let result = run(case, false);
           assert!(result.is_ok(), "Failed to parse valid input {:?}", case);
        }
    }

    #[test]
    fn test_invalid_input() {
        let cases = vec![
            "1=2",
            "-1=2",
            "-1=-1",
            "1;1=2",
            "#q01;elb001=2",
            "elb01;elb01;",
            "elb01;=2",
            "elb03 =",
            "elb03 = 1:",
            "elb03 = 1,",
            "elb03 = 1.",
            "elb03 = -",
            "elb03 = -1-,2",
            "q01:elb001 = 2",
            "q01)elb001 = 3",
            "q01.elb001 = 2",
            "q01-elb001 = 2",
            "q01;elb001 = 2_3",
            "(q01;elb001=2",
            "q01;hl0001=1  q02;hl0012=3",
            "(q01;hl0001=1 & q02;hl0012=3",
            "(q01;hl0001=1 & q02;hl0012=",
            "(q01;hl0001=1 & q02;hl0012=2]",
            "(q01;hl0001=1 & q02;hl0012= &",   
            "q01;hl0001=1 & q02;hl0012= &",   
            "q01;hl0001=1 & q02;hl0012= |",   
            "()",
            "(q01;hl0001=1  q02;hl0012=3) | q03;hl003=4",
            "q01;hl0001=1  (q02;hl0012=3 | q03;hl003=4)",
        ];

        for case in cases {
            let result = run(case, false);
            assert!(result.is_err(), "Expected parse to fail. Input: {}, Got: {:?}", case, result);
        }
    }
}
