//! 可写挂载时的预隔离（C）再收严两处（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 H1、H3 两行与第三节第 1 条，
//! 用户 2026-10-08 定「四形都修」）：
//! - `C_NEWER_ROOTS`：C 并上根环里比所选那一版新、按实例表有效、读得出的根自己的账（照影子账同一份读法）。F2 退回之后被退掉的那条根 R
//!   就是这样：影子账只管被抛弃的根、不管它，它的记录又全读不出时，C 那两份（记录点名的、记录拼根读的账）都是空的。
//! - `C_MAPPING`（收窄）：C 读一次没施加的发布（记录拼的根，或上面那条较新的根）的账读不出时，只在那次发布读得出的记录不齐（序号缺一条、
//!   缺末条）时照被抛弃根那一份从中央映射现算账里的落点；仍算不出就在取号之前拒可写（拒因另起一个成员，与 C393 那一拒分开报）。
//!   记录齐的不走这一支（照被抛弃根那一份不收窄会在 `ur` 那几形上误拒，攻方量到入库 7 条）。较新的根那次发布一条记录都读不出时：
//!   映射现算得出就并进来；现算不成时只看那条根自己的树表与中央映射树根两个指针（C834，用户 2026-10-15 定「同一指针两份都更新」，
//!   `research/prompts/c834-r3-main-verification.md`「四、用户定案」；替换攻方 `research/prompts/m3-batch37-43-code-r1-opus-output.md`
//!   第一节 Y2 那一判）：有一个指针两条位置条目指着的两个槽都读出比那条根新的实例写的自证单元头
//!   （`mount.rs` 的 `one_pointer_has_both_copies_overwritten_by_a_later_instance`）就不拒（入库 `urJ` 那一形就是这样，
//!   拒了它就再也做不成可写挂载），否则照记录不齐那一支办。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`，攻方 `research/prompts/m3-batch35-36-code-r1-opus-output.md` 第一、三节那几形）：384 槽小盘、
//! 第一个文件之后覆盖写 5 次，再录一次发布 R（覆盖写一次、一条记录；或顺序写三个数据单元、三条记录），崩在 R 的根槽落盘之后、系统配置轮换之前；
//! 取新实例的那次可写挂载 M1 里几段读回全零。每格核：M1 做成时写行那次发布的根落盘之前 R 的单元一份不少，M1 之后覆盖写、建 inode、
//! 顺序写各一次，R 的根还在根环里时 R 的单元一份不少；M1 被拒时拒在任何写之前。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions,
    record_one_publish_after_overwrites, PublishRecordedAsR, RangesReadingBackAllZero,
    RecordedPublish,
};
use singlefs_core::mount::{MountError, RollbackTarget, StillUnreadableAfterOneReread};
use singlefs_harness::memory_pool::RetainedWrite;

/// 覆盖写几次之后录 R：攻方第一、三节都取 k = 5。
const OVERWRITES_BEFORE_R: u64 = 5;

/// R 的每一条记录（两块盘各一份）。
fn every_record_of_r(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    (1..=recorded.record_count())
        .flat_map(|ordinal| recorded.record_writes_of_the_record(ordinal))
        .collect()
}

/// R 自己写的那个单元（`pointer` 指着的），两块盘各一份。
fn unit_writes_of_r_at<'recorded>(
    recorded: &'recorded RecordedPublish,
    pointer: &singlefs_core::pointer::NodePointer,
    what: &str,
) -> Vec<&'recorded RetainedWrite> {
    let writes = recorded.unit_writes_at(pointer);
    assert_eq!(writes.len(), 2, "R 自己写了{what}，两块盘各一份");
    writes
}

