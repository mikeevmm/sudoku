use crate::sudoku::{Sudoku, SudokuCell, SudokuCellUnwrap};
use itertools::Itertools;
use std::{collections::BTreeSet};

pub enum SolveError {
    Infeasible,
}

pub fn backtrack(sudoku: &mut Sudoku) -> Result<(), SolveError> {
    // Start by making a list of compatible digits
    let side = sudoku.side();
    let mut incompatible = vec![BTreeSet::<u8>::new(); side * sudoku.side()];

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
            incompatible[index].extend((1..value).chain((value + 1)..=9));
        }
    };

    for (left, right) in pairs_to_check {
        subject_to(left, right);
        subject_to(right, left);
    }

    // Now let us sort the cells by ascending cardinality OF COMPATIBILITY
    // Since we've kept track of the incompatible digits, this means sorting
    // the elements of `incompatible` by DESCENDING cardinality.
    // We also need to sort the indices in the same way, to know what corresponds
    // to what
    // Since we're iterating over the elements of `incompatible`, let's also turn them
    // into the elements that ARE compatible, into a vec sorted by ascending order.
    let (indices, compatible): (Vec<usize>, Vec<Vec<u8>>) = incompatible
        .into_iter()
        .map(|set| (1..=9).filter(|d| !set.contains(d)).collect::<Vec<u8>>())
        .enumerate()
        .sorted_unstable_by_key(|(_i, x)| -(x.len() as isize))
        .unzip();

    // Start doing the backtracking
    let mut depth = 0;
    let mut pointer = vec![0_usize; indices.len()];
    loop {
        // Have we exhausted the possibilities at this depth?
        if pointer[depth] == compatible[depth].len() {
            if depth == 0 { // Root node ran out of options
                return Err(SolveError::Infeasible);
            } else {
                sudoku.set_raw(indices[depth], SudokuCell::Empty);
                pointer[depth] = 0;

                pointer[depth - 1] += 1;
                depth -= 1;
                continue;
            }
        }

        let next_guess = compatible[depth][pointer[depth]];
        //println!("Trying depth {}, character {}", depth, pointer[depth]);
        sudoku.set_raw(indices[depth], SudokuCell::Digit(next_guess));

        //println!("{}", sudoku);
        //std::io::stdin().read_line(&mut String::new()).ok();

        // If constraint is violated, try the next compatible digit
        if violates_constraints(&sudoku) {
            // We don't need to undo the previous set_raw because it'll be overridden
            // in the next pass, either by a new value, or with Empty when we backtrack
            // to the above depth.
            pointer[depth] += 1;
        } else {
            // Otherwise, this stays feasible

            // Have we reached a fully feasible state?
            if depth == compatible.len() - 1 {
                break; // Success; we've reached a leaf.
            } else {
                depth += 1;
            }
        }
    }

    Ok(())
}

fn violates_constraints(sudoku: &Sudoku) -> bool {
    // Go over pairs of non-empty cells
    sudoku
        .nonempty()
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
        })
        .any(|((r, c), (rr, cc))| {
            sudoku.get(r, c).unwrap() == sudoku.get(rr, cc).unwrap()
        })
}