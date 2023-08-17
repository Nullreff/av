pub trait SectionData {
    const IDENTIFIER: &'static str;
    fn from_section(section: &Section) -> Result<Self, String> where Self: Sized;
    fn to_section(&self) -> Section;
}

include!("data/version.rs");