//! 读根环核落点公式（C743（重建分配器记新类时不核根住不住这一槽），D22（单元原子性怎么合成） 已定项 7 实例代号那一行「读根环时每条根照落点公式核槽位，
//! 第 0 代根照 mkfs 种根的槽另认」）：一条自证得过的根住在不是它的 txg 按落点公式该住的那一槽里，读者照这一槽这一次读坏算——
//! 择根（`recovery::choose_root`）不择它，冷启动恢复不落在它上面。合法写者只有发布路径与 mkfs，这样的根只能是错位读（读回别的槽里一条根）
//! 或手摆出来的；这里在新池新建文件之后手摆一条：照抄新池新建文件那条根、txg 加一，写进不是它落点的一个空槽。
//! 阳性对照：同一条根写进它的落点槽，择根照择它。重建分配器读根环那一遍的同一核在 `crates/singlefs-core/src/mount.rs`
//! 的 `misplaced_root_read_by_the_allocator_rebuild_tests`。
//! 判别力：`crates/mutations.tsv`「根环一簇 R1：读根环择根不核落点公式」那一行，这一条红（择到手摆的那条）。

mod common;

use common::build_pool;
use singlefs_core::address::CheckpointTxg;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, read_root_ring_slot, recover, BadRootRingSlotReading,
    JournalPolicy, RecoveryOutcome, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::memory_pool::MemoryPool;

/// 新池新建文件之后的盘面、它的系统配置与择到的那条根。
struct PoolAfterTheFirstFile {
    image: MemoryPool,
    system_configuration: SystemConfiguration,
    file_root: RootRecord,
}

fn pool_after_the_first_file(tag: &str) -> PoolAfterTheFirstFile {
    let image = build_pool(tag).memory_pool();
    let system_configuration =
        choose_system_configuration(&image).expect("新池新建文件之后择得到系统配置");
    let file_root = choose_root(&image, &system_configuration).expect("择得到新池新建文件那条根");
    PoolAfterTheFirstFile {
        image,
        system_configuration,
        file_root,
    }
}

fn reading_of(pool: &PoolAfterTheFirstFile, ring_slot: RootRingSlot) -> RootRingSlotReading {
    let immutable = &pool.system_configuration.immutable;
    read_root_ring_slot(
        &pool.image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
        ring_slot,
    )
}

fn write_root_into(pool: &mut PoolAfterTheFirstFile, ring_slot: RootRingSlot, root: &RootRecord) {
    let immutable = &pool.system_configuration.immutable;
    let device = immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let root_slot_bytes = usize::try_from(immutable.sizes.physical_block_size).expect("根槽宽");
    pool.image
        .devices
        .get_mut(&device)
        .expect("池里有这块盘")
        .write(
            slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing),
            &root.to_slot(root_slot_bytes),
        );
}

/// 场景：新池新建文件那条根照抄一份、txg 加一，写进 txg 再加一那一版的落点槽（空着，不是它自己的落点）。
/// 预期：那一槽读成「读得出、自证不过」；择根照旧择新池新建文件那条，冷启动恢复照旧读回第一个文件、落在它上面。
#[test]
fn a_self_verified_root_outside_its_formula_slot_is_read_as_bad_and_not_chosen() {
    let mut pool = pool_after_the_first_file("root-outside-its-formula-slot");
    let slots_per_region = pool
        .system_configuration
        .immutable
        .sizes
        .root_ring_slots_per_region;
    let mut planted = pool.file_root;
    planted.checkpoint_txg = CheckpointTxg(pool.file_root.checkpoint_txg.0 + 1);
    let slot_of_another_txg = target_for_publish(
        CheckpointTxg(planted.checkpoint_txg.0 + 1),
        slots_per_region,
    );
    assert_ne!(
        slot_of_another_txg,
        target_for_publish(planted.checkpoint_txg, slots_per_region),
        "前提：手摆的那一槽不是它的落点"
    );
    assert!(
        matches!(
            reading_of(&pool, slot_of_another_txg),
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)
        ),
        "前提：那一槽空着（从没写过、全 0）"
    );
    write_root_into(&mut pool, slot_of_another_txg, &planted);

    assert_eq!(
        reading_of(&pool, slot_of_another_txg),
        RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
        "不住在落点槽的根：照读得出、自证不过算"
    );
    assert_eq!(
        choose_root(&pool.image, &pool.system_configuration),
        Some(pool.file_root),
        "择根不择手摆的那条（它的 txg 最大），照旧择新池新建文件那条"
    );
    let file_key = (pool.file_root.instance, pool.file_root.checkpoint_txg);
    let recovered = recover(&pool.image, JournalPolicy::Consult);
    assert!(
        matches!(recovered.outcome, RecoveryOutcome::FileRead { root, .. } if root == file_key),
        "冷启动恢复落在新池新建文件那条根上、读回第一个文件：{:?}",
        recovered.outcome
    );
}

/// 阳性对照：同一条手摆的根写进它自己的落点槽（空着）。
/// 预期：那一槽读成自证过的那条根，择根择它（txg 最大）。核在的时候住在落点槽的根照认。
#[test]
fn the_same_root_in_its_own_formula_slot_is_read_and_chosen() {
    let mut pool = pool_after_the_first_file("root-in-its-own-formula-slot");
    let slots_per_region = pool
        .system_configuration
        .immutable
        .sizes
        .root_ring_slots_per_region;
    let mut planted = pool.file_root;
    planted.checkpoint_txg = CheckpointTxg(pool.file_root.checkpoint_txg.0 + 1);
    let own_slot = target_for_publish(planted.checkpoint_txg, slots_per_region);
    assert!(
        matches!(
            reading_of(&pool, own_slot),
            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)
        ),
        "前提：它的落点槽空着"
    );
    write_root_into(&mut pool, own_slot, &planted);

    assert_eq!(
        reading_of(&pool, own_slot),
        RootRingSlotReading::SelfVerified(planted),
        "住在落点槽的根：照自证过算"
    );
    assert_eq!(
        choose_root(&pool.image, &pool.system_configuration),
        Some(planted),
        "择根择到它（txg 最大）"
    );
}
