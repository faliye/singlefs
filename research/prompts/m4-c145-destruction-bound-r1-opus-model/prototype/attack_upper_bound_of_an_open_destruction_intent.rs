//! 攻方腿原型（m4-c145-destruction-bound-r1，副本里，不入库）：未完成销毁意图的区间上界。
//! 两块内存稀疏盘上 mkfs、取号、暖机、第一个文件，之后直接调发布函数造历史；每一版打一行
//! `name=attack_version ...`（意图 txg、lo、P、最大纪元、纪元 > lo 的待放条目数、池级 checker 判违例的不变量），
//! 并在每一版的盘拷贝上可写挂载一次（崩在这一版之后的续做），挂完再过池级 checker。
mod common;
mod common_pointer_head_corruption;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, TreeIdentifier};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::records::{birth_txg_of_a_mapping_key, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::transaction::{
    acquire_instance, publish_create_snapshot, publish_first_file, publish_overwrite,
    publish_snapshot_destruction_intent, publish_snapshot_destruction_step, warm_up,
    walk_the_deletion_queue_until_it_is_empty, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const WRITE_TIME_SECONDS: u64 = 1_788_000_000;

struct Pool {
    parameters: MakeFilesystemParameters,
    device_bytes: u64,
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    instance: InstanceGeneration,
    current: TransactionOutput,
}

fn content(seed: u32) -> Vec<u8> {
    (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + seed) % 239).expect("小于 256"))
        .collect()
}

fn pool_after_the_first_file() -> Pool {
    let width = HistoryDeviceWidth::FourGibibytes;
    let parameters = width.parameters();
    let device_bytes = width.device_bytes();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512))))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let first = content(1);
    let (instance, current) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let current = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两条根"),
            FirstFile { content: &first, write_time_seconds: WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件");
        (instance, current)
    };
    Pool { parameters, device_bytes, devices, allocator, instance, current }
}

impl Pool {
    fn image(&self) -> MemoryPool {
        MemoryPool {
            devices: self.devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
            device_size_in_bytes: self.device_bytes,
        }
    }

    fn snapshot(&mut self) -> TreeIdentifier {
        let mut writer = PoolWriter::new(&self.parameters, self.devices.as_mut_slice());
        self.current = publish_create_snapshot(&mut writer, &mut self.allocator, &self.current, self.instance).expect("建快照");
        newest_snapshot(&self.current)
    }

    fn overwrite(&mut self, seed: u32) {
        let bytes = content(seed);
        let mut writer = PoolWriter::new(&self.parameters, self.devices.as_mut_slice());
        self.current = publish_overwrite(
            &mut writer,
            &mut self.allocator,
            &self.current,
            FirstFile { content: &bytes, write_time_seconds: WRITE_TIME_SECONDS + u64::from(seed) },
            self.instance,
        )
        .expect("覆盖写");
    }

    fn intent(&mut self, snapshot: TreeIdentifier) -> u64 {
        let mut writer = PoolWriter::new(&self.parameters, self.devices.as_mut_slice());
        self.current = publish_snapshot_destruction_intent(&mut writer, &mut self.allocator, &self.current, snapshot, self.instance)
            .expect("删快照的意图");
        self.current.root.checkpoint_txg.0
    }

    fn one_batch(&mut self, snapshot: TreeIdentifier) {
        let mut writer = PoolWriter::new(&self.parameters, self.devices.as_mut_slice());
        self.current = publish_snapshot_destruction_step(
            &mut writer,
            &mut self.allocator,
            &self.current,
            snapshot,
            UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("压成 1")),
            self.instance,
        )
        .expect("一批");
    }

    fn walk_to_empty(&mut self) {
        let mut writer = PoolWriter::new(&self.parameters, self.devices.as_mut_slice());
        let walk = walk_the_deletion_queue_until_it_is_empty(
            &mut writer, &mut self.allocator, &mut self.current, UnlinkBatchSize::OneExtentLowerLeaf, self.instance,
        );
        walk.stopped_by.expect("走空");
    }
}

fn snapshots_in_chain_order(version: &TransactionOutput) -> Vec<(u64, TreeIdentifier, u64)> {
    let mut chain: Vec<(u64, TreeIdentifier, u64)> = version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| (entry.birth_txg.0, entry.tree, entry.previous_snapshot_txg.0))
        .collect();
    chain.sort_unstable_by_key(|(txg, tree, _)| (*txg, tree.0));
    chain
}

