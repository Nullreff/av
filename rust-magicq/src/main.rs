use std::{
    env,
    fs,
    process,
};
use nom::{
    error::convert_error,
    Finish,
};
use itertools::Itertools;
use magicq::showfile_parser;

fn main() {
     let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let input = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            process::exit(1);
        }
    };
    //let input = "A,\"Hello world\",0001,0.05,;";

    let result = showfile_parser(&input).finish();
    let showfile = match result {
        Ok((rem, parsed_string)) => parsed_string,
        Err(e) => {
            eprintln!("Error: {}", convert_error(input.as_str(), e));
            process::exit(1);
        }
    };

    let res = showfile.get_sections().into_iter().unique_by(|s| s.get_identifier().clone());
    for section in res {
        println!("{:?}", section.get_identifier());
    }

}