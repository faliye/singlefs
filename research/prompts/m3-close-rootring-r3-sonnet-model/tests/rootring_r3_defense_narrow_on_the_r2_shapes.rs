//! 辩方腿（m3-close-rootring-r3）：第二轮攻方 same.out 那一族盘面原样重造，F2 取法多一档 narrow（第三轮 C751 形态）。原件头注如下。
//! 原头注：攻方模型（m3-close-rootring-r2，云端攻方腿，草稿目录里的仓副本，攻补丁前的 `crates/`）：C751 这一轮的形态「所选根的实例表读不出时，
//! F2 把与所选根同实例的根当作不被抛弃（照收 [见证到的那一版, 所选根) 里读得出的同实例最新一条）」。
//! 盘面三族：
//! - `cross`：R 整次落盘（已确认）之后一次干净的可写挂载（新实例：写行一次、暖机两次），崩在这次挂载第 j 条根的根槽之后、它那次轮换之前；
//!   读的那一刻所选根的实例表读回全零（`fault=table`），或再加每块盘较新的系统配置槽读回全零（`fault=table+cfg`，与 C751 登记那一形同一个故障预算）。
//! - `registered`：C751 登记那一形（照第一轮攻方模型的造法）。
//! - `pinned656`：入库用例 `:656` 那一形（照第一轮攻方模型的造法）。
//! 每格打今天三处的真结局、core 的 F2 交回哪一条、照抄判法关开关与开开关各交回哪一条，与三处在 C755 统一判退（走读、并集）下的结局。
mod common_unwitnessed_publish;
mod rootring_r3_common;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    record_one_publish_after_overwrites, recorded_devices_reading_back_all_zero_on,
    PublishRecordedAsR, RangesReadingBackAllZero, RecordedPublish,
};
use rootring_r3_common::{
    core_fall_back_to, describe_three, fall_back_to, key_of,
    newer_system_configuration_slot_of_each_device, three_places, Trigger,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable, mount_writable_with_test_only_switches, ShadowLedger};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 在 `image` 上做一次可写挂载（`ranges` 那几段读回全零），录下这次挂载的全部写（按落盘次序）。
fn writes_of_a_writable_mount(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> Vec<RetainedWrite> {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, ranges, &stream);
    mount_writable_with_test_only_switches(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
    )
    .expect("这一次可写挂载做得成");
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    writes
}

fn root_write_indexes(writes: &[RetainedWrite]) -> Vec<usize> {
    writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect()
}

fn instance_table_ranges_of(root: &RootRecord) -> Vec<(singlefs_core::address::DeviceIdentity, u64, u64)> {
    root.instance_table
        .locations
        .iter()
        .map(|location| (location.device, location.slot.to_device_offset().0, 32768))
        .collect()
}

fn describe_key(key: Option<(u32, u64)>) -> String {
    key.map_or_else(|| "None".to_string(), |(instance, txg)| format!("({instance},{txg})"))
}

