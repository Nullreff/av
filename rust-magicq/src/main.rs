use std::str::FromStr;
use std::{
    env,
    fs,
    process,
};
use std::collections::HashMap;
use magicq::{Showfile, SectionIdentifier};

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

    let showfile = Showfile::from_str(&input).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        process::exit(1);
    });

    let res = showfile.get_sections().into_iter().fold(HashMap::new(), |mut acc, item| {
        *acc.entry(item.get_identifier()).or_insert(0) += 1;
        acc
    });

    // Print a list of CueStacks
    let cuestacks = showfile.get_sections().iter()
        .filter(|section| section.get_identifier() == &SectionIdentifier::CueStack)
        .map(|section| section[0][1].to_string());
    for cuestack in cuestacks {
        println!("{}", cuestack);
    }

    for counts in res {
        println!("{:?}", counts);
    }

}