//! F2 乙（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 7 条）：所选那一版走读失败、系统配置没见证它时，
//! 退回系统配置见证到的那一版；恢复（`recovery::recover`）、可写挂载读阶段之后建上一版那一步、只读挂载（`mounted_read::mount_read_only`）三处同一份判法
//! （`recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`）。「见证过」照攻方 `strict`：所选那一版带末条标志的记录读得出 ⇒
//! 计数器 ≤ c_见证；读不出 ⇒ 拿计数器 = c_见证 的那条记录比 (txg, 实例)；两条都读不出当见证过、不退。退到 [见证到的那一版, 所选根) 里读得出、
//! 不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H4 那一行与第三节第 1 条），
//! 所选根的实例表读不出时只认与见证到的那一版恰好相等的那条；不退到比见证到的那一版更旧的根（`F2_EXACT`，判决
//! `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与第三节第 1 条）；计数器 = c_见证 的那条记录读不出、或那一段里没有读得出的根，都不退。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R（P 是它之前那一版，已确认）。
//! - 5.2 两形：崩在 R 的根槽落盘之后、系统配置轮换之前；取新实例的那次挂载里 R 的根槽与单元读回全零（`ur`）或连 R 的记录也读回全零
//!   （`urJ`），那次挂载写行那次发布的根落盘之前断电，R 的单元已被那次挂载写掉；断电之后三处都读回 P。`ur` 那一形今天的 C 隔离得住 R 的单元，
//!   这里关着影子账（只供测试的开关 `ShadowLedger::Off`，连 C 一起关）造出 R 的单元被写掉的盘面；`urJ` 那一形 C 本来就看不见。
//! - 5.3 两形（R 已确认：整次发布连系统配置轮换都落了）：S2——R 的末条（唯一一条）记录与单元读回全零，两条都读不出、当见证过、不退，照报读不出；
//!   S1——较新的那个系统配置槽（R 那次轮换写的）与 R 的单元读回全零，c_见证 落回 P，R 算没见证、读回 P：接受的代价。
//! - 代码轮攻方（`research/prompts/m3-batch33t-code-r1-opus-output.md` 第一节）两形：A——崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元与 P 的根槽
//!   读回全零（见证到的就是 P，P 的根读不出）；B——取新实例的那次挂载里 `ur`、崩在那次挂载写行那次发布的根槽之后，下一次读时写行那次发布的单元
//!   与 P 的末条记录（计数器 = c_见证 的那一条）读回全零。两形都照报读不出，不退到 P 之前那一版、也不退到被新实例抛弃的 R。
//! - 代码轮攻方（`research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节）H4：崩在 R 的根槽之后、轮换之前，每块盘 P 那次轮换写的那一槽没写成
//!   （写回再前一次轮换的内容、两槽都自证过；两块盘都丢一次已确认的写，不是崩溃态），读的那一刻 R 的单元读回全零，c_见证 落回 P 之前那一版；三处都落到已确认的 P，四组（两种 R × 覆盖写 3、5 次）各一条。B 形那一盘上 P 的末条读得出时
//!   退到的根按 Row 的实例表滤（跳过被抛弃的 R），Row 的实例表读不出时只认恰好相等的 P。
//! - c_见证 = 0（D23 已定项 14「c_见证 = 0 … 都不退」）：设计里那一退的底上每个系统配置槽的 tail 改 0、环里伪造一条计数器 0 的 P 的记录，
//!   三处都不退。
//! - 代码轮攻方（`research/prompts/m3-batch37-43-code-r1-opus-output.md` 第二节）Y4：R 的单元、每块盘较新的系统配置槽与 P 的根槽读回全零，c_见证 落回 Pm，[Pm, R) 里
//!   读得出的只有 Pm；Pm 与 R 之间的 P 末条读得出、根读不出、不被 R 的实例表抛弃，退到 Pm 就越过它——三处照报 R 读不出。
mod common_unwitnessed_publish;

use std::cell::Cell;
use std::rc::Rc;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    image_of_devices_reading_back_all_zero, record_one_publish_after_overwrites,
    recorded_devices_reading_back_all_zero_on, root_is_in_the_ring,
    units_of_the_recorded_publish_overwritten_on, DeviceReadingBackAllZeroOnRanges,
    RangesReadingBackAllZero, RecordedPublish, RecordedPublishKind, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches, MountError, ShadowLedger,
    StillUnreadableAfterOneReread,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, recover, scan_journal, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::unit::{parse_data_unit, unit_filesystem_identifier};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
    WrittenContents,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::SharedStream;

const OVERWRITES_BEFORE_THE_RECORDED_PUBLISH: u64 = 5;

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

/// 取新实例的那次挂载里 `ranges` 读回全零、影子账按 `shadow_ledger`，写行那次发布的根落盘之前断电：交回断电那一刻的盘面。
fn image_of_a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance(
    recorded: &RecordedPublish,
    ranges: &RangesReadingBackAllZero,
    shadow_ledger: ShadowLedger,
) -> MemoryPool {
    let (mount_writes, row_root) =
        writes_of_the_mount_that_took_a_new_instance(recorded, ranges, shadow_ledger);
    let mut image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    image.apply_writes(&mount_writes[..row_root]);
    assert_ne!(
        units_of_the_recorded_publish_overwritten_on(recorded, &image),
        Vec::<(DeviceIdentity, u64)>::new(),
        "写行那次发布的根落盘之前，那次挂载已写掉 R 的单元（这一形的前提：恢复落到 R 上走读失败）"
    );
    image
}

/// 三处都读回 P：`image` 上 `ranges` 读回全零时，恢复读回 P 的内容、落到 P；只读挂载择到并打开 P；可写挂载建在 P 上。
fn assert_recovery_and_both_mounts_read_back_the_previous_version(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: recorded.previous_key,
            content: recorded.previous_content.clone(),
        },
        "恢复：所选那一版 R 走读失败、系统配置没见证它，退回见证到的 P、读回 P 的内容（F2 之前：Failed(UnitUnreadable)）"
    );
    assert_eq!(report.effective_root, Some(recorded.previous_key));
    let read_only = mount_read_only(&devices).expect("只读挂载退回 P、打得开");
    assert_eq!(
        (
            read_only.effective_root.instance,
            read_only.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "只读挂载：退回 P"
    );
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount)
        .expect("可写挂载退回 P、建得出上一版");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "可写挂载：建在 P 上"
    );
}

/// 三处都照报读不出：`image` 上 `ranges` 读回全零时，恢复落到 `chosen` 上报走读失败、只读挂载打不开、可写挂载在任何写之前拒（两块盘逐字节不变）。
fn assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    chosen: (InstanceGeneration, CheckpointTxg),
) {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let report = recover(&devices, JournalPolicy::Consult);
    assert!(
        matches!(
            report.outcome,
            RecoveryOutcome::Failed { root: Some(root), .. } if root == chosen
        ),
        "恢复落到所选的 {chosen:?}、照报走读失败，不悄悄读回别的一版：{:?}",
        report.outcome
    );
    assert!(
        mount_read_only(&devices).is_err(),
        "只读挂载照报打不开 {chosen:?}，不退到别的一版"
    );
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
    assert!(
        mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount).is_err(),
        "可写挂载照拒，不在别的一版上取号"
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices_of_the_mount, recorded.pool.device_bytes)
            == *image,
        "可写挂载在任何写之前拒：两块盘逐字节不变"
    );
}

