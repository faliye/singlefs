//! m3-retire-r1 攻方 s2（副本用例，不入库）：在冻结树副本上走一段今天的 `crates/` 接受的历史，坐实模型里两步可达——
//! ① 同一个 inode 叶容器身份被「删空摘掉」两次（第一次摘掉之后管理员回退到它还活着的那一版，回退之后续做删除队列再摘一次）；
//! ② 第二次回退的目标是被第一次回退抛弃的那一段里的根（第一次摘掉那次发布的根），回退照样做成；
//! ③ 第二次回退之后，C 在谱系父链上那一版的叶单元仍在盘上原槽、逐字节没变（「按身份覆盖」那条写法复活要的前提）。
//! 退役记录今天没有写者：用例只打印每次摘掉的身份与那次发布的 txg（一个写退役记录的写者会在那两次发布各写一条），不造退役树。
//! 每一版交池级 checker（两盘内存稀疏盘），打印违例；盘都是 `SparseBlockDevice`。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::recovery::PoolReader;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_unlink_intent,
    publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::PackedIdentity;
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const DEVICE_BYTES: u64 = 4 << 30;
const WRITE_TIME_SECONDS: u64 = 1_788_000_000;

struct Pool {
    parameters: MakeFilesystemParameters,
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    instance: InstanceGeneration,
    current: TransactionOutput,
    device_bytes: u64,
}

fn pool_after_the_first_file() -> Pool { pool_after_the_first_file_on(DEVICE_BYTES, JOURNAL_RING_DEFAULT_BYTES) }

fn pool_after_the_first_file_on(device_bytes: u64, journal_ring_bytes: u64) -> Pool {
    let parameters = MakeFilesystemParameters {
        filesystem_identifier: *b"singlefs-opus-s2",
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512))))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = vec![7u8; 3000];
    let (instance, first) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(&mut writer, &mut allocator, warmed.roots.last().expect("暖机的根"),
            FirstFile { content: &content, write_time_seconds: WRITE_TIME_SECONDS }, instance, &warmed.last_record_bytes)
            .expect("第一个文件");
        (instance, first)
    };
    Pool { parameters, devices, allocator, instance, current: first, device_bytes }
}

fn image_of(pool: &Pool) -> MemoryPool {
    MemoryPool { devices: pool.devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: pool.device_bytes }
}

fn identities(version: &TransactionOutput) -> Vec<PackedIdentity> {
    version.inode_leaf_container_contents().iter().map(|c| c.identity).collect()
}

fn show(identity: &PackedIdentity) -> String {
    format!("({},{},{},{})", identity.birth_tree.0, identity.record_type, identity.container, identity.container_birth.0)
}

/// 池级 checker 判这一版的镜像，交回红的那几条（合法历史上应当一条都没有）。
fn violations(pool: &Pool) -> Vec<String> {
    check_pool_image(&image_of(pool)).into_iter().filter_map(|(name, verdict)| match verdict {
        InvariantVerdict::Violated(detail) => Some(format!("{name}: {detail}")),
        _ => None,
    }).collect()
}

fn line(pool: &Pool, what: &str) {
    let v = &pool.current;
    println!("S2C step={what} txg={} leaves={:?} deleted_inodes={:?} checker_violations={:?}", v.root.checkpoint_txg.0,
        identities(v).iter().map(|i| format!("{}#{}", show(i), v.inode_leaf_container_contents().iter().find(|c| c.identity == *i).map_or(0, |c| c.records.len()))).collect::<Vec<_>>(),
        v.deleted_inodes.iter().map(|i| i.0).collect::<Vec<_>>(), violations(pool));
}

/// unlink 一个空 inode：意图那次，再把队列走空（空 inode 没有数据单元，直接是最后一批）。交回意图那一版与最后一批那一版。
fn unlink(pool: &mut Pool, inode: u64) -> (TransactionOutput, TransactionOutput) {
    let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
    let intent = publish_unlink_intent(&mut writer, &mut pool.allocator, &pool.current, InodeNumber(inode), WRITE_TIME_SECONDS, pool.instance)
        .unwrap_or_else(|e| panic!("unlink {inode} 意图：{e:?}"));
    let mut current = intent.clone();
    publish_unlink_steps_until_the_queue_is_empty(&mut writer, &mut pool.allocator, &mut current, UnlinkBatchSize::OneExtentLowerLeaf, pool.instance)
        .expect("走空队列");
    pool.current = current.clone();
    (intent, current)
}

fn roll_back(pool: &mut Pool, target: &TransactionOutput) {
    let t = RollbackTarget { instance: target.root.instance, checkpoint_txg: target.root.checkpoint_txg };
    roll_back_by_a_forward_publish(&pool.parameters, &mut pool.devices, &mut pool.allocator, &mut pool.current, t)
        .unwrap_or_else(|e| panic!("回退到 txg {}：{e:?}", target.root.checkpoint_txg.0));
}

