#[allow(unused_imports)]
use std::ascii::AsciiExt;

use nom::Err;
use nom::IResult;

pub use ::spec::{
    Spec,
    StrictSpec,
    MimeSpec, HttpSpec,
    Obs, Normal, Internationalized, Ascii,
    ObsNormalSwitch, InternationalizedSwitch,
    AnySpec
};


mod utils;
mod impl_spec;
mod parse_cfws;

#[derive(Debug, Clone)]
pub(crate) struct ParseResult<'a> {
    pub(crate) type_: &'a str,
    pub(crate) subtype: &'a str,
    pub(crate) params: Vec<(&'a str, &'a str)>
}

impl<'a> ParseResult<'a> {

    pub(crate) fn repr_len(&self) -> usize {
        let mut len = self.type_.len()
            + 1
            //FIXME add suffix
            + self.subtype.len();

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
            params: params
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

fn parse_media_type_params<S: Spec>(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
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
                (name, value)
            )
        ),
        call!(S::parse_space),
        eof!()
    ),  |tpl| tpl.0)
}


#[cfg(test)]
mod test {

    use ::spec::{HttpSpec, Obs};
    use super::{parse, parse_media_type_head};

    //#[cfg(feature="inner-bench")]
    #[cfg(all(feature="inner-bench", test))]
    use ::test::Bencher;


    #[test]
    fn parse_charset_utf8() {
        let pres = assert_ok!(parse::<HttpSpec<Obs>>("text/plain; charset=utf-8"));
        assert_eq!(pres.type_, "text");
        assert_eq!(pres.subtype, "plain");
        assert_eq!(pres.params, vec![("charset", "utf-8")]);
    }




    #[cfg(all(feature="inner-bench", test))]
    #[bench]
    fn parse_head(b: &mut Bencher) {
        let raw = "type/subtype";
        b.bytes = raw.as_bytes().len() as u64;
        b.iter(|| {
            parse_media_type_head::<HttpSpec<Obs>>("type/subtype")
        })
    }


}