/// `ur`：R 的记录读得出（末条，计数器大于 c_见证）。判别力：「末条读得出就当见证过」（变异「F2 末条那一支恒见证过」）时三处都不退、这一条红。
#[test]
fn a_root_whose_units_were_overwritten_while_its_record_reads_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let image = image_of_a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance(
        &recorded,
        &ranges,
        ShadowLedger::Off,
    );
    assert_recovery_and_both_mounts_read_back_the_previous_version(
        &recorded,
        &image,
        &RangesReadingBackAllZero::default(),
    );
}

/// `urJ`：R 的记录那次也读回全零，取新实例的那次挂载把它的记录槽写成自己的写行记录（计数器接在 P 后面）：R 的末条读不出，
/// 计数器 = c_见证 的那条是 P 的，(txg, 实例) 比 R 旧 ⇒ 没见证。判别力：「拿 c_见证 那条比」那一支恒见证过
/// （变异「F2 c_见证 那一支恒见证过」）时三处都不退、这一条红。
#[test]
fn a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&recorded.record_writes_of_the_record(1));
    let image = image_of_a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance(
        &recorded,
        &ranges,
        ShadowLedger::On,
    );
    assert_recovery_and_both_mounts_read_back_the_previous_version(
        &recorded,
        &image,
        &RangesReadingBackAllZero::default(),
    );
}

/// 设计里那一退（代码轮攻方 `Ru`）：崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元读回全零：R 走读失败、没见证，见证到的 P 根与单元都好，
/// 三处都退到 P：`F2_EXACT` 不把设计那一退收掉。判别力：末条那一支恒见证过（变异「F2 末条那一支恒见证过（设计那一退）」）时三处都不退、这一条红。
#[test]
fn a_root_whose_units_read_back_zero_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
    assert_recovery_and_both_mounts_read_back_the_previous_version(
        &recorded,
        &crash_image,
        &ranges,
    );
}

/// 代码轮攻方 A 形：崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元与见证到的 P 的根槽读回全零。P 的记录与单元都好，只是这一刻 P 的根读不出：
/// 只退到见证到的那一版本身，P 的根读不出就不退——三处照报 R 读不出，不退过已确认的 P、落到 P 之前那一版（F2_EXACT 之前三处都落到它，
/// 可写挂载还在它上面取号、把 P 抛弃）。判别力：退回「比所选根旧、不超过见证上限的最新可读根」（变异「F2 退到见证上限之下最新的可读根」）时
/// 三处都读回 P 之前那一版、这一条红。
#[test]
fn a_root_whose_units_and_the_witnessed_versions_root_slot_read_back_zero_is_reported_unreadable_in_recovery_and_both_mounts(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded.unit_writes())
        .with_the_root_slot_of(&crash_image, recorded.previous_key);
    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
        &recorded,
        &crash_image,
        &ranges,
        recorded.key,
    );
}

/// 代码轮攻方 B 形：取新实例的那次挂载里 R 的根槽与单元读回全零（`ur`），那次挂载落在 P、取号，崩在写行那次发布（Row）的根槽落盘之后、
/// 它那次系统配置轮换之前；下一次读时 Row 写出的单元与 P 的末条记录（计数器 = c_见证 的那一条）读回全零。Row 走读失败、没见证，
/// 见证到的那条记录读不出就不退——三处照报 Row 读不出，不退到那次挂载已按实例表抛弃了的 R（F2_EXACT 之前上限退成所选根、落到 R）。
/// 判别力：那条记录读不出时上限取所选根自己（变异「F2 见证记录读不出时上限取所选根」）时三处都读回 R、这一条红。
#[test]
fn a_row_publish_whose_units_and_the_witnessed_last_record_read_back_zero_is_reported_unreadable_instead_of_falling_back_to_the_abandoned_root(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let ranges_at_the_mount = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let (mount_writes, row_root) = writes_of_the_mount_that_took_a_new_instance(
        &recorded,
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
    let row_key = (row.instance, row.checkpoint_txg);
    assert!(
        row_key.0 > recorded.key.0 && row_key.1 > recorded.key.1,
        "写行那次发布是新实例的、txg 在 R 之后：{row_key:?}"
    );
    let row_unit_writes: Vec<&RetainedWrite> = mount_writes[..row_root]
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .collect();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&row_unit_writes)
        .with_the_last_record_of_the_publish(&image, recorded.previous_key);
    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
        &recorded, &image, &ranges, row_key,
    );
}

/// 5.3 的 S2：R 已确认，R 的记录（末条，也是计数器 = c_见证 的那一条）与单元读回全零：两条都读不出，当见证过、不退——恢复照报读不出、
/// 只读挂载照报打不开、可写挂载照拒，盘上逐字节不变。判别力：两条都读不出当没见证（变异「F2 两条都读不出就退」，攻方 `loose`）时恢复读回 P、这一条红。
#[test]
fn a_confirmed_root_whose_last_record_and_units_read_back_zero_is_not_replaced_by_the_previous_version(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let confirmed = recorded.image_with_the_whole_publish_persisted();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&recorded.record_writes_of_the_record(1));
    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
        &recorded,
        &confirmed,
        &ranges,
        recorded.key,
    );
}

/// 5.3 的 S1：R 已确认，较新的那个系统配置槽（R 那次轮换写的，每块盘一槽）与 R 的单元读回全零：c_见证 落回 P 的那一次，R 的末条读得出、
/// 计数器大于它，算没见证，恢复与只读挂载读回 P——接受的代价（F2 乙「三处都做」：已确认的版本在较新系统配置槽与
/// 它的单元同时读回全零时悄悄读回上一版）。这一条钉住这个结局；条款改了它跟着改。判别力：同 `ur` 那一条的变异（末条那一支恒见证过）时照报读不出、这一条红。
#[test]
fn a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let confirmed = recorded.image_with_the_whole_publish_persisted();
    let rotation_of_the_recorded_publish: Vec<_> = recorded
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        rotation_of_the_recorded_publish.len(),
        2,
        "R 那次发布的系统配置轮换每块盘写一槽"
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&rotation_of_the_recorded_publish);
    let devices = devices_reading_back_all_zero_on(&confirmed, &ranges);
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: recorded.previous_key,
            content: recorded.previous_content.clone(),
        },
        "恢复读回 P（R 已确认、只是这一次读不出）：用户 2026-10-07 定案接受的代价"
    );
    let read_only = mount_read_only(&devices).expect("只读挂载退回 P、打得开");
    assert_eq!(
        (
            read_only.effective_root.instance,
            read_only.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "只读挂载同样读回 P"
    );
}

/// 伪造的那条记录写进环里哪个槽：环里读得出的最大计数器之后第几条记录的槽（离开 R 与它之前写过的槽，仍在环里）。
const RECORD_SLOTS_PAST_THE_HIGHEST_COUNTER_FOR_THE_FORGED_RECORD: u64 = 8;

