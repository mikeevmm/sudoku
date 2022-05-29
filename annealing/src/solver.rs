use crate::schedule::Schedule;
use itertools::Itertools;
use sudoku::{Sudoku, SudokuCell, SudokuCellValue};

pub enum SolveError {
    Glassed,
    EmptyHint,
    IncompatibleHint,
    Infeasible,
}

pub fn anneal(
    sudoku: &mut Sudoku,
    schedule: Schedule,
    init: Option<Sudoku>,
) -> Result<(), SolveError> {
    // Start by filling in the board.

    // We don't need to respect the box, line, and column constraints, but we
    // should make sure that each integer appears.

    // For this we will just borrow the code from the backtracking version of
    // the solver, and then convert the infeasible sets into the first
    // satisfiable digit.
    let side = sudoku.side();
    let box_side = sudoku.box_side();

    let (free_indices, initial_values) = match init {
        Some(init) => init_hint(sudoku, init, side)?,
        None => init_no_hint(sudoku, side, side)?,
    };

    for (index, value) in free_indices.iter().zip(initial_values.into_iter()) {
        sudoku.set_raw(*index, sudoku::SudokuCell::Digit(value));
    }

    // Keep a list of how many violations each cell is involved in.
    // This will be used to recalculate the score of a new board
    // This amounts to keeping a second sudoku board in memory.
    let mut violation_count = vec![0_usize; side * side];

    let violations = (0..side)
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
        })
        .filter(|((r, c), (rr, cc))| sudoku.get(*r, *c).unwrap() == sudoku.get(*rr, *cc).unwrap());

    for (a, b) in violations {
        violation_count[a.0 * side + a.1] += 1;
        violation_count[b.0 * side + b.1] += 1;
    }

    // Now start doing the actual annealing:
    // We "cache" the score of the current board since it won't change unless
    // a new microstate is accepted during the annealing step
    let mut current_score: usize = violation_count.iter().sum();

    for &temperature in schedule.run() {
        if current_score == 0 {
            // No violations, we lucked into the ground state!
            break;
        }

        // Find a potential new microstate
        // The new microstate is given by swapping two elements (that are not
        // fixed)
        let (raw_a, raw_b) = {
            let mut raw_a = free_indices[alea::u64_less_than(free_indices.len() as u64) as usize];
            let mut raw_b = free_indices[alea::u64_less_than(free_indices.len() as u64) as usize];
            if raw_b < raw_a {
                std::mem::swap(&mut raw_a, &mut raw_b);
            }
            (raw_a, raw_b)
        };

        sudoku.swap_raw(raw_a, raw_b);

        // Count the number of violations after the swap;

        // TODO: is it trackable to keep this full clone() of violation_count,
        //  instead of being more careful about it?
        let old_violation_count = violation_count.clone();

        // We know that the swap means that only cells that are affected by
        // either of the swapped cells can change their violation status.  For
        // each of these other cells, remove--- if appropriate--- one violation
        // (from removing the old element), and add--- if appropriate--- one
        // violation from the new element.
        let mut recount_violations = |this: usize, other: usize| {
            let (r, c) = (this / side, this % side);
            let new_value = sudoku.get_raw(this).unwrap();
            let old_value = sudoku.get_raw(other).unwrap();

            for rr in 0..side {
                if r == rr {
                    continue;
                }

                let other_value = sudoku.get(rr, c).unwrap();
                if other_value == old_value {
                    violation_count[this] = violation_count[this].saturating_sub(1);
                    violation_count[rr * side + c] =
                        violation_count[rr * side + c].saturating_sub(1);
                }
                if other_value == new_value {
                    violation_count[this] += 1;
                    violation_count[rr * side + c] += 1;
                }
            }

            for cc in 0..side {
                if c == cc {
                    continue;
                }

                let other_value = sudoku.get(r, cc).unwrap();
                if other_value == old_value {
                    violation_count[this] = violation_count[this].saturating_sub(1);
                    violation_count[r * side + cc] =
                        violation_count[r * side + cc].saturating_sub(1);
                }
                if other_value == new_value {
                    violation_count[this] += 1;
                    violation_count[r * side + cc] += 1;
                }
            }

            for h in 0..box_side {
                for v in 0..box_side {
                    let rr = box_side * (r / box_side) + v;
                    let cc = box_side * (c / box_side) + h;

                    if rr == r || cc == c {
                        // we've already checked same row & same col
                        continue;
                    }
                    let other_value = sudoku.get(rr, cc).unwrap();
                    if other_value == old_value {
                        violation_count[this] = violation_count[this].saturating_sub(1);
                        violation_count[rr * side + cc] =
                            violation_count[rr * side + cc].saturating_sub(1);
                    }
                    if other_value == new_value {
                        violation_count[this] += 1;
                        violation_count[rr * side + cc] += 1;
                    }
                }
            }
        };

        recount_violations(raw_a, raw_b);
        recount_violations(raw_b, raw_a);

        drop(recount_violations);

        let new_score: usize = violation_count.iter().sum();

        // Test if we should approve this score
        let boltzmann = || {
            alea::f64()
                <= (f64::from(
                    i32::try_from(current_score as isize - new_score as isize)
                        .expect("Over or underflow"),
                ) / temperature)
                    .exp()
                    .min(1.)
        };
        if new_score < current_score || boltzmann() {
            // Commit to the switch
            current_score = new_score;

            //println!("{:?}", current_score);
            //println!("{}", sudoku);
            //std::io::stdin().read_line(&mut String::new()).ok();
        } else {
            // Undo the switch
            sudoku.swap_raw(raw_a, raw_b);
            violation_count = old_violation_count;
        }
    }

    // We've finished the schedule. Check if we're indeed at a solution or just
    // "glassed"
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
    for ((r, c), (rr, cc)) in pairs_to_check {
        if sudoku.get(r, c).unwrap() == sudoku.get(rr, cc).unwrap() {
            return Err(SolveError::Glassed);
        }
    }

    // Cool!
    Ok(())
}

