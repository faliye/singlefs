//! 择根胶水（`recovery::choose_root` 把根环每个槽摊成核 `singlefs_replay_prefix::choose_root_slot` 的择新键、照核交回的下标取回那条根）
//! 与它的下游，在真池的根环上：
//! - 择新键两项的次序：D22（单元原子性怎么合成） 已定项 7 txg 为主、实例代号只破平局。胶水把两项摊反时，择到实例代号高、txg 小的那条
//!   （代码轮判决 `research/prompts/m3-formal-code-r1-main-verification.md` 第二节 K1-b，攻方补的用例照本仓命名改写）。
//! - 恢复（`recovery::recover`）、只读挂载（`mounted_read::mount_read_only`）与可写挂载的读阶段（`mount::mount_writable`）
//!   拿所选槽里那条根调重放，不是根环里别的自证过的根（同一格「胶水拿所选根调重放」）。
//! - 择新键完全相同的两条根：择根取读根环次序里前面那条（已定项 24、核的平局取法）；算 F 生效值时判有效根用的那张实例表，
//!   也是前面那条根指着的（同一判决 K1-d：`newest_root_among` 文档说与 `choose_root` 同一个择法）；池级 checker
//!   （`singlefs_checker::walk::check_pool_image`）择最新根走读，同样取前面那条（代码轮第二轮判决辩方 D3）。
//!
//! 池是只做过 mkfs 的池，根照 mkfs 那条根改几项、重新编码写进 mkfs 那条根之后空着的槽（择根只读根环，不看别的结构）；
//! 只读挂载那一格要沿根读中央映射树与树表，用建过新池新建文件的池，根照新池新建文件那条根改。
mod common;

use common::{build_pool, format_pool, parameters};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration,
    effective_rollback_floor_rereading_the_newest_instance_table_once, every_root_ring_slot,
    instance_table_chain_of_root, read_root_ring_slot, readable_roots_with_ring_slots, recover,
    JournalPolicy, RecoveryOutcome, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// txg 大的那条根（排在前面的槽里）的实例代号：比另一条低。
const INSTANCE_OF_THE_NEWER_CHECKPOINT_TXG_ROOT: InstanceGeneration = InstanceGeneration(1);
/// 实例代号高的那条根（排在后面的槽里）的实例代号。
const INSTANCE_OF_THE_HIGHER_INSTANCE_ROOT: InstanceGeneration = InstanceGeneration(2);
/// txg 大的那条根比 mkfs 那条根新几个 txg。
const CHECKPOINT_TXGS_OF_THE_NEWER_CHECKPOINT_TXG_ROOT_ABOVE_THE_MKFS_ROOT: u64 = 2;
/// 实例代号高的那条根比 mkfs 那条根新几个 txg（比上一条少一个）。
const CHECKPOINT_TXGS_OF_THE_HIGHER_INSTANCE_ROOT_ABOVE_THE_MKFS_ROOT: u64 = 1;

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

/// 只做过 mkfs 的池：mkfs 那条根、读根环次序里它之后第一个与最后一个空槽。
struct MkfsPoolWithTwoEmptySlotsAfterItsRoot {
    image: MemoryPool,
    system_configuration: SystemConfiguration,
    mkfs_root: RootRecord,
    earlier_empty_slot: RootRingSlot,
    later_empty_slot: RootRingSlot,
}

fn mkfs_pool_with_two_empty_slots_after_its_root(
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

/// 前一个空槽写 txg 大、实例代号低的根，后一个空槽写实例代号高、txg 小的根；两条除这两项之外照抄 mkfs 那条根。
struct PoolWhereTheNewerCheckpointTxgSitsInAnEarlierSlot {
    image: MemoryPool,
    system_configuration: SystemConfiguration,
    mkfs_root: RootRecord,
    newer_checkpoint_txg_root: RootRecord,
    higher_instance_root: RootRecord,
}

fn pool_where_the_newer_checkpoint_txg_sits_in_an_earlier_slot(
    tag: &str,
) -> PoolWhereTheNewerCheckpointTxgSitsInAnEarlierSlot {
    let MkfsPoolWithTwoEmptySlotsAfterItsRoot {
        mut image,
        system_configuration,
        mkfs_root,
        earlier_empty_slot,
        later_empty_slot,
    } = mkfs_pool_with_two_empty_slots_after_its_root(tag);
    let mut newer_checkpoint_txg_root = mkfs_root;
    newer_checkpoint_txg_root.instance = INSTANCE_OF_THE_NEWER_CHECKPOINT_TXG_ROOT;
    newer_checkpoint_txg_root.checkpoint_txg = CheckpointTxg(
        mkfs_root.checkpoint_txg.0
            + CHECKPOINT_TXGS_OF_THE_NEWER_CHECKPOINT_TXG_ROOT_ABOVE_THE_MKFS_ROOT,
    );
    let mut higher_instance_root = mkfs_root;
    higher_instance_root.instance = INSTANCE_OF_THE_HIGHER_INSTANCE_ROOT;
    higher_instance_root.checkpoint_txg = CheckpointTxg(
        mkfs_root.checkpoint_txg.0
            + CHECKPOINT_TXGS_OF_THE_HIGHER_INSTANCE_ROOT_ABOVE_THE_MKFS_ROOT,
    );
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        earlier_empty_slot,
        &newer_checkpoint_txg_root,
    );
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        later_empty_slot,
        &higher_instance_root,
    );
    PoolWhereTheNewerCheckpointTxgSitsInAnEarlierSlot {
        image,
        system_configuration,
        mkfs_root,
        newer_checkpoint_txg_root,
        higher_instance_root,
    }
}