/// `image` 上每块盘两个系统配置槽里自证过的那几份 journal tail 改成 0、重封（c_见证 恒为 0）；再把 `key` 那次发布带末条标志的记录照抄一份，
/// 计数器改成 0、重封，每块盘写进环里一个空着的记录槽（写者的计数器从 1 起，这条只能是伪造的）。
fn image_with_every_journal_tail_zero_and_a_counter_zero_copy_of_the_last_record_of(
    image: &MemoryPool,
    key: (InstanceGeneration, CheckpointTxg),
) -> MemoryPool {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let slot_spacing_in_bytes =
        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
    let mut writes: Vec<RetainedWrite> = Vec::new();
    for (device, sparse_device) in &image.devices {
        for slot_offset in [0, slot_spacing_in_bytes] {
            if let Ok(mut slot) = SystemConfiguration::parse_slot(
                &sparse_device.read(DeviceOffsetInBytes(slot_offset), slot_bytes),
            ) {
                slot.quantities.journal_tail = 0;
                writes.push(RetainedWrite {
                    device: *device,
                    kind: StepKind::SystemConfigurationSlot,
                    is_force_unit_access: false,
                    offset: DeviceOffsetInBytes(slot_offset),
                    contents: WrittenContents::Bytes(slot.to_slot()),
                });
            }
        }
    }
    let records = scan_journal(image, &configuration);
    let mut forged_record = records
        .values()
        .find(|record| {
            (record.instance, record.checkpoint_txg) == key
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .expect("环里读得出那次发布带末条标志的记录")
        .clone();
    forged_record.counter = 0;
    let highest_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .expect("环里有记录");
    let forged_record_offset = record_offset(
        highest_counter + RECORD_SLOTS_PAST_THE_HIGHEST_COUNTER_FOR_THE_FORGED_RECORD,
        configuration.immutable.sizes.journal_ring_bytes,
    );
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("记录 4096 字节");
    let filesystem_identifier =
        unit_filesystem_identifier(&configuration.immutable.filesystem_identifier);
    for (device, sparse_device) in &image.devices {
        assert!(
            JournalRecord::parse(
                &sparse_device.read(forged_record_offset, record_bytes),
                filesystem_identifier,
            )
            .is_none(),
            "伪造的记录写进的那一槽原来没有读得出的记录"
        );
        writes.push(RetainedWrite {
            device: *device,
            kind: StepKind::JournalRecord,
            is_force_unit_access: false,
            offset: forged_record_offset,
            contents: WrittenContents::Bytes(forged_record.to_bytes()),
        });
    }
    let mut forged_image = image.clone();
    forged_image.apply_writes(&writes);
    forged_image
}

/// c_见证 = 0 不退（D23（journal 的角色与格式） 已定项 14「那条记录读不出、c_见证 = 0、那条根读不出都不退」）：设计里那一退的底上（R 的根槽落了、
/// 轮换没落，读的那一刻 R 的单元读回全零），每个系统配置槽的 journal tail 改成 0，环里另有一条 P 那次发布末条记录的副本、计数器改成 0。
/// 计数器 = c_见证 = 0 的那条记录读得出、属于 P，但 c_见证 = 0 不拿它当见证到的那一版：三处都不退、留在所选的 R 上照报读不出，可写挂载在任何写之前拒。
/// 判别力：去掉 c_见证 = 0 的守卫（变异「F2 c_见证 = 0 不另设守卫（计数器 0 的伪造记录上退到 P）」）时三处都退到 P、这一条红。
#[test]
fn a_witnessed_counter_of_zero_does_not_fall_back_even_with_a_record_carrying_counter_zero_in_recovery_and_both_mounts(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let image = image_with_every_journal_tail_zero_and_a_counter_zero_copy_of_the_last_record_of(
        &crash_image,
        recorded.previous_key,
    );
    let configuration = choose_system_configuration(&image).expect("改过的镜像上择得到系统配置");
    assert_eq!(
        configuration.quantities.journal_tail, 0,
        "择到的系统配置 journal tail 是 0"
    );
    assert!(
        scan_journal(&image, &configuration)
            .values()
            .any(|record| record.counter == 0
                && (record.instance, record.checkpoint_txg) == recorded.previous_key),
        "环里读得出一条属于 P、计数器 0 的记录"
    );
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
        &recorded,
        &image,
        &ranges,
        recorded.key,
    );
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

/// 攻方 H4（`research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节）：崩在 R 的根槽之后、轮换之前（R 没见证；P 已确认），每块盘上 P 那次轮换写的
/// 那一槽没写成（写回再前一次轮换的内容，两槽都自证过，乙入判不出不拒；两块盘都丢一次已确认的写，不是崩溃态），读的那一刻 R 的单元读回全零。c_见证 落回 P 之前那一版 Pm 的计数器，
/// R 走读失败、没见证 ⇒ 退到 [Pm, R) 里读得出、不被 R 的实例表抛弃的最新一条根，就是已确认、单元完好的 P（`F2_NEWEST`），三处都落到 P。
/// 判别力：只退到见证到的那一版本身（变异「F2_NEWEST 只退到见证到的那一版本身（H4：落到 Pm、跳过已确认的 P）」）时三处都落到 Pm、这一条红。
fn assert_the_units_of_the_recorded_publish_reading_back_zero_where_both_devices_lost_their_newest_rotation_write_fall_back_to_the_previous_version(
    publish: RecordedPublishKind,
    overwrites_before_the_recorded_publish: u64,
) {
    let recorded =
        record_one_publish_after_overwrites(overwrites_before_the_recorded_publish, publish);
    let crash_image =
        common_unwitnessed_publish::image_where_both_devices_lost_their_newest_rotation_write(
            &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
        );
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
    assert_recovery_and_both_mounts_read_back_the_previous_version(
        &recorded,
        &crash_image,
        &ranges,
    );
}

/// H4 那一组：一条记录的覆盖写 R、R 之前覆盖写 3 次。
#[test]
fn root_whose_units_read_back_zero_where_both_devices_lost_their_newest_rotation_write_falls_back_to_the_confirmed_previous_version_after_three_overwrites(
) {
    assert_the_units_of_the_recorded_publish_reading_back_zero_where_both_devices_lost_their_newest_rotation_write_fall_back_to_the_previous_version(
        RecordedPublishKind::Overwrite,
        3,
    );
}

/// H4 那一组：一条记录的覆盖写 R、R 之前覆盖写 5 次。
#[test]
fn root_whose_units_read_back_zero_where_both_devices_lost_their_newest_rotation_write_falls_back_to_the_confirmed_previous_version_after_five_overwrites(
) {
    assert_the_units_of_the_recorded_publish_reading_back_zero_where_both_devices_lost_their_newest_rotation_write_fall_back_to_the_previous_version(
        RecordedPublishKind::Overwrite,
        5,
    );
}

/// H4 那一组：三条记录的顺序写 R、R 之前覆盖写 3 次。
#[test]
fn root_of_three_records_whose_units_read_back_zero_where_both_devices_lost_their_newest_rotation_write_falls_back_to_the_confirmed_previous_version_after_three_overwrites(
) {
    assert_the_units_of_the_recorded_publish_reading_back_zero_where_both_devices_lost_their_newest_rotation_write_fall_back_to_the_previous_version(
        RecordedPublishKind::SequentialWriteOfThreeDataUnits,
        3,
    );
}

/// H4 那一组：三条记录的顺序写 R、R 之前覆盖写 5 次。
#[test]
fn root_of_three_records_whose_units_read_back_zero_where_both_devices_lost_their_newest_rotation_write_falls_back_to_the_confirmed_previous_version_after_five_overwrites(
) {
    assert_the_units_of_the_recorded_publish_reading_back_zero_where_both_devices_lost_their_newest_rotation_write_fall_back_to_the_previous_version(
        RecordedPublishKind::SequentialWriteOfThreeDataUnits,
        5,
    );
}

/// 攻方 Y4（`research/prompts/m3-batch37-43-code-r1-opus-output.md` 第二节 `Ru+cfg+Proot`）：崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元、
/// 每块盘较新的那个系统配置槽（P 那次轮换写的）与 P 的根槽读回全零。c_见证 落回 Pm，R 走读失败、没见证；[Pm, R) 里读得出、不被 R 的实例表抛弃的
/// 只有 Pm，可 Pm 与 R 之间的 P 带末条标志的记录读得出、根读不出、不被 R 的实例表抛弃——退到 Pm 就越过已确认的 P，三处照报 R 读不出、
/// 可写挂载在任何写之前拒。判别力：不看这样一次发布（变异「Y4 F2 退过一次末条读得出、根读不出的发布（落到 Pm、跳过已确认的 P）」）时
/// 三处都落到 Pm、这一条红。
#[test]
fn a_root_whose_units_the_newer_system_configuration_slots_and_the_previous_root_slot_read_back_zero_is_reported_unreadable_instead_of_skipping_the_previous_version(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges =
        RangesReadingBackAllZero(newer_system_configuration_slot_of_each_device(&crash_image))
            .with_the_writes(&recorded.unit_writes())
            .with_the_root_slot_of(&crash_image, recorded.previous_key);
    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
        &recorded,
        &crash_image,
        &ranges,
        recorded.key,
    );
}

/// 攻方 B 形那一盘（取新实例的那次挂载 M1 里 `ur`，M1 落在 P、取号，崩在写行那次发布 Row 的根槽之后、轮换之前），下一次读时只让 Row 写出的
/// 单元读回全零（`keep_the_instance_table_of_row` 为真时留下 Row 的实例表那一片照读），P 的末条读得出：Row 走读失败、没见证，见证到的是 P
/// （c_见证 是 M1 取号那一写写进去的 P 的计数器）。[P, Row) 里有 P 与 M1 按实例表抛弃了的 R，R 更新。交回那一刻的盘面与读回全零的几段。
fn image_after_a_row_publish_abandoning_the_recorded_publish_with_the_units_of_row_reading_back_zero(
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

/// 退到的根按所选根的实例表滤（`F2_NEWEST`，正推「退到的根不按实例表滤」那一处写定）：B 形那一盘上只让 Row 写出的单元（实例表那一片除外）读回全零。
/// Row 的实例表读得出、里面有 R 那个实例的行，R 的 txg 大于行里的 ⇒ R 被抛弃，[P, Row) 里不被抛弃的最新一条是 P：三处都退到 P，
/// 不退到被新实例抛弃了的 R。判别力：不按实例表滤（变异「F2_NEWEST 退到的根不按所选根的实例表滤（退到被抛弃的 R）」）时三处读回 R、这一条红。
#[test]
fn a_row_publish_whose_units_read_back_zero_falls_back_past_the_root_its_instance_table_abandoned_to_the_witnessed_version(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let (image, ranges, row_key) =
        image_after_a_row_publish_abandoning_the_recorded_publish_with_the_units_of_row_reading_back_zero(
            &recorded, true,
        );
    let report = recover(
        &devices_reading_back_all_zero_on(&image, &RangesReadingBackAllZero::default()),
        JournalPolicy::Consult,
    );
    assert!(
        matches!(report.outcome, RecoveryOutcome::FileRead { root, .. } if root == row_key),
        "没有读故障时恢复择到 Row、走得通：{:?}",
        report.outcome
    );
    assert_recovery_and_both_mounts_read_back_the_previous_version(&recorded, &image, &ranges);
}

/// 所选根的实例表读不出时只认恰好相等的那条：B 形那一盘上 Row 写出的单元连同它的实例表都读回全零。滤不了，只认与见证到的 P 恰好相等的那条：
/// 恢复与只读挂载退到 P；可写挂载判抛弃用的最新那条根（Row）的实例表重读一次仍读不出，在取号之前拒（那一拒在 D23 单子上，盘上逐字节不变）。
/// 判别力：实例表读不出时照收 [P, Row) 里最新的一条（变异「F2_NEWEST 所选根的实例表读不出时照收最新的一条（退到被抛弃的 R）」）
/// 时恢复读回 R、这一条红。
#[test]
fn a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let (image, ranges, _row_key) =
        image_after_a_row_publish_abandoning_the_recorded_publish_with_the_units_of_row_reading_back_zero(
            &recorded, false,
        );
    let devices = devices_reading_back_all_zero_on(&image, &ranges);
    let report = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: recorded.previous_key,
            content: recorded.previous_content.clone(),
        },
        "恢复：Row 的实例表读不出，只认与见证到的 P 恰好相等的那条，不退到被抛弃的 R"
    );
    let read_only = mount_read_only(&devices).expect("只读挂载退回 P、打得开");
    assert_eq!(
        (
            read_only.effective_root.instance,
            read_only.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "只读挂载：退回 P"
    );
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(&image, &ranges);
    let refusal = mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount)
        .expect_err("判抛弃用的最新那条根的实例表读不出：可写挂载拒");
    assert!(
        matches!(
            &refusal,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if matches!(
                    **still_unreadable,
                    StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
                )
        ),
        "拒因是最新那条根的实例表读不出：{refusal:?}"
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices_of_the_mount, recorded.pool.device_bytes)
            == image,
        "可写挂载在任何写之前拒：两块盘逐字节不变"
    );
}

