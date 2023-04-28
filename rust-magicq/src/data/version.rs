// V,007d,"MagicQ 1",01090307,0000,0002,;

#[derive(Debug)]
struct Version {
    value0: u64, //TODO
    name: String,
    value1: u64, //TODO
    value2: u64, //TOOD
    value3: u64, //TOOD
}

fn get_hex(value: &Value, len: usize) -> Result<u64, String> {
    match value {
        Value::Hex(v, l) if *l == len => Ok(*v),
        Value::Hex(v, l) => Err(format!("Hex value is {} characters long instead of {}", l, len)),
        _ => Err(format!("Hex value expected, got {:?} instead", value))
    }
}

fn get_string(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(*s),
        _ => Err(format!("String value expected, got {:?} instead", value))
    }
}

impl SectionData for Version {
    const IDENTIFIER: &'static str = "V";

    fn from_section(section: &Section) -> Result<Version, String> {
        let value0 = get_hex(&section[0][0], 4)?;
        let name = get_string(&section[0][1])?;
        let value1 = get_hex(&section[0][2], 8)?;
        let value2 = get_hex(&section[0][3], 4)?;
        let value3 = get_hex(&section[0][4], 4)?;
        Ok(Version {
            value0,
            name,
            value1,
            value2,
            value3,
        })
    }

    fn to_section(&self) -> Section {
        Section::new(SectionIdentifier::Version, Vec::new(), 0)
    }
}