fn newest_snapshot(version: &TransactionOutput) -> TreeIdentifier {
    snapshots_in_chain_order(version).last().expect("至少一个快照").1
}

fn violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}:{detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

struct Observation {
    max_epoch: u64,
    pending_entries_with_an_epoch_above_lo: usize,
    violations: usize,
    violations_after_the_continuation: usize,
}

/// 看现行那一版：`destroyed` 是正在删的快照（lo、P、意图 txg）。
fn observe(pool: &Pool, history: &str, step: &str, destroyed: (TreeIdentifier, u64, u64, u64)) -> Observation {
    let (snapshot, lo, predecessor, intent_txg) = destroyed;
    let version = &pool.current;
    let still_being_destroyed = version.snapshots_being_destroyed.contains(&snapshot);
    let max_epoch = version.deadlist_entries.iter().map(|entry| entry.epoch.0).max().unwrap_or(0);
    let pending_entries_with_an_epoch_above_lo = if still_being_destroyed {
        version
            .deadlist_entries
            .iter()
            .filter(|entry| {
                entry.epoch.0 > lo
                    && birth_txg_of_a_mapping_key(&entry.mapping_key).is_some_and(|birth| birth.0 > predecessor)
            })
            .count()
    } else {
        0
    };
    let found = violations(&pool.image());
    let mut cloned: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
        .devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(pool.device_bytes, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect();
    let mounted = mount_writable(&pool.parameters, &mut cloned).expect("这一版之后崩了再可写挂载");
    let after = mounted.current.version_of_trees().expect("有树").clone();
    let after_image = MemoryPool {
        devices: cloned.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: pool.device_bytes,
    };
    let found_after = violations(&after_image);
    println!(
        "name=attack_version history={history} step={step} txg={} chain={:?} head_previous={} being_destroyed={:?} lo={lo} predecessor={predecessor} intent_txg={intent_txg} deadlist_entries={} epochs={:?} max_epoch={max_epoch} max_epoch_below_intent_txg={} pending_entries_with_an_epoch_above_lo={pending_entries_with_an_epoch_above_lo} violations={} continuation_snapshots_left={} continuation_deadlist_entries_left={} violations_after_the_continuation={}",
        version.root.checkpoint_txg.0,
        snapshots_in_chain_order(version),
        version
            .tree_table_entries
            .iter()
            .find(|entry| entry.kind == singlefs_core::records::TREE_KIND_INODE)
            .map(|entry| entry.previous_snapshot_txg.0)
            .unwrap_or(0),
        version.snapshots_being_destroyed,
        version.deadlist_entries.len(),
        version.deadlist_entries.iter().map(|entry| (entry.epoch.0, birth_txg_of_a_mapping_key(&entry.mapping_key).map(|birth| birth.0))).collect::<Vec<_>>(),
        max_epoch < intent_txg,
        found.len(),
        snapshots_in_chain_order(&after).len(),
        after.deadlist_entries.len(),
        found_after.len(),
    );
    for violation in found.iter().chain(found_after.iter()) {
        println!("name=attack_violation history={history} step={step} {violation}");
    }
    Observation { max_epoch, pending_entries_with_an_epoch_above_lo, violations: found.len(), violations_after_the_continuation: found_after.len() }
}

/// 写意图、逐批走到最后一批，每一版都看。交回每一版的观察。
fn destroy_one_batch_at_a_time(pool: &mut Pool, history: &str, snapshot: TreeIdentifier) -> Vec<Observation> {
    let chain = snapshots_in_chain_order(&pool.current);
    let (lo, _, predecessor) = *chain.iter().find(|(_, tree, _)| *tree == snapshot).expect("快照在树表里");
    let intent_txg = pool.intent(snapshot);
    let destroyed = (snapshot, lo, predecessor, intent_txg);
    let mut observed = vec![observe(pool, history, "intent", destroyed)];
    let mut batch = 0;
    while pool.current.snapshots_being_destroyed.contains(&snapshot) {
        batch += 1;
        pool.one_batch(snapshot);
        observed.push(observe(pool, history, &format!("batch{batch}"), destroyed));
    }
    for observation in &observed {
        assert!(observation.max_epoch < intent_txg, "{history}：deadlist 里有纪元 ≥ 意图 txg {intent_txg} 的条目");
        assert_eq!(observation.violations, 0, "{history}：池级 checker 判违例");
        assert_eq!(observation.violations_after_the_continuation, 0, "{history}：续做之后池级 checker 判违例");
    }
    observed
}

#[test]
fn destroying_the_older_snapshot_after_the_newer_one_leaves_pending_entries_with_an_epoch_above_lo() {
    let mut pool = pool_after_the_first_file();
    let older = pool.snapshot();
    let newer = pool.snapshot();
    pool.overwrite(13);
    pool.intent(newer);
    pool.walk_to_empty();
    let observed = destroy_one_batch_at_a_time(&mut pool, "older_after_newer", older);
    assert!(
        observed.iter().any(|observation| observation.pending_entries_with_an_epoch_above_lo > 0),
        "这段历史里删旧快照时有纪元 > lo 的待放条目"
    );
}

#[test]
fn destroying_the_only_snapshot_one_batch_at_a_time() {
    let mut pool = pool_after_the_first_file();
    let only = pool.snapshot();
    pool.overwrite(17);
    destroy_one_batch_at_a_time(&mut pool, "only", only);
}

#[test]
fn destroying_the_newest_snapshot_while_an_older_one_lives() {
    let mut pool = pool_after_the_first_file();
    let older = pool.snapshot();
    pool.overwrite(19);
    let newer = pool.snapshot();
    pool.overwrite(23);
    destroy_one_batch_at_a_time(&mut pool, "newest_with_an_older_live", newer);
    destroy_one_batch_at_a_time(&mut pool, "older_after_the_newest_with_entries_of_two_epochs", older);
}

#[test]
fn destroying_the_middle_snapshot_then_the_newest_then_the_oldest() {
    let mut pool = pool_after_the_first_file();
    let oldest = pool.snapshot();
    pool.overwrite(29);
    let middle = pool.snapshot();
    pool.overwrite(31);
    let newest = pool.snapshot();
    pool.overwrite(37);
    destroy_one_batch_at_a_time(&mut pool, "middle_of_three", middle);
    destroy_one_batch_at_a_time(&mut pool, "newest_of_two", newest);
    // 删最新那个时纪元 8、birth 7 > P = 4 的四条全放了，删最旧那个时只剩纪元 4 的条目：这段历史不造「纪元 > lo 的待放条目」，只看逐版不红。
    destroy_one_batch_at_a_time(&mut pool, "oldest_last", oldest);
}

/// 直接调发布函数时 `publish_overwrite`（第一个文件）不核承诺类队列：在意图与最后一批之间插覆盖写（会话走不到这一形，事务层走得到）。
#[test]
fn overwrites_between_the_intent_and_the_final_batch_add_no_entry_at_or_above_the_intent_txg() {
    let mut pool = pool_after_the_first_file();
    let older = pool.snapshot();
    pool.overwrite(41);
    let newer = pool.snapshot();
    pool.overwrite(43);
    let chain = snapshots_in_chain_order(&pool.current);
    let (lo, _, predecessor) = *chain.iter().find(|(_, tree, _)| *tree == newer).expect("新快照");
    let intent_txg = pool.intent(newer);
    let destroyed = (newer, lo, predecessor, intent_txg);
    let mut observed = vec![observe(&pool, "overwrites_inside_the_destruction", "intent", destroyed)];
    let mut round = 0_u32;
    while pool.current.snapshots_being_destroyed.contains(&newer) {
        round += 1;
        pool.overwrite(50 + round);
        observed.push(observe(&pool, "overwrites_inside_the_destruction", &format!("overwrite{round}"), destroyed));
        pool.one_batch(newer);
        observed.push(observe(&pool, "overwrites_inside_the_destruction", &format!("batch{round}"), destroyed));
    }
    for observation in &observed {
        assert!(observation.max_epoch < intent_txg, "纪元 ≥ 意图 txg {intent_txg} 的条目");
        assert_eq!(observation.violations, 0, "池级 checker 判违例");
        assert_eq!(observation.violations_after_the_continuation, 0, "续做之后池级 checker 判违例");
    }
    destroy_one_batch_at_a_time(&mut pool, "older_after_overwrites_inside_the_destruction", older);
}

/// deadlist key 43 字节按字段 [8, 8, 1, 8, 8, 4, 6] 逐段读（盘上的序，同 must-red 世界那一份的读法）。
fn deadlist_key_fields(entry: &[u8]) -> Vec<u64> {
    let mut fields = Vec::new();
    let mut offset = 0;
    for width in [8usize, 8, 1, 8, 8, 4, 6] {
        let mut wide = [0u8; 8];
        wide[..width].copy_from_slice(&entry[offset..offset + width]);
        fields.push(u64::from_le_bytes(wide));
        offset += width;
    }
    fields
}

fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = singlefs_core::checksum::crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(node, header_end);
}

/// 现行那一版的 deadlist 叶（单层）里每一条的纪元换成 `epoch_of`（按原纪元给新纪元），重排、重封、校验和沿引用链补到根槽。
fn forge_the_epochs_of_the_newest_deadlist_leaf(image: &mut MemoryPool, epoch_of: impl Fn(u64) -> u64) {
    use singlefs_core::unit::{build_index_node, parse_index_node};
    let leaf_slot = common_pointer_head_corruption::the_slot_of(
        &common_pointer_head_corruption::tree_table_entry_of_kind(image, singlefs_core::records::TREE_KIND_DEADLIST).root,
    );
    let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
    let leaf = common_pointer_head_corruption::unit_bytes_at(image, leaf_slot, node_bytes);
    let header = parse_index_node(&leaf).expect("deadlist 叶解得开");
    assert_eq!(header.level, 0, "deadlist 树只有一层");
    let mut entries = header.entries.clone();
    for entry in &mut entries {
        let old = u64::from_le_bytes(entry[8..16].try_into().expect("8 字节"));
        entry[8..16].copy_from_slice(&epoch_of(old).to_le_bytes());
    }
    entries.sort_by_key(|entry| deadlist_key_fields(entry));
    let mut rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &entries.first().expect("至少一条")[..43],
        &entries.last().expect("至少一条")[..43],
        header.birth_txg,
        &HistoryDeviceWidth::FourGibibytes.parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽装得进 u16"),
        &entries,
    );
    reseal_index_node(&mut rebuilt);
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &rebuilt);
}

