//! 正推腿的探针（草稿目录里的副本，不入库）：截断意图那次每盘新占的槽数（分配记录树之外），在几种文件形状与目标上量。
mod common;
mod common_truncation;

use common::{build_pool, sequential_write_in_process_as_instance_one_and_advance_the_pool};
use common_truncation::{change_the_size_of_the_first_file_and_advance, payload_capacity};
use singlefs_core::transaction::TransactionUnit;

fn slots_outside_the_allocation_record_tree(units: u64, new_size_of: impl Fn(u64) -> u64, name: &str) -> (u64, Vec<String>) {
    let mut pool = build_pool(name);
    let content: Vec<u8> = (0..units * payload_capacity())
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, new_size_of(payload_capacity()));
    let outside: Vec<TransactionUnit> = intent
        .rewritten
        .iter()
        .copied()
        .filter(|role| {
            !matches!(
                role,
                TransactionUnit::AllocationTreeNodeBelowTheRoot(_) | TransactionUnit::AllocationTree
            )
        })
        .collect();
    let slots: u64 = outside.iter().map(|role| role.span_slots()).sum();
    (slots, outside.iter().map(|role| role.tag()).collect())
}

#[test]
fn probe() {
    let cases: Vec<(&str, u64, Box<dyn Fn(u64) -> u64>)> = vec![
        ("p3-mid0", 3, Box::new(|p| 100.min(p))),
        ("p150-mid143", 150, Box::new(|p| 143 * p + 9)),
        ("p150-mid149", 150, Box::new(|p| 149 * p + 9)),
        ("p150-mid0", 150, Box::new(|p| 9)),
        ("p150-to0", 150, Box::new(|_| 0)),
        ("p150-boundary144", 150, Box::new(|p| 144 * p)),
        ("p300-mid290", 300, Box::new(|p| 290 * p + 5)),
    ];
    let only = std::env::var("PROBE_CASE").unwrap_or_default();
    for (name, units, size) in cases {
        let selected = if only.is_empty() { name == "p3-mid0" || name == "p150-mid143" } else { name == only };
        if !selected { continue; }
        let (slots, tags) = slots_outside_the_allocation_record_tree(units, &size, &format!("probe-{name}"));
        println!("PROBE name={name} units={units} slots_outside_allocation_record_tree={slots} roles={tags:?}");
    }
}
