#![feature(test)]

extern crate mime;
extern crate test;

use mime::MediaType;
use mime::spec::{HttpSpec, Obs};

use test::Bencher;

//TODO check wtf. StrictSpec is SLOWER then HttpSpec<Obs> by MUTCH in the extended case??

#[bench]
fn from_str(b: &mut Bencher) {
    let s = "text/plain";
    b.bytes = s.as_bytes().len() as u64;
    b.iter(|| <MediaType<HttpSpec<Obs>>>::parse(s))
}

#[bench]
fn validate(b: &mut Bencher) {
    let s = "text/plain";
    b.bytes = s.as_bytes().len() as u64;
    b.iter(|| <MediaType<HttpSpec<Obs>>>::validate(s))
}

#[bench]
fn from_str_charset_utf8(b: &mut Bencher) {
    let s = "text/plain; charset=utf-8";
    b.bytes = s.as_bytes().len() as u64;
    b.iter(|| <MediaType<HttpSpec<Obs>>>::parse(s))
}

#[bench]
fn validate_charset_utf8(b: &mut Bencher) {
    let s = "text/plain; charset=utf-8";
    b.bytes = s.as_bytes().len() as u64;
    b.iter(|| <MediaType<HttpSpec<Obs>>>::validate(s))
}

#[bench]
fn from_str_extended(b: &mut Bencher) {
    let s = "text/plain; charset=utf-8; foo=bar";
    b.bytes = s.as_bytes().len() as u64;
    b.iter(|| <MediaType<HttpSpec<Obs>>>::parse(s))
}

#[bench]
fn validate_extended(b: &mut Bencher) {
    let s = "text/plain; charset=utf-8; foo=bar";
    b.bytes = s.as_bytes().len() as u64;
    b.iter(|| <MediaType<HttpSpec<Obs>>>::validate(s))
}
