//! harness 测试文件 `crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::format_pool;
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::memory_pool::MemoryPool;

pub fn ring_slot_holds_a_self_verified_root(
    image: &MemoryPool,
    system_configuration: &SystemConfiguration,
    ring_slot: RootRingSlot,
) -> bool {
    match read_root_ring_slot(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        ring_slot,
    ) {
        RootRingSlotReading::SelfVerified(_) => true,
        RootRingSlotReading::Bad(_) => false,
    }
}

pub fn write_root_into_ring_slot(
    image: &mut MemoryPool,
    system_configuration: &SystemConfiguration,
    ring_slot: RootRingSlot,
    root: &RootRecord,
) {
    let immutable = &system_configuration.immutable;
    let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let root_slot_bytes = usize::try_from(immutable.sizes.physical_block_size).expect("根槽宽");
    image.devices.get_mut(&device).expect("池里有这块盘").write(
        slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
        &root.to_slot(root_slot_bytes),
    );
}

/// 只做过 mkfs 的池：mkfs 那条根、读根环次序里它之后第一个与最后一个空槽。
pub struct MkfsPoolWithTwoEmptySlotsAfterItsRoot {
    pub image: MemoryPool,
    pub system_configuration: SystemConfiguration,
    pub mkfs_root: RootRecord,
    pub earlier_empty_slot: RootRingSlot,
    pub later_empty_slot: RootRingSlot,
}

pub fn mkfs_pool_with_two_empty_slots_after_its_root(
    tag: &str,
) -> MkfsPoolWithTwoEmptySlotsAfterItsRoot {
    let image = format_pool(tag).memory_pool();
    let system_configuration =
        choose_system_configuration(&image).expect("mkfs 之后择得出系统配置");
    let mkfs_root = choose_root(&image, &system_configuration).expect("mkfs 写了一条根");
    let ring_slots = every_root_ring_slot(&system_configuration.immutable.sizes);
    let mkfs_root_slot_position = ring_slots
        .iter()
        .position(|ring_slot| {
            ring_slot_holds_a_self_verified_root(&image, &system_configuration, *ring_slot)
        })
        .expect("根环里有 mkfs 那条根");
    let empty_slots_after_the_mkfs_root: Vec<RootRingSlot> = ring_slots
        [mkfs_root_slot_position + 1..]
        .iter()
        .copied()
        .filter(|ring_slot| {
            !ring_slot_holds_a_self_verified_root(&image, &system_configuration, *ring_slot)
        })
        .collect();
    let earlier_empty_slot = *empty_slots_after_the_mkfs_root
        .first()
        .expect("mkfs 那条根之后有空槽");
    let later_empty_slot = *empty_slots_after_the_mkfs_root
        .last()
        .expect("mkfs 那条根之后有空槽");
    assert!(
        earlier_empty_slot < later_empty_slot,
        "要两个不同的空槽，一前一后"
    );
    MkfsPoolWithTwoEmptySlotsAfterItsRoot {
        image,
        system_configuration,
        mkfs_root,
        earlier_empty_slot,
        later_empty_slot,
    }
}

/// 同一份造法（择新键完全相同的两条根：前面是 mkfs 那条，后面那条只把实例表指针两条位置条目的校验和改掉、写进区域 2 的槽 0——
/// 第 0 代根照 mkfs 种根的槽另认，C743），问池级 checker：
/// 它择最新根走读（I-7.2「最新的根走得完」），键相同取读根环次序里前面那条，与 `choose_root` 同（代码轮第二轮判决辩方 D3）。
/// 前面那条的实例表读得出，I-7.2 成立；取了后面那条，实例表读不出，I-7.2 判违例。
pub fn of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does(
    images: &mut PoolCheckerImages,
) {
    let MkfsPoolWithTwoEmptySlotsAfterItsRoot {
        mut image,
        system_configuration,
        mkfs_root,
        earlier_empty_slot: _,
        later_empty_slot: _,
    } = mkfs_pool_with_two_empty_slots_after_its_root("tied-choice-key-pool-checker");
    let later_genesis_slot = singlefs_core::make_filesystem::genesis_root_ring_slots()
        .last()
        .expect("mkfs 种根的槽有三个");
    let mut same_key_root_with_an_unreadable_instance_table = mkfs_root;
    for location in &mut same_key_root_with_an_unreadable_instance_table
        .instance_table
        .locations
    {
        location.unit_checksum = !location.unit_checksum;
    }
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        later_genesis_slot,
        &same_key_root_with_an_unreadable_instance_table,
    );
    assert_eq!(
        choose_root(&image, &system_configuration),
        Some(mkfs_root),
        "实现择根取前面那条（mkfs 那条）"
    );

    // checker 择最新根取读根环次序里前面那条（实例表读得出），走得完；取后面那条（实例表读不出）I-7.2 就判违例
    images.hand_out(
        "两条择根键相同的根、后面那条的实例表读不出",
        || image,
        PoolCheckerExpectation::only_the_named_invariants().and_holds("I-7.2"),
    );
}
