use std::iter::{Iterator, ExactSizeIterator};
use std::slice;
use std::marker::PhantomData;
use std::ops::Deref;
use std::fmt::{self, Debug};


use nom::Err;

use name::{Name, CHARSET};
use value::{Value, UTF_8, UTF8};


use parse::{Spec, ParseResult, parse, validate};

struct ParamIndices {
    eq_idx: usize,
    end_of_value_idx: usize
}

pub struct MediaType<S: Spec> {
    inner: AnyMediaType,
    _spec: PhantomData<S>
}

impl<S> MediaType<S>
    where S: Spec
{
    pub fn parse(input: &str) -> Result<Self, Err<&str>> {
        let parse_result: ParseResult = parse::<S>(input)?;
        let media_type: AnyMediaType = parse_result.into();
        Ok(MediaType { inner: media_type, _spec: PhantomData })
    }

    pub fn validate(input: &str) -> bool {
        validate::<S>(input)
    }
}

impl<S> Deref for MediaType<S>
    where S: Spec
{
    type Target = AnyMediaType;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> Into<AnyMediaType> for MediaType<S>
    where S: Spec
{
    fn into(self) -> AnyMediaType {
        self.inner
    }
}

pub struct AnyMediaType {
    //idx layout
    //                              /plus_idx if there is no suffix, buffer.len() if there are no parameters
    //                             /
    //  type /  subtype  + suffix  ; <space>  param_name    =   param_value  ; <space> pn = pv
    //       \           \         \          \             \                \          \
    //        \slash_idx  \plus_idx \          \             \eon_idx         \ofv_idx   \prev eov_idx + 2
    //                               \eot_idx   \prev eov_idx +2 == eot_idx + 2 if first param
    buffer: String,
    slash_idx: usize,
    /// is equal the end_type_idx if there is no plus
    plus_idx: usize,
    /// it is the index behind the last character of the subtype(inkl. suffix) which is equal to the
    /// index of the ";" of the first parameter or the len of the buffer if there are no parameter
    end_of_type: usize,
    params: Vec<ParamIndices>
}


impl AnyMediaType {

    pub fn type_(&self) -> Name {
        Name::new_unchecked(&self.buffer[..self.slash_idx])
    }

    pub fn subtype(&self) -> Name {
        Name::new_unchecked(&self.buffer[self.slash_idx+1..self.plus_idx])
    }

    pub fn suffix(&self) -> Option<Name> {
        let suffix_start = self.plus_idx+1;
        let end_idx = self.end_of_type;
        if suffix_start < end_idx {
            Some(Name::new_unchecked(&self.buffer[suffix_start..end_idx]))
        } else {
            None
        }
    }

    pub fn get_param<'a, N>(&'a self, attr: N) -> Option<Value<'a>>
        where N: PartialEq<Name<'a>>
    {
        self.params()
            .find(|nv| attr == nv.0)
            .map(|(_name, value)| value)
    }

    pub fn params(&self) -> Params {
        Params {
            iter: self.params.iter(),
            source: self.buffer.as_str(),
            last_end_idx: self.end_of_type
        }
    }

    pub fn as_str_repr(&self) -> &str {
        self.buffer.as_str()
    }

    pub fn has_utf8_charset(&self) -> bool {
        self.get_param(CHARSET)
            .map(|cs_param| {
                //FIXME use eq_ascii_case_insensitive
                cs_param == UTF_8 || cs_param == UTF8
            })
            .unwrap_or(false)
    }

}

impl<'a> From<ParseResult<'a>> for AnyMediaType {

    fn from(amtr: ParseResult<'a>) -> Self {
        let slash_idx;
        let plus_idx;
        let end_of_type;
        let mut params = Vec::new();

        let mut buffer = String::with_capacity(amtr.repr_len());
        buffer.push_str(amtr.type_);
        slash_idx = buffer.len();
        buffer.push('/');
        buffer.push_str(amtr.subtype);
        end_of_type = buffer.len();
        plus_idx = amtr.subtype.bytes()
            .rposition(|b|b==b'+')
            .unwrap_or(end_of_type);
        //get suffix from it
        if amtr.charset_utf8 {
            let len = buffer.len();
            params.push(ParamIndices {
                eq_idx: len + 9,
                end_of_value_idx: len + 15,
            });
            buffer.push_str("; charset=utf-8");
        }
        for &(name, value) in amtr.params.iter() {
            buffer.push(';');
            buffer.push(' ');
            buffer.push_str(&*name.to_ascii_lowercase());
            let eq_idx = buffer.len();
            buffer.push('=');
            //we normalize somewhere else
            buffer.push_str(value);
            let end_of_value_idx = buffer.len();
            params.push(ParamIndices {
                eq_idx, end_of_value_idx
            })
        }

        AnyMediaType {
            buffer,
            slash_idx,
            plus_idx,
            end_of_type,
            params
        }
    }
}

#[derive(Clone)]
pub struct Params<'a> {
    source: &'a str,
    last_end_idx: usize,
    iter: slice::Iter<'a, ParamIndices>
}

impl<'a> Iterator for Params<'a> {
    type Item = (Name<'a>, Value<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
            .map(|pidx| {
                let name = &self.source[self.last_end_idx+2..pidx.eq_idx];
                let value = &self.source[pidx.eq_idx+1..pidx.end_of_value_idx];
                self.last_end_idx = pidx.end_of_value_idx;
                (Name::new_unchecked(name), Value::new_unchecked(value))
            })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for Params<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a> Debug for Params<'a> {

    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        let metoo = self.clone();
        fter.debug_list()
            .entries(metoo)
            .finish()
    }
}



#[cfg(test)]
mod test {
    use super::MediaType;
    use ::parse::AnySpec;

    #[test]
    fn simple_parse() {
        let mt: MediaType<_> = assert_ok!(MediaType::<AnySpec>::parse("text/plain; charset=utf-8"));
        assert!(mt.has_utf8_charset());
        assert_eq!(mt.as_str_repr(), "text/plain; charset=utf-8");
    }

    #[test]
    fn parsing_normalizes_whitespaces() {
        let mt: MediaType<_> = assert_ok!(MediaType::<AnySpec>::parse("text/plain   ;charset=utf-8"));
        assert!(mt.has_utf8_charset());
        assert_eq!(mt.as_str_repr(), "text/plain; charset=utf-8");
    }

    //FIXME this functionality might be dropped
    #[ignore]
    #[test]
    fn parsing_normalized_utf8() {
        let mt: MediaType<_> = assert_ok!(MediaType::<AnySpec>::parse("text/plain; charset=utf8"));
        assert!(mt.has_utf8_charset());
        assert_eq!(mt.as_str_repr(), "text/plain; charset=utf-8");
    }


    #[test]
    fn params_iter_behaviour() {
        let mt: MediaType<AnySpec> = assert_ok!(MediaType::parse("test/plain; c1=abc; c2=def"));
        let mut iter = mt.params();
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.size_hint(), (2, Some(2)));

        let p1 = iter.next().unwrap();
        assert_eq!(p1.0, "c1");
        assert_eq!(p1.1, "abc");
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.size_hint(), (1, Some(1)));

        let p1 = iter.next().unwrap();
        assert_eq!(p1.0, "c2");
        assert_eq!(p1.1, "def");
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.size_hint(), (0, Some(0)));

        assert_eq!(iter.next(), None);
    }
}