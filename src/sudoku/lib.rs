use std::{path::PathBuf, io::Read, fmt::Display};

pub struct Sudoku {

}

impl Sudoku {
    pub fn parse<R: Read>(reader: R) -> Self {
        todo!()
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
