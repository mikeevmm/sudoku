use crate::sudoku::{Sudoku, SudokuCell, SudokuCellUnwrap};
use itertools::Itertools;
use std::{collections::BTreeSet, fmt::Display};

pub enum SolveError {}

pub fn backtrack(sudoku: Sudoku) -> Result<Sudoku, SolveError> {
    // Start by making a list of compatible digits
    let side = sudoku.side();
    let mut incompatible = vec![BTreeSet::<usize>::new(); side * sudoku.side()];

    // Iterate over pairs of elements.
    // We should only consider a pair if both elements lie on the same row,
    // or the same column, or are in the same box. We disregard pairs of that
    // are the same element twice.
    // To see if two elements are in a box, we note the following:
    //  Since boxes are 3 elements wide, we have that, for any position (r,c),
    //  considering these to be 0-indexed, the elements belonging to the box of
    //  (r,c) are
    //
    //        {r + (r+i)%3 - r%3 ; i=1,2} × {c + (c+i)%3 - c%3 ; i=1,2}
    //
    //  This can be checked explicitly, but follows from “wrapping around” the
    //  offset inside the box using the mod operation.
    //  It follows that for (r, r') two row indices, r≠r', they can belong to
    //  the same box iff there exists some j ∈ {1,2} such that
    //
    //          r' = r + (r+j)%3 - r%3
    //
    //  This can be rewritten as
    //
    //          r' - 3 floor(r/3) = (r%3 + j)%3
    //
    //  And for the right-hand side equal to 1, 2, and any r, there exists a
    //  j=1,2 satisfying this. Therefore, to check that (r,c) and (r',c') belong
    //  to the same box, we can check that
    //
    //      r' - 3·floor(r/3) ∈ {0,1,2}  &   c' - 3·floor(c/3) ∈ {0,1,2}
    let pairs_to_check = (0..side)
        .cartesian_product(0..side)
        .tuple_combinations()
        .filter(|((r, c), (rr, cc))| {
            if r == rr && c == cc {
                return false;
            }
            if r == rr || c == cc {
                return true;
            }
            let r_check = *rr as isize - 3 * (r / 3) as isize;
            let c_check = *cc as isize - 3 * (c / 3) as isize;
            r_check >= 0 && r_check < 3 && c_check >= 0 && c_check < 3
        });

    let mut subject_to = |this: (usize, usize), that: (usize, usize)| {
        let index = this.0 * side + this.1;
        let this_cell = sudoku.get(this.0, this.1);
        if this_cell.empty() {
            let that_cell = sudoku.get(that.0, that.1);
            if !that_cell.empty() {
                incompatible[index].insert(that_cell.unwrap());
            }
        } else {
            // If this cell has been given, we can't change it!
            // We also know that we will only see this cell (as `this`) once.
            // (We will also only see it as `that` once.)
            let value = this_cell.unwrap();
            incompatible[index].extend((1..value).chain((value+1)..9));
        }
    };

    for (left, right) in pairs_to_check {
        subject_to(left, right);
        subject_to(right, left);
    }
    todo!()
}

impl Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
