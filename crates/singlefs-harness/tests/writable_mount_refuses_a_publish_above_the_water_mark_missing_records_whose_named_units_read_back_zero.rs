//! F1 乙（用户 2026-10-07 定「缺记录就拒可写」；判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条）：可写挂载时水位之上一次
//! 没施加的发布，读得出的那几条记录的「本次发布内序号」不是 1..N 连着（I-8.9）或没有一条带末条标志，而它们点名的单元又验不过，
//! 重读一次仍这样就在取号之前拒可写（`StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`），
//! 盘上逐字节不变；读得出的记录点名的单元验得过（正常崩溃在写记录中途）不拒。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次三条记录的发布 R（顺序写三个数据单元），
//! 崩在 R 的根槽落盘之后、系统配置轮换之前；挂载时 R 的根槽、R 写出的每个单元与 R 的某一条记录读回全零（第二轮攻方 2.2 那三种藏法：
//! 中间一条 `urM`、第一条 `urF`、末条 `urL`）。这三形上 C 看不见缺的那条记录点名了哪几个单元（判别子观测不到），不拒的话那几个单元
//! 第一次覆盖写就被写掉、R 还在根环里（攻方三种藏法各 48 / 73 结点 I-7.4 红）。
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

/// R 的根槽、R 写出的每个单元与 R 的第 `hidden_record` 条记录读回全零时挂一次可写：拒成 F1 那一个成员、点名 R，盘上逐字节不变。
fn assert_the_writable_mount_is_refused_before_any_write_when_record_reads_back_zero(
    hidden_record: usize,
) {
    let recorded = recorded_publish_of_three_records();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&recorded.record_writes_of_the_record(hidden_record));
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect_err("R 缺一条记录、读得出的记录点名的单元验不过：重读一次仍这样，拒可写");
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
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes) == crash_image,
        "在取号之前拒：两块盘逐字节不变，R 的单元一个都没被写掉"
    );
}

/// `urM`：中间那一条读回全零，读得出的序号是 1、3（不连着）。判别力：判法不认序号缺口（变异「F1 不认序号缺口」）时不拒，这一条红。
#[test]
fn a_publish_whose_middle_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write(
) {
    assert_the_writable_mount_is_refused_before_any_write_when_record_reads_back_zero(2);
}

/// `urF`：第一条读回全零，读得出的序号是 2、3（不从 1 起）。判别力：同上一条（变异「F1 不认序号缺口」）。
#[test]
fn a_publish_whose_first_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write(
) {
    assert_the_writable_mount_is_refused_before_any_write_when_record_reads_back_zero(1);
}

/// `urL`：末条读回全零，读得出的序号 1、2 连着而没有一条带末条标志。判别力：判法不认缺末条（变异「F1 不认缺末条」）时不拒，这一条红。
#[test]
fn a_publish_whose_last_record_and_units_read_back_zero_refuses_the_writable_mount_before_any_write(
) {
    assert_the_writable_mount_is_refused_before_any_write_when_record_reads_back_zero(3);
}

/// 正常崩溃在写记录中途：R 的单元都落了（单元在记录之前落盘，D16（发布语义） 已定项 7），记录只落了前两条，根没落、轮换没落。
/// 读得出的两条序号 1、2、没有末条，而它们点名的单元验得过：不拒，可写挂载照常做成（R 整次不施加，所选那一版是 P）。
/// 判别力：判法不看点名的单元验不验得过（变异「F1 不看点名单元」）时这里也拒，这一条红。
#[test]
fn a_power_loss_while_writing_the_records_of_a_publish_whose_units_verify_still_mounts_writable() {
    let recorded = recorded_publish_of_three_records();
    let first_two_records: Vec<&RetainedWrite> = recorded
        .record_writes_of_the_record(1)
        .into_iter()
        .chain(recorded.record_writes_of_the_record(2))
        .collect();
    let mut crash_image = recorded.base.clone();
    let persisted: Vec<RetainedWrite> = recorded
        .writes
        .iter()
        .filter(|write| {
            write.kind == StepKind::UnitWrite
                || first_two_records
                    .iter()
                    .any(|record_write| record_write == write)
        })
        .cloned()
        .collect();
    crash_image.apply_writes(&persisted);
    let mut devices =
        devices_reading_back_all_zero_on(&crash_image, &RangesReadingBackAllZero::default());
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("崩在写记录中途、读得出的记录点名的单元验得过：可写挂载照常做成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的末条没落：R 整次不施加，所选那一版是 P"
    );
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

/// 代码轮攻方 T4 那一形（`research/prompts/m3-batch33t-code-r1-opus-output.md` 第二节第 2 小节）：写记录中途断电、根从没落，第一条记录点名的一个单元
/// 只有一份（盘 0 那一份）读回全零——单故障，另一份验得过。F1 判「点名单元验不过」要某个点名单元两份都验不过（`F1_ANY`），这里不拒，可写挂载做成、
/// 所选那一版是 P。判别力：退回两份都要验过（变异「F1 点名单元一份坏就算验不过」）时每次可写挂载都拒、没有出口，这一条红。
#[test]
fn a_power_loss_while_writing_the_records_with_one_copy_of_a_named_unit_reading_back_zero_still_mounts_writable(
) {
    let recorded = recorded_publish_of_three_records();
    let crash_image = crash_image_with_only_the_first_record_of_the_publish(&recorded);
    let both_copies = unit_writes_of_the_first_unit_named_by_the_first_record(&recorded);
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&both_copies[..1]);
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("点名的单元只坏一份、另一份验得过：F1 不拒，可写挂载做成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 只落了第一条记录：R 整次不施加，所选那一版是 P"
    );
}

/// 同上一形，那个点名单元两份都读回全零：缺记录、点名的单元两份都验不过，重读一次仍这样，在取号之前拒可写，盘上逐字节不变。
/// 判别力：`F1_ANY` 那一份判法恒交「验得过」（变异「F1 点名单元恒验得过」）时两份都坏也不拒，这一条红。
#[test]
fn a_power_loss_while_writing_the_records_with_both_copies_of_a_named_unit_reading_back_zero_refuses_the_writable_mount_before_any_write(
) {
    let recorded = recorded_publish_of_three_records();
    let crash_image = crash_image_with_only_the_first_record_of_the_publish(&recorded);
    let both_copies = unit_writes_of_the_first_unit_named_by_the_first_record(&recorded);
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&both_copies);
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect_err("缺记录、点名的单元两份都验不过：重读一次仍这样，拒可写");
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
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes) == crash_image,
        "在取号之前拒：两块盘逐字节不变"
    );
}
