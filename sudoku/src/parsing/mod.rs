use self::chars_reader::{CharReader, CharReaderError};
use std::{convert::Infallible, iter::Peekable, marker::PhantomData};

pub mod chars_reader;
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

impl<T> AllowEof for Result<T, ParseError> {
    type Return = Option<T>;
    fn eof_ok(self) -> Result<Self::Return, ParseError> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(ParseError::UnexpectedEof) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

pub trait DefaultParseError<T, P, I, E>
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

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
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

    pub fn expect_integer(&mut self) -> Result<usize, ParseError> {
        match self
            .collect_predicate(|c| c.is_ascii_digit())?
            .parse::<usize>()
        {
            Err(_) => Err(ParseError::UnexpectedEof),
            Ok(value) => Ok(value),
        }
    }

    pub fn expect_float(&mut self) -> Result<f64, ParseError> {
        let mut float_str = String::new();
        if let Some(c) = self.try_match_predicate(|c| c == '+' || c == '-')? {
            float_str.push(c);
        }
        float_str.extend(self.collect_predicate(|c| c.is_ascii_digit())?.chars());
        if self.try_match('.')? {
            float_str.push('.');
            float_str.extend(self.collect_predicate(|c| c.is_ascii_digit())?.chars());

            if self.try_match('e')? || self.try_match('E')? {
                float_str.push('e');
                if let Some(c) = self.try_match_predicate(|c| c == '+' || c == '-')? {
                    float_str.push(c);
                }
                let exponent = self.expect_integer()?;
                float_str.push_str(&exponent.to_string());
            }
        }
        let float = float_str.parse::<f64>();
        if float.is_err() {
            return Err(ParseError::UnexpectedEof);
        }
        Ok(float.unwrap())
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
            None => Ok(false),
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
            None => Ok(None),
        }
    }

    pub fn eat_space(&mut self) -> Result<bool, ParseError> {
        let mut ate_any = false;
        while self
            .try_match_predicate(|c| c.is_whitespace() && c != '\n' && c != '\r')?
            .is_some()
        {
            ate_any = true;
        }
        Ok(ate_any)
    }

    pub fn collect_predicate<K>(&mut self, predicate: K) -> Result<String, ParseError>
    where
        K: Fn(&char) -> bool,
    {
        let mut path = String::new();
        while let Some(c) = ParserCharIter::peek(&mut self.inner)? {
            if !predicate(&c) {
                break;
            }
            path.push(
                self.next()
                    .expect("The peek() before should prevent errors here."),
            );
        }
        Ok(path)
    }

    pub fn discard_predicate<K>(&mut self, predicate: K) -> Result<(), ParseError>
    where
        K: Fn(&char) -> bool,
    {
        while let Some(c) = ParserCharIter::peek(&mut self.inner)? {
            if !predicate(&c) {
                break;
            }
            self.next()
                .expect("The peek() before should prevent errors here.");
        }
        Ok(())
    }
}
