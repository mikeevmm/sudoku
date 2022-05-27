use crate::sudoku::Sudoku;
use crate::schedule::Schedule;

pub enum SolveError {

}

pub fn anneal(sudoku: &mut Sudoku, schedule: Schedule) -> Result<(), SolveError> {
    // Start by filling in the board
    // We don't need to respect the box, line, and column constraints, but we
    // should make sure that each integer appears 

    for (temperature, rounds) in schedule.run() {
        for _round in 0..rounds {
            // Find a potential new microstate

        } 
    }
    Ok(())
}