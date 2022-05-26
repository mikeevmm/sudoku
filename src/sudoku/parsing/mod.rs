use self::chars_reader::{CharReader, CharReaderError};
use std::{io::Read, iter::Peekable, error::Error, convert::Infallible};

mod chars_reader;
pub mod sudoku;

#[derive(Debug)]
pub enum ParseError {
    NotUtf8,
    IoError(std::io::Error),
    UnexpectedEof,
    UnexpectedChar(char),
    ExpectedEof,
}

struct ParserCharIter<I, E>
where
    I: Iterator<Item = Result<char, E>>,
{
    inner: Peekable<I>,
}

struct Parser<I, E>
where
    I: Iterator<Item = Result<char, E>>,
{
    inner: ParserCharIter<I, E>,
    line: usize,
    column: usize,
}

trait CanParse {}

impl CanParse for CharReaderError {}
impl CanParse for Infallible {}

trait AllowEof {
    type Return;
    fn eof_ok(self) -> Result<Self::Return, ParseError>;
}

impl AllowEof for Result<Option<char>, ParseError> {
    type Return = Option<char>;
    fn eof_ok(self) -> Result<Self::Return, ParseError> {
        match self {
            Ok(value) => Ok(value),
            Err(ParseError::UnexpectedEof) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

impl AllowEof for Result<bool, ParseError> {
    type Return = Option<bool>;
    fn eof_ok(self) -> Result<Self::Return, ParseError> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(ParseError::UnexpectedEof) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

trait DefaultParseError<T> {
    fn with_default_err_msgs<I, E>(self, parser: &Parser<I, E>) -> Result<T, String>
    where
        I: Iterator<Item = Result<char, E>>;
}

impl<T> DefaultParseError<T> for Result<T, ParseError> {
    fn with_default_err_msgs<I, CharReaderError>(self, parser: &Parser<I, CharReaderError>) -> Result<T, String>
    where
        I: Iterator<Item = Result<char, CharReaderError>>,
    {
        self.map_err(|e| parser.default_err_msg(e))
    }
}

impl<I, E> ParserCharIter<I, E>
where
    I: Iterator<Item = Result<char, E>>,
    E: CanParse,
{
    fn new(from: I) -> Self {
        Self {
            inner: from.peekable(),
        }
    }
}

impl<I> ParserCharIter<I, CharReaderError>
where
    I: Iterator<Item = Result<char, CharReaderError>>
{
    fn next(&mut self) -> Result<char, ParseError> {
        let error = self.inner.next();
        match error {
            Some(x) => match x {
                Ok(char) => Ok(char),
                Err(e) => match e {
                    CharReaderError::NotUtf8 => Err(ParseError::NotUtf8),
                    CharReaderError::Other(e) => Err(ParseError::IoError(e)),
                },
            },
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn peek(&mut self) -> Result<Option<char>, ParseError> {
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

impl<I> ParserCharIter<I, Infallible>
where
    I: Iterator<Item = Result<char, Infallible>>
{
    fn next(&mut self) -> Result<char, ParseError> {
        let error = self.inner.next();
        match error {
            Some(x) => match x {
                Ok(char) => Ok(char),
                Err(e) => unreachable!(),
            },
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn peek(&mut self) -> Result<Option<char>, ParseError> {
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

#[allow(dead_code)]
impl<I, E> Parser<I, E>
where
    I: Iterator<Item = Result<char, E>>,
    E: CanParse,
{
    fn new(from: I) -> Self {
        Self {
            inner: ParserCharIter::new(from),
            line: 0,
            column: 0,
        }
    }

    fn err(&self, message: String) -> String {
        format!("{message}\nAt {}:{}.", self.line, self.column)
    }

    fn default_err_msg(&self, err: ParseError) -> String {
        match err {
            ParseError::NotUtf8 => self.err("Found non-UTF-8 character.".to_string()),
            ParseError::IoError(e) => format!("Failed to read input, with error {}.", e),
            ParseError::UnexpectedEof => "Unexpected end of file.".to_string(),
            ParseError::UnexpectedChar(c) => {
                self.err(format!("Found unexpected character '{}'", c))
            }
            ParseError::ExpectedEof => {
                "Found trailing content, when expecting end of file.".to_string()
            }
        }
    }

    fn next(&mut self) -> Result<char, ParseError> {
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

    fn expect(&mut self, to_match: char) -> Result<(), ParseError> {
        let next = self.next()?;
        if next != to_match {
            Err(ParseError::UnexpectedChar(to_match))
        } else {
            Ok(())
        }
    }

    fn expect_eof(&mut self) -> Result<(), ParseError> {
        match self.inner.peek() {
            Ok(None) => Ok(()),
            _ => Err(ParseError::ExpectedEof),
        }
    }

    fn expect_predicate<P>(&mut self, predicate: P) -> Result<char, ParseError>
    where
        P: Fn(char) -> bool,
    {
        let next = self.next()?;
        if !predicate(next) {
            Err(ParseError::UnexpectedChar(next))
        } else {
            Ok(next)
        }
    }

    fn try_match(&mut self, to_match: char) -> Result<bool, ParseError> {
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
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn try_match_eof(&mut self) -> Result<bool, ParseError> {
        match self.inner.peek() {
            Ok(None) => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn try_match_predicate<P>(&mut self, predicate: P) -> Result<Option<char>, ParseError>
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
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn eat_space(&mut self) -> Result<(), ParseError> {
        while self
            .try_match_predicate(|c| c.is_whitespace() && c != '\n' && c != '\r')
            .eof_ok()?
            .is_some()
        {}
        Ok(())
    }
}
