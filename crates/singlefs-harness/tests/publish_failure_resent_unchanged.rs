//! 这一版的失败处置（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：发布不接受失败。
//! 一次发布落盘中途失败之后把它冻结，下一次发布之前先逐字节原样重发它（checkpoint_txg、计数器、本次发布内序号、记录标志、
//! 单元的位置与字节都不变），重发成功才建下一次发布——于是同一 (实例代号, checkpoint_txg) 不会有两条带末条标志的记录。
//!
//! 历史：A（第一个文件，txg 3，jsn 3）→ 顺序写两个数据单元的 B（txg 4，jsn 4、5，两条记录）：盘 1 写 jsn 5 那一槽时报块设备错，
//! 盘 0 上 jsn 5 已经落了。改之前接着在 A 上发一次一个数据单元的覆盖写 C：它也是 txg 4、jsn 4，只写一条记录，盘 0 上 B 的 jsn 5
//! 留在环里、与 C 共享 (实例 1, txg 4)、各带一个末条标志——C 的根落盘之后，恢复停在
//! `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided` 上（实二十报告第六节 1）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/publish_failure_resent_unchanged.rs"]
mod scenarios_of_publish_failure_resent_unchanged;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_publish_failure_resent_unchanged::publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish() {
    scenarios_of_publish_failure_resent_unchanged::publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish(&mut PoolCheckerImages::left_to_the_checker_tier());
}