/// 第一个文件（txg 3）→ 一次建 235 个空 inode（叶 1 装 1..233，叶 234 = C 装 234..236）→ unlink 236、235、234
/// （最后一次的最后一批删空 C、整片不落盘：第一次摘掉）→ 管理员回退到 unlink 234 意图那一版（C 活着、234 在删除队列里）
/// → 续做删除队列（最后一批再删空 C：第二次摘掉，同一个身份四段）→ 管理员回退到第一次摘掉那次发布的根（被第一次回退抛弃的那一段里）。
#[test]
fn the_same_leaf_identity_is_dropped_twice_and_a_second_rollback_lands_in_the_segment_the_first_rollback_abandoned() {
    let mut pool = pool_after_the_first_file();
    line(&pool, "first_file");
    {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        pool.current = publish_new_inodes(&mut writer, &mut pool.allocator, &pool.current, 235, WRITE_TIME_SECONDS, pool.instance)
            .expect("建 235 个空 inode");
    }
    line(&pool, "create_235");
    let ids = identities(&pool.current);
    assert_eq!(ids.len(), 2, "两片叶");
    let c = ids[1];
    assert_eq!(c.container, 234, "第二片容器号 = 触发分裂那条记录的号");
    for inode in [236, 235] { unlink(&mut pool, inode); line(&pool, &format!("unlink_{inode}")); }
    let (intent_234, first_drop) = unlink(&mut pool, 234);
    line(&pool, "unlink_234_first_drop");
    assert!(!identities(&first_drop).contains(&c), "第一次摘掉");
    println!("S2C name=first_drop identity={} intent_txg={} drop_publish_txg={}", show(&c), intent_234.root.checkpoint_txg.0, first_drop.root.checkpoint_txg.0);
    let c_unit = intent_234.unit(TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree(1))).clone();
    roll_back(&mut pool, &intent_234);
    line(&pool, "rollback_1_to_the_intent_of_234");
    assert!(identities(&pool.current).contains(&c), "回退之后 C 又是活叶，身份四段不变");
    assert_eq!(pool.current.deleted_inodes, vec![InodeNumber(234)], "234 在删除队列里");
    {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        let mut current = pool.current.clone();
        publish_unlink_steps_until_the_queue_is_empty(&mut writer, &mut pool.allocator, &mut current, UnlinkBatchSize::OneExtentLowerLeaf, pool.instance)
            .expect("续做删除队列");
        pool.current = current;
    }
    line(&pool, "queue_continued_second_drop");
    assert!(!identities(&pool.current).contains(&c), "第二次摘掉同一个身份");
    println!("S2C name=second_drop identity={} drop_publish_txg={}", show(&c), pool.current.root.checkpoint_txg.0);
    roll_back(&mut pool, &first_drop);
    line(&pool, "rollback_2_to_the_first_drop_root");
    assert!(!identities(&pool.current).contains(&c), "第二次回退到的那一版里 C 已摘掉");
    let image = image_of(&pool);
    let on_disk = image.read(DeviceIdentity(0), c_unit.slot.to_device_offset(), c_unit.bytes.len()).expect("读得出");
    println!("S2C name=c_version_on_the_chain slot={} bytes={} unchanged_on_disk={} lineage_tree_records={:?}", c_unit.slot.0, c_unit.bytes.len(), on_disk == c_unit.bytes,
        pool.current.lineage_containers.iter().flat_map(|c| c.contents.records.iter().map(|r| format!("{r:?}"))).collect::<Vec<_>>());
}

