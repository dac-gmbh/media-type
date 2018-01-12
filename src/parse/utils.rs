use error::{ParserErrorRef, ParserErrorKind, ExpectedChar};




#[inline]
pub fn parse_ascii_char(input: &str, pos: usize, bch: u8) -> Result<usize, ParserErrorRef> {
    debug_assert!(bch <= 0x7f, "bch should be an ascii char");
    if input.as_bytes().get(pos) != Some(&bch) {
        Err(ParserErrorKind::UnexpectedChar {
            pos, expected: ExpectedChar::Char(bch as char)
        }.with_input(input))
    } else {
        Ok(pos+1)
    }
}