use std::fmt::Display;

use crate::sudoku::Sudoku;

pub enum SolveError {

}

pub fn backtrack(sudoku: Sudoku) -> Result<Sudoku, SolveError> {
    todo!()
}

impl Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}