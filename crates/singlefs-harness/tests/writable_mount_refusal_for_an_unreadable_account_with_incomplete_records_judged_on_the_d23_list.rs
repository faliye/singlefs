//! 可写挂载的 C 读一次没施加的发布、账读不出、读得出的记录不齐、从中央映射也现算不成的那一拒（`C_MAPPING` 收窄，判决
//! `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K2 W1 / W2 H1、K2 W2 H3 两行）另起错误成员
//! （`StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`，用户 2026-10-08 定「乙：core 另起错误成员」），
//! 列进 D23（journal 的角色与格式） 已定项 14 那张单子（`RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`）；
//! 判法在崩溃镜像上复算判据（主 agent 定，被攻过零轮）：点名的是水位之上没施加的发布（或较新的有效可读根），它读得出的记录不齐，
//! 它的账两份副本都验不过（账根经新根段里的树表指针找）。「映射现算得出」那一半镜像上不复算，这一格只判必要条件。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次三条记录的发布 R（顺序写三个数据单元），
//! 崩在 R 的根槽落盘之后、系统配置轮换之前；崩溃镜像上 R 的根槽、R 的树表与中央映射树根两份盘面写成全零（不是读回全零那一层）。
//! R 的账按新根段里的树表指针找：树表两份都验不过就读不到账根（新根段里那条分配记录树根在带文件的一版上恒是空指针，
//! 判据怎么读见 `mount_transparency::account_unreadable_on_the_crash_image` 的注释，交主 agent 定）。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
    RecordedPublish,
};
use singlefs_core::mount::{
    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
};
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
    RefusalConditionNotMet, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
};

const OVERWRITES_BEFORE_R: u64 = 5;

fn recorded_publish_of_three_records() -> RecordedPublish {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    recorded
}

/// R 的根槽、树表与中央映射树根的写（两棵树的根两块盘各一份）：R 不在根环里可读、R 的账（经树表读）读不出、映射也现算不成。
fn root_slot_tree_table_and_mapping_root_writes_of_r(
    recorded: &RecordedPublish,
) -> Vec<&RetainedWrite> {
    let root_of_r = recorded.root();
    let mut writes = vec![recorded.root_slot_write()];
    for (pointer, what) in [
        (&root_of_r.tree_table, "树表"),
        (&root_of_r.mapping_root, "中央映射树根"),
    ] {
        let unit_writes = recorded.unit_writes_at(pointer);
        assert_eq!(unit_writes.len(), 2, "R 自己写了{what}，两块盘各一份");
        writes.extend(unit_writes);
    }
    writes
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

/// 崩溃镜像：R 的根槽、树表与映射根全零，末条（第 3 条，两份）也全零——读得出的第 1、2 条缺末条，记录不齐。
fn crash_image_with_the_last_record_account_and_mapping_of_r_zeroed(
    recorded: &RecordedPublish,
) -> MemoryPool {
    let mut zeroed_writes = root_slot_tree_table_and_mapping_root_writes_of_r(recorded);
    zeroed_writes.extend(recorded.record_writes_of_the_record(3));
    image_with_the_writes_zeroed(
        &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
        &zeroed_writes,
    )
}

/// 在 `crash_image` 上起一次可写挂载（不带读回全零那一层）：被拒，盘上逐字节不变；交回那一拒。
fn refusal_of_the_writable_mount_on(
    recorded: &RecordedPublish,
    crash_image: &MemoryPool,
) -> MountError {
    let mut devices =
        devices_reading_back_all_zero_on(crash_image, &RangesReadingBackAllZero::default());
    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect_err("R 的账与映射都读不出、记录不齐：重读一次仍这样，拒可写");
    assert!(
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes)
            == *crash_image,
        "在取号之前拒：两块盘逐字节不变"
    );
    refusal
}

/// 崩溃镜像上 R 缺末条、树表与映射根两份都是全零：C 读 R 的账读不出、记录不齐、映射也现算不成，拒成另起的那一个成员、点名 R
/// （不是 C393 那一拒的 `AccountOfAnAbandonedRoot`）。
/// 判别力：那一拒照旧报被抛弃根那一个成员（变异「删除线第43批 C_MAPPING 那一拒照旧报被抛弃根那一个成员」）时这一条红。
#[test]
fn a_publish_whose_account_and_mapping_are_zero_while_its_last_record_is_missing_is_refused_with_its_own_error_member(
) {
    let recorded = recorded_publish_of_three_records();
    let crash_image = crash_image_with_the_last_record_account_and_mapping_of_r_zeroed(&recorded);
    let refusal = refusal_of_the_writable_mount_on(&recorded, &crash_image);
    assert!(
        matches!(
            &refusal,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if matches!(
                    still_unreadable.as_ref(),
                    StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                        unapplied_publish: RollbackTarget { instance, checkpoint_txg },
                    } if (*instance, *checkpoint_txg) == recorded.key
                )
        ),
        "拒因是 C_MAPPING 那一个成员、点名 R {:?}：{refusal:?}",
        recorded.key
    );
}

/// 同一份崩溃镜像、同一拒：判法把它归到单子上，在崩溃镜像上复算判据为真（R 在水位之上没施加，读得出的第 1、2 条缺末条，
/// R 的树表两份都验不过、读不到账根），盘上逐字节不变、只读读出在允许集合里，拒得合法。
/// 判别力：新成员仍映成不在单子上（变异「删除线第43批 C_MAPPING 那一拒仍映成不在 D23 单子上」）时判成 `ReasonNotOnTheD23List`，这一条红。
#[test]
fn a_refusal_for_an_unreadable_account_with_incomplete_records_is_legal_on_the_d23_list_when_the_crash_image_bears_it_out(
) {
    let recorded = recorded_publish_of_three_records();
    let crash_image = crash_image_with_the_last_record_account_and_mapping_of_r_zeroed(&recorded);
    let refusal = refusal_of_the_writable_mount_on(&recorded, &crash_image);
    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
        .report
        .effective_root;
    assert_eq!(
        selected_version,
        Some(recorded.previous_key),
        "R 的根槽全零、缺末条：R 整次不施加，崩溃镜像上只读恢复落到 P"
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
            RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                unapplied_publish: recorded.key,
            }
        )
    );
}

/// 崩溃镜像上 R 的三条记录都在（序号 1、2、3，末条带标志），根槽、树表与映射根两份盘面全零：账读不出，但记录齐。
/// 拿一次报 `C_MAPPING` 那一拒的拒去判（实现在这份盘面上不走那一支：记录齐的交 F1 乙；判法不信拒因、自己复算），判据为假，拒得不合法。
/// 判别力：复算不看记录齐不齐（变异「删除线第43批 C_MAPPING 判据复算不看记录齐不齐」）时只剩「水位之上」与「账验不过」、判成合法，这一条红。
#[test]
fn a_refusal_claimed_for_an_unreadable_account_with_incomplete_records_is_illegal_on_a_crash_image_holding_every_record_of_the_publish(
) {
    let recorded = recorded_publish_of_three_records();
    let crash_image = image_with_the_writes_zeroed(
        &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
        &root_slot_tree_table_and_mapping_root_writes_of_r(&recorded),
    );
    let selected_version = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion)
        .report
        .effective_root;
    assert_eq!(
        selected_version,
        Some(recorded.previous_key),
        "R 的根槽全零、末条点名的树表验不过：R 不施加，只读恢复落到 P"
    );
    let claimed_refusal = MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
            unapplied_publish: RollbackTarget {
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
            reason: RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                unapplied_publish: recorded.key,
            },
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        }
    );
}
