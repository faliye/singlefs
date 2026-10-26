//! 挂载透明读字段（`mount_transparency::read_recovery_fields`）在恢复按 F2 乙退回见证到的那一版时照同一份判法算期望
//! （判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 那一行「`read_recovery_fields` 断言再择根、再重放落到
//! `report.effective_root`，F2 退回后两边不等、panic」与第三节第 1 条）：两种 `ComparedVersion` 都不 panic，拿来比的那一版与恢复报告一致。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次覆盖写 R（P 是它之前那一版，已确认）。两形：
//! - S1（用户 2026-10-07 定 F2 时接受的代价那一形）：R 已确认，较新的系统配置槽（R 那次轮换写的）与 R 的单元读回全零，恢复读回 P；
//! - 设计里那一退：崩在 R 的根槽落盘之后、轮换之前，R 的单元读回全零，恢复读回 P。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, record_one_publish_after_overwrites, PublishRecordedAsR,
    RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_core::recovery::RecoveryOutcome;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::mount_transparency::{
    read_recovery_fields, ComparedVersion, PosixContentRead,
};
use singlefs_harness::segments::StepKind;

const OVERWRITES_BEFORE_R: u64 = 5;

/// S1：R 整次发布都落了，读的那一刻 R 那次轮换写的两个系统配置槽与 R 的单元读回全零。交回盘面与读回全零的那几段。
fn confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero(
    recorded: &RecordedPublish,
) -> (MemoryPool, RangesReadingBackAllZero) {
    let rotation_of_r: Vec<_> = recorded
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
        .collect();
    (
        recorded.image_with_the_whole_publish_persisted(),
        RangesReadingBackAllZero::default()
            .with_the_writes(&recorded.unit_writes())
            .with_the_writes(&rotation_of_r),
    )
}

/// 设计里那一退：崩在 R 的根槽落盘之后、轮换之前，读的那一刻 R 的单元读回全零。交回盘面与读回全零的那几段。
fn unwitnessed_root_whose_units_read_back_zero(
    recorded: &RecordedPublish,
) -> (MemoryPool, RangesReadingBackAllZero) {
    (
        recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
        RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes()),
    )
}

/// 比恢复落到的那一版：读字段不 panic，恢复读回 P，拿来比的那一版就是 P、内容与恢复报告读出的相同。
fn assert_the_recovered_version_read_is_the_witnessed_version(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let fields = read_recovery_fields(&devices, ComparedVersion::TheRecoveredVersion);
    assert_eq!(
        fields.report.outcome,
        RecoveryOutcome::FileRead {
            root: recorded.previous_key,
            content: recorded.previous_content.clone(),
        },
        "恢复按 F2 退回见证到的 P、读回 P 的内容"
    );
    let compared = fields
        .compared_version
        .expect("恢复落到了一条根：有拿来比的那一版");
    assert_eq!(
        Some((compared.root.instance, compared.root.checkpoint_txg)),
        fields.report.effective_root,
        "拿来比的那一版就是恢复报告里那一版"
    );
    assert_eq!(
        compared.content,
        PosixContentRead::File(recorded.previous_content.clone()),
        "拿来比的那一版读出的内容与恢复报告读出的相同"
    );
}

/// 比恢复落到的那一版之前那一版：读字段不 panic，拿来比的那一版比恢复退到的 P 旧。
fn assert_the_version_before_the_recovered_one_is_older_than_the_witnessed_version(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let fields = read_recovery_fields(&devices, ComparedVersion::TheVersionBeforeTheRecoveredOne);
    assert_eq!(
        fields.report.effective_root,
        Some(recorded.previous_key),
        "恢复按 F2 退回见证到的 P"
    );
    let compared = fields
        .compared_version
        .expect("环里有比 P 更旧的根：有拿来比的那一版");
    assert!(
        (compared.root.checkpoint_txg, compared.root.instance)
            < (recorded.previous_key.1, recorded.previous_key.0),
        "拿来比的是恢复退到的 P 之前那一版，不是被退掉的 R 之前那一版（P）：{:?}",
        (compared.root.instance, compared.root.checkpoint_txg)
    );
}

/// 判别力：读字段拿来比的那一版不取恢复报告的那一版、换成环里最新那一条根时（变异「读字段拿环里最新那一条根当恢复落到的那一版」）这一条红。
#[test]
fn reading_the_recovered_version_of_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_compares_the_witnessed_version(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (image, ranges) =
        confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero(&recorded);
    assert_the_recovered_version_read_is_the_witnessed_version(&recorded, &image, &ranges);
}

/// 判别力：同上一条的变异。
#[test]
fn reading_the_version_before_the_recovered_one_of_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_compares_the_version_before_the_witnessed_one(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (image, ranges) =
        confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero(&recorded);
    assert_the_version_before_the_recovered_one_is_older_than_the_witnessed_version(
        &recorded, &image, &ranges,
    );
}

/// 判别力：同上。
#[test]
fn reading_the_recovered_version_of_an_unwitnessed_root_whose_units_read_back_zero_compares_the_witnessed_version(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (image, ranges) = unwitnessed_root_whose_units_read_back_zero(&recorded);
    assert_the_recovered_version_read_is_the_witnessed_version(&recorded, &image, &ranges);
}

/// 判别力：同上。
#[test]
fn reading_the_version_before_the_recovered_one_of_an_unwitnessed_root_whose_units_read_back_zero_compares_the_version_before_the_witnessed_one(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let (image, ranges) = unwitnessed_root_whose_units_read_back_zero(&recorded);
    assert_the_version_before_the_recovered_one_is_older_than_the_witnessed_version(
        &recorded, &image, &ranges,
    );
}
