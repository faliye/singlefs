//! 择根（`recovery::choose_root`，取最大那一截在 `singlefs_replay_prefix::choose_root_slot`）在真池的根环上：
//! 两条自证过的根 checkpoint_txg 相同、实例代号不同，实例代号低的那条在读根环次序里排前面——
//! 择新照 D22（单元原子性怎么合成） 已定项 7 txg 为主、平局实例代号高者赢，择排在后面的那条；
//! 两条都排在 mkfs 那条根之后，已定项 24「先逐个验证全部候选、再在有效者中择新」也要看完全部槽才择得对。
//! 读根环核落点公式（C743，D22 已定项 7 实例代号那一行）之后，一个 txg 只有一个落点槽，同 txg 两条根能并存的只剩第 0 代根照 mkfs 种根的槽另认那几槽：
//! 池是只做过 mkfs 的池，两条根照 mkfs 那条根只改实例代号（txg 仍是第 0 代的 0）、重新编码写进区域 1、区域 2 各自的槽 0（盖掉 mkfs 在那两槽种的第 0 代根；
//! 择根只读根环，不看别的结构）。mkfs 那条（实例 0）还在区域 0 的槽 0，排最前，三条平局。
mod common;

use common::format_pool;
use singlefs_core::address::InstanceGeneration;
use singlefs_core::make_filesystem::genesis_root_ring_slots;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot,
    RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::memory_pool::MemoryPool;

/// 排在前面那条根的实例代号（低）。
const LOWER_INSTANCE: InstanceGeneration = InstanceGeneration(1);
/// 排在后面那条根的实例代号（高）。
const HIGHER_INSTANCE: InstanceGeneration = InstanceGeneration(2);

fn ring_slot_holds_a_self_verified_root(
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

fn write_root_into_ring_slot(
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

#[test]
fn of_two_self_verified_roots_with_the_same_checkpoint_txg_choose_root_takes_the_higher_instance_although_it_sits_in_a_later_slot(
) {
    let mut image = format_pool("root-choice-tied-checkpoint-txg").memory_pool();
    let system_configuration =
        choose_system_configuration(&image).expect("mkfs 之后择得出系统配置");
    let mkfs_root = choose_root(&image, &system_configuration).expect("mkfs 写了一条根");
    let ring_slots = every_root_ring_slot(&system_configuration.immutable.sizes);
    let genesis_slots: Vec<RootRingSlot> = genesis_root_ring_slots().collect();
    assert!(
        genesis_slots
            .iter()
            .all(|ring_slot| ring_slot_holds_a_self_verified_root(
                &image,
                &system_configuration,
                *ring_slot
            )),
        "前提：mkfs 在三个区域的槽 0 各种了一条第 0 代根"
    );
    let earlier_slot = genesis_slots[1];
    let later_slot = genesis_slots[2];
    let position_of = |ring_slot: RootRingSlot| {
        ring_slots
            .iter()
            .position(|candidate| *candidate == ring_slot)
            .expect("种根的槽在根环里")
    };
    assert!(
        position_of(genesis_slots[0]) < position_of(earlier_slot)
            && position_of(earlier_slot) < position_of(later_slot),
        "两条根落在 mkfs 那条之后两个不同的槽里，实例代号低的那条在前"
    );
    let tied_checkpoint_txg = mkfs_root.checkpoint_txg;
    // 两条根除实例代号之外照抄 mkfs 那条根（txg 仍是第 0 代的，落点公式照 mkfs 种根的槽另认）：择根只看这两项、自证校验和与落点。
    let mut lower_instance_root = mkfs_root;
    lower_instance_root.instance = LOWER_INSTANCE;
    lower_instance_root.checkpoint_txg = tied_checkpoint_txg;
    let mut higher_instance_root = mkfs_root;
    higher_instance_root.instance = HIGHER_INSTANCE;
    higher_instance_root.checkpoint_txg = tied_checkpoint_txg;
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        earlier_slot,
        &lower_instance_root,
    );
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        later_slot,
        &higher_instance_root,
    );

    let chosen = choose_root(&image, &system_configuration).expect("根环里有自证过的根");

    assert_eq!(
        chosen.checkpoint_txg, tied_checkpoint_txg,
        "三条根 txg 平局（都是第 0 代的）"
    );
    assert_eq!(
        chosen.instance, HIGHER_INSTANCE,
        "txg 平局时实例代号高者赢，哪怕它排在后面的槽里（只比 txg 会停在排最前的 mkfs 那条实例 0，取第一个有效者也一样）"
    );
    assert_eq!(
        chosen, higher_instance_root,
        "择出的就是写进后一个槽的那条根"
    );
}