/// 胶水把择新键两项摊反（实例代号为主）时择到后一个槽里实例代号高、txg 小的那条；txg 为主择前一个槽里 txg 大的那条。
#[test]
fn a_later_checkpoint_txg_in_an_earlier_slot_beats_a_higher_instance_through_choose_root() {
    let pool = pool_where_the_newer_checkpoint_txg_sits_in_an_earlier_slot(
        "root-choice-glue-checkpoint-txg-before-instance",
    );
    assert!(
        pool.higher_instance_root.instance > pool.newer_checkpoint_txg_root.instance
            && pool.higher_instance_root.checkpoint_txg
                < pool.newer_checkpoint_txg_root.checkpoint_txg,
        "两条根的两项各大一项：只有 txg 为主才择前一个槽那条"
    );

    let chosen = choose_root(&pool.image, &pool.system_configuration).expect("根环里有自证过的根");

    assert_eq!(
        chosen, pool.newer_checkpoint_txg_root,
        "txg 为主（D22 已定项 7）：txg 大的实例 1 那条赢过实例代号高、txg 小的实例 2 那条（胶水摊反两项时择到实例 2 那条）"
    );
}

/// 恢复拿所选槽里那条根（txg 大的那条）调重放：施加之后实际走的根就是它（mkfs 池里没有它之上的 journal 记录）；
/// 拿根环里别的自证过的根（例如读根环次序里第一条，mkfs 那条）去调，走的根就成了那一条。
#[test]
fn recover_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root() {
    let pool = pool_where_the_newer_checkpoint_txg_sits_in_an_earlier_slot(
        "recover-replays-from-the-chosen-slot",
    );
    let chosen_key = (
        pool.newer_checkpoint_txg_root.instance,
        pool.newer_checkpoint_txg_root.checkpoint_txg,
    );
    assert_ne!(
        (pool.mkfs_root.instance, pool.mkfs_root.checkpoint_txg),
        chosen_key,
        "mkfs 那条根与所选根是两条不同的根"
    );

    let report = recover(&pool.image, JournalPolicy::Consult);

    assert_eq!(
        report.outcome,
        RecoveryOutcome::NoFile { root: chosen_key },
        "择到的是前一个槽里 txg 大的那条，它下面没有文件"
    );
    assert_eq!(
        report.effective_root,
        Some(chosen_key),
        "施加之后实际走的根是所选槽里那条（水位之上没有记录，原样走它），不是根环里别的自证过的根"
    );
}

/// 前一个空槽写一条只比 mkfs 那条根新一个 txg 的根，别的照抄 mkfs 那条（实例代号也照抄）。读根环次序里第一条自证过的根仍是 mkfs 那条。
struct PoolWithARootOneCheckpointTxgNewerThanTheMkfsRoot {
    image: MemoryPool,
    mkfs_root: RootRecord,
    newer_root: RootRecord,
}

fn pool_with_a_root_one_checkpoint_txg_newer_than_the_mkfs_root(
    tag: &str,
) -> PoolWithARootOneCheckpointTxgNewerThanTheMkfsRoot {
    let MkfsPoolWithTwoEmptySlotsAfterItsRoot {
        mut image,
        system_configuration,
        mkfs_root,
        earlier_empty_slot,
        later_empty_slot: _,
    } = mkfs_pool_with_two_empty_slots_after_its_root(tag);
    let mut newer_root = mkfs_root;
    newer_root.checkpoint_txg = CheckpointTxg(mkfs_root.checkpoint_txg.0 + 1);
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        earlier_empty_slot,
        &newer_root,
    );
    PoolWithARootOneCheckpointTxgNewerThanTheMkfsRoot {
        image,
        mkfs_root,
        newer_root,
    }
}

