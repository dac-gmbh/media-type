extern crate mime;

use mime::push_params_to_buffer;
use mime::spec::{MimeSpec, Ascii, Modern};

#[test]
fn see_if_everything_needed_is_exposed() {
    let mut buffer = String::new();

    let res = push_params_to_buffer::<MimeSpec<Ascii, Modern>, _, _, _>(&mut buffer, vec![
        ("key", "value"),
        ("key2", "va\x01ue")
    ]).unwrap();

    let out = res.iter().map(|indices| {
        let key = &buffer[indices.start..indices.eq_idx];
        let val = &buffer[indices.eq_idx+1..indices.end];
        (key, val)
    }).collect::<Vec<_>>();

    assert_eq!(out, vec![
        ("key", "value"),
        ("key2*", "utf-8''va%01ue")
    ]);
}