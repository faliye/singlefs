//! 正推腿的计数探针（草稿，不入库）：量「重写一个数据单元」与「整份重写 n 个单元」各发多少角色、多少条 journal 记录、换下多少落点。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use std::collections::BTreeMap;

use common::{build_pool, sequential_write_in_process_as_instance_one_and_advance_the_pool};
use common_truncation::{change_the_size_of_the_first_file_and_advance, payload_capacity};
use singlefs_core::transaction::TransactionOutput;

fn tag_histogram(version: &TransactionOutput) -> String {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for role in &version.rewritten {
        let tag = role.tag();
        let class = if tag.starts_with("t1") {
            "data".to_string()
        } else if tag.starts_with("t2下") {
            "extent_lower".to_string()
        } else if tag.starts_with("t2上") {
            "extent_upper_below_root".to_string()
        } else {
            tag.split(|c: char| c.is_ascii_digit() || c == '@' || c == '#').next().unwrap_or("").to_string() + "*"
        };
        *counts.entry(class).or_default() += 1;
    }
    counts
        .iter()
        .map(|(tag, count)| format!("{tag}x{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn line(name: &str, units: u64, version: &TransactionOutput) -> String {
    format!(
        "name={name} units={units} rewritten={} records={} released={} roles={}",
        version.rewritten.len(),
        version.earlier_records_of_this_publish.len() + 1,
        version.released.len(),
        tag_histogram(version),
    )
}

#[test]
fn print_the_cost_of_rewriting_one_data_unit_and_of_rewriting_the_whole_file() {
    let capacity = payload_capacity();
    for units in [1u64, 3, 144, 145, 288] {
        // 尾单元重写：文件最后一个单元只有 100 字节，变长到整数个净荷，重写最后那个单元（一个数据角色）。
        let mut pool = build_pool(&format!("probe-tail-{units}"));
        let content: Vec<u8> = (0..((units - 1) * capacity + 100))
            .map(|index| u8::try_from((index * 7 + 3) % 251).expect("小于 256"))
            .collect();
        let written = sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
        println!("{}", line("whole_write_first", units, &written));
        let extended = change_the_size_of_the_first_file_and_advance(&mut pool, units * capacity);
        println!("{}", line("tail_unit_rewrite", units, &extended));
        // 整份重写：同长度另一串字节，全部单元重写。
        let content_again: Vec<u8> = (0..(units * capacity))
            .map(|index| u8::try_from((index * 11 + 9) % 251).expect("小于 256"))
            .collect();
        let rewritten = sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content_again);
        println!("{}", line("whole_rewrite", units, &rewritten));
    }
}

#[test]
fn print_the_cost_of_the_truncation_intent_against_the_unlink_intent_which_appends_a_tombstone() {
    use common_deletion_in_progress::{pool_with_a_three_unit_file, publish_the_intent};
    let mut truncating = pool_with_a_three_unit_file("probe-truncation-intent");
    let truncation_intent = change_the_size_of_the_first_file_and_advance(&mut truncating, 100);
    println!("{}", line("truncation_intent_into_unit_zero", 3, &truncation_intent));
    let mut unlinking = pool_with_a_three_unit_file("probe-unlink-intent");
    let unlink_intent = publish_the_intent(&mut unlinking);
    println!("{}", line("unlink_intent_appends_a_tombstone", 3, &unlink_intent));
}
