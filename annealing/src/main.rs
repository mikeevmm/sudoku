use schedule::Schedule;
use solver::SolveError;
use std::{convert::Infallible, path::PathBuf};
use sudoku::*;

mod schedule;
mod solver;

const HEADER: &'static str = r#"annealing solver for sudoku
"#;

const USAGE: &'static str = r#"
Usage:
    annealing <schedule file> <input file> [<init file>]
    annealing --help

Options:
    --help              Print help information.
"#;

const LONG_HELP: &'static str = concat!(
    r#"
An input file of "-" denotes the input data should be read from the standard
input. The schedule file is expected to be in .schedule format, and the input
file and init file are expected to be in .soduku format.

If the annealing is successfully carried out, the program will print to stdout
a single line denoting the success of the anneal, followed by the final state in
.sudoku format, and exit with code 0. Other errors are reported to stderr, and
cause the program to exit with code 1.
The success messages can be

    SUCCESS     The .sudoku below is a solution to the given input.
    GLASS       The state was cooled into an invalid state.

The hint file, if provided, tells the annealer in what state to begin the
annealing. It follows that the hint file must agree with the input file on the
numerical clues, and must be feasible. Furthermore, hint inputs cannot contain
empty spaces.

# The .schedule format

The .schedule format describes a cooling schedule for the simulated annealing.
It consists of plain, UTF-8 encoded text, with an arbitrary number of pairs of
(floating point, integer) numbers, representing the temperature and number of
iterations for that temperature.
Lines beginning with a hash symbol (#) are ignored.
Floating point numbers take the format (in loose BNF notation):

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
    let mut init_hint: Option<Result<Sudoku, String>> = None;

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
                } else if init_hint.is_none() {
                    init_hint = Some(parsing::sudoku::parse(std::io::stdin()))
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
                } else if init_hint.is_none() {
                    init_hint = Some(parsing::sudoku::parse(reader))
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
            eprintln!("{}", USAGE);
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
            eprintln!("{}", USAGE);
            std::process::exit(1);
        }
    };

    let init_hint = match init_hint {
        Some(Ok(hint)) => Some(hint),
        Some(Err(e)) => {
            println!("Init board malformed.");
            println!("{}", e);
            std::process::exit(1);
        }
        None => None,
    };

    let result = solver::anneal(&mut input, schedule, init_hint);

    match result {
        Ok(()) => {
            println!("SUCCESS");
            println!("{}", input);
            std::process::exit(0);
        }
        Err(SolveError::Glassed) => {
            println!("GLASS");
            eprintln!(concat!(
                "The board cooled down to an unfeasible state.\n",
                "Perhaps you can start from this state and re-anneal?"
            ));
            println!("{}", input);
            std::process::exit(0);
        }
        Err(SolveError::EmptyHint) => {
            eprintln!("The hint input had empty spaces. This is not allowed.");
            std::process::exit(1);
        }
        Err(SolveError::IncompatibleHint) => {
            eprintln!("The hint input is not compatible with the input's clues.");
            std::process::exit(1);
        }
        Err(SolveError::Infeasible) => {
            eprintln!("The input is infeasible.");
            std::process::exit(1);
        }
    }
}