/// K4：谱系那一项（D28 已定项 4「容器 2 块 + 谱系树根 1 块 + 谱系树内部层数」，`admission::checkpoint_cost_of_one_lineage_container_cow`）
/// 若照搬给退役树，按「要接在后面的那一版」现读层数时，在根满长高那次发布上是不是上界。旋钮压小（一片 1 条、一个节点 2 条）照
/// `container_trees_grow_a_level_when_the_root_is_full.rs` 的谱系那一条：回退三次到第一个文件那一版，第三次长高。
/// 每次回退之前按上一版算谱系那一项与整个 ckpt_cost，回退之后数谱系树角色写了几块、归保留池的角色一共写了几块。
#[test]
fn the_lineage_term_read_on_the_version_to_build_on_against_the_blocks_the_growth_publish_writes() {
    use singlefs_core::admission::{checkpoint_cost_of_one_lineage_container_cow, checkpoint_cost_of_the_version_to_build_on, space_budget_of_role, SpaceBudgetOfARole};
    use singlefs_core::container_tree::{ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests};
    use singlefs_core::lineage_tree::{LineageContainerCapacity, LineageRecordsPerContainerCappedForTests};
    use singlefs_core::unlink::DeletionSwitches;
    let mut pool = pool_after_the_first_file();
    let first = pool.current.clone();
    pool.allocator.set_deletion_switches(DeletionSwitches {
        lineage_container_capacity: LineageContainerCapacity::CappedForTests(LineageRecordsPerContainerCappedForTests::new(1).expect("1..=2039")),
        container_tree_node_entries: ContainerTreeNodeEntries::CappedForTests(ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135")),
        ..DeletionSwitches::PRODUCT
    });
    for n in 1..=4 {
        let before = pool.current.clone();
        let term_before = checkpoint_cost_of_one_lineage_container_cow(before.lineage_tree_internal_levels()).0;
        let total_before = checkpoint_cost_of_the_version_to_build_on(Some(&before), &pool.allocator).0;
        roll_back(&mut pool, &first);
        let after = &pool.current;
        let lineage_blocks: u64 = after.rewritten.iter().filter(|r| matches!(r,
            TransactionUnit::LineageLeafContainer(_) | TransactionUnit::LineageRoot | TransactionUnit::LineageNodeBelowTheRoot(_)))
            .map(|r| r.span_slots()).sum();
        let reserve_pool_blocks: u64 = after.rewritten.iter().filter(|r| space_budget_of_role(**r) == SpaceBudgetOfARole::CheckpointReservePool)
            .map(|r| r.span_slots()).sum();
        let all_blocks: u64 = after.rewritten.iter().map(|r| r.span_slots()).sum();
        println!("S2C name=lineage_term rollback={n} txg={} internal_levels_before={} lineage_term_before={term_before} lineage_blocks_written={lineage_blocks} lineage_roles={:?} ckpt_cost_before={total_before} reserve_pool_role_blocks_written={reserve_pool_blocks} all_role_blocks_written={all_blocks} checker_violations={:?}",
            after.root.checkpoint_txg.0, before.lineage_tree_internal_levels(),
            after.rewritten.iter().filter(|r| matches!(r, TransactionUnit::LineageLeafContainer(_) | TransactionUnit::LineageRoot | TransactionUnit::LineageNodeBelowTheRoot(_))).map(|r| r.tag()).collect::<Vec<_>>(),
            violations(&pool));
    }
}

/// K4：删空摘掉那次发布（退役记录要写进的那一次）每盘新占几个槽，对 W（D28 已定项 1「环窗口」，
/// `admission::slots_one_deletion_publish_newly_takes_on_one_device_at_most`，W = 24 + Σ_ℓ min(N_ℓ, 2·(K + 26))，24 = 普通分配 16 + 固定点根 4 + 映射树根以下 4）。
/// 删除类发布整次不判式子（`admission::space_budget_of_role` 的注），空间罩在环窗口 (3S + 1) × W 里。`crates/` 今天删空一片时它右边每一片位置左移、全部重写
/// （`inode_tree::remove_record_from_leaf_containers`）。100 MiB 盘、16 MiB journal 环（`core_review_geometry_back_chain_and_empty_inode.rs` 那一档）上建 `leaves` 片满叶，
/// 把第二片（234..466）逐个 unlink 到删空，打印删空那次与之前各次删除类发布的每盘新占槽数、其中普通分配几个、W 是多少。
#[test]
fn slots_a_drop_publish_newly_takes_against_the_ring_window_bound_on_a_narrow_pool() {
    use singlefs_core::admission::{ring_window_on_one_device, space_budget_of_role, SpaceBudgetOfARole};
    use singlefs_format::{INODE_LEAF_RECORDS, ROOT_RING_REGIONS, SLOT_BYTES};
    for (device_mib, leaves) in [(100u64, 20u64), (100, 40)] {
        let mut pool = pool_after_the_first_file_on(device_mib << 20, 16 << 20);
        {
            let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
            pool.current = publish_new_inodes(&mut writer, &mut pool.allocator, &pool.current, leaves * INODE_LEAF_RECORDS - 1, WRITE_TIME_SECONDS, pool.instance)
                .expect("建满叶");
        }
        assert_eq!(identities(&pool.current).len() as u64, leaves);
        let s = RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count();
        let batch = pool.allocator.deletion_switches().batch_size.data_units_per_batch();
        let window_slots = ring_window_on_one_device(&pool.allocator, RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM, batch).0 / SLOT_BYTES;
        let w = window_slots / (ROOT_RING_REGIONS * s + 1);
        let mut max_other = (0u64, 0u64);
        let mut drop = None;
        for inode in (INODE_LEAF_RECORDS + 1..=2 * INODE_LEAF_RECORDS).rev() {
            let before = identities(&pool.current).len();
            let (intent, last) = unlink(&mut pool, inode);
            for v in [&intent, &last] {
                let all: u64 = v.rewritten.iter().map(|r| r.span_slots()).sum();
                let ordinary: u64 = v.rewritten.iter().filter(|r| space_budget_of_role(**r) == SpaceBudgetOfARole::OrdinaryAllocation).map(|r| r.span_slots()).sum();
                if identities(&last).len() < before && std::ptr::eq(v, &last) { drop = Some((v.root.checkpoint_txg.0, all, ordinary, v.rewritten.len())); }
                else if (all, ordinary) > max_other { max_other = (all, ordinary); }
            }
        }
        let (txg, all, ordinary, roles) = drop.expect("第二片删空了");
        println!("S2C name=ring_window_bound device_mib={device_mib} leaves_before={leaves} W={w} window_slots={window_slots} drop_publish_txg={txg} drop_publish_new_slots_per_device={all} of_which_ordinary_allocation={ordinary} roles={roles} other_deletion_publishes_max_new_slots={} max_ordinary={} drop_exceeds_W={} checker_violations={:?}",
            max_other.0, max_other.1, all > w, violations(&pool));
    }
}