/// 判别力：意图那一版（唯一的快照、下一侧是活头）把 deadlist 每一条的纪元改成远大于这一版 txg 的值。
/// 合法镜像上纪元 < 写它那次发布的 txg ≤ 这一版的 txg；∞ 判法下它们仍在待放集里。打出改前改后各判违例的不变量。
#[test]
fn forged_epochs_far_above_the_version_txg_inside_an_open_intent() {
    let mut pool = pool_after_the_first_file();
    let only = pool.snapshot();
    pool.overwrite(61);
    let intent_txg = pool.intent(only);
    let version_txg = pool.current.root.checkpoint_txg.0;
    let legal = pool.image();
    let before = violations(&legal);
    let mut forged = legal.clone();
    forge_the_epochs_of_the_newest_deadlist_leaf(&mut forged, |old| old + 1_000_000);
    let after = violations(&forged);
    println!(
        "name=attack_forged_epoch intent_txg={intent_txg} version_txg={version_txg} forged_epoch_offset=1000000 violations_before={} violations_after={}",
        before.len(),
        after.len()
    );
    for violation in &after {
        println!("name=attack_forged_epoch_violation {violation}");
    }
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = forged
        .devices
        .iter()
        .map(|(identity, image)| {
            let mut device = SparseBlockDevice::new(pool.device_bytes, PhysicalBlockSizeInBytes(512));
            device.image = image.clone();
            (*identity, device)
        })
        .collect();
    let mounted = mount_writable(&pool.parameters, &mut devices);
    match mounted {
        Ok(mounted) => {
            let continued = mounted.current.version_of_trees().expect("有树").clone();
            let continued_image = MemoryPool {
                devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
                device_size_in_bytes: pool.device_bytes,
            };
            let after_the_continuation = violations(&continued_image);
            println!(
                "name=attack_forged_epoch_continuation txg={} snapshots_left={} deadlist_entries_left={} violations_after_the_continuation={}",
                continued.root.checkpoint_txg.0,
                snapshots_in_chain_order(&continued).len(),
                continued.deadlist_entries.len(),
                after_the_continuation.len()
            );
            for violation in &after_the_continuation {
                println!("name=attack_forged_epoch_continuation_violation {violation}");
            }
        }
        Err(refusal) => println!("name=attack_forged_epoch_continuation refused={refusal:?}"),
    }
    assert!(before.is_empty(), "改之前是合法镜像");
}
