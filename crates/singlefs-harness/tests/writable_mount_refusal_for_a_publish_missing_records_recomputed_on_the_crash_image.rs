//! 「记录有缺口时拒可写」（D23（journal 的角色与格式） 已定项 14，F1 乙）那一拒在拒绝合法判法里归到单子上
//! （`singlefs_harness::mount_transparency::RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`），
//! 判据由判法在崩溃镜像上自己复算（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T4 那一行）：水位之上某次没施加的发布
//! 读得出的记录本次发布内序号不连续或缺末条、且某个点名单元两份副本都验不过。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次三条记录的发布 R（顺序写三个数据单元）。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
    RecordedPublish,
};
use singlefs_core::journal::JournalRecord;
use singlefs_core::mount::{
    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
    RefusalConditionNotMet, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
};
use singlefs_harness::segments::StepKind;

const OVERWRITES_BEFORE_R: u64 = 5;

fn recorded_publish_of_three_records() -> RecordedPublish {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    recorded
}

/// 写记录中途断电、只落了第一条记录（两份）：R 的单元都落了（单元在记录之前落盘），第二、三条记录、根槽与轮换都没落。
fn crash_image_with_only_the_first_record_of_the_publish(recorded: &RecordedPublish) -> MemoryPool {
    let first_record = recorded
        .writes
        .iter()
        .position(|write| write.kind == StepKind::JournalRecord)
        .expect("R 写了记录");
    let mut crash_image = recorded.base.clone();
    crash_image.apply_writes(&recorded.writes[..first_record]);
    let first_record_writes: Vec<RetainedWrite> = recorded
        .record_writes_of_the_record(1)
        .into_iter()
        .cloned()
        .collect();
    crash_image.apply_writes(&first_record_writes);
    crash_image
}

/// R 第一条记录点名的第一个单元的两份写（两块盘各一份，按盘序）。
fn unit_writes_of_the_first_unit_named_by_the_first_record(
    recorded: &RecordedPublish,
) -> Vec<&RetainedWrite> {
    let first_record = JournalRecord::parse(
        recorded.record_writes_of_the_record(1)[0]
            .bytes()
            .expect("记录写带字节"),
        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier),
    )
    .expect("R 的第一条记录解得开");
    let named = first_record
        .named
        .first()
        .expect("R 的第一条记录点名了单元");
    named
        .locations
        .iter()
        .map(|location| {
            recorded
                .unit_writes()
                .into_iter()
                .find(|write| {
                    write.device == location.device
                        && write.offset == location.slot.to_device_offset()
                })
                .expect("点名的单元是 R 写出的")
        })
        .collect()
}

/// `image` 上 `writes` 那几段盘面真的写成全零（不是读回全零的那一层：崩溃镜像本身就是这样）。
fn image_with_the_writes_zeroed(image: &MemoryPool, writes: &[&RetainedWrite]) -> MemoryPool {
    let mut zeroed = image.clone();
    for write in writes {
        let length = usize::try_from(write.length_in_bytes()).expect("一次写的长度装得进 usize");
        zeroed
            .devices
            .get_mut(&write.device)
            .expect("写落在池里的一块盘上")
            .write(write.offset, &vec![0u8; length]);
    }
    zeroed
}

/// 可写挂载在 `crash_image` 上读到 `ranges` 那几段全零：拒成 F1 那一个成员、点名 R，盘上逐字节不变；交回那一拒。
fn refusal_of_the_writable_mount_for_the_missing_records_of_r(
    recorded: &RecordedPublish,
    crash_image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> MountError {
    let mut devices = devices_reading_back_all_zero_on(crash_image, ranges);
    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect_err("R 缺记录、读到的点名单元两份都验不过：重读一次仍这样，拒可写");
    assert!(
        matches!(
            &refusal,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if matches!(
                    still_unreadable.as_ref(),
                    StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
                        publish: RollbackTarget { instance, checkpoint_txg },
                    } if (*instance, *checkpoint_txg) == recorded.key
                )
        ),
        "拒因是 F1 那一个成员、点名 R {:?}：{refusal:?}",
        recorded.key
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes)
            == *crash_image,
        "在取号之前拒：两块盘逐字节不变"
    );
    refusal
}

