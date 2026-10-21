//! 辩方腿模型（m3-close-rootring-r3，草稿目录里的仓副本，补丁前的 `crates/`，副本核心层带着第二轮正推腿的开关补丁但开关关着）：
//! 第三轮 C751 形态「只在见证到的那一版与所选根同实例时，F2 才把与所选根同实例的根当作不被抛弃」遇到的一形——
//! 实例表被同一个实例里的清扫发布重写（`InstanceTablePlan::Rewrite`），于是同实例的根不都共用一张实例表。
//! 盘面：`pool_with_a_dead_instance`（实例 1 死了、实例 2 覆盖写到 txg 19）→ 实例 3 可写挂载，末尾清扫发布（删实例 1 的行，重写实例表）→
//! 同一次挂载里再覆盖写一版 R，崩在 R 的根槽之后、轮换之前。读的那一刻 R 的实例表两份读回全零（可再加每块盘较新的系统配置槽读回全零）。
//! 每格打今天三处的真结局，与 F2 三种取法（今天 / 同实例一律收 / 只在见证同实例时收）下统一判退（并集）的结局。
#![allow(dead_code, unused_imports, reason = "整份共用模块拷进来，只用到其中一部分")]
mod common;
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
mod common_unwitnessed_publish;
mod rootring_r3_common;
#[path = "common_images_for_the_checker_tier/instance_table_sweep_at_the_end_of_a_writable_mount.rs"]
mod scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount;

