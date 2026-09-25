//! E142 第十六次跑第一段（重跑登记第五节 P3）：独立比对二进制。
//!
//! 不 `use` 模型 `e142_first_transaction_dry_run.rs` 的任何一项——两个 bin 本来互相引不到，这里连着手写
//! 一份自己的行解析、十六进制解码与连续不同字节切分，好让「模型自己算出的 equal / 不同字节段」有一条独立的路
//! 重算一遍核对，不是同一段代码在骗自己。
//!
//! 用法：`e142-region-diff-independent <模型产物路径> <crates 导出路径>`，从两份文本里各自抓
//! `name=device_region_bytes ... device=... offset=... length=... hexadecimal=...` 这一族行，按
//! (设备, 偏移, 长度) 配对（不按 `region=` 配对——R3 的教训：两块设备在同一偏移各写不同内容时按名字配对分不清）。

use std::collections::BTreeMap;
use std::env;
use std::fs;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedRegionLine {
    region: Option<String>,
    device: u32,
    offset: u64,
    length: u64,
    hexadecimal: Vec<u8>,
}

fn parse_result_line(line: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for token in line.split_whitespace() {
        if let Some((key, value)) = token.split_once('=') {
            fields.insert(key.to_string(), value.to_string());
        }
    }
    fields
}

fn hex_decode(hexadecimal: &str) -> Vec<u8> {
    (0..hexadecimal.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hexadecimal[index..index + 2], 16).expect("十六进制字符成对出现"))
        .collect()
}

fn parse_device_region_bytes_lines(text: &str) -> Vec<ParsedRegionLine> {
    text.lines()
        .filter(|line| line.contains("name=device_region_bytes "))
        .filter_map(|line| {
            let fields = parse_result_line(line);
            let hexadecimal = fields.get("hexadecimal")?;
            Some(ParsedRegionLine {
                region: fields.get("region").cloned(),
                device: fields.get("device")?.parse().ok()?,
                offset: fields.get("offset")?.parse().ok()?,
                length: fields.get("length")?.parse().ok()?,
                hexadecimal: hex_decode(hexadecimal),
            })
        })
        .collect()
}

/// 把两段字节里不同的位置切成最长的连续段，返回每段 (区域内偏移, 长度)；长度不等时缺的那一半按「不同」算。
fn diff_segments(left: &[u8], right: &[u8]) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let longer_length = left.len().max(right.len());
    let mut position = 0;
    while position < longer_length {
        if left.get(position) == right.get(position) {
            let segment_start = position;
            while position < longer_length && left.get(position) != right.get(position) {
                position += 1;
            }
            segments.push((segment_start, position - segment_start));
        } else {
            position += 1;
        }
    }
    segments
}

