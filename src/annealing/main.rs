use std::{convert::Infallible, path::PathBuf};
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
    --rate=<rate>       The annealing geometric rate [default: 0.001].

An input file of "-" denotes the input data should be read from the standard
input.

TODO: Information about the annealing.

The input file is expected to be in .soduku format.
"#,
    include_str!("../../FORMATTING.txt")
);

fn help(code: i32) -> ! {
    println!("{}", HELP);
    std::process::exit(code);
}

trait ErrorToHelp<T> {
    fn or_help(self) -> T;
}

impl<T> ErrorToHelp<T> for Result<T, ParseError> {
    fn or_help(self) -> T {
        self.unwrap_or_else(|e| match e {
            ParseError::NotUtf8 => panic!("Found non-UTF8 character while reading!"),
            ParseError::IoError(e) => panic!("IO Error: {}", e),
            ParseError::UnexpectedEof | ParseError::ExpectedEof | ParseError::UnexpectedChar(_) => {
                eprintln!("{:?}", e);
                help(1)
            }
        })
    }
}

impl ErrorToHelp<Infallible> for ParseError {
    fn or_help(self) -> Infallible {
        help(1)
    }
}

fn main() {
    let mut args = std::env::args().skip(1); // Skip the filename

    let mut rate: Option<f64> = None;
    let mut input: Option<Result<sudoku::Sudoku, String>> = None;

    while let Some(arg) = args.next() {
        // I'm sorry
        let mut parser = Parser::new(arg.chars().map::<Result<char, Infallible>, _>(|x| Ok(x)));
        if parser.try_match_str("--").or_help() {
            if parser.try_match_str("help").or_help() {
                help(0);
            }
            parser.expect_str("rate").or_help();
            parser.try_match('=').or_help();
            parser.eat_space().or_help();
            // Matched --rate
            // Parse a number
            let mut rate_str = parser.collect_predicate(|c| c.is_ascii_digit()).or_help();
            if parser.try_match('.').or_help() {
                rate_str.push('.');
                rate_str.extend(
                    parser
                        .collect_predicate(|c| c.is_ascii_digit())
                        .or_help()
                        .chars(),
                );
            }
            rate = Some(rate_str.parse::<f64>().unwrap());

            if unsafe { rate.unwrap_unchecked() } > 1. {
                eprintln!("Rate cannot be greater than 1.");
                std::process::exit(1);
            }
        } else {
            if input.is_none() {
                if parser
                    .try_match('-')
                    .and(parser.try_match_eof())
                    .or_help()
                {
                    input = Some(sudoku::parsing::sudoku::parse(std::io::stdin()));
                } else {
                    let path =
                        PathBuf::from(parser.collect_predicate(|c| !c.is_whitespace()).or_help());
                    let path_as_str = path.clone().to_string_lossy().to_string();
                    if !path.exists() {
                        eprintln!("{} does not exist.", &path_as_str);
                        std::process::exit(1);
                    }

                    let reader = std::fs::File::open(path);
                    if let Err(e) = reader {
                        eprintln!(
                            "Could not open {} for reading.\nWith error {}",
                            &path_as_str, e
                        );
                        std::process::exit(1);
                    }
                    let reader = reader.unwrap();

                    input = Some(sudoku::parsing::sudoku::parse(reader));
                }
            } else {
                help(1);
            }
        }
    }

    let input = input.unwrap_or_else(|| {
        eprintln!("No input provided.\n");
        help(1);
    });
    let rate = rate.unwrap_or(0.001);

    todo!()
}
