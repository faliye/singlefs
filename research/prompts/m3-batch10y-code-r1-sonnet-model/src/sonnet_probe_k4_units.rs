//! 正推腿探针（草稿副本里的，不进仓）：R 的某一个单元（两盘同偏移的那一对）在取新实例的那次可写挂载 M1 里读回全零，
//! 别的单元读得出：C 的隔离是否仍护住 R 的其余单元（R 在重建时是不是候选根）。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    checker_violations_on, devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    pool_after_the_mount, record_one_publish_after_overwrites, units_of_r_overwritten_on,
    PublishRecordedAsR, RangesReadingBackAllZero, OVERWRITE_BYTES,
};
use singlefs_core::mount::mount_writable;

fn i74(image: &singlefs_harness::memory_pool::MemoryPool) -> String {
    let v: Vec<String> = checker_violations_on(image).into_iter().filter(|l| l.starts_with("I-7.4")).collect();
    if v.is_empty() { "-".into() } else { format!("RED({}) {}", v.len(), v[0].chars().take(160).collect::<String>()) }
}

#[test]
fn zeroing_one_unit_of_r_at_the_first_mount() {
    for (name, kind) in [("over", PublishRecordedAsR::Overwrite), ("seq3", PublishRecordedAsR::SequentialWriteOfThreeDataUnits)] {
        let recorded = record_one_publish_after_overwrites(5, kind);
        let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
        let writes = recorded.unit_writes();
        let mut offsets: Vec<u64> = writes.iter().map(|w| w.offset.0).collect();
        offsets.sort();
        offsets.dedup();
        println!("KIND {name}: R 写了 {} 份单元写、{} 个不同偏移：{:?}", writes.len(), offsets.len(),
            offsets.iter().map(|o| (*o, writes.iter().find(|w| w.offset.0 == *o).unwrap().length_in_bytes())).collect::<Vec<_>>());
        for zeroed in &offsets {
            let group: Vec<&_> = writes.iter().filter(|w| w.offset.0 == *zeroed).copied().collect();
            let ranges = RangesReadingBackAllZero::default().with_the_writes(&group);
            let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
            let mounted = match mount_writable(&recorded.pool.parameters, &mut devices) {
                Ok(m) => m,
                Err(e) => { println!("  zero@{zeroed}: M1 refused {:?}", format!("{e:?}").chars().take(100).collect::<String>()); continue; }
            };
            let effective = (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg);
            let image_after = image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
            let over = units_of_r_overwritten_on(&recorded, &image_after);
            let mut pool = pool_after_the_mount(&recorded, &image_after, mounted);
            let mut tail = String::new();
            for step in 1..=6u32 {
                if pool.overwrite(OVERWRITE_BYTES).is_err() { break; }
                let image = pool.image();
                tail.push_str(&format!("{step}:{}/{} ", i74(&image).chars().take(3).collect::<String>(), units_of_r_overwritten_on(&recorded, &image).len()));
            }
            println!("  zero@{zeroed}: M1 ok effective={effective:?} R_key={:?} R_units_overwritten_after_M1={:?} I74_after_M1={} then {tail}",
                recorded.key, over, i74(&image_after));
        }
    }
}
