#[allow(unused_imports)]
use std::ascii::AsciiExt;

use error::ParserError;
use self::utils::parse_ascii_char;

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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParamIndices {
    pub(crate) start: usize,
    pub(crate) eq_idx: usize,
    pub(crate) end: usize
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParseResult<'a> {
    pub(crate) input: &'a str,
    pub(crate) slash_idx: usize,
    pub(crate) end_of_type_idx: usize,
    pub(crate) params: Vec<ParamIndices>
}

impl<'a> ParseResult<'a> {

    pub(crate) fn repr_len(&self) -> usize {
        self.params
            .last()
            .map(|param| param.end)
            .unwrap_or(self.end_of_type_idx)
    }
}

pub(crate) fn validate<S: Spec>(input: &str) -> bool {
    parse::<S>(input).is_ok()
}

pub(crate) fn parse<'a, S: Spec>(input: &'a str) -> Result<ParseResult, ParserError<'a>> {
    let (slash_idx, end_of_type_idx) = parse_media_type_head::<S>(input)?;
    let params = parse_media_type_params::<S>(input, end_of_type_idx)?;
    Ok(ParseResult { input, slash_idx, end_of_type_idx, params })
}



fn parse_media_type_head<S: Spec>(input: &str) -> Result<(usize, usize), ParserError> {
    let slash_idx = S::parse_token(input)?;
    let start_of_subtype = parse_ascii_char(input, slash_idx, b'/')?;
    let end_of_type_idx = at_pos!(start_of_subtype do S::parse_token | input);
    Ok((slash_idx, end_of_type_idx))
}




fn parse_media_type_params<S: Spec>(input: &str, offset: usize)
    -> Result<Vec<ParamIndices>, ParserError>
{
    let mut out = Vec::new();
    let mut offset = offset;
    loop {
        //1. parse ws
        let sc_idx = at_pos!(offset do S::parse_space | input );
        //2. if tail end { break }
        if sc_idx == input.len() { break }
        //3. parse ;
        let after_sc_idx = parse_ascii_char(input, sc_idx, b';')?;
        //4. parse ws
        let param_name_start = at_pos!(after_sc_idx do S::parse_space | input);
        //5. parse token
        let param_eq_idx = at_pos!(param_name_start do S::parse_token | input);
        //6. parse =
        let param_value_start = parse_ascii_char(input, param_eq_idx, b'=')?;
        //7. if next == '"' { parse quoted_value } else { parse unquoed_value }
        let param_end_idx;
        if input.as_bytes().get(param_value_start) == Some(&b'"') {
            param_end_idx = at_pos!(param_value_start do S::parse_quoted_string | input);
        } else {
            param_end_idx = at_pos!(param_value_start do S::parse_unquoted_value | input);
        }

        out.push(ParamIndices {
            start: param_name_start,
            eq_idx: param_eq_idx,
            end: param_end_idx
        });

        offset = param_end_idx;
    }

    Ok(out)

}


#[cfg(test)]
mod test {

    use ::spec::{HttpSpec, Obs};
    use super::{parse, ParseResult, ParamIndices};
    #[cfg(all(feature="inner-bench", test))]
    use super::parse_media_type_head;

    //#[cfg(feature="inner-bench")]
    #[cfg(all(feature="inner-bench", test))]
    use ::test::Bencher;


    #[test]
    fn parse_charset_utf8() {
        let pres: ParseResult = assert_ok!(parse::<HttpSpec<Obs>>("text/plain; charset=utf-8"));
        assert_eq!(pres.slash_idx, 4);
        assert_eq!(pres.end_of_type_idx, 10);
        assert_eq!(pres.params, vec![ParamIndices {
            start: 12,
            eq_idx: 19,
            end: 25
        }]);
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



