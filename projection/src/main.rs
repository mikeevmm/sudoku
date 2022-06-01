use itertools::Itertools;
use std::{convert::Infallible, iter::Peekable, path::PathBuf};
use sudoku::parsing;

mod solver;

const HEADER: &'static str = r#"alternating projections solver for sudoku"#;
const USAGE: &'static str = r#"
Usage:
    sudoku <iteration limit> <input file>
    sudoku --help

Options:
    --help      Print this text.
"#;
const LONG_HELP: &'static str = concat!(
    r#"
An input file of "-" denotes the input data should be read from the standard
input.

The iteration count limit should be an integer.
The input file is expected to be in .soduku format.
"#,
    include_str!("../../FORMATTING.txt")
);

trait OrUsage<T> {
    fn or_usage_msg(self, message: &str) -> T;
    fn or_usage(self) -> T;
    fn or_match_help<I>(
        self,
        parser: &mut parsing::Parser<Peekable<I>, I, Infallible>,
    ) -> Result<T, parsing::ParseError>
    where
        Peekable<I>: parsing::ParserCharIter<I, Infallible>,
        I: Iterator<Item = Result<char, Infallible>>;
}

impl<T> OrUsage<T> for Result<T, parsing::ParseError> {
    fn or_usage_msg(self, message: &str) -> T {
        match self {
            Ok(v) => v,
            Err(_) => {
                eprintln!("{}", message);
                eprintln!("{}", USAGE);
                std::process::exit(1);
            }
        }
    }

    fn or_usage(self) -> T {
        match self {
            Ok(v) => v,
            Err(_) => {
                eprintln!("{}", USAGE);
                std::process::exit(1);
            }
        }
    }

    fn or_match_help<I>(
        self,
        parser: &mut parsing::Parser<Peekable<I>, I, Infallible>,
    ) -> Result<T, parsing::ParseError>
    where
        Peekable<I>: parsing::ParserCharIter<I, Infallible>,
        I: Iterator<Item = Result<char, Infallible>>,
    {
        match self {
            Ok(_) => self,
            Err(_) => {
                if parser.try_match_str("--help").or_usage() {
                    println!("{}", HEADER);
                    println!("{}", USAGE);
                    println!("{}", LONG_HELP);
                    std::process::exit(0);
                } else {
                    self
                }
            }
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1); // Skip the filename
    let args = args.join(" ");
    let mut parse =
        parsing::Parser::new(args.chars().map::<Result<char, Infallible>, _>(|c| Ok(c)));

    parse
        .eat_space()
        .expect("Something unexpected happened while reading from stdin.");

    let max_iterations = parse
        .expect_integer()
        .or_match_help(&mut parse)
        .or_usage_msg("Expected a number of iterations.");

    parse.expect_space().or_usage();

    let input = if parse
        .try_match('-')
        .or_match_help(&mut parse)
        .or_usage_msg("Expected sudoku input.")
    {
        parsing::sudoku::parse(std::io::stdin())
    } else {
        let path = parse
            .expect_path()
            .or_match_help(&mut parse)
            .or_usage_msg("Expected sudoku input.");
        let path = PathBuf::from(path);
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

        parsing::sudoku::parse(reader)
    };

    parse
        .eat_space()
        .expect("Something unexpected happened while reading from stdin.");

    parse.expect_eof().or_usage_msg("Too many arguments.");

    let mut input = match input {
        Ok(input) => input,
        Err(e) => {
            println!("Input board malformed.");
            println!("{}", e);
            std::process::exit(1);
        }
    };

    let result = solver::solve(&mut input, max_iterations);

    match result {
        solver::SolveResult::IterationsExhausted => println!("EXHAUSTED"),
        solver::SolveResult::EarlySuccess => println!("ALL SATISFIED"),
    }

    println!("{}", input);
}
