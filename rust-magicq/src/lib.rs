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
use itertools::Itertools;

// Define the CsvValue enum
#[derive(Debug)]
pub enum Value {
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SectionIdentifier {
    Version,
    File,
    Head,
    Fixture,
    Palette,
    Group,
    FX,
    Playback,
    CueStack,
    ExecutePage,
    ExecuteItem,
    Unknown(String)
}

impl SectionIdentifier {
    fn from_string(s: &str) -> SectionIdentifier {
        match s {
            "V" => SectionIdentifier::Version,
            "T" => SectionIdentifier::File,
            "P" => SectionIdentifier::Head,
            "L" => SectionIdentifier::Fixture,
            "F" => SectionIdentifier::Palette,
            "G" => SectionIdentifier::Group,
            "W" => SectionIdentifier::FX,
            "S" => SectionIdentifier::Playback,
            "C" => SectionIdentifier::CueStack,
            "M" => SectionIdentifier::ExecutePage,
            "N" => SectionIdentifier::ExecuteItem,
            _ => SectionIdentifier::Unknown(s.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct Row {
    values: Vec<Value>,
}

impl Row {
    fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    fn get_values(&self) -> &[Value] {
        &self.values
    }
}

impl Default for Row {
    fn default() -> Self {
        Self { values: Vec::new() }
    }
}

#[derive(Debug)]
pub struct  Section {
    identifier: SectionIdentifier,
    rows: Vec<Row>,
    line_endings: usize,
}

impl Section {
    pub fn new(identifier: SectionIdentifier, rows: Vec<Row>, line_endings: usize) -> Self {
        Self { identifier, rows, line_endings }
    }

    pub fn get_identifier(&self) -> &SectionIdentifier {
        &self.identifier
    }

    pub fn get_rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn get_line_endings(&self) -> usize {
        self.line_endings
    }
}

#[derive(Debug)]
pub struct MagicQShowfile {
    headers: Vec<String>,
    sections: Vec<Section>,
}

impl MagicQShowfile {
    pub fn new(headers: Vec<String>, sections: Vec<Section>) -> Self {
        Self { headers, sections }
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }

    pub fn get_sections(&self) -> &[Section] {
        &self.sections
    }
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
            SectionIdentifier::from_string,
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
fn csv_row(input: &str) -> IResult<&str, Row, VerboseError<&str>> {
    context(
        "Row",
        alt((
            map(
                terminated(
                    many1(csv_field),
                    alt((line_ending, peek(tag(";")))),
                ),
                |r| Row::new(r)
            ),
            map(line_ending, |_| Row::default()),
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
                map(many0(line_ending), |v| v.len()),
            )),
            |(i, r, s)| Section::new(i, r, s),
        ),
    )(input)
}

pub fn showfile_parser(input: &str) -> IResult<&str, MagicQShowfile, VerboseError<&str>> {
    context(
        "Showfile",
        map(
            tuple((
                many1(parse_header),
                many1(line_ending),
                many_till(section_parser, eof),
            )),
            |(h, _, (s, _))| {
                MagicQShowfile::new(h, s)
            },
        )
    )(input)
}
