use std::marker::PhantomData;
use std::fmt::Debug;
use std::default::Default;

use error::{ParserErrorRef, ErrorKind, ExpectedChar};
use seal::Seal;

use percent_encoding::EncodeSet;

use quoted_string::parse as qs_parse;
use quoted_string::error::CoreError;
use quoted_string::spec::{GeneralQSSpec, PartialCodePoint, WithoutQuotingValidator};

pub trait Spec: Seal + GeneralQSSpec {

    type PercentEncodeSet: EncodeSet + Default;

    type UnquotedValue: WithoutQuotingValidator + Default;

    fn parse_token(input: &str) -> Result<usize, ParserErrorRef>;
    fn parse_space(input: &str) -> Result<usize, ParserErrorRef>;

    fn validate_token(input: &str) -> Result<(), ParserErrorRef> {
        let end = Self::parse_token(input)?;
        debug_assert!(end <= input.len(), "end is a index in input, so it's <= input.len()");
        if end == input.len() {
            Ok(())
        } else {
            Err(ErrorKind::UnexpectedChar {
                pos: end,
                expected: ExpectedChar::CharClass("token char")
            }.with_input(input))
        }
    }

    fn parse_unquoted_value(input: &str) -> Result<usize, ParserErrorRef> {
        //Http token is MimeToken - '{' - '}'
        let validator = Self::UnquotedValue::default();
        parse_unquoted_value(input, validator)
    }

    fn parse_quoted_string(input: &str) -> Result<usize, ParserErrorRef> {
        match qs_parse::<Self>(input) {
            //we just want the offset
            Ok(pres) => Ok(pres.quoted_string.len()),
            Err((pos, cause)) => {
                Err(ErrorKind::QuotedParamValue { pos, cause }.with_input(input))
            }
        }

    }
}


pub trait ObsNormalSwitch: Seal+Copy+Clone+Debug {}
pub trait InternationalizedSwitch: Seal+Copy+Clone+Debug {}

#[derive(Copy, Clone, Debug, Default)]
pub struct MimeSpec<
    TP: InternationalizedSwitch = Internationalized,
    O: ObsNormalSwitch = Obs
>(PhantomData<(TP,O)>);

impl<T: InternationalizedSwitch, O: ObsNormalSwitch> Seal for MimeSpec<T, O> {}

#[derive(Copy, Clone, Debug, Default)]
pub struct HttpSpec<
    O: ObsNormalSwitch = Obs
>(PhantomData<O>);

impl<O: ObsNormalSwitch> Seal for HttpSpec<O> {}

#[derive(Copy, Clone, Debug, Default)]
pub struct StrictSpec;
impl Seal for StrictSpec {}

/// # Note
///
/// Because the AnySpec is meant to be able to parse mimes from "any" spec it has to be able
/// to handle all the thinks from MIME like soft-line brakes and comments in the mime type,
/// which makes it _slower_ then e.g. HttpSpec
#[derive(Copy, Clone, Debug, Default)]
pub struct AnySpec;
impl Seal for AnySpec {}

#[derive(Copy, Clone, Debug, Default)]
pub struct Obs;
impl Seal for Obs {}
impl ObsNormalSwitch for Obs {}

#[derive(Copy, Clone, Debug, Default)]
pub struct Modern;
impl Seal for Modern {}
impl ObsNormalSwitch for Modern {}

#[derive(Copy, Clone, Debug, Default)]
pub struct Ascii;
impl Seal for Ascii {}
impl InternationalizedSwitch for Ascii {}

#[derive(Copy, Clone, Debug, Default)]
pub struct Internationalized;
impl Seal for Internationalized {}
impl InternationalizedSwitch for Internationalized {}

macro_rules! zs_conversions {
    ($($tp:ty => $tp2:ty;)*) => ($(
        impl From<$tp> for $tp2 {
            fn from(_: $tp) -> $tp2 {
                Default::default()
            }
        }
    )*);
}

zs_conversions! {
    MimeSpec<Ascii, Obs> => MimeSpec<Internationalized, Obs>;
    MimeSpec<Ascii, Modern> => MimeSpec<Internationalized, Modern>;
    MimeSpec<Ascii, Modern> => MimeSpec<Ascii, Obs>;
    MimeSpec<Ascii, Modern> => MimeSpec<Internationalized, Obs>;
    MimeSpec<Internationalized, Modern> => MimeSpec<Internationalized, Obs>;
    HttpSpec<Modern> => HttpSpec<Obs>;
    StrictSpec => HttpSpec<Modern>;
    StrictSpec => HttpSpec<Obs>;
    StrictSpec => MimeSpec<Ascii, Obs>;
    StrictSpec => MimeSpec<Ascii, Modern>;
    StrictSpec => MimeSpec<Internationalized, Obs>;
    StrictSpec => MimeSpec<Internationalized, Modern>;
    StrictSpec => AnySpec;
    HttpSpec<Modern> => AnySpec;
    HttpSpec<Obs> => AnySpec;
    MimeSpec<Ascii, Obs> => AnySpec;
    MimeSpec<Ascii, Modern> => AnySpec;
    MimeSpec<Internationalized, Obs> => AnySpec;
    MimeSpec<Internationalized, Modern> => AnySpec;
}

// It would be nicer to have it in parse but it's needed for the default impl
// and placing it in parse would lead to a circular dependency
pub(crate) fn parse_unquoted_value<V>(input: &str, mut validator: V) -> Result<usize, ParserErrorRef>
    where V: WithoutQuotingValidator
{
    let mut end_idx = None;
    for (idx, iu8) in input.bytes().enumerate() {
        if !validator.next(PartialCodePoint::from_utf8_byte(iu8)) {
            end_idx = Some(idx);
            break
        }
    }
    let pos = end_idx.unwrap_or(input.len());
    if pos == 0 {
        return Err(ErrorKind::UnquotedParamValue {
            pos, cause: CoreError::ZeroSizedValue
        }.with_input(input));
    }
    if validator.end() {
        Ok(pos)
    } else {
        return Err(ErrorKind::UnquotedParamValue {
            pos, cause: CoreError::InvalidChar
        }.with_input(input));
    }
}