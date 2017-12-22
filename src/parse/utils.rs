use nom::IResult;

use lut::media_type_chars::{MediaTypeChars, Ws, QText, ObsQText, QTextWs, ObsQTextWs};
use lut::{Table, Access};

use quoted_string::spec::{State, ParsingImpl, PartialCodePoint};
use quoted_string::error::CoreError;

#[inline]
pub fn crlf(input: &str) -> IResult<&str, &str> {
    tag!(input, "\r\n")
}

#[inline]
pub fn ws(input: &str) -> IResult<&str, char> {
    one_of!(input, " \t")
}


pub trait MimeParsingExt: ParsingImpl {
    const ALLOW_UTF8: bool;
    const OBS: bool;

    fn custom_state(state: FWSState, emit: bool) -> (State<Self>, bool);

    fn handle_normal_state(bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
        let iu8 = bch.as_u8();

        let is_qtext_ws = if Self::OBS {
            MediaTypeChars::check_at(iu8 as usize, ObsQTextWs)
        } else {
            MediaTypeChars::check_at(iu8 as usize, QTextWs)
        };

        if is_qtext_ws || (Self::ALLOW_UTF8 && iu8 > 0x7f) {
            Ok((State::Normal, true))
        } else if iu8 == b'\r' {
            Ok(Self::custom_state(FWSState::HitCr, false))
        } else {
            Err(CoreError::InvalidChar)
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FWSState {
    HitCr,
    HitNl,
    HadFws
}

impl FWSState {

    pub fn advance<Impl: MimeParsingExt>(self, bch: PartialCodePoint)
                                     -> Result<(State<Impl>, bool), CoreError>
    {
        use self::FWSState::*;
        let iu8 = bch.as_u8();
        match self {
            HitCr => {
                if iu8 == b'\n' {
                    Ok(Impl::custom_state(FWSState::HitNl, false))
                } else {
                    Err(CoreError::InvalidChar)
                }
            },
            HitNl => {
                if iu8 == b' ' || iu8 == b'\t' {
                    //FIXME emit true?
                    Ok(Impl::custom_state(FWSState::HadFws, false))
                } else {
                    Err(CoreError::InvalidChar)
                }
            },
            HadFws => {
                let lres = MediaTypeChars::lookup(iu8 as usize);
                // QText will be zero-sized so default etc. will be optimized awy
                let is_qtext = if Impl::OBS {
                    QText.check(lres)
                } else {
                    ObsQText.check(lres)
                };
                if is_qtext || (Impl::ALLOW_UTF8 && iu8 > 0x7f) {
                    Ok((State::Normal, true))
                } else if Ws.check(lres) {
                    Ok(Impl::custom_state(FWSState::HadFws, true))
                } else if iu8 == b'"' {
                    Ok((State::End, false))
                } else if iu8 == b'\\' {
                    Ok((State::QPStart, false))
                } else {
                    Err(CoreError::InvalidChar)
                }
            }
        }
    }
}
