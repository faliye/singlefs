//! F1 乙按 F2 退到的那一版再判一次（`F1_AFTER_F2`，判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H2 那一行与第三节第 1 条，
//! 用户 2026-10-08 定「四形都修」）：可写挂载读阶段判 F1 时的水位是退之前的所选那一版；所选根读得出、走读失败、F2 退回见证到的那一版之后，
//! 被退掉的那次发布到了水位之上，按退到的那一版再判一次 F1（读得出的记录序号不连续或缺末条、点名的单元有一个两份都验不过 ⇒ 重读一次仍这样就拒）。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`，攻方 `research/prompts/m3-batch35-36-code-r1-opus-output.md` 第二节那一形）：384 槽小盘、
//! 第一个文件之后覆盖写 5 次，再录一次三条记录的顺序写 R（每个数据单元一条记录，末条另点名全部元数据单元），崩在 R 的根槽落盘之后、
//! 系统配置轮换之前；取新实例的那次可写挂载 M1 里 R 的根槽读得出，R 的某几条记录与 R 写出的某一个单元读回全零。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions,
    record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
    RecordedPublish,
};
use singlefs_core::journal::JournalRecord;
use singlefs_core::mount::{
    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_harness::memory_pool::RetainedWrite;
use singlefs_harness::segments::StepKind;

const OVERWRITES_BEFORE_R: u64 = 5;

/// 这一拒是 F1 乙、点名 R 那次发布。
fn is_the_refusal_for_missing_records_of_r(
    refusal: &MountError,
    recorded: &RecordedPublish,
) -> bool {
    matches!(
        refusal,
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
            if matches!(
                **still_unreadable,
                StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify {
                    publish: RollbackTarget { instance, checkpoint_txg }
                } if (instance, checkpoint_txg) == recorded.key
            )
    )
}

/// 攻方 H2 那一格：M1 里 R 的第 1 条记录与 R 的树表（末条点名的元数据单元，R 的账在它上面）读回全零，R 的根槽读得出。
/// R 被择、走读失败，系统配置没见证 R（末条读得出、计数器大于 c_见证），退回 P；按 P 当水位，R 读得出的是第 2、3 条（序号不从 1 起），
/// 第 3 条点名的树表两份都验不过 ⇒ 在取号之前拒可写，盘上逐字节不变。
/// 判别力：退回之后不再判 F1（变异「删除线第38批 F1_AFTER_F2 F2 退回之后不按退到的那一版再判 F1」）时 M1 做成、这一条红。
#[test]
fn a_root_falling_back_to_the_witnessed_version_with_its_first_record_and_tree_table_reading_back_zero_refuses_the_writable_mount_before_any_write(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    let tree_table_writes = recorded.unit_writes_at(&recorded.root().tree_table);
    assert_eq!(tree_table_writes.len(), 2, "R 自己写了树表，两块盘各一份");
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded.record_writes_of_the_record(1))
        .with_the_writes(&tree_table_writes);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let refusal = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect_err("退回 P 之后 R 缺第 1 条、末条点名的树表两份都验不过：拒可写");
    assert!(
        is_the_refusal_for_missing_records_of_r(&refusal, &recorded),
        "拒因是 F1 乙、点名 R：{refusal:?}"
    );
    assert!(
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes) == crash_image,
        "拒在任何写之前：两块盘逐字节不变"
    );
}