#[cfg(test)]
fn find_matching_line(haystack: &[ParsedRegionLine], device: u32, offset: u64, length: u64) -> Option<&ParsedRegionLine> {
    haystack.iter().find(|line| line.device == device && line.offset == offset && line.length == length)
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    assert!(arguments.len() >= 3, "用法：e142-region-diff-independent <模型产物路径> <crates 导出路径>");
    let model_text = fs::read_to_string(&arguments[1]).unwrap_or_else(|error| panic!("读不到模型产物 {}：{error}", arguments[1]));
    let crates_text = fs::read_to_string(&arguments[2]).unwrap_or_else(|error| panic!("读不到 crates 导出 {}：{error}", arguments[2]));
    let model_lines = parse_device_region_bytes_lines(&model_text);
    let crates_lines = parse_device_region_bytes_lines(&crates_text);
    let mut crates_matched = vec![false; crates_lines.len()];
    let mut equal_count = 0u64;
    let mut unequal_count = 0u64;

    for model_line in &model_lines {
        let region = model_line.region.as_deref().unwrap_or("unknown");
        let Some(matched_index) = crates_lines.iter().position(|line| line.device == model_line.device && line.offset == model_line.offset && line.length == model_line.length) else {
            println!(
                "E7RESULT name=independent_region_diff region={region} device={} offset={} length={} matched=false",
                model_line.device, model_line.offset, model_line.length
            );
            continue;
        };
        crates_matched[matched_index] = true;
        let crates_line = &crates_lines[matched_index];
        let segments = diff_segments(&model_line.hexadecimal, &crates_line.hexadecimal);
        let equal = segments.is_empty() && model_line.hexadecimal.len() == crates_line.hexadecimal.len();
        if equal {
            equal_count += 1;
        } else {
            unequal_count += 1;
        }
        let mismatch_bytes: usize = segments.iter().map(|(_, length)| *length).sum();
        let segments_text = if segments.is_empty() {
            "none".to_string()
        } else {
            segments.iter().map(|(offset_in_region, length)| format!("{offset_in_region}:{length}")).collect::<Vec<_>>().join(",")
        };
        println!(
            "E7RESULT name=independent_region_diff region={region} device={} offset={} length={} matched=true equal={equal} mismatch_bytes={mismatch_bytes} segments={segments_text}",
            model_line.device, model_line.offset, model_line.length
        );
    }
    for (index, matched) in crates_matched.iter().enumerate() {
        if !matched {
            let line = &crates_lines[index];
            println!("E7RESULT name=independent_region_unmatched side=crates device={} offset={} length={}", line.device, line.offset, line.length);
        }
    }
    println!(
        "E7RESULT name=independent_region_diff_summary model_regions={} crates_regions={} equal={equal_count} unequal={unequal_count}",
        model_lines.len(),
        crates_lines.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_line(region: &str, device: u32, offset: u64, hexadecimal: &str) -> String {
        format!("E7RESULT name=device_region_bytes region={region} device={device} offset={offset} length={} sha256=ignored hexadecimal={hexadecimal}", hexadecimal.len() / 2)
    }

    /// P3 单测一：导出比自己 ⇒ 0 处不同。
    #[test]
    fn comparing_a_dump_against_itself_reports_zero_differences() {
        let text = format!("{}\n{}\n", sample_line("region_a", 0, 100, "aabbcc"), sample_line("region_b", 1, 200, "112233"));
        let model_lines = parse_device_region_bytes_lines(&text);
        let crates_lines = parse_device_region_bytes_lines(&text);
        for model_line in &model_lines {
            let matched = find_matching_line(&crates_lines, model_line.device, model_line.offset, model_line.length).expect("配得上");
            assert_eq!(diff_segments(&model_line.hexadecimal, &matched.hexadecimal), Vec::new(), "自己比自己 0 处不同");
        }
    }

    /// P3 单测二：改一个十六进制字符 ⇒ 恰报那一个区域、那一段。
    #[test]
    fn flipping_one_hex_character_reports_exactly_that_region_and_segment() {
        let model_text = sample_line("region_a", 0, 100, "aabbcc");
        let crates_text = sample_line("region_a", 0, 100, "aab0cc"); // 第二字节 bb → b0
        let model_lines = parse_device_region_bytes_lines(&model_text);
        let crates_lines = parse_device_region_bytes_lines(&crates_text);
        let matched = find_matching_line(&crates_lines, model_lines[0].device, model_lines[0].offset, model_lines[0].length).expect("配得上");
        let segments = diff_segments(&model_lines[0].hexadecimal, &matched.hexadecimal);
        assert_eq!(segments, vec![(1, 1)], "恰一段：区域内偏移 1、长度 1");
    }

    /// P3 单测三：删一行 ⇒ 报那个区域配不上。
    #[test]
    fn deleting_a_line_reports_that_region_as_unmatched() {
        let model_text = format!("{}\n{}\n", sample_line("region_a", 0, 100, "aabbcc"), sample_line("region_b", 0, 200, "112233"));
        let crates_text = sample_line("region_a", 0, 100, "aabbcc"); // 删掉 region_b 那一行
        let model_lines = parse_device_region_bytes_lines(&model_text);
        let crates_lines = parse_device_region_bytes_lines(&crates_text);
        let unmatched: Vec<&ParsedRegionLine> = model_lines
            .iter()
            .filter(|line| find_matching_line(&crates_lines, line.device, line.offset, line.length).is_none())
            .collect();
        assert_eq!(unmatched.len(), 1, "只有 region_b 配不上");
        assert_eq!(unmatched[0].region.as_deref(), Some("region_b"));
    }
}
