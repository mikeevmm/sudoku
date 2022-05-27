use std::{convert::Infallible, path::PathBuf};
use sudoku::parsing::{ParseError, Parser};

#[path = "../sudoku/lib.rs"]
mod sudoku;

const HEADER: &'static str = r#"annealing solver for sudoku
"#;

const USAGE: &'static str = r#"
Usage:
    annealing --rate=<rate> <input file>
    annealing --help

Options:
    --help              Print this text.
    --rate=<rate>       The annealing geometric rate [default: 0.001].
"#;

const LONG_HELP: &'static str = concat!(
    r#" An input file of "-" denotes the input data should be read from the standard
input.

TODO: Information about the annealing.

The input file is expected to be in .soduku format.
"#,
    include_str!("../../FORMATTING.txt")
);

fn short_help(code: i32) -> ! {
    println!("{}", USAGE);
    std::process::exit(code);
}

trait ErrorToHelp<T> {
    fn or_help(self) -> T;
}

fn parse_error_as_message(e: ParseError) -> ! {
    match e {
        ParseError::NotUtf8 => panic!("Found non-UTF8 character while reading!"),
        ParseError::IoError(e) => panic!("IO Error: {}", e),
        ParseError::UnexpectedEof => {
            eprintln!("I understand the arguments, but they're missing something, or misformed.");
            short_help(1);
        }
        ParseError::ExpectedEof => {
            eprintln!("I think there's trailing characters somewhere.");
            short_help(1);
        }
        ParseError::UnexpectedChar(_) => {
            eprintln!("I don't understand the arguments you've gave me.");
            short_help(1)
        }
    }
}

impl<T> ErrorToHelp<T> for Result<T, ParseError> {
    fn or_help(self) -> T {
        self.unwrap_or_else(|e| parse_error_as_message(e))
    }
}

impl ErrorToHelp<Infallible> for ParseError {
    fn or_help(self) -> Infallible {
        parse_error_as_message(self);
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
                println!("{}", HEADER);
                println!("{}", USAGE);
                println!("{}", LONG_HELP);
                std::process::exit(0);
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
            if rate_str.is_empty() {
                ParseError::UnexpectedEof.or_help();
            }
            parser.expect_eof().or_help();

            rate = Some(rate_str.parse::<f64>().unwrap());

            if unsafe { rate.unwrap_unchecked() } > 1. {
                eprintln!("Rate cannot be greater than 1.");
                std::process::exit(1);
            }
        } else {
            if input.is_none() {
                if parser.try_match('-').and(parser.try_match_eof()).or_help() {
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
                short_help(1);
            }
        }
    }

    let input = input.unwrap_or_else(|| {
        eprintln!("No input provided.\n");
        short_help(1);
    });
    let rate = rate.unwrap_or(0.001);

    todo!()
}