/// 攻方 H2 那一族逐格：R 的记录子集 {1}、{2}、{1,2} 读回全零 × R 写出的每一个单元逐个读回全零（两份），R 的根槽读得出。
/// 每一格三种结局之一，各有各的前提：退回 P 之后被 F1 乙拒（点名 R、拒在任何写之前）——读阶段之后那个单元被读得出的某条记录点名；
/// 退回 P 又做成——那个单元不被读得出的任何一条记录点名（F1 乙只看读得出的记录点名的单元）；不退、挂在 R 上（R 走得通）。
/// 做成的两种都核 R 的单元一份不少。打印每一格的结局与合计（这一版量到 39 格：被 F1 乙拒 30、退回 P 又做成 0、挂在 R 上 9）。
/// 判别力：退回之后不再判 F1（变异「删除线第38批 F1_AFTER_F2 F2 退回之后不按退到的那一版再判 F1（缺记录、逐个单元读回全零那一族）」）时
/// 那 30 格退回 P 又做成、点名单元被读得出的记录点名，这一条红。
#[test]
fn every_unit_of_a_root_falling_back_with_a_gap_in_its_records_refuses_the_writable_mount_exactly_when_a_readable_record_names_it(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut outcomes: Vec<String> = Vec::new();
    for hidden_records in [vec![1], vec![2], vec![1, 2]] {
        for (unit_index, unit_copies) in recorded.unit_writes_grouped_by_unit().iter().enumerate() {
            let cell = format!("记录 {hidden_records:?} 与 R 的第 {unit_index} 个单元读回全零");
            let record_writes: Vec<&RetainedWrite> = hidden_records
                .iter()
                .flat_map(|ordinal| recorded.record_writes_of_the_record(*ordinal))
                .collect();
            let ranges = RangesReadingBackAllZero::default()
                .with_the_writes(&record_writes)
                .with_the_writes(unit_copies);
            let named_by_a_readable_record = unit_is_named_by_a_record_not_reading_back_zero(
                &recorded,
                unit_copies,
                &hidden_records,
            );
            match mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
                &recorded,
                &crash_image,
                &ranges,
                &cell,
            ) {
                Ok(effective) if effective == recorded.key => {
                    outcomes.push(format!("{cell}：没退，挂在 R 上"));
                }
                Ok(effective) => {
                    assert_eq!(effective, recorded.previous_key, "{cell}：退的话退到 P");
                    assert!(
                        !named_by_a_readable_record,
                        "{cell}：退回 P 又做成的，那个单元不被读得出的记录点名（被点名就该被 F1 乙拒）"
                    );
                    outcomes.push(format!(
                        "{cell}：退回 P、做成（单元只被读回全零的记录点名）"
                    ));
                }
                Err(refusal) => {
                    assert!(
                        is_the_refusal_for_missing_records_of_r(&refusal, &recorded),
                        "{cell}：拒因是 F1 乙、点名 R：{refusal:?}"
                    );
                    assert!(
                        named_by_a_readable_record,
                        "{cell}：被 F1 乙拒的，那个单元被读得出的某条记录点名"
                    );
                    outcomes.push(format!("{cell}：退回 P 之后被 F1 乙拒"));
                }
            }
        }
    }
    println!("{}", outcomes.join("\n"));
    println!(
        "合计 {} 格：被 F1 乙拒 {}，退回 P、做成 {}，挂在 R 上 {}",
        outcomes.len(),
        outcomes
            .iter()
            .filter(|outcome| outcome.contains("被 F1 乙拒"))
            .count(),
        outcomes
            .iter()
            .filter(|outcome| outcome.contains("退回 P、做成"))
            .count(),
        outcomes
            .iter()
            .filter(|outcome| outcome.contains("挂在 R 上"))
            .count()
    );
}

/// R 写出的这个单元（`unit_copies`，两块盘的两份）被 R 的某条没读回全零的记录（`hidden_records` 以外的）点名。
fn unit_is_named_by_a_record_not_reading_back_zero(
    recorded: &RecordedPublish,
    unit_copies: &[&RetainedWrite],
    hidden_records: &[usize],
) -> bool {
    let filesystem_identifier =
        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier);
    (1..=recorded.record_count())
        .filter(|ordinal| !hidden_records.contains(ordinal))
        .any(|ordinal| {
            let record = JournalRecord::parse(
                recorded.record_writes_of_the_record(ordinal)[0]
                    .bytes()
                    .expect("记录写带字节"),
                filesystem_identifier,
            )
            .expect("R 的记录解得开");
            record.named.iter().any(|named| {
                named.locations.iter().any(|location| {
                    unit_copies.iter().any(|copy| {
                        copy.device == location.device
                            && copy.offset == location.slot.to_device_offset()
                    })
                })
            })
        })
}

/// 正常崩溃照不拒：崩在 R 写记录的中途（第 1 条两份都落了、第 2 条还没写），一处读故障都没有。R 的根槽没落，所选那一版是 P，
/// 读得出的第 1 条缺末条、点名的数据单元验得过（单元在记录之前落盘）：F1 乙不拒，可写挂载做成；R 走不到 F2，`F1_AFTER_F2` 不判。
#[test]
fn a_power_loss_after_the_first_record_of_a_three_record_publish_still_mounts_writable() {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    let first_record_offset = recorded.record_writes_of_the_record(1)[0].offset;
    let last_write_of_the_first_record = recorded
        .writes
        .iter()
        .rposition(|write| {
            write.kind == StepKind::JournalRecord && write.offset == first_record_offset
        })
        .expect("R 写了第 1 条记录");
    let mut crash_image = recorded.base.clone();
    crash_image.apply_writes(&recorded.writes[..=last_write_of_the_first_record]);
    let mut devices =
        devices_reading_back_all_zero_on(&crash_image, &RangesReadingBackAllZero::default());
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("崩在写记录中途、没有读故障：可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽没落：所选那一版是 P"
    );
}