/// 建过新池新建文件的池（`build_pool`）：新池新建文件那条根照抄一份、txg 加一，写进根环最后一个空槽。择根择到这一条，
/// 环里的 journal 记录都在它的水位之下，重放原样走它；读根环次序里第一条自证过的根是另一条。
/// 只读挂载要沿根把中央映射树与树表读出来，只做过 mkfs 的池打不开（实测报 `Open(Walk(UnitUnreadable { .. }))`），所以这一格用建过文件的池。
struct BuiltPoolWithACopyOfTheFileRootOneCheckpointTxgNewer {
    image: MemoryPool,
    first_root_in_ring_order: RootRecord,
    newer_root: RootRecord,
}

fn built_pool_with_a_copy_of_the_file_root_one_checkpoint_txg_newer(
    tag: &str,
) -> BuiltPoolWithACopyOfTheFileRootOneCheckpointTxgNewer {
    let built = build_pool(tag);
    let mut image = built.memory_pool();
    let system_configuration =
        choose_system_configuration(&image).expect("建过文件的池择得出系统配置");
    let file_root = choose_root(&image, &system_configuration).expect("根环里有新池新建文件那条根");
    let immutable = &system_configuration.immutable;
    let (first_self_verified_slot, first_root_in_ring_order) = readable_roots_with_ring_slots(
        &image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )[0];
    let last_empty_slot = *every_root_ring_slot(&immutable.sizes)
        .iter()
        .rev()
        .find(|ring_slot| {
            !ring_slot_holds_a_self_verified_root(&image, &system_configuration, **ring_slot)
        })
        .expect("根环里有空槽");
    assert!(
        first_self_verified_slot < last_empty_slot,
        "照抄的那条写在读根环次序里第一条自证过的根之后"
    );
    let mut newer_root = file_root;
    newer_root.checkpoint_txg = CheckpointTxg(file_root.checkpoint_txg.0 + 1);
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        last_empty_slot,
        &newer_root,
    );
    BuiltPoolWithACopyOfTheFileRootOneCheckpointTxgNewer {
        image,
        first_root_in_ring_order,
        newer_root,
    }
}

/// 只读挂载拿所选槽里那条根（txg 新的那条）调重放：施加之后的根与打开挂载态的根都是它；
/// 拿根环里别的自证过的根（例如读根环次序里第一条）去调，这两条根就不是它。
#[test]
fn mount_read_only_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root() {
    let pool = built_pool_with_a_copy_of_the_file_root_one_checkpoint_txg_newer(
        "mount-read-only-replays-from-the-chosen-slot",
    );
    let chosen_key = (pool.newer_root.instance, pool.newer_root.checkpoint_txg);
    assert_ne!(
        (
            pool.first_root_in_ring_order.instance,
            pool.first_root_in_ring_order.checkpoint_txg,
        ),
        chosen_key,
        "读根环次序里第一条自证过的根与所选根是两条不同的根"
    );

    let mounted = mount_read_only(&pool.image)
        .expect("建过文件的池加一条照抄新池新建文件那条、txg 新一个的根，只读挂得上");

    assert_eq!(
        mounted.chosen_root, pool.newer_root,
        "择到的是照抄的那条（txg 最新）"
    );
    let effective_key = (
        mounted.effective_root.instance,
        mounted.effective_root.checkpoint_txg,
    );
    let opened_key = (
        mounted.mounted.root().instance,
        mounted.mounted.root().checkpoint_txg,
    );
    assert_eq!(
        (effective_key, opened_key),
        (chosen_key, chosen_key),
        "施加之后的根与打开挂载态的根都是所选槽里那条（水位之上没有记录，原样走它），不是根环里别的自证过的根"
    );
}

