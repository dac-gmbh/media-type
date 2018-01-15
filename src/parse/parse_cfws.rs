use error::{ParserErrorKind, ParserErrorRef, ExpectedChar};


use lut::Table;
use media_type_impl_utils::lookup_tables::{MediaTypeChars, CText, VCharWs};
use media_type_impl_utils::quoted_string::MimeParsingExt;

pub fn parse_opt_cfws<E: MimeParsingExt>(input: &str) -> Result<usize, ParserErrorRef> {
    //CFWS = (1*([FWS] comment) [FWS]) / FWS
    //parse just: *([FWS] comment) [FWS]
    // which is fine as its a opt CFWS so empty "" is ok, and
    // just a comment is ok anyway and just a FWS is also ok anyway.
    let mut offset = 0;
    loop {
        offset = parse_opt_fws::<E>(input, offset)?;
        if let Some(new_offset) = opt_parse_comment::<E>(&input[offset..])? {
            offset = new_offset;
        } else {
            return Ok(offset);
        }
    }
}

fn parse_opt_fws<E: MimeParsingExt>(input: &str, offset: usize) -> Result<usize, ParserErrorRef> {
    if E::OBS {
        _parse_fws_obs(input, offset)
    } else {
        _parse_fws_modern(input, offset)
    }
}

#[inline]
fn _parse_fws_obs(input: &str, offset: usize) -> Result<usize, ParserErrorRef> {
    // obs-FWS  =  1*([CRLF] WSP)
    // parse *([CRLF] WSP) as it's optional obs-fws
    let mut offset = offset;
    loop {
        let crlfws_offset = parse_opt_crlf_seq(input, offset)?;
        let ws_offset = parse_opt_ws_seq(input, crlfws_offset);
        if offset == ws_offset {
            break
        } else {
            offset = ws_offset
        }
    }
    Ok(offset)
}

#[inline]
fn _parse_fws_modern(input: &str, offset: usize) -> Result<usize, ParserErrorRef> {
    let offset = parse_opt_ws_seq(input, offset);
    let crlf_offset = parse_opt_crlf_seq(input, offset)?;
    if crlf_offset == offset {
        Ok(offset)
    } else {
        Ok(parse_opt_ws_seq(input, crlf_offset))
    }
}


fn opt_parse_comment<E>(input: &str) -> Result<Option<usize>, ParserErrorRef>
    where E: MimeParsingExt
{
    if input.as_bytes().get(0) == Some(&b'(') {
        Ok(Some(inner_parse_comment::<E>(input, 1)?))
    } else {
        Ok(None)
    }

}

/// starts parsing after the initial '('
fn inner_parse_comment<E>(input: &str, offset: usize) -> Result<usize, ParserErrorRef>
    where E: MimeParsingExt
{
    // comment  =  "(" *([FWS] ccontent) [FWS] ")"
    // ccontent =  ctext / quoted-pair / comment
    // FWS      =  ([*WSP "\r\n"] 1*WSP) /  obs-FWS
    // obs-FWS  =  1*([CRLF] WSP)
    let mut offset = offset;
    loop {
        offset = parse_opt_fws::<E>(input, offset)?;
        if let Some(&last_byte) = input.as_bytes().get(offset) {
            //offset now points BEHIND last_byte
            offset += 1;

            if MediaTypeChars::check_at(last_byte as usize, CText)
                || (E::ALLOW_UTF8 && last_byte > 0x7f)
            { continue }

            match last_byte {
                b'(' => {
                    //UNWRAP_SAFE: only returns non if input does not starts with '('
                    // but we know it does
                    offset = inner_parse_comment::<E>(input, offset)?;
                },
                b'\\' => {
                    offset = parse_quotable::<E>(input, offset)?;
                },
                b')' => {
                    return Ok(offset);
                },
                b'\r' | b'\n'  => {
                    return Err(
                        ParserErrorKind::IllegalCrNlSeq { pos: offset - 1 }
                        .with_input(input)
                    );
                }
                _ => {
                    let charclass =
                        if E::ALLOW_UTF8 { "ctext / non-ascii-utf8 / '(' / ')' / '\\'" }
                        else { "ctext / '(' / ')' / '\\'" };

                    return Err(ParserErrorKind::UnexpectedChar {
                        //remember offset already points to the next char
                        pos: offset - 1,
                        expected: ExpectedChar::CharClass(charclass)
                    }.with_input(input));
                }
            }
        } else {
            return Err(ParserErrorKind::UnexpectedEof.with_input(input));
        }
    }
}

