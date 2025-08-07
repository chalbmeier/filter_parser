use ariadne::{self, Label, Report, ReportKind, Source};
use std::collections::HashSet;
use std::fmt;

// Error handling. Consider using thiserror crate.
#[derive(Debug, Clone)]
pub enum ParsingError {
    Report {
        message: String,
        line: usize,
        column: usize,
    },
    Internal {
        message: String,
        line: usize,
        column: usize,
    },
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ParsingError::Report { message, line, column } => {
                write!(f, "{} ({}:{})", message, line, column)
        },
            ParsingError::Internal { message, line, column } => {
                write!(f, "{} ({}:{})", message, line, column)
            }
        }
    }
}

impl std::error::Error for ParsingError { }


pub fn print_errors(source: &str, errors: &Vec<ParsingError>) {
    let source_name = "CLI";
    let formatted_errors = format_errors(errors, source_name);

    if formatted_errors.is_empty() {
        return
    }

    Report::build(ReportKind::Error, (source_name, 0..0))
        .with_message("Parsing error")
        .with_labels(formatted_errors)
        .finish()
        .print((source_name, Source::from(source)))
        .unwrap();
}

/// Converts a Vec<ParsingErros> into a Vec<ariadne::Label> which is used 
/// to build a ariadne::Report.
fn format_errors<'a>(errors: &Vec<ParsingError>, source_name: &'a str) -> Vec<ariadne::Label<(&'a str, std::ops::Range<usize>)>> {
    let mut formatted_errors = Vec::<Label<(&str, std::ops::Range<usize>)>>::new();
    let mut error_reported = HashSet::<(usize, usize)>::new();

    for error in errors.iter() {
        match error {
            ParsingError::Report {message, line, column} => {
                // report only one error per (line, column) to declutter output
                let pos = (*line, *column);
                if !error_reported.contains(&pos) {
                    let label = Label::new((source_name, *column-1..*column-1)).with_message(message); // -1 to
                                                                                        // align 0-
                                                                                        // and
                                                                                        // 1-based
                                                                                        // indexing
                    formatted_errors.push(label);
                    error_reported.insert(pos);
                } 
            },
            ParsingError::Internal {..} => {},
        }
    }
    formatted_errors

}

