//! F1 乙按 F2 退到的那一版再判一次（`F1_AFTER_F2`）之后的「重读一次」读得出的那一支：判出有一次缺记录、点名单元两份都验不过的发布，
//! 重读之前读故障撤掉，重读时点名单元验得过 ⇒ 不拒，可写挂载照常做成（D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」，
//! `.claude/rules/fs-design.md` 五条硬要求第 2 条：用重读之前的钩子撤掉读故障，强制进入这一支）。
//!
//! 造法（`tests/common_unwitnessed_publish/mod.rs`）与 `writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version.rs`
//! 第一条同一个盘面：384 槽小盘、第一个文件之后覆盖写 5 次，再录一次三条记录的顺序写 R，崩在 R 的根槽落盘之后、系统配置轮换之前；
//! 可写挂载里 R 的第 1 条记录与 R 的树表读回全零，R 的根槽读得出。不撤读故障时那一条在取号之前拒。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on_until_lifted, record_one_publish_after_overwrites,
    PublishRecordedAsR, RangesReadingBackAllZero, ReadBackAllZeroLift,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    ReadStageSettled, ShadowLedger,
};

const OVERWRITES_BEFORE_R: u64 = 5;

/// R 被择、走读失败（树表读回全零），系统配置没见证 R，F2 退回 P；按 P 当水位，R 读得出的是第 2、3 条（序号不从 1 起），第 3 条点名的树表
/// 两份都验不过 ⇒ 重读一次。重读之前的钩子撤掉读故障，重读时树表验得过：不拒，可写挂载建在 P 上做成；钩子只调了一次，读阶段没有重读
/// （那一次重读是 `F1_AFTER_F2` 的）。判别力：`F1_AFTER_F2` 重读之前不调钩子（变异「删除线第41批 F1_AFTER_F2 重读之前不调钩子
/// （读故障撤不掉、照拒）」）时重读照样验不过、在取号之前拒，这一条红。
#[test]
fn a_root_falling_back_with_its_first_record_and_tree_table_reading_back_zero_mounts_writable_when_the_reread_clears_the_fault(
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
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
    let lifting = lift.clone();
    let mut hook_calls = 0u32;
    let mut lift_before_the_reread = || {
        hook_calls += 1;
        lifting.lift();
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut lift_before_the_reread),
    )
    .expect(
        "退回 P 之后判出 R 缺记录、点名的树表验不过，重读之前读故障撤掉、重读验得过：可写挂载做成",
    );
    assert!(lift.is_lifted(), "读故障在挂载途中撤掉了");
    assert_eq!(hook_calls, 1, "钩子只在 F1_AFTER_F2 重读之前调了一次");
    assert!(
        matches!(
            mounted.output.rereads.read_stage,
            ReadStageSettled::OnTheFirstRead { .. }
        ),
        "读阶段第一遍就判定了、没有重读（那一次重读不是读阶段的）：{:?}",
        mounted.output.rereads.read_stage
    );
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "F2 退回 P，可写挂载建在 P 上"
    );
}
