use std::str;

use nom::digit;
use nom::IResult;
use nom::IResult::*;
use nom::Err::*;
use nom::ErrorKind;
use nom::Needed;

macro_rules! unimplemented_parser {
    ($name: ident, $typ: ty) => {
        fn $name(i: &[u8]) -> nom::IResult<&[u8], $typ> {
            let error : nom::Err<&[u8], u32> = nom::Err::Code(nom::ErrorKind::Custom(0));
            let result: nom::IResult<&[u8], $typ> = nom::IResult::Error(error);
            result
        }
    };
}

macro_rules! expect_success {
    ($test: expr, $output: expr, $remaining: expr) => {{
        let result = $test;
        match result {
            IResult::Done(remaining, output) => {
                assert_eq!(remaining, $remaining.as_bytes());
                assert_eq!(output, $output);
            },

            _ => panic!("unexpected failure: {:?}", result),
        }
    }}
}

macro_rules! expect_needed {
    ($test: expr) => {{
        let result = $test;
        match result {
            IResult::Incomplete(needed) => {
                match needed {
                    Needed::Unknown => {},

                    Needed::Size(size) => {
                        panic!("expected Needed(Unknown), received Needed({})", size);
                    },
                }
            },

            IResult::Done(_, _) => panic!("unexpected success: {:?}", result),
            IResult::Error(_) => panic!("unexpected failure: {:?}", result),
        }
    }};

    ($test: expr, $expected: expr) => {{
        let result = $test;
        match result {
            IResult::Incomplete(needed) => {
                match needed {
                    Needed::Unknown => {
                        panic!("expected Needed({}), received Needed(Unknown)", $expected);
                    },

                    Needed::Size(size) => {
                        assert_eq!(size, $expected);
                    },
                }
            },

            IResult::Done(_, _) => panic!("unexpected success: {:?}", result),
            IResult::Error(_) => panic!("unexpected failure: {:?}", result),
        }
    }}
}


// <source-name> ::= <positive length number> <identifier>
fn source_name(i: &[u8]) -> IResult<&[u8], &[u8]> {
    if i.len() == 0 {
        return Incomplete(Needed::Unknown);
    }

    match digit(i) {
        Error(err) => Error(err),
        Incomplete(needed) => Incomplete(needed),
        Done(i, length_bytes) => {
            if i.len() == 0 {
                return Incomplete(Needed::Unknown);
            }

            let length_str = unsafe { str::from_utf8_unchecked(length_bytes) };
            match length_str.parse::<usize>().ok() {
                Some(length) => {
                    if i.len() < length {
                        Incomplete(Needed::Size(length - i.len()))
                    } else {
                        Done(&i[length..], &i[..length])
                    }
                }

                None => {
                    Error(Code(ErrorKind::Digit))
                }
            }
        }
    }
}

#[cfg(test)]
#[test]
fn test_source_name() {
    expect_needed!(source_name(b""));
    expect_needed!(source_name(b"1"));
    expect_success!(source_name(b"1f"), b"f", "");
    expect_success!(source_name(b"1fo"), b"f", "o");

    expect_needed!(source_name(b"2f"), 1);
    expect_success!(source_name(b"2fo"), b"fo", "");
    expect_success!(source_name(b"2foo"), b"fo", "o");
}
