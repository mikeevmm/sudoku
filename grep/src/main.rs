use colored::Colorize;
use itertools::Itertools;
use std::{collections::BTreeSet, path::PathBuf};
use sudoku::*;

const HEADER: &'static str = r#"visual grepper for .sudoku
"#;

const USAGE: &'static str = r#"
Usage:
    skgrep <.sudoku file>
    skgrep --help

Options:
    --help              Print help information.
"#;

const LONG_HELP: &'static str = concat!(
    r#"
An input file of "-" denotes the input data should be read from the standard
input. 

"#,
    include_str!("../../FORMATTING.txt")
);

fn main() {
    let mut args = std::env::args().skip(1); // Skip the filename

    let input = match args.next() {
        None => {
            eprintln!("{}", USAGE);
            std::process::exit(1);
        }
        Some(string) => match string.as_str() {
            "--help" => {
                println!("{}", HEADER);
                println!("{}", USAGE);
                println!("{}", LONG_HELP);
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

    let input = match input {
        Ok(input) => input,
        Err(e) => {
            eprintln!("Input board malformed.");
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let side = input.side();
    let box_side = input.box_side();

    // Look for violations
    let mut invalid = BTreeSet::<usize>::new();
    let pairs_to_check = (0..side)
        .cartesian_product(0..side)
        .tuple_combinations()
        .filter(|((r, c), (rr, cc))| {
            if r == rr && c == cc {
                return false; // This should never happen, due to the behavior of tuple_combinations()
            }
            if r == rr || c == cc {
                return true;
            }
            (r / box_side) == (rr / box_side) && (c / box_side) == (cc / box_side)
        });

    let mut filled_count = 0;
    for ((r, c), (rr, cc)) in pairs_to_check {
        if let Some(this) = input.get(r, c).value() {
            if let Some(that) = input.get(rr, cc).value() {
                filled_count += 1;
                if this == that {
                    invalid.insert(r * side + c);
                    invalid.insert(rr * side + cc);
                }
            }
        }
    }

    let total = side * side * (side - 1) + side * side * ((side - 1) / 2 - box_side + 1);
    let filled = filled_count == total;
    drop(filled_count);

    // Print the sudoku with colors
    for c in 0..side {
        for r in 0..side {
            if let Some(value) = input.get(r, c).value() {
                if invalid.contains(&(r * side + c)) {
                    print!("{} ", value.to_string().red())
                } else if filled && invalid.len() == 0 {
                    print!("{} ", value.to_string().green());
                } else {
                    print!("{} ", value);
                }
            } else {
                print!("_ ");
            }
        }
        print!("\n");
    }
}