/// 见证记录读不出照旧不退（`F2_NEWEST` 之下）：B 形那一盘上 Row 写出的单元（实例表那一片除外）与 P 的末条记录（计数器 = c_见证 的那一条）读回全零。
/// Row 的实例表读得出、滤得了，[P, Row) 里不被抛弃的最新一条是 P——可那条见证记录读不出，见证到的是哪一版说不出，三处照报 Row 读不出。
/// 判别力：见证记录读不出时下限放到最旧（变异「F2_NEWEST 见证记录读不出时照退（下限放到最旧，退到 P）」）时三处读回 P、这一条红。
#[test]
fn a_row_publish_whose_units_and_the_witnessed_last_record_read_back_zero_is_reported_unreadable_even_with_its_instance_table_readable(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let (image, ranges, row_key) =
        image_after_a_row_publish_abandoning_the_recorded_publish_with_the_units_of_row_reading_back_zero(
            &recorded, true,
        );
    let ranges = ranges.with_the_last_record_of_the_publish(&image, recorded.previous_key);
    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
        &recorded, &image, &ranges, row_key,
    );
}

/// 在内层读法之上，点名的一段（一个根槽：退掉的 R 的，或 R 与退到的那一版之间那条根 Q 的）的第 `earliest_bad_read_number` 到第 `latest_bad_read_number` 次读读回全零（盘上的字节不动）；写到那一段之后照读。
struct DeviceWithOneRootSlotBadOnChosenReads {
    inner: DeviceReadingBackAllZeroOnRanges<SparseBlockDevice>,
    root_slot_bad_on_chosen_reads: Option<(u64, u64)>,
    earliest_bad_read_number: u64,
    latest_bad_read_number: u64,
    reads_of_the_root_slot: Rc<Cell<u64>>,
}

