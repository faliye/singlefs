use super::*;

pub(super) fn parsed(lines: &[String]) -> Vec<ResultLine> {
    lines
        .iter()
        .filter_map(|line| ResultLine::parse(line))
        .collect()
}

pub(super) fn verdict_line(lines: &[String], item: &str) -> ResultLine {
    parsed(lines)
        .into_iter()
        .find(|line| line.name() == Some("verdict") && line.field("item") == Some(item))
        .expect("有判决行")
}