fn parse_quotable<E: MimeParsingExt>(input: &str, offset: usize) -> Result<usize, ParserErrorRef>  {
    if let Some(&byte) = input.as_bytes().get(offset) {
        let valid =
            if E::OBS {
                byte <= 0x7f
            } else {
                MediaTypeChars::check_at(byte as usize, VCharWs)
            };
        if valid {
            Ok(offset + 1)
        } else {
            let charclass = if E::OBS { "quotable/obs-quotabe" } else { "quotable" };
            Err(
                ParserErrorKind::UnexpectedChar {
                    pos: offset, expected: ExpectedChar::CharClass(charclass)
                }.with_input(input)
            )
        }
    } else {
        Err(ParserErrorKind::UnexpectedEof.with_input(input))
    }
}

/// parsed both "\r\n " and "\r\n\t"
///
/// if the input does not start with `'\r'` then `offset` is returned
///
/// # Error
///
/// returns an error if the input starts with `'\r'` but does not continue with
/// either `"\n "` or `"\n\t"`
///
#[inline]
pub fn parse_opt_crlf_seq(input: &str, offset: usize) -> Result<usize, ParserErrorRef> {
    if input.as_bytes().get(offset) != Some(&b'\r') {
        Ok(offset)
    } else {
        if input.as_bytes().get(offset + 1) == Some(&b'\n') {
            if input.as_bytes().get(offset + 2).map(|bt|is_ws(*bt)).unwrap_or(false) {
               return Ok(offset + 3)
            }
        }
        Err(ParserErrorKind::IllegalCrNlSeq { pos: offset }.with_input(input))
    }
}

#[inline]
pub fn is_ws(bch: u8) -> bool {
    bch == b' ' || bch == b'\t'
}

#[inline]
pub fn parse_opt_ws_seq(input: &str, offset: usize) -> usize {
    let mut offset = offset;
    let bdata = input.as_bytes();
    while bdata.get(offset).map(|bt| is_ws(*bt)).unwrap_or(false) {
        offset += 1;
    }
    offset
}

#[cfg(test)]
mod test {

    mod opt_parse_comment {
        use media_type_impl_utils::quoted_string::{MimeObsParsing, MimeParsing};
        use super::super::*;

        #[test]
        fn empty() {
            assert_eq!(opt_parse_comment::<MimeObsParsing>("()"), Ok(Some(2)));
        }

        #[test]
        fn simple() {
            let text = "(so is a \"comment)";
            assert_eq!(
                opt_parse_comment::<MimeObsParsing>(text),
                Ok(Some(text.len()))
            );
        }

        #[test]
        fn with_quoted_pair() {
            let text = "(so is a \\(comment)";
            assert_eq!(
                opt_parse_comment::<MimeObsParsing>(text),
                Ok(Some(text.len()))
            );
        }

        #[test]
        fn with_comment() {
            let text = "(= (+ (* 2 3) 4) 10)";
            assert_eq!(
                opt_parse_comment::<MimeObsParsing>(text),
                Ok(Some(text.len()))
            );
        }


        #[test]
        fn with_fws() {
            let text = "(= (+ \r\n (* 2 3) 4) 10)";
            assert_eq!(
                opt_parse_comment::<MimeParsing>(text),
                Ok(Some(text.len()))
            );
        }

        #[test]
        fn with_fws_ons() {
            let text = "(= (+ \r\n (* 2 3) 4) 10)";
            assert_eq!(
                opt_parse_comment::<MimeObsParsing>(text),
                Ok(Some(text.len()))
            );
        }

        #[test]
        fn with_more_data_at_the_end() {
            let cmd = "(abc yay d)";
            let more = "so dada";
            assert_eq!(
                opt_parse_comment::<MimeObsParsing>((String::from(cmd) + more).as_str()),
                Ok(Some(cmd.len()))
            );
        }

