pub enum SolveError {

}

pub fn solve(sudoku: &mut sudoku::Sudoku, max_iterations: usize) -> Result<(), SolveError> {
    // Here, we will not sure the internal representation of the Sudoku, and
    // will instead work with the probability 3-tensor described in [0].
    //
    //  [0]: Chi, E., Lange, K., Techniques for Solving Sudoku Puzzles

    let side = sudoku.side();
    let box_side = sudoku.box_side();

    let mut p_tensor = ndarray::Array3::<f64>::ones((side, side, side));

    todo!()
}