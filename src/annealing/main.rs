use schedule::Schedule;
use std::{convert::Infallible, path::PathBuf};
use sudoku::*;

mod schedule;
#[path = "../sudoku/lib.rs"]
mod sudoku;

const HEADER: &'static str = r#"annealing solver for sudoku
"#;

const USAGE: &'static str = r#"
Usage:
    annealing <schedule file> <input file>
    annealing --help

Options:
    --help              Print this text.

An input file of "-" denotes the input data should be read from the standard
input. The schedule file is expected to be in .schedule format, and the input
file is expected to be in .soduku format.
"#;

const LONG_HELP: &'static str = concat!(
    r#"
# The .schedule format

The .schedule format describes a cooling schedule for the simulated annealing.
It consists of plain, UTF-8 encoded text, with an arbitrary number of pairs of
(floating point, integer) numbers, representing the temperature and number of
iterations for that temperature.
Lines beginning with a hash symbol (#) are ignored.
Floating point numbers take the format (in loose BNF notation)

float := mantissa exponent
mantissa := integer decimal
exponent := ("e" | "E") integer
integer ~= (+|-)?\d+
decimal ~= \.\d+

# The .sudoku format
"#,
    include_str!("../../FORMATTING.txt")
);

fn main() {
    let mut args = std::env::args().skip(1); // Skip the filename

    let mut schedule: Option<Result<Schedule, String>> = None;
    let mut input: Option<Result<Sudoku, String>> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                println!("{}", HEADER);
                println!("{}", USAGE);
                println!("{}", LONG_HELP);
                std::process::exit(0);
            }
            "-" => {
                if schedule.is_none() {
                    schedule = Some(schedule::parse(std::io::stdin()));
                } else if input.is_none() {
                    input = Some(parsing::sudoku::parse(std::io::stdin()));
                } else {
                    eprintln!("Too many arguments!");
                    eprintln!("{}", USAGE);
                    std::process::exit(1);
                }
            }
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

                if schedule.is_none() {
                    schedule = Some(schedule::parse(reader));
                } else if input.is_none() {
                    input = Some(parsing::sudoku::parse(reader));
                } else {
                    eprintln!("Too many arguments!");
                    eprintln!("{}", USAGE);
                    std::process::exit(1);
                }
            }
        }
    }

    let schedule = match schedule {
        Some(Ok(schedule)) => schedule,
        Some(Err(e)) => {
            eprintln!("Schedule format malformed.");
            eprintln!("{}", e);
            std::process::exit(1);
        }
        None => {
            eprintln!("No schedule file specified.");
            std::process::exit(1);
        }
    };

    let mut input = match input {
        Some(Ok(input)) => input,
        Some(Err(e)) => {
            println!("Input board malformed.");
            println!("{}", e);
            std::process::exit(1);
        }
        None => {
            eprintln!("No sudoku file specified.");
            std::process::exit(1);
        }
    };

    todo!()
}
