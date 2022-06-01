use itertools::Itertools;
use ndarray::prelude::*;
use std::collections::HashSet;
use sudoku::SudokuCellValue;

pub enum SolveResult {
    IterationsExhausted,
    EarlySuccess,
}

pub fn solve(
    sudoku: &mut sudoku::Sudoku,
    max_iterations: usize,
) -> SolveResult {
    // Here, we will not use the internal representation of the Sudoku, and
    // will instead work with the probability 3-tensor described in [0].
    //
    //  [0]: Chi, E., Lange, K., Techniques for Solving Sudoku Puzzles

    let side = sudoku.side();
    let box_side = sudoku.box_side();

    let mut tensor = ndarray::Array::<f64, _>::zeros((side, side, side));
    let mut forbidden = HashSet::<(usize, usize, usize)>::new();

    // Fix the probabilities of known digits, and forbid incompatible ones
    for r in 0..side {
        for c in 0..side {
            if let Some(digit) = sudoku.get(r, c).value() {
                for d in 0..side {
                    tensor[[r, c, d]] = if d == digit - 1 { 1.0 } else { 0.0 };
                }

                for rr in 0..side {
                    if rr == r {
                        continue;
                    }
                    forbidden.insert((rr, c, digit - 1));
                }

                for cc in 0..side {
                    if cc == c {
                        continue;
                    }
                    forbidden.insert((r, cc, digit - 1));
                }

                for v in 0..box_side {
                    for h in 0..box_side {
                        let rr = r / box_side * box_side + v;
                        let cc = c / box_side * box_side + h;
                        if rr == r && cc == c {
                            continue;
                        }
                        forbidden.insert((rr, cc, digit - 1));
                    }
                }
            }
        }
    }

    let row_except_filled = |tensor: &mut ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 3]>>,
                             row: usize,
                             d: usize|
     -> Vec<&mut f64> {
        let base_ptr = tensor.as_ptr();
        let strides = tensor.strides();
        (0..side)
            .filter(|c| !forbidden.contains(&(row, *c, d)))
            .map(|c| unsafe {
                &mut *(base_ptr.offset(
                    row as isize * strides[0] + c as isize * strides[1] + d as isize * strides[2],
                ) as *mut f64)
            })
            .collect_vec()
    };

    let column_except_filled =
        |tensor: &mut ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 3]>>,
         column: usize,
         d: usize|
         -> Vec<&mut f64> {
            let base_ptr = tensor.as_ptr();
            let strides = tensor.strides();
            (0..side)
                .filter(|r| !forbidden.contains(&(*r, column, d)))
                .map(|r| unsafe {
                    &mut *(base_ptr.offset(
                        r as isize * strides[0]
                            + column as isize * strides[1]
                            + d as isize * strides[2],
                    ) as *mut f64)
                })
                .collect_vec()
        };

    let subgrid_except_filled = |tensor: &mut ArrayBase<
        ndarray::OwnedRepr<f64>,
        Dim<[usize; 3]>,
    >,
                                 subgrid_base_row: usize,
                                 subgrid_base_col: usize,
                                 d: usize|
     -> Vec<&mut f64> {
        let base_ptr = tensor.as_ptr();
        let strides = tensor.strides();
        (0..box_side)
            .cartesian_product(0..box_side)
            .filter(|(v, h)| {
                let rr = subgrid_base_row + v;
                let cc = subgrid_base_col + h;
                !forbidden.contains(&(rr, cc, d))
            })
            .map(|(v, h)| {
                let rr = subgrid_base_row + v;
                let cc = subgrid_base_col + h;
                (rr, cc)
            })
            .map(|(rr, cc)| unsafe {
                &mut *(base_ptr.offset(
                    rr as isize * strides[0] + cc as isize * strides[1] + d as isize * strides[2],
                ) as *mut f64)
            })
            .collect_vec()
    };

    let allowed_digits = |tensor: &mut ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 3]>>,
                          row: usize,
                          column: usize|
     -> Vec<&mut f64> {
        let base_ptr = tensor.as_ptr();
        let strides = tensor.strides();
        (0..side)
            .filter(|d| !forbidden.contains(&(row, column, *d)))
            .map(|d| unsafe {
                &mut *(base_ptr.offset(
                    row as isize * strides[0]
                        + column as isize * strides[1]
                        + d as isize * strides[2],
                ) as *mut f64)
            })
            .collect::<Vec<&mut f64>>()
    };

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

    let simplex_projection = |mut y: Vec<&mut f64>| {
        if y.len() == 0 {
            return;
        }

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
        /// (row, digit)
        /// Probability of a digit along the row should be 1
        RowSimplex(usize, usize),
        /// (col, digit)
        /// Probability of a digit along the column should be 1
        ColSimplex(usize, usize),
        /// (subgrid_base_row, subgrid_base_col, digit)
        /// Probability of a digit in a subgrid should be 1
        SubgridSimplex(usize, usize, usize),
        /// (row, col, possible_digits)
        /// Probability of any digit in a cell should be 1
        DigitSimplex(usize, usize),
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
    .collect::<Vec<Constraint>>();

    eprintln!(
        "Finished computing constraints. Got {} constraints.",
        constraints.len()
    );

    for _iteration in 0..max_iterations {
        for constraint in constraints.iter() {
            match constraint {
                Constraint::RowSimplex(row, digit) => {
                    simplex_projection(row_except_filled(&mut tensor, *row, *digit))
                }
                Constraint::ColSimplex(col, digit) => {
                    simplex_projection(column_except_filled(&mut tensor, *col, *digit))
                }
                Constraint::DigitSimplex(row, col) => {
                    simplex_projection(allowed_digits(&mut tensor, *row, *col))
                }
                Constraint::SubgridSimplex(a, b, d) => {
                    simplex_projection(subgrid_except_filled(&mut tensor, *a, *b, *d))
                }
            }
        }

        // Count violations

        let mut pairs_to_check = (0..side)
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

        let all_satisfied = pairs_to_check.any(|((r, c), (rr, cc))| {
            sudoku.get(r, c).value().map_or(false, |v| {
                sudoku.get(rr, cc).value().map_or(false, |vv| v == vv)
            })
        });
        if all_satisfied {
            //println!("{:?}", tensor);
            set_according_to_tensor(sudoku, tensor);
            return SolveResult::EarlySuccess;
        }
    }

    //println!("{:?}", tensor);
    set_according_to_tensor(sudoku, tensor);
    SolveResult::IterationsExhausted
}
