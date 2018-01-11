#[doc(hidden)]
#[cfg(test)]
macro_rules! assert_ok {
    ($val:expr) => ({
        match $val {
            Ok( res ) => res,
            Err( err ) => panic!( "expected Ok(..) got Err({})", err)
        }
    });
    ($val:expr, $ctx:expr) => ({
        match $val {
            Ok( res ) => res,
            Err( err ) => panic!( "expected Ok(..) got Err({}) [ctx: {:?}]", err, $ctx)
        }
    });
}

/// this is used internally when parsing
///
/// # Example
///
/// ```ignore
/// let input = "__foobar__";
/// let offset = 2;
/// let fb_start = at_pos!(0 do parse_prefix | input);
/// let fb_end = at_pos!(fb_start do parse_foobar | input);
/// let end = at_pos!(fb_end do parse_suffix | input);
/// assert_eq!(end, input.len());
/// //which is the same
/// let fb_start = parse_prefix(&input[0..])? + 0;
/// let fb_end = parse_foobar(&input[fb_start..])? + fb_start;
/// let end = parse_suffix(&input[fb_end..])? + fb_end;
/// ```
macro_rules! at_pos {
    ($offset:ident do $pfn:path | $input:expr ) => {
        $pfn(&$input[$offset..])? + $offset
    }
}

//#[doc(hidden)]
//#[cfg(test)]
//macro_rules! assert_err {
//    ($val:expr) => ({
//        match $val {
//            Ok( val ) => panic!( "expected Err(..) got Ok({:?})", val),
//            Err( err ) => err,
//        }
//    });
//    ($val:expr, $ctx:expr) => ({
//        match $val {
//            Ok( val ) => panic!( "expected Err(..) got Ok({:?}) [ctx: {:?}]", val, $ctx),
//            Err( err ) => err,
//        }
//    });
//}