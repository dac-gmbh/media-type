#[allow(unused_imports)]
use std::ascii::AsciiExt;

use nom::Err;
use nom::IResult;
use quoted_string::ContentChars;

pub use ::spec::{
    Spec,
    StrictSpec,
    MimeSpec, HttpSpec,
    Obs, Normal, Internationalized, Ascii,
    ObsNormalSwitch, InternationalizedSwitch,
    AnySpec
};


mod utils;
mod impl_qs_spec;
mod impl_spec;
mod parse_cfws;

#[derive(Debug, Clone)]
pub(crate) struct ParseResult<'a> {
    pub(crate) type_: &'a str,
    pub(crate) subtype: &'a str,
    //most common media types do not have parameter,
    // "charset" is the most common parameter
    // today it's mainly used with the value utf-8 (and utf8 with and without quoting)
    // This means if a parameter is there is likely to be charset=utf-8 especially for Http use cases
    // (except for multipart/ media types where it is boundary="something different every time")
    // by making charset=utf-8 into a boolean it allows us to not allocate any additional memory
    // for any media type which either does not has a parameter or where it is charset=utf-8, which
    // are most. (hint: a Vec::new() does not allocate!, only if you put thinks in it it does)
    // As a sideeffect it also speeds up comparisons if one mime has a utf8 charset and the other
    // does not (because it e.g. is a completely different media type)
    pub(crate) charset_utf8: bool,
    pub(crate) params: Vec<(&'a str, &'a str)>
}

impl<'a> ParseResult<'a> {

    pub(crate) fn repr_len(&self) -> usize {
        let mut len = self.type_.len()
            + 1
            //FIXME add suffix
            + self.subtype.len();

        if self.charset_utf8 {
            len += 13;//"charset=utf-8".len()
        }

        for &(name, value) in self.params.iter() {
            len += 1 + name.len() + 1 + value.len()
        }

        len
    }
}

pub(crate) fn validate<S: Spec>(input: &str) -> bool {
    parse::<S>(input).is_ok()
}

pub(crate) fn parse<S: Spec>(input: &str) -> Result<ParseResult, Err<&str>> {
    complete!(input, call!(parse_media_type::<S>)).to_result()
}

fn parse_media_type<S: Spec>(input: &str) -> IResult<&str, ParseResult> {
    do_parse!(input,
        head: call!(parse_media_type_head::<S>) >>
        params: call!(parse_media_type_params::<S>) >>
        (ParseResult {
            type_: head.0,
            subtype: head.1,
            charset_utf8: params.0,
            params: params.1.into_iter().map(|(name, value)| {
                (name, value)
            }).collect::<Vec<_>>()
        })
    )
}

fn parse_media_type_head<S: Spec>(input: &str) -> IResult<&str, (&str, &str)> {
    do_parse!(input,
        type_: call!(S::parse_token) >>
        char!('/') >>
        //FIXME consider the suffic
        subtype: call!(S::parse_token) >>
        call!(S::parse_space) >>
        (type_, subtype)
    )
}

fn parse_media_type_params<S: Spec>(input: &str) -> IResult<&str, (bool, Vec<(&str, &str)>)>
{
    let mut has_utf8 = false;
    map!(input, tuple!(
        many0!(
            do_parse!(
                char!(';') >>
                call!(S::parse_space) >>
                name: call!(S::parse_token) >>
                char!('=') >>
                value: alt_complete!(
                    call!(S::parse_quoted_string) |
                    call!(S::parse_unquoted_value)

                ) >>
                ({
                    if !has_utf8 {
                        has_utf8 = is_charset_utf8_raw::<S>(name, value)
                    }
                    (name, value)
                })
            )
        ),
        call!(S::parse_space),
        eof!()
    ), |out| (has_utf8, out.0))
}

fn is_charset_utf8_raw<S: Spec>(name: &str, value: &str) -> bool {
    #[inline]
    fn eq<E>(left: Option<Result<char, E>>, right: char) -> bool {
        if let Some(Ok(ch)) = left {
            ch == right
        } else {
            false
        }
    }
    // check for both 'utf8' and 'utf-8' in parallel
    if name.eq_ignore_ascii_case("charset") {
        let mut chars = ContentChars::<S>::from_str(value);
        let start_ok = eq(chars.next(), 'u') &&
            eq(chars.next(), 't') &&
            eq(chars.next(), 'f');

        if start_ok {
            let next = chars.next();
            if eq(next, '8') {
                return chars.next().is_none()
            }
            return eq(next, '-') &&
                eq(chars.next(), '8') &&
                chars.next().is_none()
        }
    }
    false
}