use common_root_ring_holes::{
    content_of_the_version, fixed_geometry_of_the_ring_hole_pools, parameters_of_the_ring_hole_pools,
};
use common_unwitnessed_publish::{devices_reading_back_all_zero_on, RangesReadingBackAllZero};
use rootring_r3_common::{
    core_fall_back_to, describe_three, fall_back_to, key_of,
    newer_system_configuration_slot_of_each_device, three_places, Trigger,
};
use scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::{
    pool_with_a_dead_instance, publish_an_overwrite, sweeping_at_the_end_of_the_mount_on_any_row,
    sweeping_mount_on,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::instance_table_sweep::{
    InstanceTableSweepAtTheEndOfTheMount, InstanceTableSweepOutcome,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, instance_table_chain_of_root, recover, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes;
use singlefs_harness::segments::StepKind;

fn describe_key(key: Option<(u32, u64)>) -> String {
    key.map_or_else(|| "None".to_string(), |(instance, txg)| format!("({instance},{txg})"))
}

#[test]
fn the_narrowed_same_instance_rule_when_the_sweep_rewrote_the_table_inside_the_instance() {
    let parameters = parameters_of_the_ring_hole_pools();
    let pool = pool_with_a_dead_instance(None);
    let (mut mounted, mut devices, stream) =
        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
    assert!(
        matches!(
            mounted.output.instance_table_sweep,
            InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted { .. })
        ),
        "前提：这次挂载末尾清扫发布删了行：{:?}",
        mounted.output.instance_table_sweep
    );
    let current = mounted.current.file_version().expect("这一版带文件").clone();
    let sweep_root_table = current.root.instance_table.locations.clone();
    let after_the_sweep = publish_an_overwrite(
        &mut devices,
        &mut mounted.allocator,
        &current,
        &content_of_the_version(400),
        InstanceGeneration(3),
    );
    assert_eq!(
        after_the_sweep.root.instance_table.locations, sweep_root_table,
        "前提：清扫发布之后的覆盖写照抄清扫发布那张表"
    );
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    println!("R3SWEEP mount_root_writes={}", root_indexes.len());
    let fsid = parameters.filesystem_identifier;
    let mut cells = 0u32;
    // 崩在这次挂载的最后一条根（覆盖写 R）的根槽之后、它那次轮换之前；倒数第二条是清扫发布。
    let last = *root_indexes.last().expect("有根槽写");
    let mut image = pool.image.clone();
    image.apply_writes(&writes[..=last]);
    let root_r = RootRecord::parse_slot(writes[last].bytes().expect("根槽写带字节"), &fsid).expect("R 自证");
    let root_sweep = RootRecord::parse_slot(
        writes[root_indexes[root_indexes.len() - 2]].bytes().expect("根槽写带字节"),
        &fsid,
    )
    .expect("清扫发布的根自证");
    let root_before_sweep = RootRecord::parse_slot(
        writes[root_indexes[root_indexes.len() - 3]].bytes().expect("根槽写带字节"),
        &fsid,
    )
    .expect("清扫发布前一条根自证");
    println!(
        "R3SWEEP roots before_sweep={:?} sweep={:?} r={:?} table_same(before,sweep)={} table_same(sweep,r)={}",
        key_of(&root_before_sweep),
        key_of(&root_sweep),
        key_of(&root_r),
        root_before_sweep.instance_table.locations == root_sweep.instance_table.locations,
        root_sweep.instance_table.locations == root_r.instance_table.locations
    );
    let table_ranges: Vec<_> = root_r
        .instance_table
        .locations
        .iter()
        .map(|location| (location.device, location.slot.to_device_offset().0, 32768u64))
        .collect();
    let mut with_configuration = table_ranges.clone();
    with_configuration.extend(newer_system_configuration_slot_of_each_device(&image));
    for (fault, ranges) in [
        ("table", RangesReadingBackAllZero(table_ranges)),
        ("table+cfg", RangesReadingBackAllZero(with_configuration)),
    ] {
        cells += 1;
        let reader = devices_reading_back_all_zero_on(&image, &ranges);
        let configuration = choose_system_configuration(&reader).expect("择得到系统配置");
        let chosen = choose_root(&reader, &configuration).expect("择得到根");
        let table_readable = instance_table_chain_of_root(&reader, &chosen).is_ok();
        let recovery = match recover(&reader, JournalPolicy::Consult).outcome {
            RecoveryOutcome::FileRead { root, .. } => format!("Ok({},{})", root.0 .0, root.1 .0),
            RecoveryOutcome::NoFile { root } => format!("NoFile({},{})", root.0 .0, root.1 .0),
            RecoveryOutcome::Failed { .. } => "Fail".to_string(),
        };
        let read_only = match mount_read_only(&reader) {
            Ok(mounted) => format!(
                "Ok({},{})",
                mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0
            ),
            Err(_) => "Fail".to_string(),
        };
        let mut devices_of_the_mount = devices_reading_back_all_zero_on(&image, &ranges);
        let writable = match mount_writable(&parameters, &mut devices_of_the_mount) {
            Ok(mounted) => format!(
                "Ok({},{})",
                mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0
            ),
            Err(_) => "Fail".to_string(),
        };
        let core = core_fall_back_to(&reader, &chosen);
        let by_mode: Vec<Option<_>> = (0u8..3).map(|mode| fall_back_to(&reader, &chosen, mode)).collect();
        let matches_core = core.as_ref().map(key_of) == by_mode[0].as_ref().map(key_of);
        let shares = |target: &Option<RootRecord>| {
            target.as_ref().map_or("-".to_string(), |root| {
                (root.instance_table.locations == chosen.instance_table.locations).to_string()
            })
        };
        let mut line = format!(
            "R3SWEEP fault={fault} chosen={} table_readable={table_readable} today rec={recovery} ro={read_only} rw={writable} | f2_core={} f2_exact={} f2_same_all={} f2_same_if_witness_same_instance={} replica_matches_core={matches_core} target_shares_the_chosen_table(exact,all,narrow)=({},{},{})",
            describe_key(Some(key_of(&chosen))),
            describe_key(core.as_ref().map(key_of)),
            describe_key(by_mode[0].as_ref().map(key_of)),
            describe_key(by_mode[1].as_ref().map(key_of)),
            describe_key(by_mode[2].as_ref().map(key_of)),
            shares(&by_mode[0]),
            shares(&by_mode[1]),
            shares(&by_mode[2]),
        );
        for (mode, label) in [(0u8, "exact"), (1, "same_all"), (2, "narrow")] {
            line.push_str(&format!(
                " | union/{label}: {}",
                describe_three(&three_places(&reader, &chosen, Trigger::Union, mode))
            ));
        }
        println!("{line}");
        assert!(matches_core, "照抄的判法关着开关时要与 core 相同");
    }
    println!("R3SWEEP_SUMMARY cells={cells}");
}
