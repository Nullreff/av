use std::{
    env,
    fs,
    process,
    fmt::{self, Display, Formatter},
};
use nom::{
    branch::alt,
    bytes::complete::{tag, escaped},
    character::complete::{digit1, hex_digit1, line_ending, none_of, char, not_line_ending, alphanumeric1},
    combinator::{peek, eof, map, map_res},
    multi::{many0, many1, separated_list0, separated_list1, many_till},
    sequence::{terminated, delimited, tuple, self},
    error::{convert_error, VerboseError, context, ParseError},
    IResult, Finish, number::streaming::double, Parser, Offset, Slice,
    lib::std::ops::RangeTo, InputTake, Compare, InputLength, InputIter,
};

// Define the CsvValue enum
#[derive(Debug)]
enum Value {
    Float(f64),
    String(String),
    Hex(u64),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Hex(h) => write!(f, "0x{:X}", h),
        }
    }
}

#[derive(Debug)]
enum SectionIdentifier {
    Unknown(String)
}

#[derive(Debug)]
struct  Section {
    identifier: SectionIdentifier,
    data: Vec<Vec<Value>>,
    line_endings: usize,
}

#[derive(Debug)]
struct MagicQShowfile {
    headers: Vec<String>,
    sections: Vec<Section>,
}

fn parse_header(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    context(
        "Parsing Header", 
        map(
            delimited(
                tag("\\ "),
                not_line_ending,
                line_ending,
            ),
            |s: &str| s.to_string(),
        ),
    )(input)
}

fn parse_section_identifier(input: &str) -> IResult<&str, SectionIdentifier, VerboseError<&str>> {
    context(
        "Section Identifier",
        map(
            alphanumeric1,
            |s: &str| SectionIdentifier::Unknown(s.to_string()),
        )
    )(input)
}

// Define the string parser
fn parse_string(input: &str) -> IResult<&str, Value, VerboseError<&str>> {
    context(
        "String",
        map(
            terminated(
                alt((
                    delimited(
                        char('\"'),
                        escaped(none_of("\""), '\\', char('\"')),
                        char('\"'),
                    ),
                    map(tag("\"\""), |_| ""),
                )),
                alt((tag(","), peek(tag(";")), peek(line_ending))),
            ),
            |s: &str| Value::String(s.to_string()),
        )
    )(input)
}

// Define the floating-point parser
fn parse_float(input: &str) -> IResult<&str, Value, VerboseError<&str>> {
    context(
        "Float",
        map(
            terminated(
                alt((
                    double,
                    map(tag("nan"), |_| f64::NAN),
                )),
                alt((tag(","), peek(tag(";")), peek(line_ending))),
            ),
            Value::Float
        ),
    )(input)
}

// Define the hexadecimal parser
fn parse_hex(input: &str) -> IResult<&str, Value, VerboseError<&str>> {
    context(
        "Hex",
        map_res(
            terminated(
                hex_digit1,
                alt((tag(","), peek(tag(";")), peek(line_ending))),
            ),
            |parsed_hex: &str| -> Result<Value, std::num::ParseIntError> {
                u64::from_str_radix(parsed_hex, 16).map(Value::Hex)
            },
        ),
    )(input)
}

// Define the CSV field parser
fn csv_field(input: &str) -> IResult<&str, Value, VerboseError<&str>> {
    context(
        "Field",
        alt((parse_string, parse_hex, parse_float)),
    )(input)
}

// Define the CSV row parser
fn csv_row(input: &str) -> IResult<&str, Vec<Value>, VerboseError<&str>> {
    context(
        "Row",
        alt((
            terminated(
                many1(csv_field),
                alt((line_ending, peek(tag(";")))),
            ),
            map(line_ending, |_| Vec::new()),
        ))
    )(input)
}

// Define the section parser
fn section_parser(input: &str) -> IResult<&str, Section, VerboseError<&str>> {
    context(
        "Section",
        map(
            tuple((
                terminated(
                    parse_section_identifier, 
                    context(",", tag(",")),
                ),
                terminated(
                    many1(csv_row), 
                    context(";", tag(";")),
                ),
                many0(line_ending),
            )),
            |(i, d, l)| {
                Section{identifier: i, data: d, line_endings: l.len()}
            },
        )
    )(input)
}

fn showfile_parser(input: &str) -> IResult<&str, MagicQShowfile, VerboseError<&str>> {
    context(
        "Showfile",
        map(
            tuple((
                many1(parse_header),
                many1(line_ending),
                many_till(section_parser, eof),
            )),
            |(h, _, (s, _))| {
                MagicQShowfile{headers: h, sections: s}
            },
        )
    )(input)
}

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
    match result {
        Ok((rem, parsed_string)) => {
            println!("Parsed string: {:?}", parsed_string);
            println!("Remaining: {}", rem.len());
        },
        Err(e) => {
            eprintln!("Error: {}", convert_error(input.as_str(), e));
        }
    }
}