fn init_hint(
    sudoku: &mut Sudoku,
    hint: Sudoku,
    side: usize,
) -> Result<(Vec<usize>, Vec<usize>), SolveError> {
    Ok((0..(side * side))
        .map(|raw| -> Result<(usize, usize), SolveError> {
            let hint_here = hint.get_raw(raw).value().ok_or(SolveError::EmptyHint)?;
            if let Some(value) = sudoku.get_raw(raw).value() {
                if hint_here != value {
                    return Err(SolveError::IncompatibleHint);
                }
            }
            Ok((raw, hint_here))
        })
        .collect::<Result<Vec<(usize, usize)>, SolveError>>()?
        .into_iter()
        .unzip())
}

fn init_no_hint(
    sudoku: &mut Sudoku,
    side: usize,
    digit_range: usize,
) -> Result<(Vec<usize>, Vec<usize>), SolveError> {
    let mut digits = vec![0_usize; digit_range];
    let mut free_indices = vec![];
    for raw in 0..(side * side) {
        if let Some(value) = sudoku.get_raw(raw).value() {
            digits[value - 1] += 1;

            if digits[value - 1] > digit_range {
                return Err(SolveError::Infeasible);
            }
        } else {
            free_indices.push(raw);
        }
    }

    let initial_values = digits
        .into_iter()
        .enumerate()
        .filter_map(|(d, occurs)| {
            if occurs == digit_range {
                None
            } else {
                Some(std::iter::repeat(d + 1).take(digit_range - occurs))
            }
        })
        .flatten()
        .collect::<Vec<usize>>();

    for (raw, value) in free_indices.iter().zip(initial_values.iter()) {
        sudoku.set_raw(*raw, SudokuCell::Digit(*value));
    }

    Ok((free_indices, initial_values))
}
