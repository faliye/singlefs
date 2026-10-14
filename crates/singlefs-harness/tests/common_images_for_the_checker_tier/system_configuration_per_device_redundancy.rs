//! harness 测试文件 `crates/singlefs-harness/tests/system_configuration_per_device_redundancy.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{build_pool, parameters};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};

/// 翻掉槽里的这一个字节来制造「读得出来但自证不过」：它落在系统配置槽的保留区，
/// magic（0..4）、incompat 位图（6..38）、宽校验和字段（155..187）都不含它，
/// 于是只有覆盖整槽的那个宽校验和抓得到它（`SystemConfiguration::parse_slot` 的三道检查）。
pub const BYTE_FLIPPED_IN_THE_RESERVED_AREA_OF_THE_SLOT: u64 = 2048;

/// 一块盘上两个系统配置槽的盘内偏移：槽 0 在 0，槽 1 在槽距处（D22（单元原子性怎么合成） 已定项 16）。
pub fn system_configuration_slot_offsets() -> [DeviceOffsetInBytes; 2] {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    [DeviceOffsetInBytes(0), DeviceOffsetInBytes(spacing)]
}

/// 池级 checker 对这种镜像不许整片报「不适用」：一块盘的两个系统配置槽都自证不过时，
/// 恢复改用别的盘那一份、照常挂上，checker 早退就等于对一个挂得上的合法镜像一条都不判
/// （`.claude/kb/checks-owed.md` 的 C461）。故障注入让一块盘的两个槽先后写失败就造得出它。
pub fn the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration(
    images: &mut PoolCheckerImages,
) {
    let pool = build_pool("system-configuration-redundancy-checker-still-judges");
    let image = pool.memory_pool();
    images.hand_out(
        "干净镜像",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants(),
    );
    // 一块盘的系统配置全废不改变要判哪些不变量；不许整片报不适用：坏掉盘 0 的系统配置最多让一条从判得了变成判不了
    // （原先按「不适用」数，对拍字只分判到与没判到：部分没判的也算没判到）
    images.hand_out(
        "盘 0 两个系统配置槽都自证不过",
        || {
            let mut damaged = image;
            for offset in system_configuration_slot_offsets() {
                damaged.flip_byte(
                    DeviceIdentity(0),
                    offset,
                    BYTE_FLIPPED_IN_THE_RESERVED_AREA_OF_THE_SLOT,
                );
            }
            damaged
        },
        PoolCheckerExpectation::only_the_named_invariants()
            .and_at_most_this_many_judged_on_the_image_not_judged_here("干净镜像", 1),
    );
}
