use nom::IResult;




#[inline]
pub fn crlf(input: &str) -> IResult<&str, &str> {
    tag!(input, "\r\n")
}

#[inline]
pub fn ws(input: &str) -> IResult<&str, char> {
    one_of!(input, " \t")
}

