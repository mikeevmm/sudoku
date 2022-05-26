use self::chars_reader::{CharReader, CharReaderError};
use std::{io::Read, iter::Peekable};

use super::Sudoku;

mod chars_reader;

#[derive(Debug)]
pub enum SudokuParseError {
    NotUtf8,
    IoError(std::io::Error),
    UnexpectedEof,
    UnexpectedChar(char),
    ExpectedEof,
}

struct SudokuCharIter<R>
where
    R: Read,
{
    inner: Peekable<CharReader<R>>,
}

struct Parser<R>
where
    R: Read,
{
    inner: SudokuCharIter<R>,
    line: usize,
    column: usize,
}

trait AllowEof {
    type Return;
    fn eof_ok(self) -> Result<Self::Return, SudokuParseError>;
}

impl AllowEof for Result<Option<char>, SudokuParseError> {
    type Return = Option<char>;
    fn eof_ok(self) -> Result<Self::Return, SudokuParseError> {
        match self {
            Ok(value) => Ok(value),
            Err(SudokuParseError::UnexpectedEof) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl AllowEof for Result<bool, SudokuParseError> {
    type Return = Option<bool>;
    fn eof_ok(self) -> Result<Self::Return, SudokuParseError> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(SudokuParseError::UnexpectedEof) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

trait DefaultParseError<T> {
    fn with_default_err_msgs<R: Read>(self, parser: &Parser<R>) -> Result<T, String>;
}

impl<T> DefaultParseError<T> for Result<T, SudokuParseError> {
    fn with_default_err_msgs<R: Read>(self, parser: &Parser<R>) -> Result<T, String> {
        self.map_err(|e| parser.default_err_msg(e))
    }
}

impl<R: Read> SudokuCharIter<R> {
    fn new(inner: CharReader<R>) -> Self {
        Self {
            inner: inner.peekable(),
        }
    }

    fn next(&mut self) -> Result<char, SudokuParseError> {
        let error = self.inner.next();
        match error {
            Some(x) => match x {
                Ok(char) => Ok(char),
                Err(e) => match e {
                    CharReaderError::NotUtf8 => Err(SudokuParseError::NotUtf8),
                    CharReaderError::Other(e) => Err(SudokuParseError::IoError(e)),
                },
            },
            None => Err(SudokuParseError::UnexpectedEof),
        }
    }

    fn peek(&mut self) -> Result<Option<char>, SudokuParseError> {
        let peek = self.inner.peek();
        match peek {
            Some(char) => {
                if let Ok(char) = char {
                    return Ok(Some(char.clone()));
                }
            }
            None => {
                return Ok(None);
            }
        };

        // If we got this far we're peeking an error. Let's consume it.
        Err(self
            .next()
            .expect_err("Ok situations should have been handled above."))
    }
}

impl<R: Read> Parser<R> {
    fn new(inner: SudokuCharIter<R>) -> Self {
        Self {
            inner,
            line: 0,
            column: 0,
        }
    }

    fn err(&self, message: String) -> String {
        format!("{message}\nAt {}:{}.", self.line, self.column)
    }

    fn default_err_msg(&self, err: SudokuParseError) -> String {
        match err {
            SudokuParseError::NotUtf8 => self.err("Found non-UTF-8 character.".to_string()),
            SudokuParseError::IoError(e) => format!("Failed to read input, with error {}.", e),
            SudokuParseError::UnexpectedEof => "Unexpected end of file.".to_string(),
            SudokuParseError::UnexpectedChar(c) => {
                self.err(format!("Found unexpected character '{}'", c))
            }
            SudokuParseError::ExpectedEof => {
                "Found trailing content, when expecting end of file.".to_string()
            }
        }
    }

    fn next(&mut self) -> Result<char, SudokuParseError> {
        let next = self.inner.next();
        if let Ok(c) = next {
            if c == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
        next
    }

    fn expect(&mut self, to_match: char) -> Result<(), SudokuParseError> {
        let next = self.next()?;
        if next != to_match {
            Err(SudokuParseError::UnexpectedChar(to_match))
        } else {
            Ok(())
        }
    }

    fn expect_eof(&mut self) -> Result<(), SudokuParseError> {
        match self.inner.peek() {
            Ok(None) => Ok(()),
            _ => Err(SudokuParseError::ExpectedEof),
        }
    }

    fn expect_predicate<P>(&mut self, predicate: P) -> Result<char, SudokuParseError>
    where
        P: Fn(char) -> bool,
    {
        let next = self.next()?;
        if !predicate(next) {
            Err(SudokuParseError::UnexpectedChar(next))
        } else {
            Ok(next)
        }
    }

    fn try_match(&mut self, to_match: char) -> Result<bool, SudokuParseError> {
        let next = self.inner.peek()?;
        match next {
            Some(c) => {
                if c == to_match {
                    self.next()
                        .expect("The peek() above should already have ruled out an error.");
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Err(SudokuParseError::UnexpectedEof),
        }
    }

    fn try_match_eof(&mut self) -> Result<bool, SudokuParseError> {
        match self.inner.peek() {
            Ok(None) => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn try_match_predicate<P>(&mut self, predicate: P) -> Result<Option<char>, SudokuParseError>
    where
        P: Fn(char) -> bool,
    {
        let next = self.inner.peek()?;
        match next {
            Some(c) => {
                if predicate(c) {
                    self.next()
                        .expect("The peek() above should already have ruled out an error.");
                    Ok(Some(c))
                } else {
                    Ok(None)
                }
            }
            None => Err(SudokuParseError::UnexpectedEof),
        }
    }

    fn eat_space(&mut self) -> Result<(), SudokuParseError> {
        while let Some(c) = self
            .try_match_predicate(|c| c.is_whitespace() && c != '\n' && c != '\r')
            .eof_ok()?
        {}
        Ok(())
    }
}

pub fn parse<R: Read>(reader: R) -> Result<Sudoku, String> {
    let mut parser = Parser::new(SudokuCharIter::new(CharReader::new(reader)));

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
        SudokuParseError::UnexpectedEof
        | SudokuParseError::UnexpectedChar(_)
        | SudokuParseError::ExpectedEof => parser.err(
            concat!(
                "Finished parsing the sudoku puzzle, ",
                "but there's non-whitespace remaining in the file.",
                "Is your board not square?"
            )
            .to_string(),
        ),
        _ => parser.default_err_msg(err),
    })?;

    Ok(sudoku)
}

fn match_line<R, F>(parser: &mut Parser<R>, mut on_char: F) -> Result<(), String>
where
    R: Read,
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
                SudokuParseError::UnexpectedChar(c) => parser.err(format!(
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
