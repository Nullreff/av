use std::{
    fmt::{self, Display, Formatter},
};
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{tag, escaped},
    character::complete::{hex_digit1, line_ending, none_of, char, not_line_ending, alphanumeric1},
    combinator::{peek, eof, map, map_res, rest},
    multi::{many0, many1, many_till},
    sequence::{terminated, delimited, tuple},
    error::{VerboseError, context},
    IResult, number::streaming::double, Parser,
};

// Define the CsvValue enum
#[derive(Debug)]
pub enum Value {
    Float(f64),
    String(String),
    Hex(u64, usize),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Float(fl) => {
                if fl.is_nan() {
                    write!(f, "nan")
                } else {
                    write!(f, "{:.6}", fl)
                }
            },
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Hex(h, w) => {
                // Dirty hack because MagicQ sometimes writes out hex values
                // in both upper case and lower case and I don't know why.
                // If this breaks add a test case and figure out what the new
                // terrible hack is to keep it happy.
                if *w == 16 { 
                    write!(f, "{:0width$X}", h, width = w)
                } else {
                    write!(f, "{:0width$x}", h, width = w)
                }
            },
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
    fn from_code(s: &str) -> SectionIdentifier {
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
    
    fn to_code(&self) -> &str {
        match self {
            SectionIdentifier::Version => "V",
            SectionIdentifier::File => "T",
            SectionIdentifier::Head => "P",
            SectionIdentifier::Fixture => "L",
            SectionIdentifier::Palette => "F",
            SectionIdentifier::Group => "G",
            SectionIdentifier::FX => "W",
            SectionIdentifier::Playback => "S",
            SectionIdentifier::CueStack => "C",
            SectionIdentifier::ExecutePage => "M",
            SectionIdentifier::ExecuteItem => "N",
            SectionIdentifier::Unknown(s) => s,
        }
    }
}

#[derive(Debug)]
pub struct Row {
    values: Vec<Value>,
    trailing_comma: bool,
    trailing_newlines: usize,
}

impl Row {
    fn new(values: Vec<Value>, trailing_comma: bool, trailing_newlines: usize) -> Self {
        Self { values, trailing_comma, trailing_newlines }
    }

    fn get_values(&self) -> &[Value] {
        &self.values
    }

    fn has_trailing_comma(&self) -> bool {
        self.trailing_comma
    }

    fn get_trailing_newlines(&self) -> usize {
        self.trailing_newlines
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new(Vec::new(), false, 0)
    }
}

#[derive(Debug)]
pub struct  Section {
    identifier: SectionIdentifier,
    rows: Vec<Row>,
    trailing_newlines: usize,
}

impl Section {
    pub fn new(identifier: SectionIdentifier, rows: Vec<Row>, trailing_newlines: usize) -> Self {
        Self { identifier, rows, trailing_newlines }
    }

    pub fn get_identifier(&self) -> &SectionIdentifier {
        &self.identifier
    }

    pub fn get_rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn get_trailing_newlines(&self) -> usize {
        self.trailing_newlines
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
            SectionIdentifier::from_code,
        )
    )(input)
}

// Define the string parser
fn parse_string(input: &str) -> IResult<&str, (Value, bool), VerboseError<&str>> {
    context(
        "String",
        map(
            tuple((
                alt((
                    delimited(
                        char('\"'),
                        escaped(none_of("\""), '\\', char('\"')),
                        char('\"'),
                    ),
                    map(tag("\"\""), |_| ""),
                )),
                alt((
                    map(tag(","), |_| true),
                    map(peek(tag(";")), |_| false),
                    map(peek(line_ending), |_| false),
                )),
            )),
            |(s, c)| (Value::String(s.to_string()), c),
        )
    )(input)
}

// Define the floating-point parser
fn parse_float(input: &str) -> IResult<&str, (Value, bool), VerboseError<&str>> {
    context(
        "Float",
        map(
            tuple((
                alt((
                    double,
                    map(tag("nan"), |_| f64::NAN),
                )),
                alt((
                    map(tag(","), |_| true),
                    map(peek(tag(";")), |_| false),
                    map(peek(line_ending), |_| false),
                )),
            )),
            |(f, c)| (Value::Float(f), c)
        ),
    )(input)
}

// Define the hexadecimal parser
fn parse_hex(input: &str) -> IResult<&str, (Value, bool), VerboseError<&str>> {
    context(
        "Hex",
        map_res(
            tuple((
                hex_digit1.and(peek(rest.map(|r: &str| input.len() - r.len()))),
                alt((
                    map(tag(","), |_| true),
                    map(peek(tag(";")), |_| false),
                    map(peek(line_ending), |_| false),
                )),
            )),
            |((h, l), c)| {
                u64::from_str_radix(h, 16).map(|v| (Value::Hex(v, l), c))
            },
        ),
    )(input)
}

// Define the CSV field parser
fn csv_field(input: &str) -> IResult<&str, (Value, bool), VerboseError<&str>> {
    context(
        "Field",
        alt((parse_string, parse_hex, parse_float, )),
    )(input)
}

// Define the CSV row parser
fn csv_row(input: &str) -> IResult<&str, Row, VerboseError<&str>> {
    context(
        "Row",
        map(
            tuple((
                many1(csv_field),
                alt((
                    map(many1(line_ending), |l| l.len()),
                    map(peek(tag(";")), |_| 0),
                )),
            )),
            |(r, n)| {
                let comma = r.last().map(|t| t.1).unwrap_or(false);
                let values = r.into_iter().map(|t| t.0).collect_vec();
                Row::new(values, comma, n)
            },
        ),
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

pub fn showfile_writer(showfile: MagicQShowfile) -> String {
    let line_return = "\r\n";
    let mut sb = String::new();

    for header in showfile.get_headers() {
        sb.push_str(format!("\\ {}{}", header, line_return).as_str());
    }

    sb.push_str(line_return);

    for section in showfile.get_sections() {
        sb.push_str(section.get_identifier().to_code());
        sb.push(',');

        for row in section.get_rows() {
            for value in row.get_values() {
                sb.push_str(format!("{}", value).as_str());
                sb.push(',');
            }

            if !row.has_trailing_comma() {
                sb.pop();
            }

            for _ in 0..row.get_trailing_newlines() {
                sb.push_str(line_return);
            }
        }

        sb.push(';');
        for _ in 0..section.get_trailing_newlines() {
            sb.push_str(line_return);
        }
    }

    sb
}