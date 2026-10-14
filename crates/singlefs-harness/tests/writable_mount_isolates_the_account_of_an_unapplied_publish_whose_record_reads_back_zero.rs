//! C 的账那一份（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节「K2 T1 C 的盲区」那一行与第三节第 1 条，攻方 `C_ACCOUNT`）：
//! 可写挂载读阶段、重建分配器之后，水位之上每一次没施加的发布，用它任一条读得出的记录的新根段拼一条根、按影子账同一份读法读它的账，
//! 账里的落点并进 C 的集合（账读不出就不加，交 F1 乙），解开时机照 C（`ring`）。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`，攻方报告第三节第 1 小节那一形）：384 槽小盘、第一个文件之后覆盖写 1 次，再录一次三条记录的发布 R
//! （顺序写三个数据单元），崩在 R 的根槽落盘之后、系统配置轮换之前；取新实例的那次可写挂载 M1 里 R 的根槽与 R 的某一条记录（末条 `rL`、中间一条 `rM`、
//! 第一条 `rF`）读回全零，R 的单元读得出。F1 乙不拒（读得出的那两条点名的单元验得过），缺的那条记录单独点名的单元不在别的记录里，
//! R 的根槽读不出、影子账也不知道 R 是一条根；读得出的记录的新根段说得出 R 的树表在哪，R 的账（单元都好）读得出。
//! M1 之后照常覆盖写一次、再正常卸载：R 的单元一份都没被写掉，卸载之后池级 checker 不红 I-7.4（被抛弃根引用的单元被重新分配）。
mod common_images_for_the_checker_tier;
mod common_unwitnessed_publish;
#[path = "common_images_for_the_checker_tier/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs"]
mod scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::{OVERWRITES_BEFORE_R};
use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on,
    record_one_publish_after_overwrites, PublishRecordedAsR,
    RangesReadingBackAllZero,
};
use singlefs_core::journal::JournalRecord;
use singlefs_core::mount::mount_writable;
use singlefs_core::unit::unit_filesystem_identifier;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_last_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_middle_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero::a_publish_whose_root_slot_and_first_record_read_back_zero_keeps_its_units_through_an_overwrite_and_the_unmount,
    );
}

/// C 选法 `all`（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条：不只认带末条标志的记录）在 `C_ACCOUNT` 之下分得开的取样点
/// （`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第三类：原来那一格的末条读不出时 `C_ACCOUNT` 读得出 R 的账、把前两条点名的落点
/// 一并隔离，选法退成只认末条也照样护住）：R 的三条记录都读得出，M1 里 R 的根槽与 R 的树表读回全零。末条点名的树表两份都验不过，R 整次不施加，
/// 所选那一版是 P；三条记录齐（序号 1..3、带末条），F1 乙不判，C 的账读不出也不从映射现算；记录拼根读 R 的账经树表、读不出，`C_ACCOUNT` 什么都不加
/// ——前两条点名的落点只有 C 记录那一份护着。
/// 判别力：选法退成只认带末条标志的记录（变异「第33批T C694 的 C 选法退成只认带末条标志的记录（last）」）时前两条点名的数据单元 M1 之后空闲、这一条红。
#[test]
fn the_units_named_by_the_first_two_records_of_a_publish_whose_root_slot_and_tree_table_read_back_zero_are_isolated(
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_R,
        PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
    );
    assert_eq!(recorded.record_count(), 3, "顺序写三个数据单元切成三条记录");
    let tree_table_writes = recorded.unit_writes_at(&recorded.root().tree_table);
    assert_eq!(tree_table_writes.len(), 2, "R 自己写了树表，两块盘各一份");
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&tree_table_writes);
    let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("R 的三条记录齐、树表读不出：F1 乙不判，可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽读回全零、末条点名的树表验不过：R 整次不施加，所选那一版是 P"
    );
    let filesystem_identifier =
        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier);
    for ordinal in [1, 2] {
        let record = JournalRecord::parse(
            recorded.record_writes_of_the_record(ordinal)[0]
                .bytes()
                .expect("记录写带字节"),
            filesystem_identifier,
        )
        .expect("R 的记录解得开");
        assert!(!record.named.is_empty(), "第 {ordinal} 条记录点名了单元");
        for named in &record.named {
            for location in &named.locations {
                let device_map = mounted
                    .allocator
                    .devices
                    .iter()
                    .find(|device_map| device_map.device == location.device)
                    .expect("点名项的盘在池里");
                assert!(
                    !device_map.is_free(location.slot),
                    "R 第 {ordinal} 条记录点名的落点（盘 {:?} 槽 {:?}）M1 之后不空闲（C 隔离了它）",
                    location.device,
                    location.slot
                );
            }
        }
    }
}