/// `C_NEWER_ROOTS`（攻方 H1 最小形）：R 是一条记录的覆盖写，M1 里 R 的中央映射树根与 R 那条记录读回全零，R 的根槽读得出。
/// R 被择、走读失败，系统配置没见证 R（末条读不出，计数器 = c_见证 的那条是 P 的），退回 P；R 的账（树表、分配记录树）读得出。
/// 判别力：C 不并较新可读根的账（变异「删除线第38批 C_NEWER_ROOTS C 不并较新可读根的账（覆盖写 R 的映射根与记录读回全零）」）时
/// C 是空的，写行那次发布的根落盘之前 M1 就写掉 R 的单元（攻方量到 14 份），这一条红。
#[test]
fn a_newer_readable_root_whose_mapping_root_and_only_record_read_back_zero_keeps_its_units_through_the_mount_and_three_user_actions(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let root_of_r = recorded.root();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&unit_writes_of_r_at(
            &recorded,
            &root_of_r.mapping_root,
            "中央映射树根",
        ))
        .with_the_writes(&every_record_of_r(&recorded));
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let effective =
        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
            &recorded,
            &crash_image,
            &ranges,
            "覆盖写 R、映射根与记录读回全零",
        )
        .expect("R 的映射根与记录读回全零：可写挂载退到 P、做得成");
    assert_eq!(
        effective, recorded.previous_key,
        "R 走读失败、系统配置没见证它，所选那一版退到 P"
    );
}

/// `C_NEWER_ROOTS`（攻方 H1 的 sw3 那一形）：R 是三条记录的顺序写，M1 里 R 的中央映射树根与三条记录都读回全零，R 的根槽读得出、账读得出。
/// 判别力：同上那一条变异的 sw3 那一行（「……（顺序写 R 的映射根与三条记录读回全零）」）时这一条红。
#[test]
fn a_newer_readable_root_of_three_records_whose_mapping_root_and_every_record_read_back_zero_keeps_its_units_through_the_mount_and_three_user_actions(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    let root_of_r = recorded.root();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&unit_writes_of_r_at(
            &recorded,
            &root_of_r.mapping_root,
            "中央映射树根",
        ))
        .with_the_writes(&every_record_of_r(&recorded));
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let effective =
        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
            &recorded,
            &crash_image,
            &ranges,
            "顺序写 R、映射根与三条记录读回全零",
        )
        .expect("R 的映射根与三条记录读回全零：可写挂载退到 P、做得成");
    assert_eq!(effective, recorded.previous_key, "所选那一版退到 P");
}

/// `C_NEWER_ROOTS` 连同 `C_MAPPING`（攻方 H1 里账读不出的那几格）：R 是一条记录的覆盖写，M1 里 R 的树表与那条记录读回全零，R 的根槽读得出。
/// 退回 P 之后 R 是较新的可读根，它的账（经树表读）读不出，那次发布一条记录都读不出 ⇒ 映射现算得出就并进 C（这一格条款没写，
/// 取法见 `mount::placements_in_the_accounts_of_newer_readable_roots` 那一支的注释）。
/// 判别力：一条记录都读不出时不从映射现算（变异「删除线第38批 C_MAPPING 较新可读根一条记录都读不出时不从映射现算」）时 C 是空的、这一条红。
#[test]
fn a_newer_readable_root_whose_tree_table_and_only_record_read_back_zero_keeps_its_units_recomputed_from_the_mapping(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let root_of_r = recorded.root();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&unit_writes_of_r_at(
            &recorded,
            &root_of_r.tree_table,
            "树表",
        ))
        .with_the_writes(&every_record_of_r(&recorded));
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let effective =
        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
            &recorded,
            &crash_image,
            &ranges,
            "覆盖写 R、树表与记录读回全零",
        )
        .expect("R 的账读不出、映射现算得出：可写挂载退到 P、做得成");
    assert_eq!(effective, recorded.previous_key, "所选那一版退到 P");
}

/// `C_MAPPING`（攻方 H3 根藏的那一半）：R 是三条记录的顺序写，M1 里 R 的根槽、末条与 R 的树表读回全零。所选那一版是 P，读得出的第 1、2 条
/// 缺末条、点名的数据单元验得过（F1 乙不拒）；记录拼根读 R 的账读不出 ⇒ 记录不齐，从中央映射现算 R 引用的落点。
/// 判别力：没施加的发布的账读不出时不从映射现算（变异「删除线第38批 C_MAPPING 没施加的发布的账读不出时不从映射现算」）时末条单独点名的单元
/// （第三个数据单元与全部元数据单元）没人护，这一条红。
#[test]
fn a_publish_whose_root_slot_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    let root_of_r = recorded.root();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.record_writes_of_the_record(3))
        .with_the_writes(&unit_writes_of_r_at(
            &recorded,
            &root_of_r.tree_table,
            "树表",
        ));
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let effective =
        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
            &recorded,
            &crash_image,
            &ranges,
            "顺序写 R、根槽、末条与树表读回全零",
        )
        .expect("R 的账读不出、映射现算得出：可写挂载做得成");
    assert_eq!(
        effective, recorded.previous_key,
        "R 的根槽读不出，所选那一版是 P"
    );
}

