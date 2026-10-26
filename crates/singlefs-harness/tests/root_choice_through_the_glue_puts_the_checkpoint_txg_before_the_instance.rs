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
//! 池是只做过 mkfs 的池，根照 mkfs 那条根改几项、重新编码写进它的 txg 按落点公式该住的那一槽（读根环核落点公式，C743；第 0 代根照 mkfs 种根的槽另认，
//! 择新键与 mkfs 那条完全相同的那条写进区域 2 的槽 0；择根只读根环，不看别的结构）；
//! 只读挂载那一格要沿根读中央映射树与树表，用建过新池新建文件的池，根照新池新建文件那条根改。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs"]
mod scenarios_of_root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance;

use common::{build_pool, parameters};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance::{
    mkfs_pool_with_two_empty_slots_after_its_root, ring_slot_holds_a_self_verified_root,
    write_root_into_ring_slot, MkfsPoolWithTwoEmptySlotsAfterItsRoot,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::genesis_root_ring_slots;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration,
    effective_rollback_floor_rereading_the_newest_instance_table_once,
    instance_table_chain_of_root, readable_roots_with_ring_slots, recover, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

/// txg 大的那条根（排在前面的槽里）的实例代号：比另一条低。
const INSTANCE_OF_THE_NEWER_CHECKPOINT_TXG_ROOT: InstanceGeneration = InstanceGeneration(1);

/// 实例代号高的那条根（排在后面的槽里）的实例代号。
const INSTANCE_OF_THE_HIGHER_INSTANCE_ROOT: InstanceGeneration = InstanceGeneration(2);

/// txg 大的那条根比 mkfs 那条根新几个 txg：txg 3 落区域 0 的槽 1（S = 8），在读根环次序里排在区域 2 前面。
const CHECKPOINT_TXGS_OF_THE_NEWER_CHECKPOINT_TXG_ROOT_ABOVE_THE_MKFS_ROOT: u64 = 3;

/// 实例代号高的那条根比 mkfs 那条根新几个 txg（比上一条少一个）：txg 2 落区域 2 的槽 0（盖掉 mkfs 种在那里的第 0 代根）。
const CHECKPOINT_TXGS_OF_THE_HIGHER_INSTANCE_ROOT_ABOVE_THE_MKFS_ROOT: u64 = 2;

/// txg 大、实例代号低的根与实例代号高、txg 小的根各写进自己的 txg 按落点公式该住的那一槽：前一条落的槽在读根环次序里排前面；
/// 两条除这两项之外照抄 mkfs 那条根。
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
        earlier_empty_slot: _,
        later_empty_slot: _,
    } = mkfs_pool_with_two_empty_slots_after_its_root(tag);
    let slots_per_region = system_configuration
        .immutable
        .sizes
        .root_ring_slots_per_region;
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
    let slot_of_the_newer_checkpoint_txg_root =
        target_for_publish(newer_checkpoint_txg_root.checkpoint_txg, slots_per_region);
    let slot_of_the_higher_instance_root =
        target_for_publish(higher_instance_root.checkpoint_txg, slots_per_region);
    assert!(
        slot_of_the_newer_checkpoint_txg_root < slot_of_the_higher_instance_root,
        "前提：txg 大的那条落的槽在读根环次序里排前面（区域号为主序）"
    );
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        slot_of_the_newer_checkpoint_txg_root,
        &newer_checkpoint_txg_root,
    );
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        slot_of_the_higher_instance_root,
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

/// 一条只比 mkfs 那条根新一个 txg 的根写进它按落点公式该住的那一槽（txg 1：区域 1 的槽 0，盖掉 mkfs 种在那里的第 0 代根），
/// 别的照抄 mkfs 那条（实例代号也照抄）。读根环次序里第一条自证过的根仍是 mkfs 那条（区域 0 的槽 0）。
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
        earlier_empty_slot: _,
        later_empty_slot: _,
    } = mkfs_pool_with_two_empty_slots_after_its_root(tag);
    let mut newer_root = mkfs_root;
    newer_root.checkpoint_txg = CheckpointTxg(mkfs_root.checkpoint_txg.0 + 1);
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        target_for_publish(
            newer_root.checkpoint_txg,
            system_configuration
                .immutable
                .sizes
                .root_ring_slots_per_region,
        ),
        &newer_root,
    );
    PoolWithARootOneCheckpointTxgNewerThanTheMkfsRoot {
        image,
        mkfs_root,
        newer_root,
    }
}

/// 建过新池新建文件的池（`build_pool`）：新池新建文件那条根照抄一份、txg 加一，写进它按落点公式该住的那一槽（那一槽空着）。择根择到这一条，
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
    let mut newer_root = file_root;
    newer_root.checkpoint_txg = CheckpointTxg(file_root.checkpoint_txg.0 + 1);
    let slot_of_the_newer_root = target_for_publish(
        newer_root.checkpoint_txg,
        immutable.sizes.root_ring_slots_per_region,
    );
    assert!(
        !ring_slot_holds_a_self_verified_root(
            &image,
            &system_configuration,
            slot_of_the_newer_root
        ),
        "前提：照抄的那条的落点槽空着"
    );
    assert!(
        first_self_verified_slot < slot_of_the_newer_root,
        "照抄的那条写在读根环次序里第一条自证过的根之后"
    );
    write_root_into_ring_slot(
        &mut image,
        &system_configuration,
        slot_of_the_newer_root,
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

/// 择新键（txg, 实例代号）完全相同的两条根：前面是 mkfs 那条（区域 0 的槽 0），后面那条照抄它、只把实例表指针的两条位置条目的校验和改掉（表读不出），
/// 写进区域 2 的槽 0（第 0 代根照 mkfs 种根的槽另认，C743；一个非 0 的 txg 只有一个落点槽，择新键完全相同的两条根只能是第 0 代的）。
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
        later_empty_slot: _,
    } = mkfs_pool_with_two_empty_slots_after_its_root("tied-choice-key-effective-floor");
    let later_genesis_slot = genesis_root_ring_slots()
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance::of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance::of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does,
    );
}