impl DeviceWithOneRootSlotBadOnChosenReads {
    fn lift_when_written_over(&mut self, offset_in_bytes: u64, length_in_bytes: u64) {
        if let Some((start, length)) = self.root_slot_bad_on_chosen_reads {
            if offset_in_bytes < start + length && start < offset_in_bytes + length_in_bytes {
                self.root_slot_bad_on_chosen_reads = None;
            }
        }
    }
}

impl BlockDevice for DeviceWithOneRootSlotBadOnChosenReads {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let buffer_length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        if let Some((start, length)) = self.root_slot_bad_on_chosen_reads {
            if offset.0 < start + length && start < offset.0 + buffer_length {
                let read_number = self.reads_of_the_root_slot.get() + 1;
                self.reads_of_the_root_slot.set(read_number);
                if self.earliest_bad_read_number <= read_number
                    && read_number <= self.latest_bad_read_number
                {
                    let from = usize::try_from(start.saturating_sub(offset.0)).expect("缓冲内偏移");
                    let to =
                        usize::try_from((start + length).min(offset.0 + buffer_length) - offset.0)
                            .expect("缓冲内偏移");
                    buffer[from..to].fill(0);
                }
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.lift_when_written_over(
            offset.0,
            u64::try_from(bytes.len()).expect("写长装得进 u64"),
        );
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.lift_when_written_over(offset.0, length);
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// `image` 上 `ranges` 读回全零，`root_slot_write` 写的那个根槽从第 `earliest_bad_read_number` 次读起读回全零：
/// `only_this_read` 时只坏这一次，否则一直坏到被写掉。
fn devices_with_the_root_slot_bad_on_chosen_reads(
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    root_slot_write: &RetainedWrite,
    only_this_read: bool,
    earliest_bad_read_number: u64,
    reads_of_the_root_slot: &Rc<Cell<u64>>,
) -> Vec<(DeviceIdentity, DeviceWithOneRootSlotBadOnChosenReads)> {
    let latest_bad_read_number = if only_this_read {
        earliest_bad_read_number
    } else {
        u64::MAX
    };
    devices_reading_back_all_zero_on(image, ranges)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceWithOneRootSlotBadOnChosenReads {
                    inner,
                    root_slot_bad_on_chosen_reads: (identity == root_slot_write.device)
                        .then_some((root_slot_write.offset.0, root_slot_write.length_in_bytes())),
                    earliest_bad_read_number,
                    latest_bad_read_number,
                    reads_of_the_root_slot: Rc::clone(reads_of_the_root_slot),
                },
            )
        })
        .collect()
}

/// 判退那一形一格的盘面：每块盘底下那份内存盘此刻的字节。
fn image_of_the_devices_with_the_root_slot_bad_on_chosen_reads(
    devices: &[(DeviceIdentity, DeviceWithOneRootSlotBadOnChosenReads)],
    device_size_in_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.inner.image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

/// 判退那一形一格可写挂载的拒因是不是 C 读一条比退到的那一版新的根那一份账（`C_NEWER_ROOTS`，`mount.rs` 的 `placements_in_the_accounts_of_newer_readable_roots`）：
/// 那条根（退掉的 R，或 R 与退到的那一版之间读阶段读得出的 Q）比退到的那一版新、按实例表有效；它的单元与记录这次挂载都读回全零，账读不出、一条记录都读不出、
/// 映射现算不成，重读一次仍不成、现行账里也没有凭据 ⇒ 拒，点名它的 (实例代号, checkpoint_txg)（`key_of_the_root_fallen_back_from`），在任何写之前
/// （它的单元可能还在，不拒就会在写行那次发布里写掉它们）。别的拒因交 `false`。
fn refused_on_the_account_of_the_root_fallen_back_from(
    refusal: &MountError,
    key_of_the_root_fallen_back_from: (InstanceGeneration, CheckpointTxg),
) -> bool {
    matches!(
        refusal,
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
            if matches!(
                still_unreadable.as_ref(),
                StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                    unapplied_publish,
                } if (unapplied_publish.instance, unapplied_publish.checkpoint_txg)
                    == key_of_the_root_fallen_back_from
            )
    )
}

/// R 写出的单元里的数据单元写（每块盘一份）：单元写的字节解得开数据单元头的那几份。R 的账（树表、分配记录树）与映射树的节点不在其内。
fn data_unit_writes_of_the_recorded_publish(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    recorded
        .unit_writes()
        .into_iter()
        .filter(|write| {
            write
                .bytes()
                .is_some_and(|bytes| parse_data_unit(bytes).is_ok())
        })
        .collect()
}

/// 判退那一形读回全零的几段，R 的账读得出的那一种：R 的数据单元与 R 的记录。R 的根读得出、走到文件读不到数据单元 ⇒ 走读失败（字节丢了）、
/// 没见证，退到 P；R 的账读得出，C 读 R 那一份账做得成，挂载做得成、R 的单元照护。
fn ranges_with_the_data_units_and_the_record_of_the_recorded_publish_reading_back_zero(
    recorded: &RecordedPublish,
) -> RangesReadingBackAllZero {
    let data_unit_writes = data_unit_writes_of_the_recorded_publish(recorded);
    assert!(
        !data_unit_writes.is_empty(),
        "覆盖写的 R 写了一个数据单元（这一形的前提：读回全零的是数据单元、不是 R 的账）"
    );
    RangesReadingBackAllZero::default()
        .with_the_writes(&data_unit_writes)
        .with_the_writes(&recorded.record_writes_of_the_record(1))
}

