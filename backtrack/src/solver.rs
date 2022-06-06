use sudoku::{Sudoku, SudokuCell, SudokuCellValue};
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
    let mut incompatible = vec![BTreeSet::<usize>::new(); side * side];

    // Iterate over pairs of elements.
    // We should only consider a pair if both elements lie on the same row,
    // or the same column, or are in the same box. We disregard pairs of that
    // are the same element twice.
    // TODO: This could probably be optimized.
    let pairs_to_check = (0..side)
        .cartesian_product(0..side)
        .tuple_combinations()
        .filter(|((r, c), (rr, cc))| {
            if r == rr && c == cc {
                return false; // This should never happen, due to the behavior of tuple_combinations()
            }
            if r == rr || c == cc {
                return true;
            }
            (r / box_side) == (rr / box_side) && (c / box_side) == (cc / box_side)
        });

    let mut subject_to = |this: (usize, usize), that: (usize, usize)| {
        let index = this.0 * side + this.1;
        let this_cell = sudoku.get(this.0, this.1);

        if this_cell.is_empty() {
            if let Some(value) = sudoku.get(that.0, that.1).value() {
                incompatible[index].insert(value);
            }
        } else {
            incompatible[index].extend(1..=digit_range);
        }
    };

    for (left, right) in pairs_to_check {
        subject_to(left, right);
        subject_to(right, left);
    }

    drop(subject_to);

    // Now let us sort the cells by ascending cardinality OF COMPATIBILITY
    // Since we've kept track of the incompatible digits, this means sorting
    // the elements of `incompatible` by DESCENDING cardinality.
    // We also need to sort the indices in the same way, to know what corresponds
    // to what
    // Since we're iterating over the elements of `incompatible`, let's also turn them
    // into the elements that ARE compatible, into a vec sorted by ascending order.
    let (indices, compatible): (Vec<usize>, Vec<Vec<usize>>) = incompatible
        .into_iter()
        .map(|set| {
            (1..=digit_range)
                .filter(|d| !set.contains(d))
                .collect::<Vec<usize>>()
        })
        .enumerate()
        .filter(|(_, x)| x.len() > 0)
        .sorted_unstable_by_key(|(_i, x)| x.len() as isize)
        .unzip();
    
    // Start doing the backtracking
    let mut depth = 0; // The index of the string character being tested.
    let mut pointer = vec![0_usize; indices.len()]; // The character being tested, for each depth.
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
    let side = sudoku.side();
    let box_side = sudoku.box_side();
    let (r, c) = (last_changed / side, last_changed % side);

    // Check row
    for cc in 0..side {
        if cc == c {
            continue;
        }
        let element = sudoku.get(r, cc);
        if let Some(value) = element.value() {
            if value == new_value {
                return true;
            }
        }
    }

    // Check column
    for rr in 0..side {
        if rr == r {
            continue;
        }
        if let Some(value) = sudoku.get(rr, c).value() {
            if value == new_value {
                return true;
            }
        }
    }

    // Check box
    for h in 0..box_side {
        for v in 0..box_side {
            let rr = box_side * (r / box_side) + v;
            let cc = box_side * (c / box_side) + h;

            if rr == r || cc == c { // we've already checked same row & same col
                continue;
            }

            if let Some(value) = sudoku.get(rr, cc).value() {
                if value == new_value {
                    return true;
                }
            }
        }
    }

    return false;
}
