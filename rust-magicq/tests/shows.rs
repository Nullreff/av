use testsgenerator::generate_tests;
use magicq::showfile_parser;
use nom::{
    error::convert_error,
    Finish,
};

// See testsgenerator/src/lib.rs
generate_tests!("../events");
generate_tests!("../show");