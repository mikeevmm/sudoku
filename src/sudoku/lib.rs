use std::fmt::Display;

pub mod parsing;

#[derive(Debug, Clone)]
pub enum SudokuCell {
    Empty,
    Digit(usize),
}

impl TryFrom<char> for SudokuCell {
    type Error = char;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if value == '_' {
            return Ok(SudokuCell::Empty);
        }
        if let Some(d) = value.to_digit(10) {
            return Ok(SudokuCell::Digit(d as usize));
        }
        return Err(value);
    }
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

    pub fn set(&mut self, row: usize, column: usize, value: SudokuCell) -> Result<(), ()> {
        let index = row * self.side + column;
        if index >= self.values.len() {
            return Err(());
        }
        self.values[index] = value;
        Ok(())
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, cell) in self.values.iter().enumerate() {
            if i % self.side == 0 && i > 0 {
                write!(f, "\n")?;
            }
            match cell {
                SudokuCell::Empty => write!(f, "_ ")?,
                SudokuCell::Digit(d) => write!(f, "{} ", d)?,
            }
        }
        Ok(())
    }
}
