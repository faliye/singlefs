//! 攻方模型（m3-close-rootring-r1，草稿目录里的仓副本）：C751 用户选的支「所选根的实例表读不出、而退到的那一版之上还有读得出、没法按实例表判
//! 抛弃与否的根时，只读挂载不退、照报所选那一版打不开」在两份盘面上判得怎样：C751 登记行那一形（覆盖写 R，崩在 R 的根槽之后、轮换之前；读的那一刻
//! R 的单元、每块盘较新的系统配置槽、R 的根指着的实例表读回全零）与入库用例 `:656` 那一形。两种读法：字面（表读不出时 [W, 所选根) 里每条读得出的根都
//! 「没法判」）与同实例可判（与所选根同一个实例代号的根表里不会有它的行，判得出不被抛弃）。下面的造法照抄入库用例那份文件里的几个函数。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    record_one_publish_after_overwrites, recorded_devices_reading_back_all_zero_on,
    PublishRecordedAsR, RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_test_only_switches, ShadowLedger};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots, recover, scan_journal, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 取新实例的那次挂载里 `ranges` 读回全零、影子账按 `shadow_ledger`：交回那次挂载的全部写（按落盘次序）与写行那次发布的根槽写在其中的下标。
fn writes_of_the_mount_that_took_a_new_instance(
    recorded: &RecordedPublish,
    ranges: &RangesReadingBackAllZero,
    shadow_ledger: ShadowLedger,
) -> (Vec<RetainedWrite>, usize) {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(&crash_image, ranges, &stream);
    mount_writable_with_test_only_switches(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        shadow_ledger,
    )
    .expect("系统配置没见证 R：取新实例的那次可写挂载做得成");
    let (mount_writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let row_root = mount_writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("那次挂载写行那次发布有根槽写");
    (mount_writes, row_root)
}
/// 每块盘上较新的那个系统配置槽（两槽里自证过、槽世代号大的那一槽）：盘、偏移、长度。
fn newer_system_configuration_slot_of_each_device(
    image: &MemoryPool,
) -> Vec<(DeviceIdentity, u64, u64)> {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let slot_spacing_in_bytes =
        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
    image
        .devices
        .iter()
        .map(|(device, sparse_device)| {
            let newer_slot_offset = [0, slot_spacing_in_bytes]
                .into_iter()
                .filter_map(|offset| {
                    SystemConfiguration::parse_slot(
                        &sparse_device.read(DeviceOffsetInBytes(offset), slot_bytes),
                    )
                    .ok()
                    .map(|slot| (slot.quantities.slot_generation, offset))
                })
                .max()
                .map(|(_, offset)| offset)
                .expect("每块盘至少一槽自证过");
            (*device, newer_slot_offset, SYSTEM_CONFIGURATION_SLOT_BYTES)
        })
        .collect()
}
/// 攻方 B 形那一盘（取新实例的那次挂载 M1 里 `ur`，M1 落在 P、取号，崩在写行那次发布 Row 的根槽之后、轮换之前），下一次读时只让 Row 写出的
/// 单元读回全零（`keep_the_instance_table_of_row` 为真时留下 Row 的实例表那一片照读），P 的末条读得出：Row 走读失败、没见证，见证到的是 P
/// （c_见证 是 M1 取号那一写写进去的 P 的计数器）。[P, Row) 里有 P 与 M1 按实例表抛弃了的 R，R 更新。交回那一刻的盘面与读回全零的几段。
fn image_after_a_row_publish_abandoning_r_with_the_units_of_row_reading_back_zero(
    recorded: &RecordedPublish,
    keep_the_instance_table_of_row: bool,
) -> (
    MemoryPool,
    RangesReadingBackAllZero,
    (InstanceGeneration, CheckpointTxg),
) {
    let ranges_at_the_mount = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let (mount_writes, row_root) = writes_of_the_mount_that_took_a_new_instance(
        recorded,
        &ranges_at_the_mount,
        ShadowLedger::On,
    );
    let mut image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    image.apply_writes(&mount_writes[..=row_root]);
    let row = RootRecord::parse_slot(
        mount_writes[row_root].bytes().expect("根槽写带字节"),
        &recorded.pool.parameters.filesystem_identifier,
    )
    .expect("写行那次发布的根自证");
    let row_unit_writes: Vec<&RetainedWrite> = mount_writes[..row_root]
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter(|write| {
            !keep_the_instance_table_of_row
                || !row.instance_table.locations.iter().any(|location| {
                    location.device == write.device
                        && location.slot.to_device_offset() == write.offset
                })
        })
        .collect();
    (
        image,
        RangesReadingBackAllZero::default().with_the_writes(&row_unit_writes),
        (row.instance, row.checkpoint_txg),
    )
}

fn key(root: &RootRecord) -> (u32, u64) {
    (root.instance.0, root.checkpoint_txg.0)
}

/// 一份盘面：今天三处的结局、F2 今天退到哪一条，[退到的那一条, 所选根) 之上读得出的根与它们的实例代号，两种读法的条件真不真。
fn judge(label: &str, image: &MemoryPool, ranges: &RangesReadingBackAllZero) {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let configuration = choose_system_configuration(&devices).expect("择得到系统配置");
    let chosen = choose_root(&devices, &configuration).expect("择得到根");
    let recovery = match recover(&devices, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, .. } => format!("Ok({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::NoFile { root } => format!("NoFile({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { .. } => "Fail".to_string(),
    };
    let read_only = match mount_read_only(&devices) {
        Ok(mounted) => format!("Ok({},{})", mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0),
        Err(_) => "Fail".to_string(),
    };
    let records = scan_journal(&devices, &configuration);
    let fallen_back_to =
        singlefs_core::recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
            &devices, &configuration, &chosen, &chosen, &records,
        );
    let table_readable =
        singlefs_core::recovery::instance_table_chain_of_root(&devices, &chosen).is_ok();
    let above: Vec<RootRecord> = match fallen_back_to {
        Some(fallen) => readable_roots(
            &devices,
            &configuration.immutable.region_devices,
            &configuration.immutable.sizes,
            &configuration.immutable.filesystem_identifier,
        )
        .into_iter()
        .filter(|root| key(root) != key(&chosen))
        .filter(|root| {
            (root.checkpoint_txg, root.instance) > (fallen.checkpoint_txg, fallen.instance)
                && (root.checkpoint_txg, root.instance) < (chosen.checkpoint_txg, chosen.instance)
        })
        .collect(),
        None => Vec::new(),
    };
    let literal = !table_readable && !above.is_empty();
    let same_instance_decidable = !table_readable
        && above.iter().any(|root| root.instance != chosen.instance);
    let newest_same_instance_root_above = above
        .iter()
        .filter(|root| root.instance == chosen.instance)
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .map(key);
    println!(
        "C751 {label} chosen={:?} table_readable={table_readable} today rec={recovery} ro={read_only} f2_fallback={:?} readable_roots_strictly_between={:?} | literal_branch_ro_refuses={literal} same_instance_reading_ro_refuses={same_instance_decidable} newest_same_instance_root_between={:?}",
        key(&chosen),
        fallen_back_to.as_ref().map(key),
        above.iter().map(key).collect::<Vec<_>>(),
        newest_same_instance_root_above
    );
}

#[test]
fn the_registered_c751_shape_and_the_pinned_656_shape() {
    for overwrites in [3u64, 5] {
        for (kind_label, kind) in [
            ("ow", PublishRecordedAsR::Overwrite),
            ("sw3", PublishRecordedAsR::SequentialWriteOfThreeDataUnits),
        ] {
            let recorded = record_one_publish_after_overwrites(overwrites, kind);
            let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let mut ranges =
                RangesReadingBackAllZero(newer_system_configuration_slot_of_each_device(&crash_image))
                    .with_the_writes(&recorded.unit_writes());
            for location in recorded.root().instance_table.locations {
                ranges.0.push((location.device, location.slot.to_device_offset().0, 32768));
            }
            judge(&format!("registered_c751 kind={kind_label} k={overwrites}"), &crash_image, &ranges);
            if kind == PublishRecordedAsR::Overwrite {
                let (image, ranges, _row_key) =
                    image_after_a_row_publish_abandoning_r_with_the_units_of_row_reading_back_zero(
                        &recorded, false,
                    );
                judge(&format!("pinned_656 kind={kind_label} k={overwrites}"), &image, &ranges);
            }
        }
    }
}