/// `C_MAPPING`（攻方 H3 根读得出的那一半）：同上，只是 R 的根槽读得出。R 被择、走读失败、退回 P（末条读不出，计数器 = c_见证 的那条是 P 的），
/// F1 乙按 P 再判一次：读得出的第 1、2 条点名的数据单元验得过、不拒；R 既是没施加的发布、又是较新的可读根，两处都按映射现算。
#[test]
fn a_newer_readable_root_whose_last_record_and_tree_table_read_back_zero_keeps_its_units_recomputed_from_the_mapping(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    let root_of_r = recorded.root();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded.record_writes_of_the_record(3))
        .with_the_writes(&unit_writes_of_r_at(
            &recorded,
            &root_of_r.tree_table,
            "树表",
        ));
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let effective =
        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
            &recorded,
            &crash_image,
            &ranges,
            "顺序写 R、末条与树表读回全零、根槽读得出",
        )
        .expect("R 的账读不出、映射现算得出：可写挂载退到 P、做得成");
    assert_eq!(effective, recorded.previous_key, "所选那一版退到 P");
}

/// `C_MAPPING` 仍算不出就拒：同「根藏的那一半」，再加 R 的中央映射树根读回全零——账读不出、记录不齐、映射也现算不成，重读一次仍是这样，
/// 现行账（P 的）里也没有 C694 的 B 那道凭据（R 比 P 新），在取号之前拒可写，拒因是 `AccountOfAnUnappliedPublishWithIncompleteReadableRecords`、点名 R。
/// 判别力：现算不成也不拒（变异「删除线第38批 C_MAPPING 现算不成也不拒」）时可写挂载做成、这一条红。
#[test]
fn a_publish_whose_account_and_mapping_read_back_zero_while_its_last_record_is_missing_refuses_the_writable_mount_before_any_write(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    let root_of_r = recorded.root();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.record_writes_of_the_record(3))
        .with_the_writes(&unit_writes_of_r_at(
            &recorded,
            &root_of_r.tree_table,
            "树表",
        ))
        .with_the_writes(&unit_writes_of_r_at(
            &recorded,
            &root_of_r.mapping_root,
            "中央映射树根",
        ));
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let refusal =
        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
            &recorded,
            &crash_image,
            &ranges,
            "顺序写 R、根槽、末条、树表与映射根读回全零",
        )
        .expect_err("账与映射都读不出、记录不齐：可写挂载在取号之前拒");
    assert!(
        matches!(
            &refusal,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if matches!(
                    **still_unreadable,
                    StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                        unapplied_publish: RollbackTarget { instance, checkpoint_txg }
                    } if (instance, checkpoint_txg) == recorded.key
                )
        ),
        "拒因是 C 的映射那一支另起的成员、点名 R：{refusal:?}"
    );
}

