# filter_parser

A command-line parser for SOEP-style filter expressions, written in Rust.

![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/rust-stable-orange)

---

## Installation

filter_parser requires the latest stable version of [Rust](https://www.rust-lang.org/tools/install). If you have Rust installed, run:

```console
$ git clone https://github.com/chalbmeier/filter_parser
$ cd filter_parser
$ cargo build --release
```
---

## Usage

Run the command followed by a filter expression in quotation marks. Valid filters return an unformatted syntax tree:
```console
$ ./target/release/filter_parser "q01;elb0001=1"
Filter { left: Set { question: Some(Token { variant: Identifier, lexeme: "q01", literal: None, line: 1, column: 1 }), item: Token { variant: Identifier, lexeme: "elb0001", literal: None, line: 1, column: 5 } }, operator: Token { variant: Equal, lexeme: "=", literal: None, line: 1, column: 12 }, right: Literal { value: Token { variant: Number, lexeme: "1", literal: None, line: 1, column: 13 } } }
```
Invalid filters return a formatted error message:
```console
$ ./target/release/filter_parser "q01;elb0001="
Error: Parsing error
   ╭─[ CLI:1:1 ]
   │
 1 │ q01;elb0001=
   │             │ 
   │             ╰─ Expected number, list of numbers, range, or item
───╯
```

---

## Description

filter_parser is a recursive descent parser for SOEP-style filter syntax. The [SOEP](https://www.diw.de/en/diw_01.c.615551.en/research_infrastructure__socio-economic_panel__soep.html) is a large German panel survey that uses filters to navigate respondents through a sequence of survey questions. Filters and questions are defined in the [SOEP metadatabase](https://git.soep.de/kwenzig/publicecoredoku), which forms the backbone of the survey process from start to end. Although there is no official, fully spelled out syntax for filters (how dare you, SOEP!), a working definition has been established:

Filters typically look like this: `q01;hl0001=1`. `q01` identifies a question and `hl0001` identifies an item. The two identifiers are separated by a `;` and are followed by a symbol for comparison: `=`, `==`, `!=`, `>`, `>=`, `<`, `<=`. Next is a number (`1`), or a list of numbers `-2,-1,1`, or a range `1:4`, or an item identifier `hl0012`, or a combination of question and item identifier `q02;hl0012`.

Filters can be combined with the logical operators `&` and `|`: `q01;hl0001=1 & q02;hl0012=1`.  Both `&` and `|` are internally left-associative, implying that `q01;hl0001=1 | q02;hl0012=1 | q03;hl0013=1` is the same as `(q01;hl0001=1 | q02;hl0012=1) | q03;hl0013=1`. `&` has precedence over `|`, implying that `q01;hl0001=1 | q02;hl0012=1 & q03;hl0013=1` is the same as `q01;hl0001=1 | (q02;hl0012=1 & q03;hl0013=1)`. Filter expressions can include brackets to control the order of logical operations.

Question identifiers are optional to allow for the increasing use of filters to control preloads and other pre-survey information. Hence, for example, `e=2` and `e=f` are valid filter expressions.

A possible future extension of filter_parser is to allow for mathematical operations on the right-hand side such as `q01;elb0001>q02;elb0002/12`. 

---

## Credits

This work is very much inspired by the great book ["Crafting Interpreters" by Robert Nystrom](https://craftinginterpreters.com/).

---

