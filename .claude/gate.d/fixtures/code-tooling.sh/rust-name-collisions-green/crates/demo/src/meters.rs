pub struct Meter;

impl Meter {
    fn reading(&self) -> u32 {
        2
    }
}

impl std::fmt::Display for Meter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("meter")
    }
}
