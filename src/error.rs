use std::fmt::{self, Display};
use std::error::Error as StdError;
use quoted_string::error::CoreError;

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum  ParserError<'a> {

    QuotedParamValue {
        input: &'a str,
        pos: usize,
        cause: CoreError
    },
    UnquotedParamValue {
        input: &'a str,
        pos: usize,
        cause: CoreError
    },
    UnexpectedChar {
        input: &'a str,
        pos: usize,
        expected: ExpectedChar
    },
    UnexpectedEof {
        input: &'a str
    },

    IllegalCrNlSeq {
        input: &'a str,
        pos: usize
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

impl<'a> Display for ParserError<'a> {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        use self::ParserError::*;
        match *self {
            QuotedParamValue { input, pos, cause } => {
                write!(
                    fter,
                    "parsing quoted parameter failed on: {:?} at byte {:?} because of {:?} ({})",
                    input, pos, cause, cause
                )
            },
            UnquotedParamValue { input, pos, cause } => {
                write!(
                    fter,
                    "parsing unquoted parameter failed on: {:?} at byte {:?} because of {:?} ({})",
                    input, pos, cause, cause
                )
            },
            UnexpectedChar { input, pos, expected } => {
                write!(
                    fter,
                    "hit unexpected char {:?} while parsing {:?} at {} expected {}",
                    one_char_str(input, pos), input, pos, expected
                )
            },
            UnexpectedEof { input } => {
                write!(fter, "hit eof unexpectedly in {:?}", input)
            },

            IllegalCrNlSeq { input, pos } => {
                write!(fter, "hit invalid \"\\r\\n \"/\"\\r\\n\\t\" seq in {:?} at {}", input, pos)
            }
        }
    }
}

impl<'a> StdError for ParserError<'a> {

    fn description(&self) -> &str {
        use self::ParserError::*;
        match *self {
            QuotedParamValue {..} => "parsing quoted parameter value failed",
            UnquotedParamValue {..} => "parsing unquoted parameter value failed",
            UnexpectedChar { .. } => "parsing hit an unexpected character",
            UnexpectedEof { .. } => "parsing unexpectedly hit eof",
            IllegalCrNlSeq { .. } => r#"parsing found a illegal "\r\n "/"\r\n\t" seqence"#
        }
    }

    fn cause(&self) -> Option<&StdError> {
        use self::ParserError::*;
        match self {
            &QuotedParamValue { ref cause, ..} => Some(cause as &StdError),
            &UnquotedParamValue { ref cause, ..} => Some(cause as &StdError),
            _ => None
        }
    }
}
