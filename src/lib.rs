//! # Mime
//!
//! Mime is now Media Type, technically, but `Mime` is more immediately
//! understandable, so the main type here is `Mime`.
//!
//! ## What is Mime?
//!
//! Example mime string: `text/plain`
//!
//! ```
//! let plain_text: mime::Mime = "text/plain".parse().unwrap();
//! assert_eq!(plain_text, mime::TEXT_PLAIN);
//! ```
//!
//! ## Inspecting Mimes
//!
//! ```
//! let mime = mime::TEXT_PLAIN;
//! match (mime.type_(), mime.subtype()) {
//!     (mime::TEXT, mime::PLAIN) => println!("plain text!"),
//!     (mime::TEXT, _) => println!("structured text"),
//!     _ => println!("not text"),
//! }
//! ```
#![doc(html_root_url = "https://docs.rs/mime/0.3.5")]
#![cfg_attr(feature = "inner-bench", feature(test))]


//#![deny(warnings)]
//#![deny(missing_docs)]
//#![deny(missing_debug_implementations)]

#[cfg(all(feature = "inner-bench", test))]
extern crate test;

extern crate media_type_parser_utils;
extern crate quoted_string;
extern crate lut;


pub use quoted_string::AsciiCaseInsensitiveEq;
pub use self::name::*;
pub use self::value::*;
pub use self::media_type::{MediaType, AnyMediaType, Params};

#[macro_use]
mod macros;
pub mod error;
mod name;
mod value;
pub mod spec;
mod parse;
mod media_type;

mod seal {
    // trick to make implementing traits in external crates impossible
    pub trait Seal {}
}


