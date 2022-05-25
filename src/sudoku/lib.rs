use std::fmt::Display;

pub mod parsing;

#[derive(Debug, Clone)]
pub enum SudokuCell {
    Empty,
    Digit(usize),
}

pub struct Sudoku {
    side: usize,
    values: Vec<SudokuCell>, // Row-major
}

impl Sudoku {
    pub fn empty(side: usize) -> Self {
        Sudoku {
            side,
            values: vec![SudokuCell::Empty; side * side],
        }
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}