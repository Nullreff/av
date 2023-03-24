use nom::{
    branch::alt,
    bytes::complete::{tag, escaped, take_till},
    character::complete::{digit1, hex_digit1, line_ending, none_of, char},
    combinator::map_res,
    multi::many1,
    sequence::{delimited, tuple},
    IResult,
};
use std::fmt::{self, Display, Formatter};

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
enum DefinitionIdentifier {
    Unknown(String)
}

fn parse_definition_identifier(input: &str) -> IResult<&str, DefinitionIdentifier> {
    println!("parse_definition_identifier: {}", input);
    let (input, parsed_string) = take_till(|c| c == ',' || c == '\n' || c == ';')(input)?;
    Ok((input, DefinitionIdentifier::Unknown(parsed_string.to_string())))
}

// Define the integer parser
fn parse_integer(input: &str) -> IResult<&str, CsvValue> {
    println!("parse_integer: {}", input);
    map_res(digit1, |s: &str| s.parse::<i64>().map(CsvValue::Integer))(input)
}

// Define the string parser
fn parse_string(input: &str) -> IResult<&str, CsvValue> {
    println!("parse_string: {}", input);
    let (input, parsed_string) = delimited(
        char('\"'),
        escaped(none_of("\""), '\\', char('\"')),
        char('\"'),
    )(input)?;

    Ok((input, CsvValue::String(parsed_string.to_string())))
}

// Define the floating-point parser
fn parse_float(input: &str) -> IResult<&str, CsvValue> {
    println!("parse_float: {}", input);
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
    )(input)
}

// Define the hexadecimal parser
fn parse_hex(input: &str) -> IResult<&str, CsvValue> {
    println!("parse_hex: {}", input);
    map_res(
        hex_digit1,
        |parsed_hex: &str| -> Result<CsvValue, std::num::ParseIntError> {
            u64::from_str_radix(parsed_hex, 16).map(CsvValue::Hex)
        },
    )(input)
}

// Define the CSV field parser
fn csv_field(input: &str) -> IResult<&str, CsvValue> {
    println!("csv_field: {}", input);
    let (input, field) = alt((parse_string, parse_float, parse_hex))(input)?;
    Ok((input, field))
}

// Define the CSV row parser
fn csv_row(input: &str) -> IResult<&str, Vec<CsvValue>> {
    println!("csv_row: {}", input);
    let (input, row) = many1(delimited(tag(""), csv_field, alt((tag(","), line_ending))))(input)?;
    Ok((input, row))
}

// Define the section parser
fn section_parser(input: &str) -> IResult<&str, Vec<Vec<CsvValue>>> {
    println!("section_parser");
    let (input, definition_identifier) = parse_definition_identifier(input)?;
    let (input, _) = tag(",")(input)?;
    println!("Content: {:?}", definition_identifier);
    let (input, section_content) = many1(csv_row)(input)?;
    println!("Content: {:?}", section_content);
    println!("Input: {}", input);
    let (input, _) = tag(";")(input)?;

    Ok((input, section_content))
}

fn main() {
    let input = "A,\"Hello world\",0001,0.05,;";

    let result = section_parser(input);
    match result {
        Ok((_, parsed_string)) => println!("Parsed string: {:?}", parsed_string),
        Err(e) => println!("Error: {:?}", e),
    }

}