/// 可写挂载的读阶段拿所选槽里那条根（txg 新的那条）调重放：挂载交回的施加之后的根就是它；
/// 拿根环里别的自证过的根（例如读根环次序里第一条，mkfs 那条）去调，施加之后的根就成了那一条。
#[test]
fn mount_writable_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root() {
    let pool = pool_with_a_root_one_checkpoint_txg_newer_than_the_mkfs_root(
        "mount-writable-replays-from-the-chosen-slot",
    );
    let chosen_key = (pool.newer_root.instance, pool.newer_root.checkpoint_txg);
    assert_ne!(
        (pool.mkfs_root.instance, pool.mkfs_root.checkpoint_txg),
        chosen_key,
        "mkfs 那条根与所选根是两条不同的根"
    );
    let mut devices: Vec<_> = pool
        .image
        .devices
        .iter()
        .map(|(identity, sparse_device)| {
            let mut device = SparseBlockDevice::new(
                pool.image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters().geometry.physical_block_size),
            );
            device.image = sparse_device.clone();
            (*identity, device)
        })
        .collect();

    let mounted = mount_writable(&parameters(), &mut devices)
        .expect("只做过 mkfs 的池加一条照抄 mkfs 那条、txg 新一个的根，可写挂得上");

    assert_eq!(
        mounted.output.chosen_root, pool.newer_root,
        "择到的是前一个槽里 txg 新的那条"
    );
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg,
        ),
        chosen_key,
        "施加之后的根是所选槽里那条（水位之上没有记录，原样走它），不是根环里别的自证过的根"
    );
}

/// 择新键（txg, 实例代号）完全相同的两条根：前面是 mkfs 那条，后面那条照抄它、只把实例表指针的两条位置条目的校验和改掉（表读不出）。
/// 择根取前面那条（只有严格更新才换）；F 生效值按表判有效根，读的也是前面那条指着的那张表，读得出，算出 mkfs 那一版的 F。
/// 取了后面那条就读不出表，重读一次仍读不出，报错。
#[test]
fn of_two_roots_with_the_same_choice_key_the_effective_floor_reads_the_instance_table_of_the_earlier_slot_as_choose_root_does(
) {
    let MkfsPoolWithTwoEmptySlotsAfterItsRoot {
        mut image,
        system_configuration,
        mkfs_root,
        earlier_empty_slot: _,
        later_empty_slot,
    } = mkfs_pool_with_two_empty_slots_after_its_root("tied-choice-key-effective-floor");
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
        later_empty_slot,
        &same_key_root_with_an_unreadable_instance_table,
    );
    assert!(
        instance_table_chain_of_root(&image, &mkfs_root).is_ok()
            && instance_table_chain_of_root(
                &image,
                &same_key_root_with_an_unreadable_instance_table
            )
            .is_err(),
        "前面那条的实例表读得出，后面那条的读不出：两条择新键相同、指着的表不同"
    );

    let chosen = choose_root(&image, &system_configuration).expect("根环里有自证过的根");
    let effective_floor = effective_rollback_floor_rereading_the_newest_instance_table_once(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );

    assert_eq!(
        chosen, mkfs_root,
        "择新键完全相同取读根环次序里前面那条（D22 已定项 24、核的平局取法）"
    );
    assert_eq!(
        effective_floor,
        Ok(mkfs_root.rollback_floor),
        "F 生效值判有效根读的是择根择的那条根指着的表（前面那条，读得出），不是后面那条（读不出，重读一次仍读不出就报错）"
    );
}

/// 同一份造法（择新键完全相同的两条根：前面是 mkfs 那条，后面那条只把实例表指针两条位置条目的校验和改掉），问池级 checker：
/// 它择最新根走读（I-7.2「最新的根走得完」），键相同取读根环次序里前面那条，与 `choose_root` 同（代码轮第二轮判决辩方 D3）。
/// 前面那条的实例表读得出，I-7.2 成立；取了后面那条，实例表读不出，I-7.2 判违例。
#[test]
fn of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does(
) {
    let MkfsPoolWithTwoEmptySlotsAfterItsRoot {
        mut image,
        system_configuration,
        mkfs_root,
        earlier_empty_slot: _,
        later_empty_slot,
    } = mkfs_pool_with_two_empty_slots_after_its_root("tied-choice-key-pool-checker");
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
        later_empty_slot,
        &same_key_root_with_an_unreadable_instance_table,
    );
    assert_eq!(
        choose_root(&image, &system_configuration),
        Some(mkfs_root),
        "实现择根取前面那条（mkfs 那条）"
    );

    let newest_root_walk_verdict = check_pool_image(&image)
        .into_iter()
        .find(|(invariant, _)| *invariant == "I-7.2")
        .map(|(_, verdict)| verdict);
    assert_eq!(
        newest_root_walk_verdict,
        Some(InvariantVerdict::Holds),
        "checker 择最新根取读根环次序里前面那条（实例表读得出），走得完；取后面那条（实例表读不出）I-7.2 就判违例"
    );
}
