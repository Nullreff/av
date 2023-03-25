use testsgenerator::generate_tests;
use magicq::{showfile_parser, showfile_writer};
use nom::{error::convert_error, Finish};
use pretty_assertions::assert_eq;

// See testsgenerator/src/lib.rs
generate_tests!("../events");
generate_tests!("../show");