//! # Media TYpe
//!
#![cfg_attr(feature = "inner-bench", feature(test))]


//#![deny(warnings)]
//#![deny(missing_docs)]
//#![deny(missing_debug_implementations)]

#[cfg(all(feature = "inner-bench", test))]
extern crate test;

extern crate percent_encoding;
extern crate media_type_impl_utils;
extern crate quoted_string;
extern crate lut;


pub use quoted_string::AsciiCaseInsensitiveEq;
pub use self::name::*;
pub use self::value::*;
pub use self::media_type::{MediaType, AnyMediaType, Params};

#[cfg(feature="expose-param-utils")]
pub use parse::ParamIndices;
#[cfg(feature="expose-param-utils")]
pub use gen::push_params_to_buffer;

#[macro_use]
mod macros;
pub mod error;
mod name;
mod value;
pub mod spec;
mod parse;
mod media_type;
mod gen;

mod seal {
    // trick to make implementing traits in external crates impossible
    pub trait Seal {}
}