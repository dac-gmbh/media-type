use std::fmt::{self, Display};
use std::error::Error as StdError;
use quoted_string::error::CoreError;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExpectedChar {
    Char(char),
    CharClass(&'static str),
}

impl Display for ExpectedChar {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        use self::ExpectedChar::*;
        match *self {
            Char(ch) => write!(fter, "{:?}", ch),
            CharClass(chc) => write!(fter, "{:?}", chc)
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ParserErrorKind {

    QuotedParamValue {
        pos: usize,
        cause: CoreError
    },

    UnquotedParamValue {
        pos: usize,
        cause: CoreError
    },

    UnexpectedChar {
        pos: usize,
        expected: ExpectedChar
    },

    UnexpectedEof,

    IllegalCrNlSeq {
        pos: usize
    }
}

impl ParserErrorKind {

    pub fn with_input(self, input: &str) -> ParserErrorRef {
        ParserErrorRef::new(input, self)
    }

    fn description(&self) -> &str {
        use self::ParserErrorKind::*;
        match *self {
            QuotedParamValue {..} => "parsing quoted parameter value failed",
            UnquotedParamValue {..} => "parsing unquoted parameter value failed",
            UnexpectedChar { .. } => "parsing hit an unexpected character",
            UnexpectedEof { .. } => "parsing unexpectedly hit eof",
            IllegalCrNlSeq { .. } => r#"parsing found a illegal "\r\n "/"\r\n\t" seqence"#
        }
    }

    fn cause(&self) -> Option<&StdError> {
        use self::ParserErrorKind::*;
        match self {
            &QuotedParamValue { ref cause, ..} => Some(cause as &StdError),
            &UnquotedParamValue { ref cause, ..} => Some(cause as &StdError),
            _ => None
        }
    }

    fn display(&self, input: &str, fter: &mut fmt::Formatter) -> fmt::Result {
        use self::ParserErrorKind::*;
        match *self {
            QuotedParamValue { pos, cause } => {
                write!(
                    fter,
                    "parsing quoted parameter failed on: {:?} at byte {:?} because of {:?} ({})",
                    input, pos, cause, cause
                )
            },
            UnquotedParamValue { pos, cause } => {
                write!(
                    fter,
                    "parsing unquoted parameter failed on: {:?} at byte {:?} because of {:?} ({})",
                    input, pos, cause, cause
                )
            },
            UnexpectedChar {  pos, expected } => {
                write!(
                    fter,
                    "hit unexpected char {:?} while parsing {:?} at {} expected {}",
                    one_char_str(input, pos), input, pos, expected
                )
            },
            UnexpectedEof => {
                write!(fter, "hit eof unexpectedly in {:?}", input)
            },

            IllegalCrNlSeq { pos } => {
                write!(fter, "hit invalid \"\\r\\n \"/\"\\r\\n\\t\" seq in {:?} at {}", input, pos)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParserErrorRef<'a> {
    input: &'a str,
    kind: ParserErrorKind
}

impl<'a> ParserErrorRef<'a> {

    pub fn new(input: &'a str, kind: ParserErrorKind) -> Self {
        ParserErrorRef { input, kind }
    }

    pub fn input(&self) -> &'a str {
        self.input
    }

    pub fn kind(&self) -> ParserErrorKind {
        self.kind
    }

    pub fn to_owned(&self) -> ParserError {
        ParserError::new(self.input, self.kind)
    }
}

impl<'a> Display for ParserErrorRef<'a> {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        self.kind.display(self.input, fter)
    }
}


impl<'a> StdError for ParserErrorRef<'a> {

    fn description(&self) -> &str {
        self.kind.description()
    }

    fn cause(&self) -> Option<&StdError> {
        self.kind.cause()
    }
}

impl<'a> From<ParserErrorRef<'a>> for ParserError {
    fn from(pref: ParserErrorRef<'a>) -> Self {
        pref.to_owned()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParserError {
    input: String,
    kind: ParserErrorKind
}

impl ParserError {

    pub fn new<I: Into<String>>(input: I, kind: ParserErrorKind) -> Self {
        ParserError { input: input.into(), kind }
    }

    pub fn input(&self) -> &str {
        self.input.as_ref()
    }

    pub fn kind(&self) -> ParserErrorKind {
        self.kind
    }

    //Deref, Borrow, AsRef can not be implemented unless rust has
    // AssociatedTypeConstructors, at last wrt. lifetimes
    pub fn as_ref(&self) -> ParserErrorRef {
        ParserErrorRef {
            input: self.input.as_ref(),
            kind: self.kind
        }
    }
}

impl Display for ParserError {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        self.kind.display(self.input.as_ref(), fter)
    }
}

impl StdError for ParserError {
    fn description(&self) -> &str {
        self.kind.description()
    }

    fn cause(&self) -> Option<&StdError> {
        self.kind.cause()
    }
}


fn one_char_str(inp: &str, offset: usize) -> &str {
    inp.get(offset..)
        .map(|tail: &str| {
            let first_char_len = tail.chars().next().map(|ch| ch.len_utf8()).unwrap_or(0);
            //INDEX_SAFE: if there is no char it's 0, ..0 is always valid if there is a char
            // indexing the substring only containing the existing first char is also valid
            &tail[..first_char_len]
        })
        .unwrap_or("<[BUG] invalid str index in error>")
}