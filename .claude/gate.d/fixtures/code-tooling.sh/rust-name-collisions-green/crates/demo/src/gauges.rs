pub struct Gauge;

impl Gauge {
    fn reading(&self) -> u32 {
        1
    }
}

impl std::fmt::Display for Gauge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("gauge")
    }
}

#[cfg(feature = "fast")]
fn chosen_path() -> u32 {
    1
}

#[cfg(not(feature = "fast"))]
fn chosen_path() -> u32 {
    2
}
