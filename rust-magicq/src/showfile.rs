use std::{
    fmt::{self, Display, Formatter},
    ops::{Index, IndexMut},
    str::FromStr,
};
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{tag, escaped},
    character::complete::{hex_digit1, line_ending, none_of, char, not_line_ending, alphanumeric1},
    combinator::{peek, eof, map, map_res, rest},
    multi::{many0, many1, many_till},
    sequence::{terminated, delimited, tuple},
    error::{VerboseError, context, convert_error},
    IResult, number::streaming::double, Parser, Finish,
};

static LINE_RETURN: &str = "\n";

#[derive(Debug)]
pub struct Header(String);

impl Header {
    pub fn new(value: &str) -> Header {
        Header(value.to_string())
    }

    pub fn parse(input: &str) -> IResult<&str, Header, VerboseError<&str>> {
        context(
            "Parsing Header", 
            map(
                delimited(
                    tag("\\ "),
                    not_line_ending,
                    line_ending,
                ),
                Header::new,
            ),
        )(input)
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Header(value) = self;
        write!(f, "\\ {}{}", value, LINE_RETURN)
    }
}

#[derive(Debug)]
pub enum Value {
    Float(f64),
    String(String),
    Hex(u64, usize),
}

impl Value {
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

    fn parse_float(input: &str) -> IResult<&str, (Value, bool), VerboseError<&str>> {
        context(
            "Float",
            map(
                tuple((
                    alt((
                        double,
                        map(tag("nan"), |_| f64::NAN),
                        map(tag("-nan"), |_| -f64::NAN),
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

    pub fn parse(input: &str) -> IResult<&str, (Value, bool), VerboseError<&str>> {
        context(
            "Field",
            alt((Self::parse_string, Self::parse_hex, Self::parse_float)),
        )(input)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Float(fl) => {
                // Dirty hack because MagicQ sometimes writes out both
                // nan and -nan. Please don't ask why it needs -nan.
                if fl.is_nan() {
                    write!(f, "{}", if fl.is_sign_positive() {
                        "nan"
                    } else {
                        "-nan"
                    })
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
    Settings,
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
    pub fn from_code(s: &str) -> SectionIdentifier {
        match s {
            "V" => SectionIdentifier::Version,
            "T" => SectionIdentifier::Settings,
            "P" => SectionIdentifier::Head,
            "L" => SectionIdentifier::Fixture,
            "F" => SectionIdentifier::Palette,
            "G" => SectionIdentifier::Group,
            "W" => SectionIdentifier::FX,
            "S" => SectionIdentifier::Playback,
            "C" => SectionIdentifier::CueStack,
            "M" => SectionIdentifier::ExecutePage,
            "N" => SectionIdentifier::ExecuteItem,
            //"r" => SectionIdentifier::Unknown("r"),
            //"Q" => SectionIdentifier::Unknown("Q"),
            //"R" => SectionIdentifier::Unknown("R"),
            //"Z" => SectionIdentifier::Unknown("Z"),
            //"J" => SectionIdentifier::Unknown("J"),
            //"u" => SectionIdentifier::Unknown("u"),
            //"H" => SectionIdentifier::Unknown("H"),
            //"E1" => SectionIdentifier::Unknown("E1"),
            //"Y" => SectionIdentifier::Unknown("Y"),
            _ => SectionIdentifier::Unknown(i.to_string()),
        }
    }
    
    pub fn to_code(&self) -> &str {
        match self {
            SectionIdentifier::Version => "V",
            SectionIdentifier::Settings => "T",
            SectionIdentifier::Head => "P",
            SectionIdentifier::Fixture => "L",
            SectionIdentifier::Palette => "F",
            SectionIdentifier::Group => "G",
            SectionIdentifier::FX => "W",
            SectionIdentifier::Playback => "S",
            SectionIdentifier::CueStack => "C",
            SectionIdentifier::ExecutePage => "M",
            SectionIdentifier::ExecuteItem => "N",
            //SectionIdentifier::Unknown("r") => "r",
            //SectionIdentifier::Unknown("Q") => "Q",
            //SectionIdentifier::Unknown("R") => "R",
            //SectionIdentifier::Unknown("Z") => "Z",
            //SectionIdentifier::Unknown("J") => "J",
            //SectionIdentifier::Unknown("u") => "u",
            //SectionIdentifier::Unknown("H") => "H",
            //SectionIdentifier::Unknown("E1") => "E1",
            //SectionIdentifier::Unknown("Y") => "Y",
            SectionIdentifier::Unknown(s) => s,
        }
    }

    fn parse(input: &str) -> IResult<&str, SectionIdentifier, VerboseError<&str>> {
        context(
            "Section Identifier",
            map(
                alphanumeric1,
                SectionIdentifier::from_code,
            )
        )(input)
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

    fn parse(input: &str) -> IResult<&str, Row, VerboseError<&str>> {
        context(
            "Row",
            map(
                tuple((
                    many1(Value::parse),
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

    pub fn has_trailing_comma(&self) -> bool {
        self.trailing_comma
    }

    pub fn get_trailing_newlines(&self) -> usize {
        self.trailing_newlines
    }
}

impl<'a> IntoIterator for &'a Row {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl Index<usize> for Row {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl IndexMut<usize> for Row {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new(Vec::new(), false, 0)
    }
}

impl Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let last_index = self.values.len() - 1;

        for (i, value) in self.values.iter().enumerate() {
            let has_comma = i != last_index || self.has_trailing_comma();
            write!(f, "{}{}", value, if has_comma {","} else {""})?;
        }

        write!(f, "{}", LINE_RETURN.repeat(self.get_trailing_newlines()))?;

        Ok(())
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

    pub fn parse(input: &str) -> IResult<&str, Section, VerboseError<&str>> {
        context(
            "Section",
            map(
                tuple((
                    terminated(
                        SectionIdentifier::parse, 
                        context(",", tag(",")),
                    ),
                    terminated(
                        many1(Row::parse), 
                        context(";", tag(";")),
                    ),
                    map(many0(line_ending), |v| v.len()),
                )),
                |(i, r, s)| Section::new(i, r, s),
            ),
        )(input)
    }

    pub fn get_identifier(&self) -> &SectionIdentifier {
        &self.identifier
    }

    pub fn get_trailing_newlines(&self) -> usize {
        self.trailing_newlines
    }
}

impl<'a> IntoIterator for &'a Section {
    type Item = &'a Row;
    type IntoIter = std::slice::Iter<'a, Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

impl Index<usize> for Section {
    type Output = Row;

    fn index(&self, index: usize) -> &Self::Output {
        &self.rows[index]
    }
}

impl IndexMut<usize> for Section {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.rows[index]
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},", self.get_identifier().to_code())?;

        for row in self.rows.iter() {
            write!(f, "{}", row)?;
        }

        write!(f, ";")?;
        write!(f, "{}", LINE_RETURN.repeat(self.get_trailing_newlines()))?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Showfile {
    headers: Vec<Header>,
    sections: Vec<Section>,
}

impl Showfile {
    pub fn new(headers: Vec<Header>, sections: Vec<Section>) -> Self {
        Self { headers, sections }
    }

    pub fn parse(input: &str) -> IResult<&str, Showfile, VerboseError<&str>> {
        context(
            "Showfile",
            map(
                tuple((
                    many1(Header::parse),
                    many1(line_ending),
                    many_till(Section::parse, eof),
                )),
                |(h, _, (s, _))| {
                    Showfile::new(h, s)
                },
            )
        )(input)
    }

    pub fn get_headers(&self) -> &[Header] {
        &self.headers
    }

    pub fn get_sections(&self) -> &[Section] {
        &self.sections
    }
}

impl FromStr for Showfile {
    type Err = String;

    fn from_str(input: &str) -> Result<Showfile, String> {
        let result = Self::parse(input).finish();
        match result {
            Ok((_, s)) => Ok(s),
            Err(e) => Err(convert_error(input, e)),
        }
    }
}

impl Display for Showfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for header in self.get_headers() {
            write!(f, "{}", header)?;
        }

        write!(f, "{}", LINE_RETURN)?;

        for section in self.get_sections() {
            write!(f, "{}", section)?;
        }

        Ok(())
    }
}