/// 一格：今天三处的真结局、F2 三份交回、统一判退的几种结局，打一行 `R3NARROW`。交回照抄的判法关开关时与 core 是否相同。
fn judge(label: &str, recorded: &RecordedPublish, image: &MemoryPool, ranges: &RangesReadingBackAllZero) -> bool {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let configuration = choose_system_configuration(&devices).expect("择得到系统配置");
    let chosen = choose_root(&devices, &configuration).expect("择得到根");
    let table_readable =
        singlefs_core::recovery::instance_table_chain_of_root(&devices, &chosen).is_ok();
    let recovery = match recover(&devices, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, .. } => format!("Ok({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::NoFile { root } => format!("NoFile({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { .. } => "Fail".to_string(),
    };
    let read_only = match mount_read_only(&devices) {
        Ok(mounted) => format!("Ok({},{})", mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0),
        Err(_) => "Fail".to_string(),
    };
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
    let writable = match mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount) {
        Ok(mounted) => format!(
            "Ok({},{})",
            mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0
        ),
        Err(_) => "Fail".to_string(),
    };
    let core = core_fall_back_to(&devices, &chosen);
    let replica_off = fall_back_to(&devices, &chosen, 0);
    let replica_on = fall_back_to(&devices, &chosen, 1);
    let replica_narrow = fall_back_to(&devices, &chosen, 2);
    let replica_matches_core = core.as_ref().map(key_of) == replica_off.as_ref().map(key_of);
    let shares_table = |target: &Option<RootRecord>| {
        target.as_ref().map_or("-".to_string(), |root| {
            (root.instance_table.locations == chosen.instance_table.locations).to_string()
        })
    };
    let mut line = format!(
        "R3NARROW {label} chosen={} table_readable={table_readable} today rec={recovery} ro={read_only} rw={writable} | f2_core={} f2_replica_off={} f2_same_instance={} f2_narrow={} replica_matches_core={replica_matches_core} target_shares_the_chosen_table(off,on,narrow)=({},{},{})",
        describe_key(Some(key_of(&chosen))),
        describe_key(core.as_ref().map(key_of)),
        describe_key(replica_off.as_ref().map(key_of)),
        describe_key(replica_on.as_ref().map(key_of)),
        describe_key(replica_narrow.as_ref().map(key_of)),
        shares_table(&replica_off),
        shares_table(&replica_on),
        shares_table(&replica_narrow),
    );
    for (trigger, name) in [(Trigger::OwnReader, "own"), (Trigger::Walk, "walk"), (Trigger::Union, "union")] {
        for (kept, rule) in [(0u8, "exact"), (1u8, "same"), (2u8, "narrow")] {
            line.push_str(&format!(
                " | {name}/{rule}: {}",
                describe_three(&three_places(&devices, &chosen, trigger, kept))
            ));
        }
    }
    println!("{line}");
    replica_matches_core
}

/// 第一轮攻方模型 `:656` 那一形的造法：崩在 R 的根槽之后、轮换之前；取新实例的那次挂载里 R 的根槽与 R 的单元读回全零（落在 P、取号），
/// 崩在写行那次发布 Row 的根槽之后、轮换之前；下一次读时 Row 写出的单元读回全零。
fn pinned_656_image(recorded: &RecordedPublish) -> (MemoryPool, RangesReadingBackAllZero) {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let at_the_mount = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let writes = writes_of_a_writable_mount(recorded, &crash_image, &at_the_mount);
    let row_root = root_write_indexes(&writes)[0];
    let mut image = crash_image;
    image.apply_writes(&writes[..=row_root]);
    let row_units: Vec<&RetainedWrite> = writes[..row_root]
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .collect();
    (image, RangesReadingBackAllZero::default().with_the_writes(&row_units))
}

#[test]
fn the_same_instance_rule_on_the_cross_instance_witness_and_on_the_two_shapes_of_the_first_round() {
    let mut cells = 0u32;
    let mut replica_mismatches = 0u32;
    for (kind_label, kind) in [
        ("ow", PublishRecordedAsR::Overwrite),
        ("sw3", PublishRecordedAsR::SequentialWriteOfThreeDataUnits),
    ] {
        for overwrites in [3u64, 5] {
            let recorded = record_one_publish_after_overwrites(overwrites, kind);
            let fsid = recorded.pool.parameters.filesystem_identifier;
            let confirmed = recorded.image_with_the_whole_publish_persisted();
            let writes = writes_of_a_writable_mount(&recorded, &confirmed, &RangesReadingBackAllZero::default());
            for (ordinal, root_index) in root_write_indexes(&writes).into_iter().enumerate() {
                let mut image = confirmed.clone();
                image.apply_writes(&writes[..=root_index]);
                let root_of_the_crash = RootRecord::parse_slot(
                    writes[root_index].bytes().expect("根槽写带字节"),
                    &fsid,
                )
                .expect("这次挂载写的根自证");
                let table_only = RangesReadingBackAllZero(instance_table_ranges_of(&root_of_the_crash));
                let mut with_configuration = table_only.0.clone();
                with_configuration.extend(newer_system_configuration_slot_of_each_device(&image));
                for (fault, ranges) in [
                    ("table", table_only),
                    ("table+cfg", RangesReadingBackAllZero(with_configuration)),
                ] {
                    let label = format!(
                        "shape=cross kind={kind_label} k={overwrites} crash_after_mount_root#{} fault={fault}",
                        ordinal + 1
                    );
                    cells += 1;
                    replica_mismatches += u32::from(!judge(&label, &recorded, &image, &ranges));
                }
            }
            let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let mut registered = newer_system_configuration_slot_of_each_device(&crash_image);
            registered.extend(instance_table_ranges_of(&recorded.root()));
            let registered = RangesReadingBackAllZero(registered).with_the_writes(&recorded.unit_writes());
            cells += 1;
            replica_mismatches += u32::from(!judge(
                &format!("shape=registered kind={kind_label} k={overwrites}"),
                &recorded,
                &crash_image,
                &registered,
            ));
            if kind == PublishRecordedAsR::Overwrite {
                let (image, ranges) = pinned_656_image(&recorded);
                cells += 1;
                replica_mismatches += u32::from(!judge(
                    &format!("shape=pinned656 kind={kind_label} k={overwrites}"),
                    &recorded,
                    &image,
                    &ranges,
                ));
            }
        }
    }
    println!("R3NARROW_SUMMARY cells={cells} replica_mismatches={replica_mismatches}");
    assert_eq!(replica_mismatches, 0, "照抄的 F2 判法关着开关时要与 core 逐格相同");
}
