use super::*;
use crate::sudoku::Sudoku;
use std::io::Read;

pub fn parse<R: Read>(reader: R) -> Result<Sudoku, String> {
    let mut parser = Parser::new(CharReader::new(reader));

    // Read the first line. This will give a hint as to the size of the board.
    let mut first_line = Vec::<char>::new();
    match_line(&mut parser, |_i, c| {
        first_line.push(c);
        Ok(())
    })?;

    let side = first_line.len();

    if side == 0 {
        return Err(concat!(
            "I don't know how to solve a 0 by 0 board! ",
            "Maybe it's already trivially solved?"
        )
        .to_string());
    }

    // We've read the first line.
    // We can instantiate a board of the correct size, and start filling it in
    let mut sudoku = Sudoku::empty(side);

    // Plug back in the information from the first line.
    for (i, c) in first_line.into_iter().enumerate() {
        let d = c
            .try_into()
            .map_err(|c| format!("Sorry, I don't know how to read '{}' as a cell.", c))?;
        sudoku.set(0, i, d);
    }

    // Parse the rest of the lines;
    // We expect (dimensions - 1) lines remaining!
    for line in 1..side {
        match_line(&mut parser, |i, c| {
            if i >= side {
                return Err(format!("There are too many elements on line {}!", line));
            }
            let d = c
                .try_into()
                .map_err(|c| format!("Sorry, I don't know how to read '{}' as a cell.", c))?;
            sudoku.set(line, i, d);
            Ok(())
        })?;
    }

    // If after eating all the remaining whitespace we are not at EOF, then
    // the file is misformatted.
    parser.eat_space().with_default_err_msgs(&parser)?;
    parser.expect_eof().map_err(|err| match err {
        ParseError::UnexpectedEof | ParseError::UnexpectedChar(_) | ParseError::ExpectedEof => {
            parser.err(
                concat!(
                    "Finished parsing the sudoku puzzle, ",
                    "but there's non-whitespace remaining in the file.",
                    "Is your board not square?"
                )
                .to_string(),
            )
        }
        _ => parser.default_err_msg(err),
    })?;

    Ok(sudoku)
}

fn match_line<I, F>(parser: &mut Parser<Peekable<I>, I, CharReaderError>, mut on_char: F) -> Result<(), String>
where
    I: Iterator<Item = Result<char, CharReaderError>>,
    F: FnMut(usize, char) -> Result<(), String>,
{
    if let Ok(true) = parser.try_match_eof() {
        return Err(concat!(
            "I expected to see more lines of sudoku, but the file ended.\n",
            "Is your board not square?"
        )
        .to_string());
    }

    // We allow initial empty space
    parser.eat_space().with_default_err_msgs(&parser)?;

    let mut index = 0;
    loop {
        let next = parser
            .expect_predicate(|c| c.is_digit(10) || c == '_')
            .map_err(|err| match err {
                ParseError::UnexpectedChar(c) => parser.err(format!(
                    "Expected a digit or an underscore, but found a '{}'.",
                    c
                )),
                _ => parser.default_err_msg(err),
            })?;

        on_char(index, next)?;
        index += 1;

        // Eat trailing whitespace
        parser.eat_space().with_default_err_msgs(&parser)?;

        // If we match an EOF or new line, we've finished parsing the line
        if parser.try_match_eof().with_default_err_msgs(&parser)? {
            break; // Matched EOF
        }

        parser.try_match('\r').with_default_err_msgs(&parser)?;
        if parser.try_match('\n').with_default_err_msgs(&parser)? {
            break; // Matched new line
        }
    }

    Ok(())
}
