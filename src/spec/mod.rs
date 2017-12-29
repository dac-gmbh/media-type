use std::marker::PhantomData;
use std::fmt::Debug;

use nom::{IResult, ErrorKind};
use seal::Seal;


use quoted_string::error::CoreError;
use quoted_string::spec::{GeneralQSSpec, PartialCodePoint, WithoutQuotingValidator};

pub trait Spec: Seal + GeneralQSSpec {
    type UnquotedValue: WithoutQuotingValidator + Default;

    fn parse_token(input: &str) -> IResult<&str, &str>;
    fn parse_space(input: &str) -> IResult<&str, &str>;

    fn parse_unquoted_value(input: &str) -> IResult<&str, &str> {
        //Http token is MimeToken - '{' - '}'
        let validator = Self::UnquotedValue::default();
        parse_unquoted_value(input, validator)
    }

    fn parse_quoted_string(input: &str) -> IResult<&str, &str> {
        use quoted_string::parse;
        match parse::<Self>(input) {
            Ok(p) => IResult::Done(p.tail, p.quoted_string),
            Err((_pos, e)) => {
                let err = ErrorKind::Custom(e.id() as u32);
                IResult::Error(error_code!(err))
            }
        }
    }
}


pub trait ObsNormalSwitch: Seal+Copy+Clone+Debug {}
pub trait InternationalizedSwitch: Seal+Copy+Clone+Debug {}

#[derive(Copy, Clone, Debug)]
pub struct MimeSpec<
    TP: InternationalizedSwitch = Ascii,
    O: ObsNormalSwitch = Normal
>(PhantomData<(TP,O)>);

impl<T: InternationalizedSwitch, O: ObsNormalSwitch> Seal for MimeSpec<T, O> {}

#[derive(Copy, Clone, Debug)]
pub struct HttpSpec<
    O: ObsNormalSwitch = Normal
>(PhantomData<O>);

impl<O: ObsNormalSwitch> Seal for HttpSpec<O> {}

#[derive(Copy, Clone, Debug)]
pub struct StrictSpec;
impl Seal for StrictSpec {}

/// # Note
///
/// Because the AnySpec is meant to be able to parse mimes from "any" spec it has to be able
/// to handle all the thinks from MIME like soft-line brakes and comments in the mime type,
/// which makes it _slower_ then e.g. HttpSpec
#[derive(Copy, Clone, Debug)]
pub struct AnySpec;
impl Seal for AnySpec {}

#[derive(Copy, Clone, Debug)]
pub struct Obs;
impl Seal for Obs {}
impl ObsNormalSwitch for Obs {}

#[derive(Copy, Clone, Debug)]
pub struct Normal;
impl Seal for Normal {}
impl ObsNormalSwitch for Normal {}

#[derive(Copy, Clone, Debug)]
pub struct Ascii;
impl Seal for Ascii {}
impl InternationalizedSwitch for Ascii {}

#[derive(Copy, Clone, Debug)]
pub struct Internationalized;
impl Seal for Internationalized {}
impl InternationalizedSwitch for Internationalized {}


// It would be nicer to have it in parse but it's needed for the default impl
// and placing it in parse would lead to a circular dependency
pub(crate) fn parse_unquoted_value<V>(input: &str, mut validator: V) -> IResult<&str, &str>
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
        let err = ErrorKind::Custom(CoreError::ZeroSizedValue.id() as u32);
        return IResult::Error(error_position!(err, input));
    }
    if validator.end() {
        let (parsed, rest) = input.split_at(end_idx.unwrap_or(input.len()));
        return IResult::Done(rest, parsed)
    } else {
        let err = ErrorKind::Custom(CoreError::InvalidChar.id() as u32);
        return IResult::Error(error_position!(err, &input[pos..]));
    }
}