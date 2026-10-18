//! 池级 checker 判全池系统配置槽节点大小都不是格式常量 `NODE_BYTES` 的镜像（C840，用户 2026-10-15 弹窗定「当自证不过」）：
//! 只有 I-8.5（tail 槽有效集非空）红，别的每一条报不适用。判的是池级 checker 自己（手造一张镜像、断言判器的结论），
//! 读者与择系统配置那一侧在 `system_configuration_slot_with_a_foreign_node_size_is_not_chosen.rs`。
//!
//! 坏镜像的造法在 `common_foreign_node_size_planted_in_system_configuration_slots`：只改那 4 字节、整槽校验和重封，拒它的只能是节点大小那一判。
//! 盘：两个文件镜像上 mkfs → 取号 → 暖机 → 新池新建文件（`common::build_pool`），整条录制流施加到内存镜像上再改。

mod common;
mod common_foreign_node_size_planted_in_system_configuration_slots;

use common::build_pool;
use common_foreign_node_size_planted_in_system_configuration_slots::plant_a_foreign_node_size_into_every_slot;
use singlefs_checker::image::{
    InvariantVerdict, SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
};
use singlefs_checker::walk::check_pool_image;

/// 场景：两块盘四个系统配置槽的节点大小都换成 8192。
/// 预期：池级 checker 判 I-8.5（tail 槽有效集非空）违例、只有它红，别的每一条报不适用（这不是一个挂得上的镜像）。
#[test]
fn every_slot_with_a_foreign_node_size_reddens_the_tail_slot_invariant_of_the_pool_checker() {
    let mut image = build_pool("foreign-node-size-checker-every-slot").memory_pool();
    plant_a_foreign_node_size_into_every_slot(&mut image);
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, found)| matches!(found, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        violated,
        [SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
        "只有 I-8.5 红：{verdicts:?}"
    );
    let judged_other_than_not_applicable: Vec<&(&str, InvariantVerdict)> = verdicts
        .iter()
        .filter(|(name, found)| {
            *name != SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
                && !matches!(found, InvariantVerdict::NotApplicable(_))
        })
        .collect();
    assert!(
        judged_other_than_not_applicable.is_empty(),
        "全池每一槽都自证不过时别的每一条都该报不适用：{judged_other_than_not_applicable:?}"
    );
}