/// D23（journal 的角色与格式） 已定项 14 第 3 条，判退那一形：崩在 R 的根槽之后、轮换之前，这次挂载里 R 的数据单元与记录读回全零（R 的账读得出），
/// 读阶段读到 R 的根（R 的根槽第一次读是好的），R 走读失败、没见证 ⇒ 退到 P。R 的根槽第几次读读回全零都扫一遍（只这一次 / 从这一次起一直）：
/// 取第一个 txg 那一遍直接读盘读坏的那几格都在里面，第几次读落在那一遍里不写死。
/// 拒在 C 读 R 那一份账上（[`refused_on_the_account_of_the_root_fallen_back_from`]）的格照收，盘上逐字节不变。
/// 做成的格：R 的记录读不出，记录那一半盖不住 R；写行 txg 都要大于 R，池级 checker 无违例（重建分配器读根环那一遍读坏 R 的格，C 照读阶段读到的 R 护它的单元）。
/// 至少一格做成。每格打一行 `FIRST_TXG_AFTER_THE_FALL_BACK_CELL`（`mount=ok` 或 `mount=refused_on_the_account_of_r`）。
#[test]
fn root_fallen_back_from_and_read_bad_after_the_read_stage_still_numbers_the_new_instance_above_it()
{
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges =
        ranges_with_the_data_units_and_the_record_of_the_recorded_publish_reading_back_zero(
            &recorded,
        );
    let root_slot_write = recorded.root_slot_write();
    let txg_of_the_recorded_publish = recorded.key.1 .0;
    let mut cells_mounted = 0usize;
    for only_this_read in [true, false] {
        for earliest_bad_read_number in 2..=16u64 {
            let reads_of_the_root_slot_of_the_recorded_publish = Rc::new(Cell::new(0u64));
            let mut devices = devices_with_the_root_slot_bad_on_chosen_reads(
                &crash_image,
                &ranges,
                root_slot_write,
                only_this_read,
                earliest_bad_read_number,
                &reads_of_the_root_slot_of_the_recorded_publish,
            );
            let mounted = match mount_writable(&recorded.pool.parameters, &mut devices) {
                Ok(mounted) => mounted,
                Err(refusal) => {
                    assert!(
                        refused_on_the_account_of_the_root_fallen_back_from(&refusal, recorded.key),
                        "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：可写挂载做成，或拒在退掉的 R 那一份账上；得到 {refusal:?}"
                    );
                    assert!(
                        image_of_the_devices_with_the_root_slot_bad_on_chosen_reads(
                            &devices,
                            recorded.pool.device_bytes
                        ) == crash_image,
                        "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：拒在任何写之前，盘上逐字节不变"
                    );
                    println!(
                        "FIRST_TXG_AFTER_THE_FALL_BACK_CELL only_this_read={only_this_read} earliest_bad_read_number={earliest_bad_read_number} reads={} mount=refused_on_the_account_of_r",
                        reads_of_the_root_slot_of_the_recorded_publish.get()
                    );
                    continue;
                }
            };
            cells_mounted += 1;
            let effective = (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg,
            );
            let row = mounted.output.row_publish.root().checkpoint_txg.0;
            let image_after = image_of_the_devices_with_the_root_slot_bad_on_chosen_reads(
                &devices,
                recorded.pool.device_bytes,
            );
            let violated: Vec<&'static str> =
                singlefs_checker::walk::check_pool_image(&image_after)
                    .into_iter()
                    .filter(|(_, verdict)| {
                        matches!(
                            verdict,
                            singlefs_checker::image::InvariantVerdict::Violated(_)
                        )
                    })
                    .map(|(identifier, _)| identifier)
                    .collect();
            println!(
                "FIRST_TXG_AFTER_THE_FALL_BACK_CELL only_this_read={only_this_read} earliest_bad_read_number={earliest_bad_read_number} reads={} mount=ok effective={effective:?} row={row} txg_of_r={txg_of_the_recorded_publish} violated={violated:?}",
                reads_of_the_root_slot_of_the_recorded_publish.get()
            );
            assert_eq!(
                effective, recorded.previous_key,
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：R 走读失败、没见证，退到 P"
            );
            assert!(
                row > txg_of_the_recorded_publish,
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：写行 txg {row} 大于退掉的 R {txg_of_the_recorded_publish}"
            );
            assert_eq!(
                violated,
                Vec::<&'static str>::new(),
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：池级 checker 无违例"
            );
        }
    }
    assert!(
        cells_mounted > 0,
        "至少一格可写挂载做成（读阶段读到 R、之后读环读不到 R 那条路）；一格都没做成"
    );
}

/// I-7.4（回退候选集里每一个根所引用的块不被重新分配）在判退那一形上：崩在 R 的根槽之后、轮换之前，这次挂载里 R 的全部单元与记录读回全零（R 的账读不出），
/// 读阶段读到 R 的根，R 走读失败、没见证 ⇒ 退到 P；R 比 P 新、按实例表有效，在回退候选集里。R 的根槽第几次读读回全零都扫一遍（只这一次 / 从这一次起一直），
/// 重建分配器读根环那一遍读坏 R 的那几格都在里面：C 照读阶段读到的 R 读它那一份账，读不成、记录一条都读不出、映射现算不成、重读一次仍不成、现行账里没有凭据
/// ⇒ 每一格都拒在 R 那一份账上（[`refused_on_the_account_of_the_root_fallen_back_from`]），在任何写之前、盘上逐字节不变；没有一格做成可写挂载
/// （做成就在写行那次发布里写掉 R 的单元）。每格打一行 `ROOT_FALLEN_BACK_FROM_UNREADABLE_ACCOUNT_CELL`。
/// 判别力：C 只看读根环那一遍读出的根（变异「C_NEWER_ROOTS 不并读阶段择到的根」）时，那一遍读坏 R 的格做成可写挂载、这一条红。
#[test]
fn root_fallen_back_from_whose_units_and_record_read_back_zero_and_read_bad_after_the_read_stage_is_refused_on_its_account_in_every_cell(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&recorded.record_writes_of_the_record(1));
    let root_slot_write = recorded.root_slot_write();
    for only_this_read in [true, false] {
        for earliest_bad_read_number in 2..=16u64 {
            let reads_of_the_root_slot_of_the_recorded_publish = Rc::new(Cell::new(0u64));
            let mut devices = devices_with_the_root_slot_bad_on_chosen_reads(
                &crash_image,
                &ranges,
                root_slot_write,
                only_this_read,
                earliest_bad_read_number,
                &reads_of_the_root_slot_of_the_recorded_publish,
            );
            let refusal = match mount_writable(&recorded.pool.parameters, &mut devices) {
                Ok(mounted) => panic!(
                    "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读（R 的根槽共读了 {} 次）：R 的账读不出、没有凭据，可写挂载却做成了，\
                     落在 {:?}、写行 txg {}",
                    reads_of_the_root_slot_of_the_recorded_publish.get(),
                    (
                        mounted.output.effective_root.instance,
                        mounted.output.effective_root.checkpoint_txg
                    ),
                    mounted.output.row_publish.root().checkpoint_txg.0
                ),
                Err(refusal) => refusal,
            };
            println!(
                "ROOT_FALLEN_BACK_FROM_UNREADABLE_ACCOUNT_CELL only_this_read={only_this_read} earliest_bad_read_number={earliest_bad_read_number} reads={}",
                reads_of_the_root_slot_of_the_recorded_publish.get()
            );
            assert!(
                refused_on_the_account_of_the_root_fallen_back_from(&refusal, recorded.key),
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：拒在退掉的 R 那一份账上；得到 {refusal:?}"
            );
            assert!(
                image_of_the_devices_with_the_root_slot_bad_on_chosen_reads(
                    &devices,
                    recorded.pool.device_bytes
                ) == crash_image,
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：拒在任何写之前，盘上逐字节不变"
            );
        }
    }
}

