use std::{
    env,
    fs,
    process,
    fmt::{self, Display, Formatter},
};
use nom::{
    branch::alt,
    bytes::complete::{tag, escaped, take_till},
    character::complete::{digit1, hex_digit1, line_ending, none_of, char},
    combinator::{eof, map, map_res},
    multi::{many0, many1, separated_list1, many_till},
    sequence::{delimited, tuple, self},
    error::{convert_error, VerboseError, context},
    IResult, Finish, InputTake,
};

// Define the CsvValue enum
#[derive(Debug)]
enum CsvValue {
    Integer(i64),
    Float(f64),
    String(String),
    Hex(u64),
}

impl Display for CsvValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CsvValue::Integer(i) => write!(f, "{}", i),
            CsvValue::Float(fl) => write!(f, "{}", fl),
            CsvValue::String(s) => write!(f, "\"{}\"", s),
            CsvValue::Hex(h) => write!(f, "0x{:X}", h),
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
    data: Vec<Vec<CsvValue>>,
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
                take_till(|c| c == '\n'),
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
            take_till(|c| c == ',' || c == '\n' || c == ';'),
            |s: &str| SectionIdentifier::Unknown(s.to_string()),
        )
    )(input)
}

// Define the integer parser
fn parse_integer(input: &str) -> IResult<&str, CsvValue, VerboseError<&str>> {
    context(
        "Integer",
        map_res(digit1, |s: &str| s.parse::<i64>().map(CsvValue::Integer))
    )(input)
}

// Define the string parser
fn parse_string(input: &str) -> IResult<&str, CsvValue, VerboseError<&str>> {
    context(
        "String",
        map(
            delimited(
                char('\"'),
                escaped(none_of("\""), '\\', char('\"')),
                char('\"'),
            ),
            |s: &str| CsvValue::String(s.to_string()),
        )
    )(input)
}

// Define the floating-point parser
fn parse_float(input: &str) -> IResult<&str, CsvValue, VerboseError<&str>> {
    context(
        "Float",
        map_res(
            tuple((
                digit1,
                tag("."),
                digit1
            )),
            |(int_part, _, frac_part)| -> Result<CsvValue, std::num::ParseFloatError> {
                println!("{}.{}", int_part, frac_part);
                format!("{}.{}", int_part, frac_part)
                    .parse::<f64>()
                    .map(CsvValue::Float)
            },
        )
    )(input)
}

// Define the hexadecimal parser
fn parse_hex(input: &str) -> IResult<&str, CsvValue, VerboseError<&str>> {
    context(
        "Hex",
        map_res(
            hex_digit1,
            |parsed_hex: &str| -> Result<CsvValue, std::num::ParseIntError> {
                u64::from_str_radix(parsed_hex, 16).map(CsvValue::Hex)
            },
        ),
    )(input)
}

// Define the CSV field parser
fn csv_field(input: &str) -> IResult<&str, CsvValue, VerboseError<&str>> {
    context(
        "Field",
        alt((parse_string, parse_float, parse_hex)),
    )(input)
}

// Define the CSV row parser
fn csv_row(input: &str) -> IResult<&str, Vec<CsvValue>, VerboseError<&str>> {
    context(
        "Row",
        many1(delimited(tag(""), csv_field, tag(",")))
    )(input)
}

// Define the section parser
fn section_parser(input: &str) -> IResult<&str, Section, VerboseError<&str>> {
    context(
        "Section",
        map(
            tuple((
                parse_section_identifier,
                tag(","),
                separated_list1(line_ending, csv_row),
                tag(";"),
                many0(line_ending),
            )),
            |(i, _, d, _, l)| {
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