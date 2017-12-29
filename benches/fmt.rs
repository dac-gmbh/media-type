#![feature(test)]

extern crate mime;
extern crate test;

use mime::MediaType;
use mime::spec::{HttpSpec, Obs};

use test::Bencher;

//TODO implement Display for Media Type
#[ignore]
#[bench]
fn bench_fmt(_b: &mut Bencher) {
//    use std::fmt::Write;
//    let mime = MediaType::<HttpSpec<Obs>>::parse("text/plain; charset=utf-8").unwrap();
//    b.bytes = mime.to_string().as_bytes().len() as u64;
//    let mut s = String::with_capacity(64);
//    b.iter(|| {
//        let _ = write!(s, "{}", mime);
//        ::test::black_box(&s);
//        unsafe { s.as_mut_vec().set_len(0); }
//    })
}
