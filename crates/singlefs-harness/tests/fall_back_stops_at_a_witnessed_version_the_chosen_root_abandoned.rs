//! C757（退回下界单独起作用的那一形没有用例）：F2 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根
//! （`recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`，D23（journal 的角色与格式）
//! 已定项 14「所选根走读失败、系统配置没见证它时退回见证到的那一版」）。区间的左端 `witnessed_publish <= order` 只在「见证到的那一版被所选根的
//! 实例表抛弃」这一形单独起作用：见证到的那一版 W 不在候选里，区间里别的根一条都没有时就不退；没有左端就退到 W 之下。
//!
//! 盘面（造法同 `recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` 的攻方 B 形，`tests/common_unwitnessed_publish/mod.rs`）：
//! 384 槽小盘、第一个文件之后覆盖写 5 次（P 是最后一次），再录一次覆盖写 W（一条记录；W 的根槽落了、系统配置轮换没落）。
//! 取新实例的那次挂载里 W 的根槽与单元读回全零，那次挂载落在 P、取号、写行（Row）抛弃 W，崩在 Row 的根槽之后、它那次轮换之前；
//! W 的记录读得出，Row 的写行记录接在它后面（计数器比它大一），W 的记录还在环里。
//! 之后一个系统配置槽的 journal tail 是 W 那条记录的计数器（W 那次轮换只在这一槽落了、取号那一刻这一槽读不出——这里直接改那一槽造出盘面，
//! 推的，没走那段历史），下一次读时 Row 写出的单元（实例表那一片除外）与 Row 的写行记录读回全零：Row 走读失败；
//! Row 的末条读不出，拿计数器 = c_见证 的那条（W 的末条）比，W 比 Row 旧 ⇒ Row 没见证；见证到的 W 被 Row 的实例表抛弃，
//! [W, Row) 里一条不被抛弃的根都没有 ⇒ 不退，三处照报 Row 读不出。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    image_of_devices_reading_back_all_zero, record_one_publish_after_overwrites,
    recorded_devices_reading_back_all_zero_on, PublishRecordedAsR, RangesReadingBackAllZero,
    RecordedPublish,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::mount::{mount_writable, mount_writable_with_test_only_switches, ShadowLedger};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, recover, scan_journal, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const OVERWRITES_BEFORE_W: u64 = 5;

/// 取新实例的那次挂载里 `ranges` 读回全零：交回那次挂载的全部写（按落盘次序）与写行那次发布的根槽写在其中的下标。
fn writes_of_the_mount_that_took_a_new_instance(
    recorded: &RecordedPublish,
    ranges: &RangesReadingBackAllZero,
) -> (Vec<RetainedWrite>, usize) {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(&crash_image, ranges, &stream);
    mount_writable_with_test_only_switches(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
    )
    .expect("系统配置没见证 W：取新实例的那次可写挂载做得成");
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

/// 盘 0 上两个系统配置槽里槽世代号小的那一槽（不是择系统配置时用的那一份）的 journal tail 改成 `journal_tail`、重封。
fn image_with_the_older_system_configuration_slot_of_device_zero_witnessing(
    image: &MemoryPool,
    journal_tail: u64,
) -> MemoryPool {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let slot_spacing_in_bytes =
        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
    let device_zero = image.devices.get(&DeviceIdentity(0)).expect("池里有盘 0");
    let (older_slot_offset, mut older_slot) = [0, slot_spacing_in_bytes]
        .into_iter()
        .filter_map(|offset| {
            SystemConfiguration::parse_slot(
                &device_zero.read(DeviceOffsetInBytes(offset), slot_bytes),
            )
            .ok()
            .map(|slot| (offset, slot))
        })
        .min_by_key(|(_, slot)| slot.quantities.slot_generation)
        .expect("盘 0 至少一槽自证过");
    older_slot.quantities.journal_tail = journal_tail;
    let mut witnessing_image = image.clone();
    witnessing_image.apply_writes(&[RetainedWrite {
        device: DeviceIdentity(0),
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(older_slot_offset),
        contents: WrittenContents::Bytes(older_slot.to_slot()),
    }]);
    witnessing_image
}

/// 交回 Row 之后那一刻的盘面（没改系统配置槽）、下一次读时读回全零的几段（Row 写出的单元除实例表那一片、Row 的写行记录）、
/// Row 的 (实例, txg) 与 W 那条记录的计数器。
fn image_after_a_row_publish_abandoning_w(
    recorded: &RecordedPublish,
) -> (
    MemoryPool,
    RangesReadingBackAllZero,
    (InstanceGeneration, CheckpointTxg),
    u64,
) {
    let ranges_at_the_mount = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let (mount_writes, row_root) =
        writes_of_the_mount_that_took_a_new_instance(recorded, &ranges_at_the_mount);
    let mut image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    image.apply_writes(&mount_writes[..=row_root]);
    let row = RootRecord::parse_slot(
        mount_writes[row_root].bytes().expect("根槽写带字节"),
        &recorded.pool.parameters.filesystem_identifier,
    )
    .expect("写行那次发布的根自证");
    let row_key = (row.instance, row.checkpoint_txg);
    let configuration = choose_system_configuration(&image).expect("镜像上择得到系统配置");
    let records = scan_journal(&image, &configuration);
    let last_record_counter_of = |key: (InstanceGeneration, CheckpointTxg)| {
        records
            .values()
            .find(|record| {
                (record.instance, record.checkpoint_txg) == key
                    && record.place_in_publish
                        == JournalRecordPlaceInPublish::LastRecordOfThePublish
            })
            .map(|record| record.counter)
    };
    let last_record_counter_of_w =
        last_record_counter_of(recorded.key).expect("前提：W 的记录还在环里、读得出");
    let row_record_counter = last_record_counter_of(row_key).expect("Row 的写行记录读得出");
    assert_eq!(
        row_record_counter,
        last_record_counter_of_w + 1,
        "前提：Row 的写行记录接在 W 的记录后面"
    );
    let row_unit_writes: Vec<&RetainedWrite> = mount_writes[..row_root]
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter(|write| {
            !row.instance_table.locations.iter().any(|location| {
                location.device == write.device && location.slot.to_device_offset() == write.offset
            })
        })
        .collect();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&row_unit_writes)
        .with_the_last_record_of_the_publish(&image, row_key);
    (image, ranges, row_key, last_record_counter_of_w)
}

