use lut::{Table, Access, Any};
use lut::media_type_chars::{
    MediaTypeChars,
    QText, QTextWs,
    DQuoteOrEscape, Ws,
    RestrictedToken, VCharWs,
    Token, HttpToken
};
use quoted_string::error::CoreError;
use quoted_string::spec::{
    PartialCodePoint,
    ParsingImpl,
    State,
    WithoutQuotingValidator,
    QuotingClassifier, QuotingClass,
};

use super::utils::{MimeParsingExt, FWSState};


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct AnyParsingImpl;

impl ParsingImpl for AnyParsingImpl {

    fn can_be_quoted(_bch: PartialCodePoint) -> bool {
        true
    }

    fn handle_normal_state(_bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
        Ok((State::Normal, true))
    }

}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct NormalParsingImpl;

impl ParsingImpl for NormalParsingImpl {

    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        MediaTypeChars::check_at(bch.as_u8() as usize, VCharWs)
    }

    fn handle_normal_state(bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
        if MediaTypeChars::check_at(bch.as_u8() as usize, QTextWs) {
            Ok((State::Normal, true))
        } else {
            Err(CoreError::InvalidChar)
        }
    }

}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct StrictParsingImpl;

impl ParsingImpl for StrictParsingImpl {

    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        let iu8 = bch.as_u8();
        iu8 == b'"' || iu8 == b'\\'
    }

    fn handle_normal_state(bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
        if MediaTypeChars::check_at(bch.as_u8() as usize, QTextWs) {
            Ok((State::Normal, true))
        } else {
            Err(CoreError::InvalidChar)
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct NormalQuoting;

impl QuotingClassifier for NormalQuoting {

    fn classify_for_quoting(pcp: PartialCodePoint) -> QuotingClass {
        let idx = pcp.as_u8() as usize;
        let lres = MediaTypeChars::lookup(idx);
        if QTextWs.check(lres) {
            QuotingClass::QText
        } else if DQuoteOrEscape.check(lres) && idx <= 0x7f {
            QuotingClass::NeedsQuoting
        } else {
            QuotingClass::Invalid
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct RestrictedTokenValidator {
    count: usize
}

impl WithoutQuotingValidator for RestrictedTokenValidator {
    fn next(&mut self, pcp: PartialCodePoint) -> bool {
        let iu8 = pcp.as_u8();
        let res =
            if self.count == 0 {
                //FIXME use iu8.is_ascii_alphanumeric() once stable (1.24)
                iu8 < 0x7f && (iu8 as char).is_alphanumeric()
            } else {
                MediaTypeChars::check_at(iu8 as usize, RestrictedToken)
            };
        if res {
            self.count += 1;
        }
        res
    }


    fn end(&self) -> bool {
        0 < self.count && self.count < 128
    }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct HttpObsQuoting;

impl QuotingClassifier for HttpObsQuoting {

    fn classify_for_quoting(pcp: PartialCodePoint) -> QuotingClass {
        let idx = pcp.as_u8() as usize;
        let lres = MediaTypeChars::lookup(idx);
        if idx > 0x7f || QTextWs.check(lres) {
            QuotingClass::QText
        } else if DQuoteOrEscape.check(lres) {
            QuotingClass::NeedsQuoting
        } else {
            QuotingClass::Invalid
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct HttpObsParsingImpl;

impl ParsingImpl for HttpObsParsingImpl {

    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        let idx = bch.as_u8() as usize;
        idx > 0x7f || MediaTypeChars::check_at(idx, QTextWs)
    }
    fn handle_normal_state(bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
        let idx = bch.as_u8() as usize;
        if idx > 0x7f || MediaTypeChars::check_at(idx, QTextWs) {
            Ok((State::Normal, true))
        } else {
            Err(CoreError::InvalidChar)
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct HttpTokenValidator;

impl HttpTokenValidator {
    pub fn new() -> Self {
        Default::default()
    }
}

impl WithoutQuotingValidator for HttpTokenValidator {
    fn next(&mut self, pcp: PartialCodePoint) -> bool {
        MediaTypeChars::check_at(pcp.as_u8() as usize, HttpToken)
    }

    fn end(&self) -> bool {
        true
    }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct MimeTokenValidator {
    len_valid: bool
}

impl MimeTokenValidator {
    pub fn new() -> Self {
        Default::default()
    }
}

impl WithoutQuotingValidator for MimeTokenValidator {
    fn next(&mut self, pcp: PartialCodePoint) -> bool {
        MediaTypeChars::check_at(pcp.as_u8() as usize, Token)
    }
    fn end(&self) -> bool {
        true
    }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct AnyQuoting;

impl QuotingClassifier for AnyQuoting {
    fn classify_for_quoting(pcp: PartialCodePoint) -> QuotingClass {
        let iu8 = pcp.as_u8();
        if iu8 == b'"' || iu8 == b'\\' {
            QuotingClass::NeedsQuoting
        } else {
            QuotingClass::QText
        }
    }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct MimeObsQuoting;

impl QuotingClassifier for MimeObsQuoting {
    fn classify_for_quoting(pcp: PartialCodePoint) -> QuotingClass {
        let iu8 = pcp.as_u8();
        if MediaTypeChars::check_at(iu8 as usize, QTextWs) {
            QuotingClass::QText
        } else if iu8 <= 0x7f {
            QuotingClass::NeedsQuoting
        } else {
            QuotingClass::Invalid
        }
    }
}

macro_rules! def_mime_parsing {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            utf8 = $utf8:tt;
            obsolte_syntax = $obs:tt;
        }
        fn can_be_quoted($nm:ident: PartialCodePoint) -> bool
            $body:block
    ) => (
        $(#[$meta])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        pub struct $name(FWSState);
        impl MimeParsingExt for $name {
            const ALLOW_UTF8: bool = $utf8;
            const OBS: bool = $obs;

            fn custom_state(state: FWSState, emit: bool) -> (State<Self>, bool) {
                (State::Custom($name(state)), emit)
            }
        }

        impl ParsingImpl for $name {
            fn can_be_quoted($nm: PartialCodePoint) -> bool {
                $body
            }

            fn handle_normal_state(bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
                <Self as MimeParsingExt>::handle_normal_state(bch)
            }

            fn advance(&self, bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
                self.0.advance(bch)
            }
        }
    );
}

def_mime_parsing! {
    pub struct MimeObsParsing {
        utf8 = false;
        obsolte_syntax = true;
    }
    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        // obs syntax allows any us-ascii in quoted-pairs
        bch.as_u8() <= 0x7f
    }
}

def_mime_parsing! {
    pub struct MimeObsParsingUtf8 {
        utf8 = true;
        obsolte_syntax = true;
    }
    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        // Internationalized Mail does not extend quoted-pairs just qtext ...
        // obs syntax allows any us-ascii in quoted-pairs
        bch.as_u8() <= 0x7f
    }
}

def_mime_parsing! {
    pub struct MimeParsing {
        utf8 = false;
        obsolte_syntax = false;
    }
    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        // VCHAR / WS == QText + Ws + DQuoteOrEscape
        let idx = bch.as_u8() as usize;
        MediaTypeChars::check_at(idx, Any::new(Ws) | QText | DQuoteOrEscape)
    }
}

def_mime_parsing! {
    pub struct MimeParsingUtf8 {
        utf8 = true;
        obsolte_syntax = false;
    }
    fn can_be_quoted(bch: PartialCodePoint) -> bool {
        // Internationalized Mail does not extend quoted-pairs just qtext ...
        let idx = bch.as_u8() as usize;
        MediaTypeChars::check_at(idx, Any::new(Ws) | QText | DQuoteOrEscape)
    }
}


#[cfg(test)]
mod test {



}