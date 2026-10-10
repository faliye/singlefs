//! F2 乙（用户 2026-10-07 定「三处都做」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 7 条）：所选那一版走读失败、系统配置没见证它时，
//! 退回系统配置见证到的那一版；恢复（`recovery::recover`）、可写挂载读阶段之后建上一版那一步、只读挂载（`mounted_read::mount_read_only`）三处同一份判法
//! （`recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`）。「见证过」照攻方 `strict`：所选那一版带末条标志的记录读得出 ⇒
//! 计数器 ≤ c_见证；读不出 ⇒ 拿计数器 = c_见证 的那条记录比 (txg, 实例)；两条都读不出当见证过、不退。退到 [见证到的那一版, 所选根) 里读得出、
//! 不被所选根的实例表抛弃的最新一条根（`F2_NEWEST`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H4 那一行与第三节第 1 条），
//! 所选根的实例表读不出时只认与见证到的那一版恰好相等的那条；不退到比见证到的那一版更旧的根（`F2_EXACT`，判决
//! `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行与第三节第 1 条）；计数器 = c_见证 的那条记录读不出、或那一段里没有读得出的根，都不退。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R（P 是它之前那一版，已确认）。
//! - 第二轮攻方 5.2 两形：崩在 R 的根槽落盘之后、系统配置轮换之前；取新实例的那次挂载里 R 的根槽与单元读回全零（`ur`）或连 R 的记录也读回全零
//!   （`urJ`），那次挂载写行那次发布的根落盘之前断电，R 的单元已被那次挂载写掉；断电之后三处都读回 P。`ur` 那一形今天的 C 隔离得住 R 的单元，
//!   这里关着影子账（只供测试的开关 `ShadowLedger::Off`，连 C 一起关）造出 R 的单元被写掉的盘面；`urJ` 那一形 C 本来就看不见。
//! - 5.3 两形（R 已确认：整次发布连系统配置轮换都落了）：S2——R 的末条（唯一一条）记录与单元读回全零，两条都读不出、当见证过、不退，照报读不出；
//!   S1——较新的那个系统配置槽（R 那次轮换写的）与 R 的单元读回全零，c_见证 落回 P，R 算没见证、读回 P：用户 2026-10-07 定案接受的代价。
//! - 代码轮攻方（`research/prompts/m3-batch33t-code-r1-opus-output.md` 第一节）两形：A——崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元与 P 的根槽
//!   读回全零（见证到的就是 P，P 的根读不出）；B——取新实例的那次挂载里 `ur`、崩在那次挂载写行那次发布的根槽之后，下一次读时写行那次发布的单元
//!   与 P 的末条记录（计数器 = c_见证 的那一条）读回全零。两形都照报读不出，不退到 P 之前那一版、也不退到被新实例抛弃的 R。
//! - 代码轮攻方（`research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节）H4：崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元与 P 那次轮换写的
//!   较新系统配置槽读回全零，c_见证 落回 P 之前那一版；三处都落到已确认的 P，四组（两种 R × 覆盖写 3、5 次）各一条。B 形那一盘上 P 的末条读得出时
//!   退到的根按 Row 的实例表滤（跳过被抛弃的 R），Row 的实例表读不出时只认恰好相等的 P。
//! - c_见证 = 0（删除线第 41 批，D23 已定项 14「c_见证 = 0 … 都不退」）：设计里那一退的底上每个系统配置槽的 tail 改 0、环里伪造一条计数器 0 的 P 的记录，
//!   三处都不退。
//! - 代码轮攻方（`research/prompts/m3-batch37-43-code-r1-opus-output.md` 第二节）Y4：H4 那一形再加 P 的根槽读回全零，c_见证 落回 Pm，[Pm, R) 里
//!   读得出的只有 Pm；Pm 与 R 之间的 P 末条读得出、根读不出、不被 R 的实例表抛弃，退到 Pm 就越过它——三处照报 R 读不出（用户 2026-10-08 定「两形都修」）。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    image_of_devices_reading_back_all_zero, record_one_publish_after_overwrites,
    recorded_devices_reading_back_all_zero_on, units_of_r_overwritten_on, PublishRecordedAsR,
    RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
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
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const OVERWRITES_BEFORE_R: u64 = 5;

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
        units_of_r_overwritten_on(recorded, &image),
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
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
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
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
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
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
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
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
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
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
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
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
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
/// 计数器大于它，算没见证，恢复与只读挂载读回 P——用户 2026-10-07 定案接受的代价（F2 乙「三处都做」，交用户时写明：已确认的版本在较新系统配置槽与
/// 它的单元同时读回全零时悄悄读回上一版）。这一条钉住这个结局；条款改了它跟着改。判别力：同 `ur` 那一条的变异（末条那一支恒见证过）时照报读不出、这一条红。
#[test]
fn a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let confirmed = recorded.image_with_the_whole_publish_persisted();
    let rotation_of_r: Vec<_> = recorded
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        rotation_of_r.len(),
        2,
        "R 那次发布的系统配置轮换每块盘写一槽"
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&rotation_of_r);
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
/// 判别力：去掉 c_见证 = 0 的守卫（变异「删除线第41批 F2 c_见证 = 0 不另设守卫（计数器 0 的伪造记录上退到 P）」）时三处都退到 P、这一条红。
#[test]
fn a_witnessed_counter_of_zero_does_not_fall_back_even_with_a_record_carrying_counter_zero_in_recovery_and_both_mounts(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
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

/// 攻方 H4（`research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节）：崩在 R 的根槽之后、轮换之前（R 没见证；P 已确认，它那次轮换写的
/// 系统配置槽在每块盘上都是较新的那一槽），读的那一刻 R 的单元与每块盘较新的那个系统配置槽读回全零。c_见证 落回 P 之前那一版 Pm 的计数器，
/// R 走读失败、没见证 ⇒ 退到 [Pm, R) 里读得出、不被 R 的实例表抛弃的最新一条根，就是已确认、单元完好的 P（`F2_NEWEST`），三处都落到 P。
/// 判别力：只退到见证到的那一版本身（变异「删除线第38批 F2_NEWEST 只退到见证到的那一版本身（H4：落到 Pm、跳过已确认的 P）」）时三处都落到 Pm、这一条红。
fn assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
    publish: PublishRecordedAsR,
    overwrites_before_r: u64,
) {
    let recorded = record_one_publish_after_overwrites(overwrites_before_r, publish);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges =
        RangesReadingBackAllZero(newer_system_configuration_slot_of_each_device(&crash_image))
            .with_the_writes(&recorded.unit_writes());
    assert_recovery_and_both_mounts_read_back_the_previous_version(
        &recorded,
        &crash_image,
        &ranges,
    );
}