/// D8（核心索引结构） 已定项 8 ②（号永不重发）在判退那一形上：R 是发过树号的发布（建快照那一次发两个号，这里在崩溃镜像上把 R 的根槽换成水位高两个号、照样自证的同一条根；
/// R 的数据单元与记录这次挂载读回全零，R 的账读得出，记录那一半盖不住 R 的水位）。读阶段读到 R 的根、退到 P，R 的根槽第几次读读回全零都扫一遍
/// （同 `root_fallen_back_from_and_read_bad_after_the_read_stage_still_numbers_the_new_instance_above_it`）：取水位那一遍直接读盘读坏的那几格都在里面。
/// 拒在 R 那一份账上（[`refused_on_the_account_of_the_root_fallen_back_from`]）的格照收、盘上逐字节不变；做成的格写行那次发布带的水位都等于 R 的，
/// 不落回 P 的；至少一格做成。
/// 这里只钉水位：改写过的 R 的根与盘上 R 的记录带的水位不再相同，不拿这张盘面问池级 checker。
#[test]
fn root_fallen_back_from_and_read_bad_after_the_read_stage_still_raises_the_tree_identifier_watermark_of_the_new_instance_to_its_own(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let mut root_having_issued_tree_identifiers = recorded.root();
    let watermark_of_the_overwrite = root_having_issued_tree_identifiers.tree_identifier_watermark;
    let watermark_after_issuing_tree_identifiers = watermark_of_the_overwrite + 2;
    root_having_issued_tree_identifiers.tree_identifier_watermark =
        watermark_after_issuing_tree_identifiers;
    let root_slot_write = RetainedWrite {
        contents: WrittenContents::Bytes(root_having_issued_tree_identifiers.to_slot(
            usize::try_from(recorded.root_slot_write().length_in_bytes()).expect("根槽宽"),
        )),
        device: recorded.root_slot_write().device,
        kind: recorded.root_slot_write().kind,
        is_force_unit_access: recorded.root_slot_write().is_force_unit_access,
        offset: recorded.root_slot_write().offset,
    };
    let mut crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    crash_image.apply_writes(std::slice::from_ref(&root_slot_write));
    let ranges =
        ranges_with_the_data_units_and_the_record_of_the_recorded_publish_reading_back_zero(
            &recorded,
        );
    let mut cells_mounted = 0usize;
    for only_this_read in [true, false] {
        for earliest_bad_read_number in 2..=16u64 {
            let reads_of_the_root_slot = Rc::new(Cell::new(0u64));
            let mut devices = devices_with_the_root_slot_bad_on_chosen_reads(
                &crash_image,
                &ranges,
                &root_slot_write,
                only_this_read,
                earliest_bad_read_number,
                &reads_of_the_root_slot,
            );
            let mounted = match mount_writable(&recorded.pool.parameters, &mut devices) {
                Ok(mounted) => mounted,
                Err(refusal) => {
                    assert!(
                        refused_on_the_account_of_the_root_fallen_back_from(&refusal, recorded.key),
                        "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：可写挂载做成，或拒在退掉的 R 那一份账上；得到 {refusal:?}"
                    );
                    assert!(
                        image_of_the_devices_with_the_root_slot_bad_on_chosen_reads(
                            &devices,
                            recorded.pool.device_bytes
                        ) == crash_image,
                        "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：拒在任何写之前，盘上逐字节不变"
                    );
                    continue;
                }
            };
            cells_mounted += 1;
            assert_eq!(
                (
                    mounted.output.effective_root.instance,
                    mounted.output.effective_root.checkpoint_txg
                ),
                recorded.previous_key,
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：R 走读失败、没见证，退到 P"
            );
            assert_eq!(
                mounted.output.effective_root.tree_identifier_watermark,
                watermark_of_the_overwrite,
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：退到的 P 带的水位低于 R 的（这一形的前提）"
            );
            assert_eq!(
                mounted.output.row_publish.root().tree_identifier_watermark,
                watermark_after_issuing_tree_identifiers,
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：写行那次发布带的水位等于退掉的 R 的"
            );
        }
    }
    assert!(
        cells_mounted > 0,
        "至少一格可写挂载做成（读阶段读到 R、之后读环读不到 R 那条路）；一格都没做成"
    );
}

/// Q 那一形的盘面：录的那次发布当 Q（它之前那一版是 P），Q 整次发布落完之后会话里再覆盖写一次当 R、R 整次发布也落完，
/// 再把每块盘的系统配置槽写回 Q 之前那一刻的字节（Q、R 两次轮换都没落，c_见证 是 P 的）。交回盘面、R 的 (实例, txg) 与 R 写出的单元落的几段
/// （每块盘一份，按录制流里的单元写认）。
/// **这是两块盘都丢掉两次轮换写的盘面，不是任何一次崩溃的状态**：Q 那次发布在它的轮换与末尾屏障之后才返回，R 的写都在那之后
/// （同 `common_unwitnessed_publish::image_where_both_devices_lost_their_newest_rotation_write` 的说明，丢写归 C6（块层语义假设写错））；
/// 用这份盘面的用例钉的是产品在这张盘上的行为，不说这张盘走得到。
fn image_with_the_intermediate_and_the_recorded_publish_persisted_and_both_rotations_lost(
    recorded_intermediate_publish: &mut RecordedPublish,
) -> (
    MemoryPool,
    (InstanceGeneration, CheckpointTxg),
    RangesReadingBackAllZero,
) {
    let geometry = fixed_geometry_of_the_384_slot_small_devices();
    let operations_before_the_recorded_publish = recorded_intermediate_publish.pool.devices[0]
        .1
        .recorded_operations()
        .len();
    recorded_intermediate_publish
        .pool
        .overwrite(OVERWRITE_BYTES)
        .expect("384 槽小盘上 Q 之后再覆盖写一次落得下");
    let root_of_the_recorded_publish = *recorded_intermediate_publish.pool.session().current.root();
    let operations_of_the_recorded_publish = recorded_intermediate_publish.pool.devices[0]
        .1
        .recorded_operations()
        .split_off(operations_before_the_recorded_publish);
    assert_eq!(
        operations_of_the_recorded_publish
            .iter()
            .filter(|operation| geometry.classify(operation) == StepKind::RootRecordFua)
            .count(),
        1,
        "R 那次覆盖写只写一条根（准入够，没推抬 F）"
    );
    let unit_ranges_of_the_recorded_publish = RangesReadingBackAllZero(
        operations_of_the_recorded_publish
            .iter()
            .filter(|operation| geometry.classify(operation) == StepKind::UnitWrite)
            .map(|operation| (operation.device, operation.offset.0, operation.length))
            .collect(),
    );
    assert!(
        !unit_ranges_of_the_recorded_publish.0.is_empty(),
        "R 那次覆盖写写了单元（这一形的前提：R 的单元读回全零、R 走读失败）"
    );
    let mut image = recorded_intermediate_publish.pool.image();
    let configuration = choose_system_configuration(&image).expect("镜像上择得到系统配置");
    let slot_spacing_in_bytes =
        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
    for (device, sparse_device_before_the_intermediate_publish) in
        &recorded_intermediate_publish.base.devices
    {
        for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let slot_offset = DeviceOffsetInBytes(slot_index * slot_spacing_in_bytes);
            let slot_before_the_intermediate_publish =
                sparse_device_before_the_intermediate_publish.read(slot_offset, slot_bytes);
            image
                .devices
                .get_mut(device)
                .expect("池里有这块盘")
                .write(slot_offset, &slot_before_the_intermediate_publish);
        }
    }
    (
        image,
        (
            root_of_the_recorded_publish.instance,
            root_of_the_recorded_publish.checkpoint_txg,
        ),
        unit_ranges_of_the_recorded_publish,
    )
}

