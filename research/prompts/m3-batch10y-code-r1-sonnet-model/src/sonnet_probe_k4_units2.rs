//! 正推腿探针（草稿副本里的，不进仓）：与 sonnet_probe_k4_units 同一个造法，再带上单元角色，与「写行那次发布的根落盘之前断电」的盘面。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    checker_violations_on, devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    recorded_devices_reading_back_all_zero_on, record_one_publish_after_overwrites,
    units_of_r_overwritten_on, PublishRecordedAsR, RangesReadingBackAllZero,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::root_record::RootRecord;
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const UNIT_SLOT: u64 = 16384;

fn i74_text(image: &singlefs_harness::memory_pool::MemoryPool) -> String {
    let v: Vec<String> = checker_violations_on(image).into_iter().filter(|l| l.starts_with("I-7.4") || l.starts_with("I-4.8")).collect();
    if v.is_empty() { "-".into() } else { v.iter().map(|s| s.chars().take(70).collect::<String>()).collect::<Vec<_>>().join(" | ") }
}

#[test]
fn single_unit_zero_then_power_loss_before_the_row_root() {
    let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::Overwrite);
    let root = RootRecord::parse_slot(
        recorded.root_slot_write().bytes().expect("根槽写带字节"),
        &recorded.pool.parameters.filesystem_identifier,
    ).expect("R 自证");
    let role = |offset: u64| -> String {
        let slot = offset / UNIT_SLOT;
        let mut names = vec![];
        for (n, p) in [("树表", &root.tree_table), ("实例表", &root.instance_table), ("映射根", &root.mapping_root), ("分配记录树根", &root.allocation_record_tree_root)] {
            if p.locations.iter().any(|l| l.slot.0 == slot) { names.push(n); }
        }
        if names.is_empty() { "其它（数据/映射条目/树节点…）".to_string() } else { names.join("+") }
    };
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let writes = recorded.unit_writes();
    let mut offsets: Vec<u64> = writes.iter().map(|w| w.offset.0).collect();
    offsets.sort(); offsets.dedup();
    for zeroed in &offsets {
        let group: Vec<&_> = writes.iter().filter(|w| w.offset.0 == *zeroed).copied().collect();
        let ranges = RangesReadingBackAllZero::default().with_the_writes(&group);
        // M1 录自己的写
        let stream = SharedStream::retaining_contents();
        let mut devices = recorded_devices_reading_back_all_zero_on(&crash_image, &ranges, &stream);
        let mounted = match mount_writable(&recorded.pool.parameters, &mut devices) { Ok(m) => m, Err(e) => { println!("zero@{zeroed} ({}): refused {e:?}", role(*zeroed)); continue; } };
        let selected = (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg);
        let (mount_writes, _s, _i) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
        let row_root = mount_writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("写行有根槽写");
        let mut before_row_root = crash_image.clone();
        before_row_root.apply_writes(&mount_writes[..row_root]);
        let over = units_of_r_overwritten_on(&recorded, &before_row_root).len();
        let report = recover(&before_row_root, JournalPolicy::Consult);
        println!(
            "zero@{zeroed} 角色={} | M1 选 {selected:?} | 根落盘前断电：R 的单元被写掉 {over} 份；恢复择 {:?}，结局 {}; checker I-7.4/I-4.8: {}",
            role(*zeroed), report.effective_root, format!("{:?}", report.outcome).chars().take(90).collect::<String>(), i74_text(&before_row_root)
        );
        let _ = devices_reading_back_all_zero_on; // 保持 use
    }
}
