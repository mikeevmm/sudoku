use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use solver::SolveError;
use sudoku::parsing;

mod solver;

const HELP: &'static str = concat!(
    r#"backtrack solver for sudoku

Usage:
    sudoku [--benchmark=<file>] <input file>
    sudoku --help

Options:
    --help      Print this text.

An input file of "-" denotes the input data should be read from the standard
input.

The input file is expected to be in .soduku format.
"#,
    include_str!("../../FORMATTING.txt")
);

fn main() {
    let mut args = std::env::args().skip(1); // Skip the filename

    let mut input = None;
    let mut benchmark: Option<BufWriter<Box<dyn Write>>> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                println!("{}", HELP);
                std::process::exit(0);
            }
            "-" => {
                input = Some(parsing::sudoku::parse(std::io::stdin()));
            }
            other => {
                if other.starts_with("--benchmark") {
                    // Parse a benchmark file path
                    let mut parser = sudoku::parsing::Parser::new(
                        other
                            .chars()
                            .map::<Result<char, std::convert::Infallible>, _>(|c| Ok(c))
                            .peekable(),
                    );
                    parser.expect_str("--benchmark").unwrap();
                    let path = if parser.try_match('=').unwrap() {
                        parser.collect_predicate(|_| true).unwrap()
                    } else {
                        match args.next() {
                            Some(path) => path,
                            None => {
                                println!("{}", HELP);
                                std::process::exit(1);
                            }
                        }
                    };
                    benchmark = if path.as_str() == "-" {
                        Some(std::io::BufWriter::new(
                            Box::new(std::io::stdout()) as Box<dyn Write>
                        ))
                    } else {
                        let file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(path)
                            .unwrap();
                        Some(std::io::BufWriter::new(Box::new(file)))
                    };
                } else {
                    // Parse an input path
                    let path = PathBuf::from(other);
                    let path_as_str = path.clone().to_string_lossy().to_string();
                    if !path.exists() {
                        eprintln!("{} does not exist.", &path_as_str);
                        std::process::exit(1);
                    }

                    let reader = std::fs::File::open(path);
                    if let Err(e) = reader {
                        eprintln!(
                            "could not open {} for reading.\nwith error {}",
                            &path_as_str, e
                        );
                        std::process::exit(1);
                    }
                    let reader = reader.unwrap();

                    input = Some(parsing::sudoku::parse(reader));
                }
            }
        }
    }

    if input.is_none() {
        eprintln!("{}", HELP);
        std::process::exit(1);
    };

    let input = match input.unwrap() {
        Ok(input) => input,
        Err(e) => {
            println!("Input board malformed.");
            println!("{}", e);
            std::process::exit(1);
        }
    };

    match benchmark {
        Some(writer) => run_benchmark(input, writer),
        None => run(input),
    };
}

fn run(mut input: sudoku::Sudoku) {
    let result = solver::backtrack(&mut input);

    match result {
        Ok(()) => {
            eprintln!("Success.");
            println!("{}", input);
            std::process::exit(0);
        }
        Err(SolveError::Infeasible) => {
            eprintln!(
                "The input board is infeasible. This is as far as I got:\n{}",
                input
            );
            std::process::exit(1);
        }
    }
}

fn run_benchmark<O: Write>(input: sudoku::Sudoku, mut out: BufWriter<O>) {
    // Run the function 100 times, append the average to the file.
    use std::sync::mpsc;
    use std::thread;
    use std::time;

    let (time_tx, time_rx) = mpsc::channel::<Option<u128>>();
    let thread_iterations = 1;
    let thread_count = thread::available_parallelism().unwrap().get() / 2;

    eprintln!(
        "Benchmarking {} iterations.",
        thread_iterations * thread_count
    );

    for _thread in 0..thread_count {
        let time_tx = time_tx.clone();
        let input = input.clone();
        thread::spawn(move || {
            for _ in 0..thread_iterations {
                let mut input = input.clone();
                let now = time::Instant::now();
                let result = solver::backtrack(&mut input);
                let elapsed = now.elapsed().as_millis();
                match result {
                    Ok(()) => time_tx.send(Some(elapsed)),
                    Err(_) => time_tx.send(None),
                }
                .ok();
            }
        });
    }
    drop(time_tx);

    while let Ok(time) = time_rx.recv() {
        match time {
            Some(time) => {
                out.write(format!("{}\n", time).as_bytes()).unwrap();
            }
            None => {
                out.write("-1\n".as_bytes()).unwrap();
            }
        }
    }

    out.flush().unwrap();
}
