use std::{collections::BTreeSet, fmt::Display};

pub mod parsing;

#[derive(Debug, Clone)]
pub enum SudokuCell {
    Empty,
    Digit(u8),
}

impl SudokuCell {
    pub fn empty(&self) -> bool {
        match self {
            SudokuCell::Empty => true,
            _ => false,
        }
    }
}

pub trait SudokuCellUnwrap {
    fn unwrap(self) -> u8;
}

impl SudokuCellUnwrap for SudokuCell {
    fn unwrap(self) -> u8 {
        match self {
            SudokuCell::Empty => panic!("Tried to unwrap an empty sudoku cell."),
            SudokuCell::Digit(d) => d,
        }
    }
}

impl SudokuCellUnwrap for &SudokuCell {
    fn unwrap(self) -> u8 {
        match self {
            SudokuCell::Empty => panic!("Tried to unwrap an empty sudoku cell."),
            SudokuCell::Digit(d) => *d,
        }
    }
}

impl TryFrom<char> for SudokuCell {
    type Error = char;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if value == '_' {
            return Ok(SudokuCell::Empty);
        }
        if let Some(d) = value.to_digit(10) {
            return Ok(SudokuCell::Digit(d as u8));
        }
        return Err(value);
    }
}

pub struct Sudoku {
    side: usize,
    values: Vec<SudokuCell>, // Row-major
    set: BTreeSet<usize>,
}

impl Sudoku {
    pub fn empty(side: usize) -> Self {
        Sudoku {
            side,
            values: vec![SudokuCell::Empty; side * side],
            set: BTreeSet::new(),
        }
    }

    pub fn side(&self) -> usize {
        self.side
    }

    pub fn set(&mut self, row: usize, column: usize, value: SudokuCell) {
        let index = row * self.side + column;
        match value {
            SudokuCell::Digit(_) => self.set.insert(index),
            SudokuCell::Empty => self.set.remove(&index),
        };
        self.values[index] = value;
    }

    pub fn get(&self, row: usize, column: usize) -> &SudokuCell {
        let index = row * self.side + column;
        &self.values[index]
    }

    pub fn set_raw(&mut self, index: usize, value: SudokuCell) {
        match value {
            SudokuCell::Digit(_) => self.set.insert(index),
            SudokuCell::Empty => self.set.remove(&index),
        };
        self.values[index] = value;
    }

    pub fn nonempty(&self) -> impl Iterator<Item = (usize, usize)> + Clone + '_ {
        self.set.iter().map(|i| (i / self.side, i % self.side))
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
