//! 定长记录的存放与外排序（登记 5.1 第 4 步）的单测。

use super::*;

#[test]
fn spilled_records_come_back_in_the_same_order_as_in_memory() {
    let records: Vec<ArmRecord> = (0..1_000u64)
        .map(|index| ArmRecord {
            key: Digest128(u128::from((index * 7_919) % 97)),
            ordinal: 1_000 - index,
            outcome: Digest128(u128::from(index % 3)),
            elements: u32::try_from(index % 11).expect("小于 11"),
        })
        .collect();
    let mut in_memory = ArmRecordStore::new(usize::MAX);
    let mut spilled = ArmRecordStore::new(64);
    for record in &records {
        in_memory.push(*record);
        spilled.push(*record);
    }
    assert!(spilled.runs.len() >= 15, "每 64 条溢出一段");
    let mut first = Vec::new();
    in_memory.visit_sorted(&mut |record| first.push(record));
    let mut second = Vec::new();
    spilled.visit_sorted(&mut |record| second.push(record));
    assert_eq!(first.len(), 1_000, "一条不少");
    assert_eq!(first, second, "外排序与内存排序逐条相同");
}
