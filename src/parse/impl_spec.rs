
use nom::IResult;

use quoted_string::spec::GeneralQSSpec;


use media_type_parser_utils::quoted_string_spec::{self as impl_qs_spec, MimeParsingExt};

use spec::*;

impl GeneralQSSpec for AnySpec {
    type Quoting = impl_qs_spec::AnyQuoting;
    type Parsing = impl_qs_spec::AnyParsingImpl;
}

impl GeneralQSSpec for StrictSpec {
    type Quoting = impl_qs_spec::NormalQuoting;
    type Parsing = impl_qs_spec::StrictParsingImpl;
}

impl GeneralQSSpec for HttpSpec<Normal> {
    type Quoting = impl_qs_spec::NormalQuoting;
    type Parsing = impl_qs_spec::NormalParsingImpl;
}

impl GeneralQSSpec for HttpSpec<Obs> {
    type Quoting = impl_qs_spec::HttpObsQuoting;
    type Parsing = impl_qs_spec::HttpObsParsingImpl;
}

impl GeneralQSSpec for MimeSpec<Ascii, Normal> {
    type Quoting = impl_qs_spec::NormalQuoting;
    type Parsing = impl_qs_spec::MimeParsing;
}

impl GeneralQSSpec for MimeSpec<Ascii, Obs> {
    type Quoting = impl_qs_spec::MimeObsQuoting;
    type Parsing = impl_qs_spec::MimeObsParsing;
}

impl GeneralQSSpec for MimeSpec<Internationalized, Normal> {
    type Quoting = impl_qs_spec::NormalQuoting;
    type Parsing = impl_qs_spec::MimeParsingUtf8;
}

impl GeneralQSSpec for MimeSpec<Internationalized, Obs> {
    type Quoting = impl_qs_spec::MimeObsQuoting;
    type Parsing = impl_qs_spec::MimeObsParsingUtf8;
}


impl Spec for StrictSpec {
    fn parse_token(input: &str) -> IResult<&str, &str> {
        let validator = impl_qs_spec::RestrictedTokenValidator::default();
        parse_unquoted_value(input, validator)
    }

    fn parse_space(input: &str) -> IResult<&str, &str> {
        parse_opt_ws(input)
    }

    type UnquotedValue = impl_qs_spec::HttpTokenValidator;

}

impl Spec for AnySpec {
    fn parse_token(input: &str) -> IResult<&str, &str> {
        let validator = impl_qs_spec::MimeTokenValidator::default();
        parse_unquoted_value(input, validator)
    }

    fn parse_space(input: &str) -> IResult<&str, &str> {
        use super::parse_cfws::parse_opt_cfws;
        parse_opt_cfws::<<MimeSpec<Internationalized, Obs> as GeneralQSSpec>::Parsing>(input)
    }

    type UnquotedValue = impl_qs_spec::MimeTokenValidator;
}


impl<O> Spec for HttpSpec<O>
    where O: ObsNormalSwitch, HttpSpec<O>: GeneralQSSpec
{
    fn parse_token(input: &str) -> IResult<&str, &str> {
        Self::parse_unquoted_value(input)
    }

    fn parse_space(input: &str) -> IResult<&str, &str> {
        parse_opt_ws(input)
    }

    type UnquotedValue = impl_qs_spec::HttpTokenValidator;

}

impl<I, O> Spec for MimeSpec<I, O>
    where O: ObsNormalSwitch,
          I: InternationalizedSwitch,
          MimeSpec<I, O>: GeneralQSSpec,
          <MimeSpec<I,O> as GeneralQSSpec>::Parsing: MimeParsingExt
{

    type UnquotedValue = impl_qs_spec::MimeTokenValidator;

    fn parse_token(input: &str) -> IResult<&str, &str> {
        Self::parse_unquoted_value(input)
    }

    fn parse_space(input: &str) -> IResult<&str, &str> {
        use super::parse_cfws::parse_opt_cfws;
        parse_opt_cfws::<<MimeSpec<I,O> as GeneralQSSpec>::Parsing>(input)
    }

}

fn parse_opt_ws(input: &str) -> IResult<&str, &str> {
    input.bytes()
        .position(|iu8| iu8 != b' ' && iu8 != b'\t')
        .map(|idx| IResult::Done(&input[idx..], &input[..idx]))
        .unwrap_or(IResult::Done("", input))
}