#[cfg(notyet)]
//#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn test_type_() {
        assert_eq!(TEXT_PLAIN.type_(), TEXT);
    }


    #[test]
    fn test_subtype() {
        assert_eq!(TEXT_PLAIN.subtype(), PLAIN);
        assert_eq!(TEXT_PLAIN_UTF_8.subtype(), PLAIN);
        let mime = Mime::from_str("text/html+xml").unwrap();
        assert_eq!(mime.subtype(), HTML);
    }

    #[test]
    fn test_matching() {
        match (TEXT_PLAIN.type_(), TEXT_PLAIN.subtype()) {
            (TEXT, PLAIN) => (),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_suffix() {
        assert_eq!(TEXT_PLAIN.suffix(), None);
        let mime = Mime::from_str("text/html+xml").unwrap();
        assert_eq!(mime.suffix(), Some(XML));
    }

    #[test]
    fn test_mime_fmt() {
        let mime = TEXT_PLAIN;
        assert_eq!(mime.to_string(), "text/plain");
        let mime = TEXT_PLAIN_UTF_8;
        assert_eq!(mime.to_string(), "text/plain; charset=utf-8");
    }

    #[test]
    fn test_mime_from_str() {
        assert_eq!(Mime::from_str("text/plain").unwrap(), TEXT_PLAIN);
        assert_eq!(Mime::from_str("TEXT/PLAIN").unwrap(), TEXT_PLAIN);
        assert_eq!(Mime::from_str("text/plain; charset=utf-8").unwrap(), TEXT_PLAIN_UTF_8);
        assert_eq!(Mime::from_str("text/plain;charset=\"utf-8\"").unwrap(), TEXT_PLAIN_UTF_8);

        // quotes + semi colon
        Mime::from_str("text/plain;charset=\"utf-8\"; foo=bar").unwrap();
        Mime::from_str("text/plain;charset=\"utf-8\" ; foo=bar").unwrap();

        let upper = Mime::from_str("TEXT/PLAIN").unwrap();
        assert_eq!(upper, TEXT_PLAIN);
        assert_eq!(upper.type_(), TEXT);
        assert_eq!(upper.subtype(), PLAIN);


        let extended = Mime::from_str("TEXT/PLAIN; CHARSET=UTF-8; FOO=BAR").unwrap();
        assert_eq!(extended, "text/plain; charset=utf-8; foo=BAR");
        assert_eq!(extended.get_param("charset").unwrap(), "utf-8");
        assert_eq!(extended.get_param("foo").unwrap(), "BAR");

        Mime::from_str("multipart/form-data; boundary=--------foobar").unwrap();

        // stars
        assert_eq!("*/*".parse::<Mime>().unwrap(), STAR_STAR);
        assert_eq!("image/*".parse::<Mime>().unwrap(), "image/*");
        assert_eq!("text/*; charset=utf-8".parse::<Mime>().unwrap(), "text/*; charset=utf-8");

        // parse errors
        Mime::from_str("f o o / bar").unwrap_err();
        Mime::from_str("text\n/plain").unwrap_err();
        Mime::from_str("text\r/plain").unwrap_err();
        Mime::from_str("text/\r\nplain").unwrap_err();
        Mime::from_str("text/plain;\r\ncharset=utf-8").unwrap_err();
        Mime::from_str("text/plain; charset=\r\nutf-8").unwrap_err();
        Mime::from_str("text/plain; charset=\"\r\nutf-8\"").unwrap_err();
    }

    #[test]
    fn test_case_sensitive_values() {
        let mime = Mime::from_str("multipart/form-data; charset=BASE64; boundary=ABCDEFG").unwrap();
        assert_eq!(mime.get_param(CHARSET).unwrap(), "bAsE64");
        assert_eq!(mime.get_param(BOUNDARY).unwrap(), "ABCDEFG");
        assert_ne!(mime.get_param(BOUNDARY).unwrap(), "abcdefg");
    }

    #[test]
    fn test_get_param() {
        assert_eq!(TEXT_PLAIN.get_param("charset"), None);
        assert_eq!(TEXT_PLAIN.get_param("baz"), None);

        assert_eq!(TEXT_PLAIN_UTF_8.get_param("charset"), Some(UTF_8));
        assert_eq!(TEXT_PLAIN_UTF_8.get_param("baz"), None);

        let mime = Mime::from_str("text/plain; charset=utf-8; foo=bar").unwrap();
        assert_eq!(mime.get_param(CHARSET).unwrap(), "utf-8");
        assert_eq!(mime.get_param("foo").unwrap(), "bar");
        assert_eq!(mime.get_param("baz"), None);


        let mime = Mime::from_str("text/plain;charset=\"utf-8\"").unwrap();
        assert_eq!(mime.get_param(CHARSET), Some(UTF_8));
    }

    #[test]
    fn test_mime_with_dquote_quoted_pair() {
        let mime = Mime::from_str(r#"application/x-custom; title="the \" char""#).unwrap();
        assert_eq!(mime.get_param("title").unwrap(), "the \" char");
    }

    #[test]
    fn test_params() {
        let mime = TEXT_PLAIN;
        let mut params = mime.params();
        assert_eq!(params.next(), None);

        let mime = Mime::from_str("text/plain; charset=utf-8; foo=bar").unwrap();
        let mut params = mime.params();
        assert_eq!(params.next(), Some((CHARSET, UTF_8)));

        let (second_param_left, second_param_right) = params.next().unwrap();
        assert_eq!(second_param_left, "foo");
        assert_eq!(second_param_right, "bar");

        assert_eq!(params.next(), None);
    }

    #[test]
    fn test_name_eq() {
        assert_eq!(TEXT, TEXT);
        assert_eq!(TEXT, "text");
        assert_eq!("text", TEXT);
        assert_eq!(TEXT, "TEXT");
    }

    #[test]
    fn test_value_eq() {
        let param = Value {
            source: "ABC",
        };

        assert_eq!(param, param);
        assert_eq!(param, "ABC");
        assert_eq!("ABC", param);
        assert_ne!(param, "abc");
        assert_ne!("abc", param);
    }

    #[test]
    fn test_mime_with_utf8_values() {
        let mime = Mime::from_str(r#"application/x-custom; param="Straße""#).unwrap();
        assert_eq!(mime.get_param("param").unwrap(), "Straße");
    }

    #[test]
    fn test_mime_with_multiple_plus() {
        let mime = Mime::from_str(r#"application/x-custom+bad+suffix"#).unwrap();
        assert_eq!(mime.type_(), "application");
        assert_eq!(mime.subtype(), "x-custom+bad");
        assert_eq!(mime.suffix().unwrap(), "suffix");
    }

    #[test]
    fn test_mime_param_with_empty_quoted_string() {
        let mime = Mime::from_str(r#"application/x-custom;param="""#).unwrap();
        assert_eq!(mime.get_param("param").unwrap(), "");
    }

    #[test]
    fn test_mime_param_with_tab() {
        let mime = Mime::from_str("application/x-custom;param=\"\t\"").unwrap();
        assert_eq!(mime.get_param("param").unwrap(), "\t");
    }

    #[test]
    fn test_mime_param_with_quoted_tab() {
        let mime = Mime::from_str("application/x-custom;param=\"\\\t\"").unwrap();
        assert_eq!(mime.get_param("param").unwrap(), "\t");
    }

    #[test]
    fn test_reject_tailing_half_quoted_pair() {
        let mime = Mime::from_str(r#"application/x-custom;param="\""#);
        assert!(mime.is_err());
    }

    #[test]
    fn test_parameter_eq_is_order_independent() {
        let mime_a = Mime::from_str(r#"application/x-custom; param1=a; param2=b"#).unwrap();
        let mime_b = Mime::from_str(r#"application/x-custom; param2=b; param1=a"#).unwrap();
        assert_eq!(mime_a, mime_b);
    }

    #[test]
    fn test_parameter_eq_is_order_independent_with_str() {
        let mime_a = Mime::from_str(r#"application/x-custom; param1=a; param2=b"#).unwrap();
        let mime_b = r#"application/x-custom; param2=b; param1=a"#;
        assert_eq!(mime_a, mime_b);
    }

    #[test]
    fn test_name_eq_is_case_insensitive() {
        let mime1 = Mime::from_str(r#"text/x-custom; abc=a"#).unwrap();
        let mime2 = Mime::from_str(r#"text/x-custom; aBc=a"#).unwrap();
        assert_eq!(mime1, mime2);
    }
}

