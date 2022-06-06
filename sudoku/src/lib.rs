use std::fmt::Display;

pub mod parsing;

#[derive(Debug, Clone)]
pub enum SudokuCell {
    Empty,
    Digit(usize),
}

impl SudokuCell {
    pub fn is_empty(&self) -> bool {
        match self {
            SudokuCell::Empty => true,
            _ => false,
        }
    }
}

pub trait SudokuCellValue {
    fn value(&self) -> Option<usize>;
    fn unwrap(self) -> usize;
}

impl SudokuCellValue for SudokuCell {
    fn value(&self) -> Option<usize> {
        match self {
            SudokuCell::Empty => None,
            SudokuCell::Digit(d) => Some(*d),
        }
    }

    fn unwrap(self) -> usize {
        match self {
            SudokuCell::Empty => panic!("Tried to unwrap an empty sudoku cell."),
            SudokuCell::Digit(d) => d,
        }
    }
}

impl SudokuCellValue for &SudokuCell {
    fn value(&self) -> Option<usize> {
        match self {
            SudokuCell::Empty => None,
            SudokuCell::Digit(d) => Some(*d),
        }
    }

    fn unwrap(self) -> usize {
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
            return Ok(SudokuCell::Digit(d as usize));
        }
        return Err(value);
    }
}

impl TryFrom<String> for SudokuCell {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.chars().all(|c| c == '_') {
            Ok(SudokuCell::Empty)
        } else if let Ok(value) = value.parse::<usize>() {
            Ok(SudokuCell::Digit(value))
        } else {
            Err(value)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sudoku {
    side: usize,
    box_side: usize,
    values: Vec<SudokuCell>, // Row-major
}

impl Sudoku {
    pub fn empty(side: usize) -> Self {
        Sudoku {
            side,
            box_side: (side as f32).sqrt() as usize,
            values: vec![SudokuCell::Empty; side * side],
        }
    }

    pub fn side(&self) -> usize {
        self.side
    }

    pub fn box_side(&self) -> usize {
        self.box_side
    }

    pub fn set(&mut self, row: usize, column: usize, value: SudokuCell) {
        let index = row * self.side + column;
        self.values[index] = value;
    }

    pub fn get(&self, row: usize, column: usize) -> &SudokuCell {
        let index = row * self.side + column;
        &self.values[index]
    }

    pub fn set_raw(&mut self, index: usize, value: SudokuCell) {
        self.values[index] = value;
    }

    pub fn get_raw(&self, index: usize) -> &SudokuCell {
        &self.values[index]
    }

    pub fn swap_raw(&mut self, raw_a: usize, raw_b: usize) {
        self.values.swap(raw_a, raw_b);
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
