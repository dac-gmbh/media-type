use nom::{IResult, ErrorKind};

use lut::Table;
use lut::media_type_chars::{MediaTypeChars, CText, VCharWs};

use quoted_string::error::CoreError;

use super::utils::{crlf, ws, MimeParsingExt};

//TODO OPTIMIZE crate loop0/1 macro as a version of many0/1 which does not create a vector


pub fn parse_opt_cfws<E: MimeParsingExt>(input: &str) -> IResult<&str, &str> {
    //CFWS = (1*([FWS] comment) [FWS]) / FWS
    //parse just: *([FWS] comment) [FWS]
    // which is fine as its a opt CFWS so empty "" is ok, and
    // just a comment is ok anyway and just a FWS is also ok anyway.
    recognize!(input, tuple!(
        many0!(tuple!(
            call!(parse_opt_fws::<E>),
            call!(parse_comment::<E>)
        )),
        call!(parse_opt_fws::<E>)
    ))
}

fn parse_opt_fws<E: MimeParsingExt>(input: &str) -> IResult<&str, &str> {
    // obs-FWS  =  1*([CRLF] WSP)
    if E::OBS {
        //parse *([CRLF] WSP) as it's optional fws
        recognize!(input, many0!(tuple!(
            opt!(crlf),
            ws
        )))
    } else {
        recognize!(input, tuple!(
            many0!(ws),
            opt!(tuple!(
                crlf,
                many1!(ws)
            ))
        ))
    }
}

fn parse_quoted_pair<E: MimeParsingExt>(input: &str) -> IResult<&str, char> {
    let bytes = input.as_bytes();
    let valid =
        if bytes.len() >= 2  && bytes[0] == b'\\' {
            if E::OBS {
                bytes[1] <= 0x7f
            } else {
                MediaTypeChars::check_at(bytes[1] as usize, VCharWs)
            }
        } else { false };

    if valid {
        //there are no non-ascii quoted-pairs
        IResult::Done(&input[2..], bytes[1] as char)
    } else {
        IResult::Error(error_code!(ErrorKind::Custom(CoreError::InvalidChar.id() as u32)))
    }
}

fn parse_comment<E: MimeParsingExt>(input: &str) -> IResult<&str, &str> {
    // comment  =  "(" *([FWS] ccontent) [FWS] ")"
    // ccontent =  ctext / quoted-pair / comment
    // FWS      =  ([*WSP "\r\n"] 1*WSP) /  obs-FWS
    // obs-FWS  =  1*([CRLF] WSP)
    recognize!(input, tuple!(
        char!('('),
        many0!(tuple!(
            call!(parse_opt_fws::<E>),
            alt!(
                call!(parse_quoted_pair::<E>) => { |_| ()} |
                call!(parse_comment::<E>) => { |_| ()} |
                call!(one_ctext_char::<E>)=> { |_| ()}
            )
        )),
        call!(parse_opt_fws::<E>),
        char!(')')
    ))
}

fn one_ctext_char<E: MimeParsingExt>(input: &str) -> IResult<&str, ()> {
    if input.is_empty() {
        let err = ErrorKind::Custom(CoreError::InvalidChar.id() as u32);
        return IResult::Error(error_code!(err));
    }
    let first_byte = input.as_bytes()[0];
    if MediaTypeChars::check_at(first_byte as usize, CText) {
        //SLICE_SAFE: we know it's only one byte long as it is CText
        IResult::Done(&input[1..], ())
    } else if E::ALLOW_UTF8 && first_byte > 0x7f {
        //UNWRAP_SAFE: len > 0 is assured
        let offset = input.chars().next().unwrap().len_utf8();
        IResult::Done(&input[offset..], ())
    } else {
        IResult::Error(error_code!(ErrorKind::Custom(CoreError::InvalidChar.id() as u32)))
    }
}