/// 下界单独起作用的那一形：见证到的 W 被 Row 的实例表抛弃，[W, Row) 里没有可退的根——恢复照报 Row 读不出、只读挂载打不开、
/// 可写挂载在任何写之前拒，不退到 W 之下的 P。对照：系统配置照旧（c_见证 是 P 的计数器）时同一盘面退到 P。
/// 判别力：只去掉下界 `witnessed_publish <= order`（`crates/mutations.tsv` 里「C757」那一行）时恢复退到 P、这一条红。
#[test]
fn a_row_publish_whose_units_and_record_read_back_zero_does_not_fall_back_below_the_witnessed_version_its_instance_table_abandoned(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_W, PublishRecordedAsR::Overwrite);
    let (image, ranges, row_key, last_record_counter_of_w) =
        image_after_a_row_publish_abandoning_w(&recorded);

    // 对照：c_见证 是 P 的计数器，见证到的就是 P，[P, Row) 里 W 被抛弃、P 不被抛弃 ⇒ 退到 P。
    let report_without_the_witness_of_w = recover(
        &devices_reading_back_all_zero_on(&image, &ranges),
        JournalPolicy::Consult,
    );
    assert_eq!(
        report_without_the_witness_of_w.outcome,
        RecoveryOutcome::FileRead {
            root: recorded.previous_key,
            content: recorded.previous_content.clone(),
        },
        "对照：系统配置见证 P 时 Row 走读失败退到 P"
    );

    let image = image_with_the_older_system_configuration_slot_of_device_zero_witnessing(
        &image,
        last_record_counter_of_w,
    );
    let devices = devices_reading_back_all_zero_on(&image, &ranges);
    let report = recover(&devices, JournalPolicy::Consult);
    assert!(
        matches!(
            report.outcome,
            RecoveryOutcome::Failed { root: Some(root), .. } if root == row_key
        ),
        "恢复：见证到的 W 被 Row 的实例表抛弃、[W, Row) 里没有可退的根 ⇒ 落在 Row 上照报走读失败，不退到 W 之下的 P：{:?}",
        report.outcome
    );
    assert!(
        mount_read_only(&devices).is_err(),
        "只读挂载照报打不开 Row，不退到 W 之下"
    );
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(&image, &ranges);
    assert!(
        mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount).is_err(),
        "可写挂载照拒，不在 W 之下的 P 上取号"
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices_of_the_mount, recorded.pool.device_bytes)
            == image,
        "可写挂载在任何写之前拒：两块盘逐字节不变"
    );
}
