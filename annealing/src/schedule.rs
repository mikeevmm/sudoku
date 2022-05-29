use sudoku::parsing::AllowEof;

use crate::parsing::chars_reader::CharReader;
use crate::parsing::{self, DefaultParseError};
use std::io::Read;

pub struct Schedule {
    pub temperatures: Vec<f64>,
    pub rounds: Vec<usize>,
}

impl Schedule {
    pub fn run(&self) -> impl Iterator<Item = &f64> {
        self.temperatures
            .iter()
            .zip(self.rounds.iter())
            .map(|(t, &r)| (0..r).map(move |_| t))
            .flatten()
    }
}

pub fn parse<R: Read>(from: R) -> Result<Schedule, String> {
    let mut parser = parsing::Parser::new(CharReader::new(from));

    let mut temperatures = vec![];
    let mut rounds = vec![];

    while !parser.try_match_eof().with_default_err_msgs(&parser)? {
        // This will run once per line

        // Consume initial whitespace
        parser.eat_space().with_default_err_msgs(&parser)?;

        // If we see an '#', just discard everything until a newline is found
        if parser.try_match('#').with_default_err_msgs(&parser)? {
            parser
                .discard_predicate(|&c| c != '\n')
                .with_default_err_msgs(&parser)?;
            parser
                .expect('\n')
                .eof_ok()
                .with_default_err_msgs(&parser)?;
            continue;
        }

        // Match a temperature and a number of iterations.
        temperatures.push(parser.expect_float().with_default_err_msgs(&parser)?);
        parser.eat_space().with_default_err_msgs(&parser)?;
        rounds.push(parser.expect_integer().with_default_err_msgs(&parser)?);

        // Eat trailing whitespace
        parser.eat_space().with_default_err_msgs(&parser)?;

        parser.try_match('\n').with_default_err_msgs(&parser)?;
    }

    if temperatures.len() == 0 {
        return Err("Empty schedule file.".to_string());
    }

    Ok(Schedule {
        temperatures,
        rounds,
    })
}
