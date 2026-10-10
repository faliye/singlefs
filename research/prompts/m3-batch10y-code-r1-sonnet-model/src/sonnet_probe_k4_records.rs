//! 正推腿探针（草稿副本里的，不进仓）：R（顺序写三个数据单元，三条记录）的某几条记录、某个单元在取新实例的那次可写挂载里读回全零：
//! 挂载做成还是拒、拒的是不是该拒（本该放行的可写挂载被拒）。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    checker_violations_on, devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    record_one_publish_after_overwrites, units_of_r_overwritten_on, PublishRecordedAsR, RangesReadingBackAllZero,
};
use singlefs_core::mount::mount_writable;

fn i74(image: &singlefs_harness::memory_pool::MemoryPool) -> String {
    let v: Vec<String> = checker_violations_on(image).into_iter().filter(|l| l.starts_with("I-7.4")).collect();
    if v.is_empty() { "-".into() } else { "RED".into() }
}

#[test]
fn records_and_units_of_r_reading_back_zero() {
    let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::SequentialWriteOfThreeDataUnits);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let records = recorded.record_count();
    println!("R 一共 {records} 条记录");
    let unit_writes = recorded.unit_writes();
    let mut offsets: Vec<u64> = unit_writes.iter().map(|w| w.offset.0).collect();
    offsets.sort(); offsets.dedup();
    // 各种子集：每条记录单独、全部记录、记录 i + 一个数据单元（前三个偏移是 32 KiB 的数据单元）
    let mut cases: Vec<(String, RangesReadingBackAllZero)> = vec![];
    for i in 1..=records {
        cases.push((format!("记录 {i}"), RangesReadingBackAllZero::default().with_the_writes(&recorded.record_writes_of_the_record(i))));
    }
    let mut all_records = RangesReadingBackAllZero::default();
    for i in 1..=records { all_records = all_records.with_the_writes(&recorded.record_writes_of_the_record(i)); }
    cases.push(("全部记录".into(), all_records.clone()));
    for i in 1..=records {
        let mut r = RangesReadingBackAllZero::default().with_the_writes(&recorded.record_writes_of_the_record(i));
        let data: Vec<&_> = unit_writes.iter().filter(|w| w.offset.0 == offsets[0]).copied().collect();
        r = r.with_the_writes(&data);
        cases.push((format!("记录 {i} + 第一个数据单元"), r));
    }
    for (name, ranges) in cases {
        let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
        match mount_writable(&recorded.pool.parameters, &mut devices) {
            Ok(m) => {
                let image_after = image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
                println!("{name}: M1 ok 选 {:?}，R 的单元被写掉 {} 份，I-7.4 {}", (m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0), units_of_r_overwritten_on(&recorded, &image_after).len(), i74(&image_after));
            }
            Err(e) => println!("{name}: M1 拒 {}", format!("{e:?}").chars().take(160).collect::<String>()),
        }
    }
}