/// 攻方 Y2（`research/prompts/m3-batch37-43-code-r1-opus-output.md` 第一节，`u8M+u9T` 那一格）：R 是一条记录的覆盖写，M1 里 R 那条记录、
/// R 的中央映射树根与树表读回全零，R 的根槽读得出。R 走读失败、系统配置没见证它，退回 P；R 是较新的可读根，账读不出、那次发布一条记录都读不出、
/// 映射也现算不成，R 的树表与中央映射树根两个指针指着的槽都读回全零、读不出自证单元头，没有一个指针两份都被比 R 新的实例盖掉（C834 的判法）
/// ⇒ 照记录不齐那一支办：重读一次仍这样、现行账（P 的）里没有
/// C694 的 B 那道凭据，在取号之前拒可写，拒因是 `AccountOfAnUnappliedPublishWithIncompleteReadableRecords`、点名 R，R 的单元一份不少。
/// 判别力：这一支不拒（变异「删除线第45批 Y2（C834 判法）较新可读根记录全读不出、映射现算不成、没有一个指针两份都被更新的实例盖掉也不拒」）时 M1 做成、
/// 写行那次发布的根落盘之前写掉 R 的单元（攻方量到 14 份），这一条红。
#[test]
fn a_newer_readable_root_whose_only_record_mapping_root_and_tree_table_read_back_zero_refuses_the_writable_mount_before_any_write(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let root_of_r = recorded.root();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&every_record_of_r(&recorded))
        .with_the_writes(&unit_writes_of_r_at(
            &recorded,
            &root_of_r.mapping_root,
            "中央映射树根",
        ))
        .with_the_writes(&unit_writes_of_r_at(
            &recorded,
            &root_of_r.tree_table,
            "树表",
        ));
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let refusal =
        mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
            &recorded,
            &crash_image,
            &ranges,
            "覆盖写 R、记录、映射根与树表读回全零、根槽读得出",
        )
        .expect_err(
            "R 的账与映射都读不出、记录一条都读不出、两个指针都没有两份被更新的实例盖掉：可写挂载在取号之前拒",
        );
    assert!(
        matches!(
            &refusal,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if matches!(
                    **still_unreadable,
                    StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                        unapplied_publish: RollbackTarget { instance, checkpoint_txg }
                    } if (instance, checkpoint_txg) == recorded.key
                )
        ),
        "拒因是 C 的映射那一支的成员、点名 R：{refusal:?}"
    );
}

/// 攻方 H1 那一族的覆盖写那一形逐格（`C_NEWER_ROOTS` 与 `C_MAPPING` 合用）：R 是一条记录的覆盖写，M1 里 R 那条记录与 R 写出的某一个单元
/// （逐个取，两块盘的两份）读回全零，R 的根槽读得出。每一格要么做成、R 的单元一份不少，要么在任何写之前拒。
#[test]
fn every_unit_of_a_newer_readable_root_of_one_record_reading_back_zero_with_its_record_keeps_its_units(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    assert_every_unit_reading_back_zero_with_every_record_keeps_the_units_of_r(&recorded);
}

/// 同上，R 是三条记录的顺序写，三条记录都读回全零（攻方第一节 sw3 `rec_hidden=111` 那一行）。
#[test]
fn every_unit_of_a_newer_readable_root_of_three_records_reading_back_zero_with_every_record_keeps_its_units(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_every_unit_reading_back_zero_with_every_record_keeps_the_units_of_r(&recorded);
}

/// R 写出的每一个单元逐个连同 R 的全部记录读回全零（根槽读得出），各挂一次：做成的核 R 的单元一份不少，拒的只许是 C 的映射那一支拒
/// （点名 R）。打印每一格的结局与合计。
fn assert_every_unit_reading_back_zero_with_every_record_keeps_the_units_of_r(
    recorded: &RecordedPublish,
) {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut outcomes: Vec<String> = Vec::new();
    for (unit_index, unit_copies) in recorded.unit_writes_grouped_by_unit().iter().enumerate() {
        let cell = format!("R 的第 {unit_index} 个单元与全部记录读回全零");
        let ranges = RangesReadingBackAllZero::default()
            .with_the_writes(unit_copies)
            .with_the_writes(&every_record_of_r(recorded));
        match mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
            recorded,
            &crash_image,
            &ranges,
            &cell,
        ) {
            Ok(effective) => outcomes.push(format!("{cell}：做成，落到 {effective:?}")),
            Err(refusal) => {
                assert!(
                    matches!(
                        &refusal,
                        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                            if matches!(
                                **still_unreadable,
                                StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords {
                                    unapplied_publish: RollbackTarget { instance, checkpoint_txg }
                                } if (instance, checkpoint_txg) == recorded.key
                            )
                    ),
                    "{cell}：拒只许是 C 的映射那一支拒、点名 R：{refusal:?}"
                );
                outcomes.push(format!("{cell}：拒（映射现算不成）"));
            }
        }
    }
    println!("{}", outcomes.join("\n"));
    println!(
        "合计 {} 格：做成 {}，拒 {}",
        outcomes.len(),
        outcomes
            .iter()
            .filter(|outcome| outcome.contains("做成"))
            .count(),
        outcomes
            .iter()
            .filter(|outcome| outcome.contains("：拒"))
            .count()
    );
}
