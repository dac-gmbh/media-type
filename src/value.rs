use std::cmp::{PartialEq, Eq};
use std::fmt;
use std::borrow::Cow;

use quoted_string::{self, ContentChars, AsciiCaseInsensitiveEq};
use parse::AnySpec;

pub static UTF_8: Value = Value { source: "utf-8" };
pub static UTF8: Value = Value { source: "utf8" };


/// A parameter value section of a `Mime`.
/// 
/// Except for the `charset` parameter, parameters 
/// are compared case sensitive
#[derive(Clone, Copy, Hash)]
pub struct Value<'a> {
    source: &'a str,
}

impl<'a> Value<'a> {

    /// crates a Value from a `source` str, assuming it's a valid quoted-string/token
    ///
    /// It is seen as a _bug_ to pass in invalid input e.g. a malformed quoted-string. If
    /// it's done the result it a malformed Value, for which e.g. the `Eq` implementation
    /// is not correct as comparing it to itself might yield false.
    pub(crate) fn new_unchecked(source: &'a str) -> Value<'a> {
        Value { source }
    }


    /// Returns the underlying representation.
    ///
    /// The underlying representation differs from the content,
    /// as it can contain quotes surrounding the content and
    /// quoted-pairs, even if non of them are necessary to
    /// represent the content.
    ///
    /// For example the representation `r#""a\"\ b""#` corresponds
    /// to the content `r#""a" b"#`. Another semantically  equivalent
    /// (i.e. with the same content) representation  is `r#""a\" b""`
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mime = r#"text/plain; param="abc def""#.parse::<mime::Mime>().unwrap();
    /// let param = mime.get_param("param").unwrap();
    /// assert_eq!(param.as_str_repr(), r#""abc def""#);
    /// ```
    pub fn as_str_repr(&self) -> &'a str {
        self.source
    }

    /// Returns the content of this instance.
    ///
    /// It differs to the representation in that it will remove the
    /// quotation marks from the quoted string and will "unquote"
    /// quoted pairs.
    ///
    /// If the underlying representation is a quoted string containing
    /// quoted-pairs `Cow::Owned` is returned.
    ///
    /// If the underlying representation is a quoted-string without
    /// quoted-pairs `Cow::Borrowed` is returned as normal
    /// str slicing can be used to strip the surrounding double quoted.
    ///
    /// If the underlying representation is not a quoted-string
    /// `Cow::Borrowed` is returned, too.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::borrow::Cow;
    ///
    /// let raw_mime = r#"text/plain; p1="char is \""; p2="simple"; p3=simple2"#;
    /// let mime = raw_mime.parse::<mime::Mime>().unwrap();
    ///
    /// let param1 = mime.get_param("p1").unwrap();
    /// let expected: Cow<'static, str> = Cow::Owned(r#"char is ""#.into());
    /// assert_eq!(param1.to_content(), expected);
    ///
    /// let param2 = mime.get_param("p2").unwrap();
    /// assert_eq!(param2.to_content(), Cow::Borrowed("simple"));
    ///
    /// let param3 = mime.get_param("p3").unwrap();
    /// assert_eq!(param3.to_content(), Cow::Borrowed("simple2"));
    /// ```
    ///
    pub fn to_content(&self) -> Cow<'a, str> {
        if self.is_quoted() {
            quoted_string::to_content::<AnySpec>(self.source)
                .expect("[BUG] can not convert valid quoted string to content")
        } else {
            Cow::Borrowed(self.source)
        }
    }

    #[inline]
    pub fn is_quoted(&self) -> bool {
        self.source.bytes().next() == Some(b'"')
    }
}

impl<'a, 'b> PartialEq<Value<'b>> for Value<'a> {
    #[inline]
    fn eq(&self, other: &Value<'b>) -> bool {
        match (self.is_quoted(), other.is_quoted()) {
            (true, true) => {
                let left_content_chars = ContentChars::<AnySpec>::from_str(self.source);
                let right_content_chars = ContentChars::<AnySpec>::from_str(other.source);
                left_content_chars == right_content_chars
            }
            (true, false) => {
                let left_content_chars = ContentChars::<AnySpec>::from_str(self.source);
                left_content_chars == other.source
            }
            (false, true) => {
                let right_content_chars = ContentChars::<AnySpec>::from_str(other.source);
                right_content_chars == self.source
            }
            (false, false) => {
                self.source == other.source
            }
        }
    }
}

