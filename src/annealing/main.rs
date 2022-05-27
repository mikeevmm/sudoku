use std::convert::Infallible;
use sudoku::parsing::{ParseError, Parser};

#[path = "../sudoku/lib.rs"]
mod sudoku;

const HELP: &'static str = concat!(
    r#"annealing solver for sudoku

Usage:
    annealing --rate=<rate> <input file>
    annealing --help

Options:
    --help              Print this text.
    --rate=<rate>       The annealing geometric rate.

An input file of "-" denotes the input data should be read from the standard
input.

TODO: Information about the annealing.

The input file is expected to be in .soduku format.
"#,
    include_str!("../../FORMATTING.txt")
);

trait ErrorToHelp<T> {
    fn or_help(self, code: i32) -> T;
}

impl<T> ErrorToHelp<T> for Result<T, ParseError> {
    fn or_help(self, code: i32) -> T {
        self.unwrap_or_else(|_| {
            println!("{}", HELP);
            std::process::exit(code);
        })
    }
}

impl ErrorToHelp<Infallible> for ParseError {
    fn or_help(self, code: i32) -> Infallible {
        println!("{}", HELP);
        std::process::exit(code);
    }
}

fn main() {
    let mut args = std::env::args().skip(1); // Skip the filename

    while let Some(arg) = args.next() {
        // I'm sorry
        let mut parser = Parser::new(arg.chars().map::<Result<char, Infallible>, _>(|x| Ok(x)));

        match parser.try_match_str("--") {
            Err(ParseError::UnexpectedEof) => {
                ParseError::UnexpectedEof.or_help(1);
            }
            Err(e) => panic!("{:?}", e),
            Ok(false) => {}
            Ok(true) => {
                // Matched --
                if parser.try_match_str("help").or_help(1) {

                }
                parser.expect_str("rate").or_help(1);
                parser.try_match('=').or_help(1);
                parser.eat_space().or_help(1);
                // Matched --rate

            }
        }
    }

    todo!()
}
