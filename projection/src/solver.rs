use itertools::Itertools;
use ndarray::prelude::*;
use std::collections::{HashMap, HashSet};
use sudoku::SudokuCellValue;

pub enum SolveResult {
    IterationsExhausted,
    Success,
}

pub fn solve(sudoku: &mut sudoku::Sudoku, max_iterations: usize) -> SolveResult {
    // Here, we will not use the internal representation of the Sudoku, and
    // will instead work with the probability 3-tensor described in [0].
    //
    //  [0]: Chi, E., Lange, K., Techniques for Solving Sudoku Puzzles

    let side = sudoku.side();
    let box_side = sudoku.box_side();

    let mut tensor = ndarray::Array::<f64, _>::zeros((side, side, side));

    let influence_pairs = (0..side)
        .cartesian_product(0..side)
        .tuple_combinations()
        .filter(|((r, c), (rr, cc))| {
            if r == rr || c == cc {
                return true;
            }
            (r / box_side) == (rr / box_side) && (c / box_side) == (cc / box_side)
        });

    // Precompute the valid elements of the rows, columns, subgrids and cells.
    let mut row_digit_simplexes =
        HashMap::<(usize, usize), Vec<&mut f64>>::with_capacity(side * side);
    let mut column_digit_simplexes =
        HashMap::<(usize, usize), Vec<&mut f64>>::with_capacity(side * side);
    let mut subgrid_digit_simplexes =
        HashMap::<(usize, usize, usize), Vec<&mut f64>>::with_capacity(side * side);
    let mut cell_simplexes = HashMap::<(usize, usize), Vec<&mut f64>>::with_capacity(side * side);

    {
        let digit_can_go_here = |row, column, d| {
            if !sudoku.get(row, column).is_empty() {
                return false;
            }

            for rr in 0..side {
                if rr == column {
                    continue;
                }
                if let Some(digit) = sudoku.get(rr, column).value() {
                    if digit - 1 == d {
                        return false;
                    }
                }
            }
            for cc in 0..side {
                if cc == column {
                    continue;
                }
                if let Some(digit) = sudoku.get(row, cc).value() {
                    if digit - 1 == d {
                        return false;
                    }
                }
            }
            for v in 0..box_side {
                for h in 0..box_side {
                    let rr = row / box_side * box_side + v;
                    let cc = column / box_side * box_side + h;
                    if let Some(digit) = sudoku.get(rr, cc).value() {
                        if digit - 1 == d {
                            return false;
                        }
                    }
                }
            }
            return true;
        };

        let base_ptr = tensor.as_ptr();
        let strides = tensor.strides();

        for row in 0..side {
            for d in 0..side {
                let valid_cols = (0..side).filter(|cc| digit_can_go_here(row, *cc, d));
                let simplex = valid_cols
                    .map(|cc| unsafe {
                        &mut *(base_ptr.offset(
                            row as isize * strides[0]
                                + cc as isize * strides[1]
                                + d as isize * strides[2],
                        ) as *mut f64)
                    })
                    .collect_vec();
                row_digit_simplexes.insert((row, d), simplex);
            }
        }

        for column in 0..side {
            for d in 0..side {
                let valid_rows = (0..side).filter(|rr| digit_can_go_here(*rr, column, d));
                let simplex = valid_rows
                    .map(|rr| unsafe {
                        &mut *(base_ptr.offset(
                            rr as isize * strides[0]
                                + column as isize * strides[1]
                                + d as isize * strides[2],
                        ) as *mut f64)
                    })
                    .collect_vec();
                column_digit_simplexes.insert((column, d), simplex);
            }
        }

        for subgrid_v_index in 0..box_side {
            for subgrid_h_index in 0..box_side {
                for d in 0..side {
                    let subgrid_base_row = subgrid_v_index * box_side;
                    let subgrid_base_col = subgrid_h_index * box_side;
                    let valid_subgrid_positions = (0..box_side)
                        .cartesian_product(0..box_side)
                        .filter(|(v, h)| {
                            digit_can_go_here(subgrid_base_row + v, subgrid_base_col + h, d)
                        })
                        .map(|(v, h)| (subgrid_base_row + v, subgrid_base_col + h));
                    let simplex = valid_subgrid_positions
                        .map(|(rr, cc)| unsafe {
                            &mut *(base_ptr.offset(
                                rr as isize * strides[0]
                                    + cc as isize * strides[1]
                                    + d as isize * strides[2],
                            ) as *mut f64)
                        })
                        .collect_vec();
                    subgrid_digit_simplexes
                        .insert((subgrid_base_row, subgrid_base_col, d), simplex);
                }
            }
        }

        for row in 0..side {
            for column in 0..side {
                let valid_digits_here = (0..side).filter(|d| digit_can_go_here(row, column, *d));
                let simplex = valid_digits_here
                    .map(|d| unsafe {
                        &mut *(base_ptr.offset(
                            row as isize * strides[0]
                                + column as isize * strides[1]
                                + d as isize * strides[2],
                        ) as *mut f64)
                    })
                    .collect_vec();
                cell_simplexes.insert((row, column), simplex);
            }
        }
    }

    let set_according_to_tensor =
        |sudoku: &mut sudoku::Sudoku,
         tensor: ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 3]>>| {
            for r in 0..side {
                for c in 0..side {
                    let mut best_prob = 0.;
                    for (index, prob) in tensor.slice(s![r, c, ..]).iter().enumerate() {
                        if *prob > best_prob {
                            best_prob = *prob;
                            sudoku.set(r, c, sudoku::SudokuCell::Digit(index + 1));
                        }
                    }
                }
            }
        };

    let simplex_projection = |y: &mut [&mut f64]| {
        // Following the formulation of Algorithm 1 [0].
        // Insertion sort; we need to preserve a copy of y anyway
        // (I started by implementing quick sort in place and was very proud)
        let w = {
            let mut w = Vec::<f64>::with_capacity(side);

            for element in y.iter() {
                let insert_in = match w.binary_search_by(|e| {
                    e.partial_cmp(element)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .reverse()
                }) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                w.insert(insert_in, **element);
            }
            w
        };

        let mut cw = 0.;
        let mut k = 0;
        for j in 0..w.len() {
            cw += w[j];
            if w[j] <= (cw - 1.) / ((j + 1) as f64) {
                cw -= w[j];
                break;
            }
            k = j;
        }
        let lambda = (cw - 1.) / ((k + 1) as f64);

        // Project
        for i in 0..y.len() {
            *y[i] = (*y[i] - lambda).max(0.);
        }

        debug_assert!(y.iter().all(|x| **x >= 0.));
        debug_assert!((y.iter().map(|x: &&mut f64| **x).sum::<f64>() - 1.).abs() <= 1e-6);
    };

    #[derive(Debug)]
    enum Constraint {
        /// (row, digit - 1)
        /// Probability of a digit along the row should be 1
        RowSimplex(usize, usize),
        /// (col, digit - 1)
        /// Probability of a digit along the column should be 1
        ColSimplex(usize, usize),
        /// (subgrid_base_row, subgrid_base_col, digit - 1)
        /// Probability of a digit in a subgrid should be 1
        SubgridSimplex(usize, usize, usize),
        /// (row, col, possible_digits - 1)
        /// Probability of any digit in a cell should be 1
        DigitSimplex(usize, usize),
        /// (row, col, digit - 1)
        /// Probability of this digit in this place is 1
        Known(usize, usize, usize),
    }

    let constraints = ((0..side)
        .cartesian_product(0..side)
        .filter(|(r, d)| {
            !(0..side).any(|c| {
                sudoku
                    .get(*r, c)
                    .value()
                    .map_or(false, |digit| digit - 1 == *d)
            })
        })
        .map(|(r, d)| Constraint::RowSimplex(r, d)))
    .chain(
        (0..side)
            .cartesian_product(0..side)
            .filter(|(c, d)| {
                !(0..side).any(|r| {
                    sudoku
                        .get(r, *c)
                        .value()
                        .map_or(false, |digit| digit - 1 == *d)
                })
            })
            .map(|(c, d)| Constraint::ColSimplex(c, d)),
    )
    .chain(
        (0..box_side)
            .cartesian_product(0..box_side)
            .cartesian_product(0..side)
            .filter(|((a, b), d)| {
                !(0..box_side).cartesian_product(0..box_side).any(|(v, h)| {
                    sudoku
                        .get(a * box_side + v, b * box_side + h)
                        .value()
                        .map_or(false, |digit| digit - 1 == *d)
                })
            })
            .map(|((a, b), d)| Constraint::SubgridSimplex(a * box_side, b * box_side, d)),
    )
    .chain((0..side).cartesian_product(0..side).filter_map(
        |(r, c)| match sudoku.get(r, c).value() {
            Some(_digit) => None,
            None => Some(Constraint::DigitSimplex(r, c)),
        },
    ))
    .chain((0..side).cartesian_product(0..side).filter_map(|(r, c)| {
        sudoku
            .get(r, c)
            .value()
            .map(|digit| Constraint::Known(r, c, digit - 1))
    }))
    .collect::<Vec<Constraint>>();

    eprintln!(
        "Finished computing constraints. Got {} constraints.",
        constraints.len()
    );

    for _iteration in 0..max_iterations {
        for constraint in constraints.iter() {
            match constraint {
                Constraint::RowSimplex(row, d) => {
                    simplex_projection(row_digit_simplexes.get_mut(&(*row, *d)).unwrap())
                }
                Constraint::ColSimplex(col, d) => {
                    simplex_projection(column_digit_simplexes.get_mut(&(*col, *d)).unwrap())
                }
                Constraint::DigitSimplex(row, col) => {
                    simplex_projection(cell_simplexes.get_mut(&(*row, *col)).unwrap())
                }
                Constraint::SubgridSimplex(a, b, d) => {
                    simplex_projection(subgrid_digit_simplexes.get_mut(&(*a, *b, *d)).unwrap())
                }
                Constraint::Known(row, col, d) => {
                    for dd in 0..side {
                        tensor[[*row, *col, dd]] = if dd == *d { 1. } else { 0. };
                    }
                }
            }
        }

        // Count violations

        set_according_to_tensor(sudoku, tensor.clone());
        let some_violation = influence_pairs.clone().any(|((r, c), (rr, cc))| {
            sudoku.get(r, c).value().map_or(false, |v| {
                sudoku.get(rr, cc).value().map_or(false, |vv| v == vv)
            })
        });
        if !some_violation {
            //println!("{:?}", tensor);
            return SolveResult::Success;
        }
    }

    //println!("{:?}", tensor);
    //set_according_to_tensor(sudoku, tensor);
    SolveResult::IterationsExhausted
}
