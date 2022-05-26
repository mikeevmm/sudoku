use std::path::PathBuf;
use sudoku::parsing;

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

fn main() {
    let mut args = std::env::args().skip(1); // Skip the filename
    let args = args.map(|arg| arg.chars().chain(' ')).flatten();

    let input = match args.next() {
        None => {
            eprintln!("{}", HELP);
            std::process::exit(1);
        }
        Some(string) => match string.as_str() {
            "--help" => {
                println!("{}", HELP);
                std::process::exit(0);
            }
            "-" => parsing::sudoku::parse(std::io::stdin()),
            path => {
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
            }
        },
    };

    let mut input = match input {
        Ok(input) => input,
        Err(e) => {
            println!("Input board malformed.");
            println!("{}", e);
            std::process::exit(1);
        }
    };

    let result = solver::backtrack(&mut input);

    match result {
        Ok(()) => {
            println!("Success. Solution:\n{}", input);
            std::process::exit(0);
        }
        Err(SolveError::Infeasible) => {
            eprintln!("The input board is infeasible. This is as far as I got:\n{}", input);
            std::process::exit(1);
        }
    }
}
