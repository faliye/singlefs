//! U11、U13 的新形态（第 4 次重跑登记第一节 1.2）：管理员回退是挂着时的一次向前发布；回退到最旧候选与 deferred 为零的合成基底的单测。

use super::{
    q7c_self_test, run_forward_rollback_scenario, run_rollback_to_the_oldest_candidate,
    synthetic_base_with_zero_deferred, Emitter,
};
use singlefs_core::address::DeviceIdentity;

/// U11（第 4 次重跑登记第一节 1.2、第九节）：A → B → 干净重开 → C → 同一次挂载里挂着回退到 A → 普通重开。
/// K12：回退那次发布的 txg = C 的 txg + 1、实例代号不变、实例表行数不变；普通重开之后按实例表判被抛弃的根 0 条、每盘隔离 0 槽。
/// M32（`crates/mutations.tsv`）把期望的隔离改回挂载时回退那一形的 54 槽，这条测试必须红。
#[test]
fn rolling_back_by_a_forward_publish_keeps_the_instance_and_isolates_nothing_after_a_plain_remount()
{
    let scenario = run_forward_rollback_scenario();
    assert_eq!(
        scenario.txg_of_the_rollback_publish,
        scenario.txg_before_the_rollback + 1,
        "K12：回退那次发布的 txg = 现行 + 1"
    );
    assert_eq!(
        scenario.instance_after, scenario.instance_before,
        "K12：挂着回退不取号，实例代号不变"
    );
    assert_eq!(
        scenario.instance_table_rows_after, scenario.instance_table_rows_before,
        "K12：挂着回退不写行，实例表行数不变"
    );
    assert_eq!(
        scenario.isolated_after_the_plain_remount,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "K12：挂着回退不抛弃任何根，普通重开之后每盘隔离 0 槽"
    );
    assert_eq!(
        scenario.abandoned_roots_after_the_plain_remount, 0,
        "K12：按实例表判被抛弃的根 0 条"
    );
}

/// U13 (a)（第 4 次重跑登记第一节 1.2、第九节）：覆盖写 72 次之后在同一个进程里挂着回退到候选集里 txg 最小的根：
/// K12 的前三样成立。第 5 项只报数（产物 `name=u13a_rollback_to_oldest_candidate`），这里不钉。
#[test]
fn rolling_back_to_the_oldest_candidate_after_seventy_two_overwrites_is_one_forward_publish() {
    // 池级 checker 那几格经给定状态表两段判：配置开了 GPU 而这份二进制没编进 gpu 特性时判段不跑，照本次未跑交回
    if singlefs_verification::crash_amplification::judging_test_not_run_without_the_gpu_feature(
        "e156_allocation_basis_counts::rolling_back_to_the_oldest_candidate_after_seventy_two_overwrites_is_one_forward_publish",
    ) {
        return;
    }
    let rollback = run_rollback_to_the_oldest_candidate();
    assert_eq!(rollback.overwrites, 72, "3N = 3 × 3S = 72（S = 8）");
    assert_eq!(
        rollback.txg_after,
        rollback.txg_before + 1,
        "K12：txg = 现行 + 1"
    );
    assert_eq!(
        rollback.instance_after, rollback.instance_before,
        "K12：实例代号不变"
    );
    assert_eq!(
        rollback.instance_table_rows_after, rollback.instance_table_rows_before,
        "K12：实例表行数不变"
    );
}

/// U13 (b)（第 4 次重跑登记第一节 1.2、第九节）：Q7c① 的转色在 β_syn（造出来的基底，第 5 项为 0）上核：`flip1` 为真、`flip2` 为假。
/// M33、M34（`crates/mutations.tsv`）分别改坏 `q7c_self_test` 里两处比较符号，这条测试必须红。
#[test]
fn synthetic_base_with_zero_deferred_flips_the_q7c1_self_test() {
    let base = synthetic_base_with_zero_deferred();
    let (_item1, _item2, item5) =
        super::mirror_accounting_row_slots(&base.pool, base.accounting_slot);
    assert_eq!(item5, 0, "β_syn 的第 5 项为 0（造法：β0 第 5 项 − 1）");
    let mut emitter = Emitter { emitted: 0 };
    let (flip1, flip2) = q7c_self_test(
        &mut emitter,
        "test_beta_syn",
        &base.pool,
        base.accounting_slot,
        base.referenced,
    );
    assert!(
        flip1,
        "U13 (b)：第 5 项为 0 的基底上 Q7c① 必须转色——去掉『减 defer』之后坏镜像由红变绿，G27 仍判红"
    );
    assert!(
        !flip2,
        "U13 (b)：Q7c② 在第 5 项为 0 的基底上不适用，不应转色"
    );
}
