use testsgenerator::generate_tests;
use std::str::FromStr;
use magicq::Showfile;
use similar_asserts::assert_eq;

// See testsgenerator/src/lib.rs
generate_tests!("../events");
generate_tests!("../show");