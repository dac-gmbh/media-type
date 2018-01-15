
use quoted_string::spec::GeneralQSSpec;
use media_type_impl_utils::quoted_string::{self as impl_qs_spec, MimeParsingExt};
use media_type_impl_utils::percent_encoding::{MimePercentEncodeSet, HttpPercentEncodeSet};

use error::ParserErrorRef;
use spec::*;

impl GeneralQSSpec for AnySpec {
    type Quoting = impl_qs_spec::AnyQuoting;
    type Parsing = impl_qs_spec::AnyParsingImpl;
}

impl GeneralQSSpec for StrictSpec {
    type Quoting = impl_qs_spec::NormalQuoting;
    type Parsing = impl_qs_spec::StrictParsingImpl;
}

impl GeneralQSSpec for HttpSpec<Modern> {
    type Quoting = impl_qs_spec::NormalQuoting;
    type Parsing = impl_qs_spec::NormalParsingImpl;
}

impl GeneralQSSpec for HttpSpec<Obs> {
    type Quoting = impl_qs_spec::NormalUtf8Quoting;
    type Parsing = impl_qs_spec::HttpObsParsingImpl;
}

impl GeneralQSSpec for MimeSpec<Ascii, Modern> {
    type Quoting = impl_qs_spec::NormalQuoting;
    type Parsing = impl_qs_spec::MimeParsing;
}

impl GeneralQSSpec for MimeSpec<Ascii, Obs> {
    type Quoting = impl_qs_spec::MimeObsQuoting;
    type Parsing = impl_qs_spec::MimeObsParsing;
}

impl GeneralQSSpec for MimeSpec<Internationalized, Modern> {
    type Quoting = impl_qs_spec::NormalUtf8Quoting;
    type Parsing = impl_qs_spec::MimeParsingUtf8;
}

impl GeneralQSSpec for MimeSpec<Internationalized, Obs> {
    type Quoting = impl_qs_spec::MimeObsUtf8Quoting;
    type Parsing = impl_qs_spec::MimeObsParsingUtf8;
}


impl Spec for StrictSpec {
    //is Http is like mime but forbids '{','}' so using it here is fine
    type PercentEncodeSet = HttpPercentEncodeSet;

    fn parse_token(input: &str) -> Result<usize, ParserErrorRef> {
        let validator = impl_qs_spec::StrictTokenValidator::default();
        parse_unquoted_value(input, validator)
    }

    fn parse_space(input: &str) -> Result<usize, ParserErrorRef> {
        Ok(parse_opt_ws(input))
    }

    type UnquotedValue = impl_qs_spec::HttpTokenValidator;

}

impl Spec for AnySpec {
    //is Http is like mime but forbids '{','}' so we use the mime set
    type PercentEncodeSet = MimePercentEncodeSet;

    fn parse_token(input: &str) -> Result<usize, ParserErrorRef> {
        let validator = impl_qs_spec::MimeTokenValidator::default();
        parse_unquoted_value(input, validator)
    }

    fn parse_space(input: &str) -> Result<usize, ParserErrorRef> {
        use super::parse_cfws::parse_opt_cfws;
        parse_opt_cfws::<<MimeSpec<Internationalized, Obs> as GeneralQSSpec>::Parsing>(input)
    }

    type UnquotedValue = impl_qs_spec::MimeTokenValidator;
}


impl<O> Spec for HttpSpec<O>
    where O: ObsNormalSwitch, HttpSpec<O>: GeneralQSSpec
{
    type PercentEncodeSet = HttpPercentEncodeSet;

    fn parse_token(input: &str) -> Result<usize, ParserErrorRef> {
        Self::parse_unquoted_value(input)
    }

    fn parse_space(input: &str) -> Result<usize, ParserErrorRef> {
        Ok(parse_opt_ws(input))
    }

    type UnquotedValue = impl_qs_spec::HttpTokenValidator;

}

impl<I, O> Spec for MimeSpec<I, O>
    where O: ObsNormalSwitch,
          I: InternationalizedSwitch,
          MimeSpec<I, O>: GeneralQSSpec,
          <MimeSpec<I,O> as GeneralQSSpec>::Parsing: MimeParsingExt
{

    type PercentEncodeSet = MimePercentEncodeSet;
    type UnquotedValue = impl_qs_spec::MimeTokenValidator;

    fn parse_token(input: &str) -> Result<usize, ParserErrorRef> {
        Self::parse_unquoted_value(input)
    }

    fn parse_space(input: &str) -> Result<usize, ParserErrorRef> {
        use super::parse_cfws::parse_opt_cfws;
        parse_opt_cfws::<<MimeSpec<I,O> as GeneralQSSpec>::Parsing>(input)
    }

}

fn parse_opt_ws(input: &str) -> usize {
    input.bytes()
        .position(|iu8| iu8 != b' ' && iu8 != b'\t')
        .unwrap_or(input.len())
}
