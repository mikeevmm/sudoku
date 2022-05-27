use crate::sudoku::{Sudoku, SudokuCell, SudokuCellValue};
use itertools::Itertools;
use std::collections::BTreeSet;

pub enum SolveError {
    Infeasible,
}

pub fn backtrack(sudoku: &mut Sudoku) -> Result<(), SolveError> {
    // Start by making a list of compatible digits
    let side = sudoku.side();
    let box_side = sudoku.box_side();
    let digit_range = box_side * box_side;
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
    //
    //  The logic follows analogously for a box side different from 3.
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
            let r_check = *rr as isize - (box_side * (r / box_side)) as isize;
            let c_check = *cc as isize - (box_side * (c / box_side)) as isize;
            r_check >= 0 && (r_check as usize) < box_side && c_check >= 0 && (c_check as usize) < box_side
        });

    let mut subject_to = |this: (usize, usize), that: (usize, usize)| {
        let index = this.0 * side + this.1;
        let this_cell = sudoku.get(this.0, this.1);
        if this_cell.is_empty() {
            let that_cell = sudoku.get(that.0, that.1);
            if !that_cell.is_empty() {
                incompatible[index].insert(that_cell.unwrap());
            }
        } else {
            // If this cell has been given, we can't change it!
            // We also know that we will only see this cell (as `this`) once.
            // (We will also only see it as `that` once.)
            incompatible[index].extend((1..=digit_range));
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
    let (indices, compatible): (Vec<usize>, Vec<Vec<usize>>) = incompatible
        .into_iter()
        .map(|set| (1..=digit_range).filter(|d| !set.contains(d)).collect::<Vec<usize>>())
        .enumerate()
        .sorted_unstable_by_key(|(_i, x)| -(x.len() as isize))
        .unzip();

    // Start doing the backtracking
    let mut depth = 0;
    let mut pointer = vec![0_usize; indices.len()];
    loop {
        // Have we exhausted the possibilities at this depth?
        if pointer[depth] == compatible[depth].len() {
            if depth == 0 {
                // Root node ran out of options
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
        // We only need to check whether the new addition violates a constraint,
        //  because we knew that we were in a sane state the previous iteration.
        if violates_constraints(&sudoku, indices[depth], next_guess) {
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

fn violates_constraints(sudoku: &Sudoku, last_changed: usize, new_value: usize) -> bool {
    let (r, c) = (last_changed / sudoku.side(), last_changed % sudoku.side());
    let side = sudoku.side();
    let box_side = sudoku.box_side();

    // Check row
    for cc in 0..side {
        if cc == c {
            continue;
        }
        let element = sudoku.get(r, cc);
        if element.is_empty() {
            continue;
        }
        if element.unwrap() == new_value {
            return true;
        }
    }

    // Check column
    for rr in 0..side {
        if rr == r {
            continue;
        }
        let element = sudoku.get(rr, c);
        if element.is_empty() {
            continue;
        }
        if element.unwrap() == new_value {
            return true;
        }
    }

    // Check box
    for h in 1..box_side {
        for v in 1..box_side {
            let rr = box_side*(r/box_side) + (r + v)%box_side;
            let cc = box_side*(c/box_side) + (c + h)%box_side;

            let element = sudoku.get(rr, cc);
            if element.is_empty() {
                continue;
            }
            if element.unwrap() == new_value {
                return true;
            }
        }
    }

    return false;
}