/// H4 那一组：一条记录的覆盖写 R、R 之前覆盖写 3 次。
#[test]
fn a_root_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_three_overwrites(
) {
    assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
        PublishRecordedAsR::Overwrite,
        3,
    );
}

/// H4 那一组：一条记录的覆盖写 R、R 之前覆盖写 5 次。
#[test]
fn a_root_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_five_overwrites(
) {
    assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
        PublishRecordedAsR::Overwrite,
        5,
    );
}

/// H4 那一组：三条记录的顺序写 R、R 之前覆盖写 3 次。
#[test]
fn a_root_of_three_records_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_three_overwrites(
) {
    assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
        3,
    );
}

/// H4 那一组：三条记录的顺序写 R、R 之前覆盖写 5 次。
#[test]
fn a_root_of_three_records_whose_units_and_the_newer_system_configuration_slots_read_back_zero_falls_back_to_the_confirmed_previous_version_after_five_overwrites(
) {
    assert_the_units_of_r_and_the_newer_system_configuration_slots_reading_back_zero_fall_back_to_the_previous_version(
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
        5,
    );
}

/// 攻方 Y4（`research/prompts/m3-batch37-43-code-r1-opus-output.md` 第二节 `Ru+cfg+Proot`）：崩在 R 的根槽之后、轮换之前，读的那一刻 R 的单元、
/// 每块盘较新的那个系统配置槽（P 那次轮换写的）与 P 的根槽读回全零。c_见证 落回 Pm，R 走读失败、没见证；[Pm, R) 里读得出、不被 R 的实例表抛弃的
/// 只有 Pm，可 Pm 与 R 之间的 P 带末条标志的记录读得出、根读不出、不被 R 的实例表抛弃——退到 Pm 就越过已确认的 P，三处照报 R 读不出、
/// 可写挂载在任何写之前拒。判别力：不看这样一次发布（变异「删除线第45批 Y4 F2 退过一次末条读得出、根读不出的发布（落到 Pm、跳过已确认的 P）」）时
/// 三处都落到 Pm、这一条红。
#[test]
fn a_root_whose_units_the_newer_system_configuration_slots_and_the_previous_root_slot_read_back_zero_is_reported_unreadable_instead_of_skipping_the_previous_version(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
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

/// 退到的根按所选根的实例表滤（`F2_NEWEST`，正推「退到的根不按实例表滤」那一处写定）：B 形那一盘上只让 Row 写出的单元（实例表那一片除外）读回全零。
/// Row 的实例表读得出、里面有 R 那个实例的行，R 的 txg 大于行里的 ⇒ R 被抛弃，[P, Row) 里不被抛弃的最新一条是 P：三处都退到 P，
/// 不退到被新实例抛弃了的 R。判别力：不按实例表滤（变异「删除线第38批 F2_NEWEST 退到的根不按所选根的实例表滤（退到被抛弃的 R）」）时三处读回 R、这一条红。
#[test]
fn a_row_publish_whose_units_read_back_zero_falls_back_past_the_root_its_instance_table_abandoned_to_the_witnessed_version(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (image, ranges, row_key) =
        image_after_a_row_publish_abandoning_r_with_the_units_of_row_reading_back_zero(
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
/// 判别力：实例表读不出时照收 [P, Row) 里最新的一条（变异「删除线第38批 F2_NEWEST 所选根的实例表读不出时照收最新的一条（退到被抛弃的 R）」）
/// 时恢复读回 R、这一条红。
#[test]
fn a_row_publish_whose_units_and_instance_table_read_back_zero_falls_back_only_to_the_exactly_witnessed_version(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (image, ranges, _row_key) =
        image_after_a_row_publish_abandoning_r_with_the_units_of_row_reading_back_zero(
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
/// 判别力：见证记录读不出时下限放到最旧（变异「删除线第38批 F2_NEWEST 见证记录读不出时照退（下限放到最旧，退到 P）」）时三处读回 P、这一条红。
#[test]
fn a_row_publish_whose_units_and_the_witnessed_last_record_read_back_zero_is_reported_unreadable_even_with_its_instance_table_readable(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (image, ranges, row_key) =
        image_after_a_row_publish_abandoning_r_with_the_units_of_row_reading_back_zero(
            &recorded, true,
        );
    let ranges = ranges.with_the_last_record_of_the_publish(&image, recorded.previous_key);
    assert_recovery_and_both_mounts_report_the_chosen_version_unreadable(
        &recorded, &image, &ranges, row_key,
    );
}