        #[test]
        fn obs_with_bad_fws_no_cr() {
            let text = "(= (+ \n (* 2 3) 4) 10)";
            let res = opt_parse_comment::<MimeObsParsing>(text);
            assert_eq!(res, Err(
                ParserErrorKind::IllegalCrNlSeq { pos: 6 }
                    .with_input("(= (+ \n (* 2 3) 4) 10)")
            ));
        }

        #[test]
        fn with_bad_fws_no_cr() {
            let text = "(= (+ \n (* 2 3) 4) 10)";
            let res = opt_parse_comment::<MimeParsing>(text);
            assert_eq!(res, Err(
                ParserErrorKind::IllegalCrNlSeq { pos: 6 }
                    .with_input("(= (+ \n (* 2 3) 4) 10)")
            ));
        }

        #[test]
        fn with_bad_fws_twice_in_row() {
            let res = opt_parse_comment::<MimeParsing>("(= (+ \r\n \r\n  (* 2 3) 4) 10)");
            assert_eq!(res, Err(
                ParserErrorKind::IllegalCrNlSeq { pos: 9 }
                    .with_input("(= (+ \r\n \r\n  (* 2 3) 4) 10)")
            ));
        }

        #[test]
        fn with_fws_twice_in_row_obs_grammar() {
            let text = "(= (+ \r\n \r\n  (* 2 3) 4) 10)";
            assert_eq!(
                opt_parse_comment::<MimeObsParsing>(text),
                Ok(Some(text.len()))
            );
        }

        #[test]
        fn not_a_comment() {
            let text = "  (noop)";
            let res = opt_parse_comment::<MimeParsing>(text);
            assert_eq!(res, Ok(None));
        }
    }


    mod _parse_fws_modern {
        use super::super::_parse_fws_modern;

        #[test]
        fn crlf_space() {
            let text = "\r\n ";
            assert_eq!(_parse_fws_modern(text, 0), Ok(3));
        }

        #[test]
        fn crlf_tab() {
            let text = "\r\n\t";
            assert_eq!(_parse_fws_modern(text, 0), Ok(3));
        }

        #[test]
        fn ws_then_crlf() {
            let text = "  \r\n ";
            assert_eq!(_parse_fws_modern(text, 0), Ok(5));
        }

        #[test]
        fn ws_then_crlf_then_ws() {
            let text = "  \r\n   abcde";
            assert_eq!(_parse_fws_modern(text, 0), Ok(7));
        }

        #[test]
        fn wsonly() {
            let text = "     ";
            assert_eq!(_parse_fws_modern(text, 0), Ok(5));
        }

        #[test]
        fn no_fws() {
            let text = "";
            assert_eq!(_parse_fws_modern(text, 0), Ok(0));
        }
    }

    mod parse_opt_crlf_seq {
        use super::super::parse_opt_crlf_seq;

        #[test]
        fn non_crlf() {
            let text = "abc";
            assert_eq!(parse_opt_crlf_seq(text, 0), Ok(0));
        }

        #[test]
        fn crnl_space() {
            let text = "\r\n ";
            assert_eq!(parse_opt_crlf_seq(text, 0), Ok(3));
        }

        #[test]
        fn crnl_tab() {
            let text = "\r\n\t";
            assert_eq!(parse_opt_crlf_seq(text, 0), Ok(3));
        }
    }

    mod parse_opt_ws_seq {
        use super::super::parse_opt_ws_seq;

        #[test]
        fn no_ws() {
            let text = "";
            assert_eq!(parse_opt_ws_seq(text, 0), 0)
        }

        #[test]
        fn spaces() {
            let text = "   ";
            assert_eq!(parse_opt_ws_seq(text, 0), 3)
        }

        #[test]
        fn spaces_then_more() {
            let text = "   abc";
            assert_eq!(parse_opt_ws_seq(text, 0), 3)
        }

        #[test]
        fn mixed_spaces() {
            let text = " \t\t \t";
            assert_eq!(parse_opt_ws_seq(text, 0), 5)
        }

        #[test]
        fn start_offset() {
            let text = "a \t\t \t";
            assert_eq!(parse_opt_ws_seq(text, 2), text.len())
        }
    }
}
