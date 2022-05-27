use self::chars_reader::{CharReader, CharReaderError};
use std::{convert::Infallible, iter::Peekable, marker::PhantomData};

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

pub struct Parser<P, I, E>
where
    P: ParserCharIter<I, E>,
    I: Iterator<Item = Result<char, E>>,
{
    phantom_iter: PhantomData<I>,
    phantom_err: PhantomData<E>,
    inner: P,
    line: usize,
    column: usize,
}

pub trait AllowEof {
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

trait DefaultParseError<T, P, I, E>
where
    I: Iterator<Item = Result<char, E>>,
    P: ParserCharIter<I, E>,
{
    fn with_default_err_msgs(self, parser: &Parser<P, I, E>) -> Result<T, String>;
}

impl<T, I> DefaultParseError<T, Peekable<I>, I, CharReaderError> for Result<T, ParseError>
where
    I: Iterator<Item = Result<char, CharReaderError>>,
    Peekable<I>: ParserCharIter<I, CharReaderError>,
{
    fn with_default_err_msgs(
        self,
        parser: &Parser<Peekable<I>, I, CharReaderError>,
    ) -> Result<T, String> {
        self.map_err(|e| parser.default_err_msg(e))
    }
}

pub trait ParserCharIter<I, E>
where
    I: Iterator<Item = Result<char, E>>,
{
    fn next(&mut self) -> Result<char, ParseError>;
    fn peek(&mut self) -> Result<Option<char>, ParseError>;
}

impl<I> ParserCharIter<I, CharReaderError> for Peekable<I>
where
    I: Iterator<Item = Result<char, CharReaderError>>,
{
    fn next(&mut self) -> Result<char, ParseError> {
        let error = <Peekable<I> as Iterator>::next(self);
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
        let peek = Peekable::<I>::peek(self);
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
        Err(
            <Peekable<I> as ParserCharIter<I, CharReaderError>>::next(self)
                .expect_err("Ok situations should have been handled above."),
        )
    }
}

impl<I> ParserCharIter<I, Infallible> for Peekable<I>
where
    I: Iterator<Item = Result<char, Infallible>>,
{
    fn next(&mut self) -> Result<char, ParseError> {
        let error = <Peekable<I> as Iterator>::next(self);
        match error {
            Some(x) => match x {
                Ok(char) => Ok(char),
                Err(_) => unreachable!(),
            },
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn peek(&mut self) -> Result<Option<char>, ParseError> {
        let peek = Peekable::<I>::peek(self);
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
        Err(<Peekable<I> as ParserCharIter<I, Infallible>>::next(self)
            .expect_err("Ok situations should have been handled above."))
    }
}

impl<I, E> Parser<Peekable<I>, I, E>
where
    I: Iterator<Item = Result<char, E>>,
    Peekable<I>: ParserCharIter<I, E>,
{
    pub fn new(from: I) -> Self {
        Self {
            phantom_iter: PhantomData,
            phantom_err: PhantomData,
            inner: from.peekable(),
            line: 0,
            column: 0,
        }
    }

    pub fn err(&self, message: String) -> String {
        format!("{message}\nAt {}:{}.", self.line, self.column)
    }

    pub fn default_err_msg(&self, err: ParseError) -> String {
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

    pub fn next(&mut self) -> Result<char, ParseError> {
        let next = ParserCharIter::next(&mut self.inner);
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

    pub fn expect(&mut self, to_match: char) -> Result<(), ParseError> {
        let next = self.next()?;
        if next != to_match {
            Err(ParseError::UnexpectedChar(to_match))
        } else {
            Ok(())
        }
    }

    pub fn expect_str(&mut self, to_match: &str) -> Result<(), ParseError> {
        for char in to_match.chars() {
            self.expect(char)?;
        }
        Ok(())
    }

    pub fn expect_eof(&mut self) -> Result<(), ParseError> {
        match ParserCharIter::peek(&mut self.inner) {
            Ok(None) => Ok(()),
            _ => Err(ParseError::ExpectedEof),
        }
    }

    pub fn expect_predicate<K>(&mut self, predicate: K) -> Result<char, ParseError>
    where
        K: Fn(char) -> bool,
    {
        let next = self.next()?;
        if !predicate(next) {
            Err(ParseError::UnexpectedChar(next))
        } else {
            Ok(next)
        }
    }

    pub fn try_match(&mut self, to_match: char) -> Result<bool, ParseError> {
        let next = ParserCharIter::peek(&mut self.inner)?;
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

    pub fn try_match_str(&mut self, to_match: &str) -> Result<bool, ParseError> {
        for char in to_match.chars() {
            if !self.try_match(char)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn try_match_eof(&mut self) -> Result<bool, ParseError> {
        match ParserCharIter::peek(&mut self.inner) {
            Ok(None) => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn try_match_predicate<K>(&mut self, predicate: K) -> Result<Option<char>, ParseError>
    where
        K: Fn(char) -> bool,
    {
        let next = ParserCharIter::peek(&mut self.inner)?;
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

    pub fn eat_space(&mut self) -> Result<(), ParseError> {
        while self
            .try_match_predicate(|c| c.is_whitespace() && c != '\n' && c != '\r')
            .eof_ok()?
            .is_some()
        {}
        Ok(())
    }
}