/// 崩溃镜像上 R 只落了第一条记录，它点名的一个单元两份盘面都是全零：可写挂载拒成 F1，判法把这一拒归到单子上、在崩溃镜像上复算判据为真
/// （读得出的序号只有 1、没有末条；那个点名单元两份都验不过），盘上逐字节不变、只读读出在允许集合里，拒得合法。
/// 判别力：F1 那一拒照旧归「不在单子上」（变异「F1 的拒照旧归到不在 D23 单子上」）时判成 `ReasonNotOnTheD23List`，这一条红。
#[test]
fn a_refusal_for_a_publish_missing_records_whose_named_unit_fails_on_both_copies_of_the_crash_image_is_legal(
) {
    let recorded = recorded_publish_of_three_records();
    let both_copies = unit_writes_of_the_first_unit_named_by_the_first_record(&recorded);
    let crash_image = image_with_the_writes_zeroed(
        &crash_image_with_only_the_first_record_of_the_publish(&recorded),
        &both_copies,
    );
    let refusal = refusal_of_the_writable_mount_for_the_missing_records_of_r(
        &recorded,
        &crash_image,
        &RangesReadingBackAllZero::default(),
    );
    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
        .report
        .effective_root;
    assert_eq!(
        selected_version,
        Some(recorded.previous_key),
        "R 整次不施加：崩溃镜像上只读恢复落到 P"
    );
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &crash_image,
            selected_version,
            &refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
        ),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify
        )
    );
}

/// 同一拒（可写挂载读到那个点名单元两份都是全零），崩溃镜像上那两份却验得过：判据在崩溃镜像上复算为假，判法照报拒得不合法。
/// 判别力：复算不看点名单元验不验得过（变异「F1 判据复算不看点名单元」）时只剩「缺记录」一半、判成合法，这一条红。
#[test]
fn a_refusal_for_a_publish_missing_records_is_illegal_on_a_crash_image_whose_named_units_verify() {
    let recorded = recorded_publish_of_three_records();
    let crash_image = crash_image_with_only_the_first_record_of_the_publish(&recorded);
    let both_copies = unit_writes_of_the_first_unit_named_by_the_first_record(&recorded);
    let refusal = refusal_of_the_writable_mount_for_the_missing_records_of_r(
        &recorded,
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&both_copies),
    );
    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
        .report
        .effective_root;
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &crash_image,
            selected_version,
            &refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
        ),
        WritableMountRefusalVerdict::Illegal {
            reason:
                RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        }
    );
}

/// 崩溃镜像上 R 的三条记录都在（序号 1、2、3，末条带标志），根槽与 R 写出的单元两份盘面都是全零：点名的单元验不过，但记录没有缺口。
/// 拿一次报 F1 的拒去判（实现在这份盘面上不报它：判法不信拒因、自己复算），判据为假，拒得不合法。
/// 判别力：复算不看序号与末条（变异「F1 判据复算不看记录缺口」）时只剩「点名单元验不过」一半、判成合法，这一条红。
#[test]
fn a_refusal_claimed_for_missing_records_is_illegal_on_a_crash_image_holding_every_record_of_the_publish(
) {
    let recorded = recorded_publish_of_three_records();
    let mut zeroed_writes = recorded.unit_writes();
    zeroed_writes.push(recorded.root_slot_write());
    let crash_image = image_with_the_writes_zeroed(
        &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
        &zeroed_writes,
    );
    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
        .report
        .effective_root;
    assert_eq!(
        selected_version,
        Some(recorded.previous_key),
        "R 的根槽全零、点名的单元验不过：R 不施加，只读恢复落到 P"
    );
    let claimed_refusal = MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
            publish: RollbackTarget {
                instance: recorded.key.0,
                checkpoint_txg: recorded.key.1,
            },
        },
    ));
    assert_eq!(
        judge_writable_mount_refusal_of_a_crash_state(
            &crash_image,
            selected_version,
            &claimed_refusal,
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            ReadFaultsInTheCrashState::NoneInjected,
        ),
        WritableMountRefusalVerdict::Illegal {
            reason:
                RefusalReasonOnTheD23List::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        }
    );
}
