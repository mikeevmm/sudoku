use self::chars_reader::{CharReader, CharReaderError};
use std::{fmt::Display, io::Read, iter::Peekable, path::PathBuf};

mod chars_reader;

pub struct Sudoku {}

#[derive(Debug)]
pub enum SudokuParseError {
    NotUtf8,
    IoError(std::io::Error),
    UnexpectedEof,
    ExpectedChar(char),
    ExpectedEof,
    ExpectedOneOf(String),
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
}

trait AllowEof {
    type Return;
    fn eof_ok(self) -> Result<Self::Return, SudokuParseError>;
}

impl AllowEof for Result<Option<char>, SudokuParseError> {
    type Return = Option<char>;
    fn eof_ok(self) -> Result<Option<char>, SudokuParseError> {
        match self {
            Ok(value) => Ok(value),
            Err(SudokuParseError::UnexpectedEof) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl AllowEof for Result<char, SudokuParseError> {
    type Return = Option<char>;
    fn eof_ok(self) -> Result<Option<char>, SudokuParseError> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(SudokuParseError::UnexpectedEof) => Ok(None),
            Err(err) => Err(err),
        }
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
        Self { inner }
    }

    fn expect(&mut self, to_match: char) -> Result<(), SudokuParseError> {
        let next = self.inner.next()?;
        if next != to_match {
            Err(SudokuParseError::ExpectedChar(to_match))
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

    fn try_match(&mut self, to_match: char) -> Result<bool, SudokuParseError> {
        let next = self.inner.peek()?;
        match next {
            Some(c) => {
                if c == to_match {
                    self.inner
                        .next()
                        .expect("The peek() above should already have ruled out an error.");
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Err(SudokuParseError::UnexpectedEof),
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
                    Ok(Some(c))
                } else {
                    Ok(None)
                }
            }
            None => Err(SudokuParseError::UnexpectedEof),
        }
    }
}

impl Sudoku {
    pub fn parse<R: Read>(reader: R) -> Result<Self, SudokuParseError> {
        todo!()
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for SudokuParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