// Value uses ContentChars for Eq _which is only PartialEq, not Eq_ but
// it's partial eq because of the possiblility of errors in the input,
// as we know the input is valid (it's a but if it isn't) we can implement
// Eq for it
impl<'a> Eq for Value<'a> {}


impl<'a> PartialEq<str> for Value<'a> {
    fn eq(&self, other: &str) -> bool {
        if self.is_quoted() {
            let content_chars = ContentChars::<AnySpec>::from_str(self.source);
            content_chars == other
        } else {
            self.source == other
        }
    }
}

impl<'a, 'b> PartialEq<&'b str> for Value<'a> {
    #[inline]
    fn eq(&self, other: & &'b str) -> bool {
        self == *other
    }
}


impl<'a, 'b> PartialEq<Value<'b>> for &'a str {
    #[inline]
    fn eq(&self, other: &Value<'b>) -> bool {
        other == self
    }
}

impl<'a> PartialEq<Value<'a>> for str {
    #[inline]
    fn eq(&self, other: &Value<'a>) -> bool {
        other == self
    }
}

impl<'a> From<Value<'a>> for Cow<'a, str> {
    #[inline]
    fn from(value: Value<'a>) -> Self {
        value.to_content()
    }
}

impl<'a> fmt::Debug for Value<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.source, f)
    }
}

impl<'a> fmt::Display for Value<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.source, f)
    }
}


impl<'a, 'b> AsciiCaseInsensitiveEq<Value<'b>> for Value<'a> {
    fn eq_ignore_ascii_case(&self, other: &Value<'b>) -> bool {
        match (self.is_quoted(), other.is_quoted()) {
            (true, true) => {
                let left_cc = ContentChars::<AnySpec>::from_str(self.source);
                let right_cc = ContentChars::<AnySpec>::from_str(other.source);
                left_cc.eq_ignore_ascii_case(&right_cc)
            }
            (true, false) => {
                let left_cc = ContentChars::<AnySpec>::from_str(self.source);
                left_cc.eq_ignore_ascii_case(other.source)
            }
            (false, true) => {
                let right_cc = ContentChars::<AnySpec>::from_str(other.source);
                right_cc.eq_ignore_ascii_case(self.source)
            }
            (false, false) => {
                self.source.eq_ignore_ascii_case(other.source)
            }
        }
    }
}

impl<'a> AsciiCaseInsensitiveEq<str> for Value<'a> {
    fn eq_ignore_ascii_case(&self, other: &str) -> bool {
        if self.is_quoted() {
            let content_chars = ContentChars::<AnySpec>::from_str(self.source);
            content_chars.eq_ignore_ascii_case(other)
        } else {
            self.source.eq_ignore_ascii_case(other)
        }
    }
}
impl<'a> AsciiCaseInsensitiveEq<Value<'a>> for str {
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &Value<'a>) -> bool {
        other.eq_ignore_ascii_case(self)
    }
}
impl<'a, 'b> AsciiCaseInsensitiveEq<&'b str> for Value<'a> {
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &&'b str) -> bool {
        self.eq_ignore_ascii_case(*other)
    }
}
impl<'a, 'b> AsciiCaseInsensitiveEq<Value<'a>> for &'b str {
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &Value<'a>) -> bool {
        other.eq_ignore_ascii_case(*self)
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::cmp::PartialEq;
    use std::fmt::Debug;

    use AsciiCaseInsensitiveEq;
    use super::Value;

    fn bidi_eq<A: Debug+PartialEq<B>, B: Debug+PartialEq<A>>(left: A, right: B) {
        assert_eq!(left, right);
        assert_eq!(right, left);
    }

    fn bidi_eq_iac<
        A: Debug+AsciiCaseInsensitiveEq<B>,
        B: Debug+AsciiCaseInsensitiveEq<A>
    >(left: A, right: B)
    {
        assert!(left.eq_ignore_ascii_case(&right));
        assert!(right.eq_ignore_ascii_case(&left));
    }

    fn bidi_ne_iac<
        A: Debug+AsciiCaseInsensitiveEq<B>,
        B: Debug+AsciiCaseInsensitiveEq<A>
    >(left: A, right: B)
    {
        assert!(!left.eq_ignore_ascii_case(&right));
        assert!(!right.eq_ignore_ascii_case(&left));
    }

