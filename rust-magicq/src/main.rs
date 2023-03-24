use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{alpha1, digit1, hex_digit1, line_ending, not_line_ending},
    combinator::map_res,
    multi::many1,
    sequence::{delimited, preceded, tuple},
    IResult,
};

// Define the integer parser
fn parse_integer(input: &str) -> IResult<&str, i64> {
    map_res(digit1, str::parse::<i64>)(input)
}

// Define the string parser
fn parse_string(input: &str) -> IResult<&str, String> {
    delimited(tag("\""), take_till(|c| c == '\"'), tag("\""))(input)
}

// Define the floating-point parser
fn parse_float(input: &str) -> IResult<&str, f64> {
    map_res(
        tuple((
            digit1,
            tag("."),
            digit1
        )),
        |(int_part, _, frac_part)| -> Result<f64, std::num::ParseFloatError> {
            format!("{}.{}", int_part, frac_part).parse::<f64>()
        },
    )(input)
}

// Define the hexadecimal parser
fn parse_hex(input: &str) -> IResult<&str, u64> {
    map_res(preceded(tag("0x"), hex_digit1), |s: &str| u64::from_str_radix(s, 16))(input)
}

// Define the CSV field parser
fn csv_field(input: &str) -> IResult<&str, String> {
    let (input, field) = alt((
        map_res(parse_integer, |i: i64| i.to_string()),
        parse_string,
        map_res(parse_float, |f: f64| f.to_string()),
        map_res(parse_hex, |h: u64| format!("0x{:X}", h)),
        take_till(|c| c == ',' || c == '\n' || c == ';'),
    ))(input)?;
    Ok((input, field))
}

// Define the CSV row parser
fn csv_row(input: &str) -> IResult<&str, Vec<String>> {
    let (input, row) = many1(delimited(tag(""), csv_field, alt((tag(","), line_ending))))(input)?;
    Ok((input, row))
}

// Define the section parser
fn section_parser(input: &str) -> IResult<&str, Vec<Vec<String>>> {
    let (input, first_field) = alpha1(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, mut section_content) = many1(csv_row)(input)?;
    let (input, _) = tag(";")(input)?;

    section_content[0].insert(0, first_field.to_string());

    Ok((input, section_content))
}

fn main() {
    let input = "A id,name,age
                 1,\"John\",0x1E
                 2,\"Jane\",28;
                 B title,year,rating
                 \"The Matrix\",1999,8.7
                 \"Inception\",2010,8.8;";

    let mut remaining_input = input;

    while let Ok((input, section_content)) = section_parser(remaining_input) {
        for record in section_content {
            println!("{:?}", record);
        }
       
    }
}