//! 里程碑「覆盖写、释放、回退与复用」增补 2 第 20a 行那一格（代码三方第一轮打中，判决 `research/prompts/m2-wave1-code-r1-main-verification.md` 第二节第 1 行；
//! 暖机那几次空发布一起挪到取号之前）的后续：可写挂载取号之后要发的那一串——写行一次、暖机 1–3 次
//! （D16（发布语义） 已定项 8 戊）——在**取号之前**整串预演，预演不过在任何写之前返回。
//!
//! 此前这里钉的是「分配记录树只有一个节点（10 + 20 的条目，812 条）装不下」那道墙在取号之前拒：一路覆盖写把那一个节点填满，
//! 写行那次、暖机第 1 次、暖机第 2 次各有一个池在取号之前被拒。分配记录树按绝对槽号按位置寻址之后（D8（核心索引结构） 已定项 14，
//! K1）那道墙拆了，这里改钉它的反面：同样的三个池（49 次覆盖写 + 1 次空发布、48 + 1、47 + 2），可写挂载都做成——
//! 取号、写行、暖机都发出去，挂载之后的那一版分配记录多于 812 条、每块盘上装着记录的叶不止一片，池级 checker 全绿、
//! 冷启动读回挂载之前最后写的内容。
//!
//! 取号之前那一串的预演走的就是发布路径落盘之前那一段（`transaction::prepare_the_version_publish` 等），写行那次在取号之前被拒的别的原因
//! （实例表旧链核不过、映射树的形状）钉在 `instance_table_chain_read.rs`、
//! `accounting_and_mapping_tree_split.rs`；暖机那几次在取号之前被拒（`MountError::WarmUpAdmissionRefusedBeforeAcquisition`）
//! 拆墙之后只有坏盘面或只供测试的开关下走得到，这一轮没有用例钉它。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/row_publish_admission.rs"]
mod scenarios_of_row_publish_admission;

use common_images_for_the_checker_tier::PoolCheckerImages;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_row_publish_admission::writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_row_publish_admission::writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_row_publish_admission::writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds()
{
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_row_publish_admission::writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds,
    );
}