    fn bidi_ne<A: Debug+PartialEq<B>, B: Debug+PartialEq<A>>(left: A, right: B) {
        assert_ne!(left, right);
        assert_ne!(right, left);
    }

    #[test]
    fn test_value_eq_str() {
        let value = Value {
            source: "abc",
        };
        let value_quoted = Value {
            source: "\"abc\"",
        };
        let value_quoted_with_esacpes = Value {
            source: "\"a\\bc\"",
        };

        bidi_eq(value, "abc");
        bidi_ne(value, "\"abc\"");
        bidi_ne(value, "\"a\\bc\"");

        bidi_eq(value_quoted, "abc");
        bidi_ne(value_quoted, "\"abc\"");
        bidi_ne(value_quoted, "\"a\\bc\"");

        bidi_eq(value_quoted_with_esacpes, "abc");
        bidi_ne(value_quoted_with_esacpes, "\"abc\"");
        bidi_ne(value_quoted_with_esacpes, "\"a\\bc\"");


        assert_ne!(value, "aBc");
        assert_ne!(value_quoted, "aBc");
        assert_ne!(value_quoted_with_esacpes, "aBc");
    }

    #[test]
    fn test_value_eq_value() {
        let value = Value {
            source: "abc",
        };
        let value_quoted = Value {
            source: "\"abc\"",
        };
        let value_quoted_with_esacpes = Value {
            source: "\"a\\bc\"",
        };
        assert_eq!(value, value);
        assert_eq!(value_quoted, value_quoted);
        assert_eq!(value_quoted_with_esacpes, value_quoted_with_esacpes);

        bidi_eq(value, value_quoted);
        bidi_eq(value, value_quoted_with_esacpes);
        bidi_eq(value_quoted, value_quoted_with_esacpes);
    }


    #[test]
    fn as_str_repr() {
        let value = Value { source: "\"ab cd\"" };
        assert_eq!(value, "ab cd");
        assert_eq!(value.as_str_repr(), "\"ab cd\"");
    }

    #[test]
    fn to_content_not_quoted() {
        let value = Value { source: "abc" };
        assert_eq!(value.to_content(), Cow::Borrowed("abc"));
    }

    #[test]
    fn to_content_quoted_simple() {
        let value = Value { source: "\"ab cd\"" };
        assert_eq!(value.to_content(), Cow::Borrowed("ab cd"));
    }

    #[test]
    fn to_content_with_quoted_pair() {
        let value = Value { source: "\"ab\\\"cd\"" };
        assert_eq!(value, "ab\"cd");
        let expected: Cow<'static, str> = Cow::Owned("ab\"cd".into());
        assert_eq!(value.to_content(), expected);
    }

    #[test]
    fn value_eq_value_ignore_ascii_case() {
        let l = Value::new_unchecked("abc");
        let lq = Value::new_unchecked("\"abc\"");
        let c = Value::new_unchecked("ABC");
        let cq = Value::new_unchecked("\"ABC\"");

        assert!(l.eq_ignore_ascii_case(&l));
        assert!(c.eq_ignore_ascii_case(&c));
        assert!(lq.eq_ignore_ascii_case(&lq));
        assert!(cq.eq_ignore_ascii_case(&cq));
        bidi_eq_iac(l, lq);
        bidi_eq_iac(l, c);
        bidi_eq_iac(l, cq);
        bidi_eq_iac(lq, c);
        bidi_eq_iac(lq, cq);
        bidi_eq_iac(c, cq);
    }

    #[test]
    fn value_eq_str() {
        let val = Value::new_unchecked("abc");

        bidi_eq(val, "abc");
        bidi_ne(val, "\"abc\"");
        bidi_eq_iac(val, "aBc");
        bidi_ne_iac(val, "\"aBc\"");

        assert_eq!(&val, "abc");
        assert_eq!("abc", &val);
        assert!(<str as AsciiCaseInsensitiveEq<Value>>::eq_ignore_ascii_case("aBc", &val));
        assert!(<Value as AsciiCaseInsensitiveEq<str>>::eq_ignore_ascii_case(&val, "aBc"));

    }
}