/// Q 的根槽从第几次读起一直读回全零（从 1 数）：第 1 次读是读阶段择根那一遍（经读阶段缓存，读阶段之内不再打到盘上），
/// 从第 2 次读起是判退那两问（各带一次重读）、取第一个 txg、取水位、F 生效值与重建分配器读根环那一遍（判抛弃用的实例表取读阶段择到的根，那一步不读根环）。
const FIRST_READ_OF_THE_ROOT_SLOT_OF_THE_INTERMEDIATE_PUBLISH_AFTER_THE_READ_STAGE: u64 = 2;

/// I-7.4（回退候选集里每一个根所引用的块不被重新分配）在判退那一形上，P 与 R 之间还有一条根 Q（P < Q < R）：
/// [`image_with_the_intermediate_and_the_recorded_publish_persisted_and_both_rotations_lost`] 那张盘（c_见证 是 P 的），这次挂载里 R 的单元、Q 的单元与 Q 的记录（覆盖写一条）读回全零，
/// Q 的根槽从读阶段之后那一次读起一直读回全零（[`FIRST_READ_OF_THE_ROOT_SLOT_OF_THE_INTERMEDIATE_PUBLISH_AFTER_THE_READ_STAGE`]）。读阶段择到 R、也读到了 Q；R 走读失败、没见证，
/// 判退那两问读 Q 的根槽都读坏，Q 的记录又读不出（不是「末条读得出、根读不出」那一形），[P, R) 里读得出的只有 P ⇒ 退到 P。
/// Q 比 P 新、按实例表有效，在回退候选集里；重建分配器读根环那一遍读不到 Q，C 照读阶段读到的 Q 读它那一份账：读不成、记录一条都读不出、映射现算不成、
/// 重读一次仍不成、现行账里没有凭据 ⇒ 拒在 Q 那一份账上（[`refused_on_the_account_of_the_root_fallen_back_from`]），在任何写之前、盘上逐字节不变
/// （做成就在写行那次发布里写掉 Q 的单元）。打一行 `ROOT_BETWEEN_THE_VERSION_FALLEN_BACK_TO_AND_THE_ROOT_FALLEN_BACK_FROM_CELL`。
/// 判别力：C 只并读阶段择到的那一条根（变异「C_NEWER_ROOTS 只并读阶段择到的根、不并读阶段读到的全部根」）时 C 看不见 Q、不拒，这一条红。
#[test]
fn root_read_by_the_read_stage_between_the_version_fallen_back_to_and_the_root_fallen_back_from_whose_units_and_record_read_back_zero_is_refused_on_its_account(
) {
    let mut recorded_intermediate_publish = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_RECORDED_PUBLISH,
        RecordedPublishKind::Overwrite,
    );
    let (image, key_of_the_recorded_publish, unit_ranges_of_the_recorded_publish) =
        image_with_the_intermediate_and_the_recorded_publish_persisted_and_both_rotations_lost(
            &mut recorded_intermediate_publish,
        );
    let key_of_the_previous_version = recorded_intermediate_publish.previous_key;
    let key_of_the_intermediate_publish = recorded_intermediate_publish.key;
    assert!(
        key_of_the_previous_version.1 < key_of_the_intermediate_publish.1 && key_of_the_intermediate_publish.1 < key_of_the_recorded_publish.1,
        "P < Q < R：{key_of_the_previous_version:?} {key_of_the_intermediate_publish:?} {key_of_the_recorded_publish:?}"
    );
    assert!(
        root_is_in_the_ring(&image, key_of_the_intermediate_publish)
            && root_is_in_the_ring(&image, key_of_the_recorded_publish),
        "Q 与 R 的根都在根环里"
    );
    assert_eq!(
        choose_system_configuration(&image)
            .expect("镜像上择得到系统配置")
            .quantities
            .journal_tail,
        choose_system_configuration(&recorded_intermediate_publish.base)
            .expect("Q 之前那一刻择得到系统配置")
            .quantities
            .journal_tail,
        "系统配置见证到的是 P（Q、R 两次轮换都没落）"
    );
    let ranges = unit_ranges_of_the_recorded_publish
        .with_the_writes(&recorded_intermediate_publish.unit_writes())
        .with_the_writes(&recorded_intermediate_publish.record_writes_of_the_record(1));
    let reads_of_the_root_slot_of_the_intermediate_publish = Rc::new(Cell::new(0u64));
    let mut devices = devices_with_the_root_slot_bad_on_chosen_reads(
        &image,
        &ranges,
        recorded_intermediate_publish.root_slot_write(),
        false,
        FIRST_READ_OF_THE_ROOT_SLOT_OF_THE_INTERMEDIATE_PUBLISH_AFTER_THE_READ_STAGE,
        &reads_of_the_root_slot_of_the_intermediate_publish,
    );
    let refusal = match mount_writable(&recorded_intermediate_publish.pool.parameters, &mut devices) {
        Ok(mounted) => panic!(
            "Q 的根槽从第 {FIRST_READ_OF_THE_ROOT_SLOT_OF_THE_INTERMEDIATE_PUBLISH_AFTER_THE_READ_STAGE} 次读起读坏（共读了 {} 次）：Q 的账读不出、没有凭据，可写挂载却做成了，\
             落在 {:?}、写行 txg {}",
            reads_of_the_root_slot_of_the_intermediate_publish.get(),
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg
            ),
            mounted.output.row_publish.root().checkpoint_txg.0
        ),
        Err(refusal) => refusal,
    };
    println!(
        "ROOT_BETWEEN_THE_VERSION_FALLEN_BACK_TO_AND_THE_ROOT_FALLEN_BACK_FROM_CELL reads={}",
        reads_of_the_root_slot_of_the_intermediate_publish.get()
    );
    assert!(
        refused_on_the_account_of_the_root_fallen_back_from(
            &refusal,
            key_of_the_intermediate_publish
        ),
        "拒在 Q 那一份账上；得到 {refusal:?}"
    );
    assert!(
        image_of_the_devices_with_the_root_slot_bad_on_chosen_reads(
            &devices,
            recorded_intermediate_publish.pool.device_bytes
        ) == image,
        "拒在任何写之前，盘上逐字节不变"
    );
}
