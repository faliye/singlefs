//! 树表里的只读快照条目（码 14 快照 inode 根、码 15 快照 extent 根，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
//! 第三节 S1 与第七节岔路 1）在核心层读树表那一步怎么判：一个快照一对，两种可以各有多条；两种条数相等、每条码 15 的头 ID 恰是一条
//! 只属于它的码 14 的树 ID、码 14 至多 `SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION` 条、码 14 的头 ID 是活头 inode 树的树 ID，
//! 四样任一样不成立，读树表的五个读者都判树表损坏；都成立时五个读者都放行（冷走读跳过它们、用户可见根那一读把它们的根指针记下，
//! 判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S2），从盘上重建那一版把它们配成对交出来，不解它们指着的根。
//! 树表是手造的：第一个文件那一版（txg 3）的十二条条目之后按树 ID 升序接上快照条目（树 ID 从那条根的水位起连发，根的水位跟着抬），
//! 树表单元重建、写回它两条位置条目指的槽；读者拿换过树表指针校验和与水位的那条根去读。盘都是内存稀疏盘。

use singlefs_core::address::{DeviceIdentity, TreeIdentifier};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::JournalRecord;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mounted_read::{open_pool_for_read, OpenPoolForReadFailure};
use singlefs_core::records::{
    TreeTableEntry, SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, TREE_KIND_EXTENT, TREE_KIND_INODE,
    TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::{
    allocation_records_under_root, rebuild_version, rebuild_version_with_its_snapshot_entries,
    user_visible_tree_root_pointers, walk_to_file, PoolReader, RebuildVersionFailure,
    RecoveryFailure, SnapshotTreeTableEntries, UserVisibleTreeRootPointers,
    TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS,
    TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY,
    TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER,
    TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const DEVICE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-snap-s1";
const WRITE_TIME_SECONDS: u64 = 1_788_100_000;
/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);

fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

/// 两块内存盘上 mkfs → 取号 → 暖机 → 第一个文件（txg 3）之后的池与第一个文件那次发布的输出。
fn pool_after_the_first_file() -> (MemoryPool, TransactionOutput) {
    let parameters = MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
    let mut allocator: PoolAllocator =
        allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content: Vec<u8> = (0..3000u32)
        .map(|index| u8::try_from((index * 137 + 3) % 251).expect("小于 256"))
        .collect();
    let first = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件")
    };
    let image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    };
    (image, first)
}

/// 一条根指着的树表单元，按它第一条位置条目读回来。
fn tree_table_unit_of(image: &MemoryPool, root: &RootRecord) -> Vec<u8> {
    let location = root.tree_table.locations[0];
    PoolReader::read(
        image,
        location.device,
        location.slot.to_device_offset(),
        node_bytes(),
    )
    .expect("树表单元读得到")
}

/// 第一个文件那一版树表里的十二条条目（盘上原样、按盘上次序）。
fn entries_of_the_first_file_version(image: &MemoryPool, root: &RootRecord) -> Vec<Vec<u8>> {
    parse_index_node(&tree_table_unit_of(image, root))
        .expect("树表单元解得开")
        .entries
}

fn entry_of_kind(entries: &[Vec<u8>], kind: u16) -> TreeTableEntry {
    entries
        .iter()
        .map(|entry_bytes| TreeTableEntry::parse(entry_bytes).expect("写者写的条目解得开"))
        .find(|entry| entry.kind == kind)
        .expect("第一个文件那一版的树表里这种树恰好一条")
}

/// 用 `entries` 照原树表单元的头重建一个树表单元（key 区间取首条与末条的树 ID），写回它两条位置条目指的槽（两块盘各一份），
/// 交回一条树表指针两条校验和换过、树 ID 水位换成 `tree_identifier_watermark`、别的字段与 `root` 相同的根。
fn root_whose_tree_table_carries(
    image: &mut MemoryPool,
    root: &RootRecord,
    entries: &[Vec<u8>],
    tree_identifier_watermark: u64,
) -> RootRecord {
    let header = parse_index_node(&tree_table_unit_of(image, root)).expect("树表单元解得开");
    let key_of = |entry_bytes: &Vec<u8>| entry_bytes[..8].to_vec();
    let rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &key_of(entries.first().expect("树表有条目")),
        &key_of(entries.last().expect("树表有条目")),
        header.birth_txg,
        &FILESYSTEM_IDENTIFIER,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽 200"),
        entries,
    );
    let checksum = crc32_castagnoli(&rebuilt);
    let mut rewritten_root = *root;
    for location in &mut rewritten_root.tree_table.locations {
        image
            .devices
            .get_mut(&location.device)
            .expect("池里有这块盘")
            .write(location.slot.to_device_offset(), &rebuilt);
        location.unit_checksum = checksum;
    }
    rewritten_root.tree_identifier_watermark = tree_identifier_watermark;
    rewritten_root
}

/// 一个快照照判决 S1 的字段取值造的那一对条目：码 14 抄活头 inode 树条目（根指针、`previous_snapshot_txg` 都是捕获那一刻活头的），
/// 树 ID 换成 `snapshot_inode_root_tree`、诞生 txg 换成 S.txg（这里取那条根的 txg）、头 ID 是活头 inode 树的树 ID；
/// 码 15 抄活头 extent 树条目（`previous_snapshot_txg` 是 0），树 ID 是码 14 的下一个号、头 ID 是码 14 的树 ID。
fn snapshot_entry_pair(
    live_inode_tree_entry: TreeTableEntry,
    live_extent_tree_entry: TreeTableEntry,
    snapshot_inode_root_tree: u64,
    snapshot_txg: u64,
) -> SnapshotTreeTableEntries {
    let mut inode_root = live_inode_tree_entry;
    inode_root.kind = TREE_KIND_SNAPSHOT_INODE_ROOT;
    inode_root.tree = TreeIdentifier(snapshot_inode_root_tree);
    inode_root.birth_txg.0 = snapshot_txg;
    inode_root.head_identifier = live_inode_tree_entry.tree.0;
    let mut extent_root = live_extent_tree_entry;
    extent_root.kind = TREE_KIND_SNAPSHOT_EXTENT_ROOT;
    extent_root.tree = TreeIdentifier(snapshot_inode_root_tree + 1);
    extent_root.birth_txg.0 = snapshot_txg;
    extent_root.head_identifier = snapshot_inode_root_tree;
    SnapshotTreeTableEntries {
        inode_root,
        extent_root,
    }
}

/// 第一个文件那一版加 `pair_count` 个快照：快照的树 ID 从那条根的水位起每个快照连发两个（判决 S2「新树号从水位连发两个」）。
struct VersionWithSnapshots {
    image: MemoryPool,
    first: TransactionOutput,
    original_entries: Vec<Vec<u8>>,
    pairs: Vec<SnapshotTreeTableEntries>,
}

fn version_with_snapshot_pairs(pair_count: u64) -> VersionWithSnapshots {
    let (image, first) = pool_after_the_first_file();
    let original_entries = entries_of_the_first_file_version(&image, &first.root);
    let live_inode_tree_entry = entry_of_kind(&original_entries, TREE_KIND_INODE);
    let live_extent_tree_entry = entry_of_kind(&original_entries, TREE_KIND_EXTENT);
    let watermark = first.root.tree_identifier_watermark;
    let pairs = (0..pair_count)
        .map(|snapshot_position| {
            snapshot_entry_pair(
                live_inode_tree_entry,
                live_extent_tree_entry,
                watermark + 2 * snapshot_position,
                first.root.checkpoint_txg.0,
            )
        })
        .collect();
    VersionWithSnapshots {
        image,
        first,
        original_entries,
        pairs,
    }
}

impl VersionWithSnapshots {
    /// 十二条原样条目之后接上 `snapshot_entries`（调用方按树 ID 升序给），树 ID 水位抬到最大那个快照树 ID 之上。
    fn root_carrying(&mut self, snapshot_entries: &[TreeTableEntry]) -> RootRecord {
        let mut entries = self.original_entries.clone();
        entries.extend(snapshot_entries.iter().map(TreeTableEntry::to_bytes));
        let watermark = snapshot_entries
            .iter()
            .map(|entry| entry.tree.0 + 1)
            .max()
            .unwrap_or(self.first.root.tree_identifier_watermark);
        let root = self.first.root;
        root_whose_tree_table_carries(&mut self.image, &root, &entries, watermark)
    }

    fn entries_of_every_pair(&self) -> Vec<TreeTableEntry> {
        self.pairs
            .iter()
            .flat_map(|pair| [pair.inode_root, pair.extent_root])
            .collect()
    }
}

/// 读树表的五个读者各自读一条根的结局（读下去了是 `Ok(())`）。
#[derive(Debug, PartialEq, Eq)]
struct TreeTableReaderOutcomes {
    rebuild_version: Result<(), RebuildVersionFailure>,
    allocation_records_under_root: Result<(), RecoveryFailure>,
    walk_to_file: Result<(), RecoveryFailure>,
    user_visible_tree_root_pointers: Result<(), RecoveryFailure>,
    open_pool_for_read: Result<(), OpenPoolForReadFailure>,
}

fn outcomes_of_every_tree_table_reader(
    image: &MemoryPool,
    root: &RootRecord,
    record_standing_for_root: &JournalRecord,
) -> TreeTableReaderOutcomes {
    let mut mapping_fallbacks = 0;
    TreeTableReaderOutcomes {
        rebuild_version: rebuild_version(image, root, Some(record_standing_for_root.clone()))
            .map(|_rebuilt| ()),
        allocation_records_under_root: allocation_records_under_root(image, root)
            .map(|_records| ()),
        walk_to_file: walk_to_file(image, root, &mut mapping_fallbacks).map(|_content| ()),
        user_visible_tree_root_pointers: user_visible_tree_root_pointers(image, root)
            .map(|_pointers| ()),
        open_pool_for_read: open_pool_for_read(image, root).map(|_mounted| ()),
    }
}

fn every_reader_refuses_the_tree_table_with(what: &'static str) -> TreeTableReaderOutcomes {
    let damaged = RecoveryFailure::UnitMalformed { what };
    TreeTableReaderOutcomes {
        rebuild_version: Err(RebuildVersionFailure::Walk(damaged.clone())),
        allocation_records_under_root: Err(damaged.clone()),
        walk_to_file: Err(damaged.clone()),
        user_visible_tree_root_pointers: Err(damaged.clone()),
        open_pool_for_read: Err(OpenPoolForReadFailure::Walk(damaged)),
    }
}

/// 两个快照（两对码 14 / 码 15）：同一种出现两条不再判「同一种树两条」，从盘上重建那一版照常建得出，快照条目按码 14 的树 ID 升序配成对交出来，
/// 逐字段与写进去的相等；重建出来的那一版与不带快照条目那一份的读法（`rebuild_version`）交回的相同。
/// 对照：第一个文件那一版原样（没有快照条目）交空表。
#[test]
fn a_tree_table_carrying_two_snapshot_pairs_is_rebuilt_and_hands_back_both_pairs() {
    let mut version = version_with_snapshot_pairs(2);
    let record = version.first.record.clone();
    let unchanged = rebuild_version_with_its_snapshot_entries(
        &version.image,
        &version.first.root,
        Some(record.clone()),
    )
    .expect("第一个文件那一版建得出");
    assert_eq!(
        unchanged.snapshot_entries,
        Vec::new(),
        "对照：没有快照条目的树表交空表"
    );

    let snapshot_entries = version.entries_of_every_pair();
    let root = version.root_carrying(&snapshot_entries);
    let rebuilt =
        rebuild_version_with_its_snapshot_entries(&version.image, &root, Some(record.clone()))
            .expect("两对快照条目的树表：重建照常建得出");
    assert_eq!(
        rebuilt.snapshot_entries, version.pairs,
        "两对快照条目按码 14 的树 ID 升序配成对交出来，逐字段与写进去的相等"
    );
    assert_eq!(
        Ok(rebuilt.version),
        rebuild_version(&version.image, &root, Some(record)),
        "重建出来的那一版与丢掉快照条目的那一份读法交回的相同"
    );
}

/// 两个快照（两对码 14 / 码 15）的合法树表：读树表的五个读者全部放行（判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
/// 第二节 S2：冷走读对码 14 / 15 跳过、照常读出活头那一版的文件，用户可见根那一读把它们计进用户可见根）。用户可见根那一读交回的
/// 四条快照根指针按盘上次序、逐字节是写进去的那四条条目的根指针；inode 树、extent 树两条与不带快照条目的那一版读出的相同。
#[test]
fn a_tree_table_carrying_two_snapshot_pairs_is_admitted_by_every_reader() {
    let mut version = version_with_snapshot_pairs(2);
    let record = version.first.record.clone();
    let without_snapshots = user_visible_tree_root_pointers(&version.image, &version.first.root)
        .expect("第一个文件那一版读得出用户可见根");
    let mut mapping_fallbacks = 0;
    // 换树表之前读：`root_carrying` 把树表单元写回原来那两个槽，原来那条根的树表校验和随之对不上。
    let file_without_snapshots =
        walk_to_file(&version.image, &version.first.root, &mut mapping_fallbacks)
            .expect("第一个文件那一版冷走读得出文件");
    let snapshot_entries = version.entries_of_every_pair();
    let root = version.root_carrying(&snapshot_entries);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
        TreeTableReaderOutcomes {
            rebuild_version: Ok(()),
            allocation_records_under_root: Ok(()),
            walk_to_file: Ok(()),
            user_visible_tree_root_pointers: Ok(()),
            open_pool_for_read: Ok(()),
        },
        "两对合法的快照条目：五个读者都放行"
    );
    assert!(
        walk_to_file(&version.image, &root, &mut mapping_fallbacks) == Ok(file_without_snapshots),
        "冷走读跳过快照条目：读回的文件与不带快照条目的那一版相同"
    );
    assert_eq!(
        user_visible_tree_root_pointers(&version.image, &root),
        Ok(UserVisibleTreeRootPointers {
            inode_tree: without_snapshots.inode_tree,
            extent_tree: without_snapshots.extent_tree,
            snapshot_tree_roots: snapshot_entries
                .iter()
                .map(|entry| TreeTableEntry::root_pointer_bytes(&entry.to_bytes()).to_vec())
                .collect(),
        }),
        "用户可见根：活头两棵照旧，四条快照条目的根指针按盘上次序记下"
    );
}

/// 两对之外多一条码 14（树号接在最后一条之后，头 ID 是活头 inode 树）：码 14 三条、码 15 两条，条数不相等，五个读者都判树表损坏
/// （判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S1：配对判定的「码 14 比码 15 多一条」那一边）。
#[test]
fn a_tree_table_with_one_more_snapshot_inode_root_than_snapshot_extent_roots_is_refused_by_every_reader(
) {
    let mut version = version_with_snapshot_pairs(2);
    let record = version.first.record.clone();
    let mut snapshot_entries = version.entries_of_every_pair();
    let mut lone_inode_root = version.pairs[1].inode_root;
    lone_inode_root.tree = TreeIdentifier(version.pairs[1].extent_root.tree.0 + 1);
    snapshot_entries.push(lone_inode_root);
    let root = version.root_carrying(&snapshot_entries);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
        every_reader_refuses_the_tree_table_with(
            TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER
        ),
        "码 14 三条、码 15 两条：五个读者都判树表损坏"
    );
}

/// 两对，两条码 15 的头 ID 都指第二个快照的码 14（树号较大的那一条）：条数相等、每条码 15 都指得到一条码 14，但第一个快照的码 14 没有码 15、
/// 第二个的有两条，五个读者都判树表损坏（判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S1：配对那一判写成
/// 「码 14 的树号不大于码 15 的头 ID」时第一条码 15 会配上第一个快照的码 14，这张坏表被收下）。
#[test]
fn two_snapshot_extent_roots_claiming_the_later_snapshot_inode_root_are_refused_by_every_reader() {
    let mut version = version_with_snapshot_pairs(2);
    let record = version.first.record.clone();
    version.pairs[0].extent_root.head_identifier = version.pairs[1].inode_root.tree.0;
    let snapshot_entries = version.entries_of_every_pair();
    let root = version.root_carrying(&snapshot_entries);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
        every_reader_refuses_the_tree_table_with(
            TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY
        ),
        "两条码 15 配第二个快照的码 14：五个读者都判树表损坏"
    );
}

/// 两对之外多一条码 15（头 ID 指着第一个快照的码 14）：码 14 两条、码 15 三条，条数不相等，五个读者都判树表损坏。
#[test]
fn a_tree_table_with_one_more_snapshot_extent_root_than_snapshot_inode_roots_is_refused_by_every_reader(
) {
    let mut version = version_with_snapshot_pairs(2);
    let record = version.first.record.clone();
    let mut snapshot_entries = version.entries_of_every_pair();
    let mut lone_extent_root = version.pairs[0].extent_root;
    lone_extent_root.tree = TreeIdentifier(version.pairs[1].extent_root.tree.0 + 1);
    snapshot_entries.push(lone_extent_root);
    let root = version.root_carrying(&snapshot_entries);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
        every_reader_refuses_the_tree_table_with(
            TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER
        ),
        "码 14 两条、码 15 三条：五个读者都判树表损坏"
    );
}

/// 两对，第二个快照的码 15 头 ID 改成一个不是任何码 14 的号（活头 inode 树的树 ID）：条数相等、这一条码 15 配不上码 14，
/// 五个读者都判树表损坏。
#[test]
fn a_snapshot_extent_root_whose_head_is_not_any_snapshot_inode_root_is_refused_by_every_reader() {
    let mut version = version_with_snapshot_pairs(2);
    let record = version.first.record.clone();
    let live_inode_tree = version.pairs[1].inode_root.head_identifier;
    version.pairs[1].extent_root.head_identifier = live_inode_tree;
    let snapshot_entries = version.entries_of_every_pair();
    let root = version.root_carrying(&snapshot_entries);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
        every_reader_refuses_the_tree_table_with(
            TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY
        ),
        "一条码 15 的头 ID 不是任何码 14 的树 ID：五个读者都判树表损坏"
    );
}

/// 两对，两条码 15 的头 ID 都指第一个快照的码 14：条数相等、每条码 15 都指得到一条码 14，但第二个快照的码 14 没有码 15、
/// 第一个的有两条，五个读者都判树表损坏（与「指不到」报同一个）。
#[test]
fn two_snapshot_extent_roots_claiming_the_same_snapshot_inode_root_are_refused_by_every_reader() {
    let mut version = version_with_snapshot_pairs(2);
    let record = version.first.record.clone();
    version.pairs[1].extent_root.head_identifier = version.pairs[0].inode_root.tree.0;
    let snapshot_entries = version.entries_of_every_pair();
    let root = version.root_carrying(&snapshot_entries);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
        every_reader_refuses_the_tree_table_with(
            TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY
        ),
        "两条码 15 配同一条码 14：五个读者都判树表损坏"
    );
}

/// 第 11 个快照：`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`（10）对之外再多一对，五个读者都判树表损坏。
/// 边界：恰好 10 对时重建照常建得出、交回 10 对。
#[test]
fn an_eleventh_snapshot_is_refused_by_every_reader_while_ten_are_rebuilt() {
    let pairs_allowed = u64::try_from(SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION).expect("10");
    let mut version = version_with_snapshot_pairs(pairs_allowed + 1);
    let record = version.first.record.clone();
    let every_snapshot_entry = version.entries_of_every_pair();
    let ten_pairs_of_entries = &every_snapshot_entry[..2 * SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION];
    let root_with_ten = version.root_carrying(ten_pairs_of_entries);
    let rebuilt_with_ten = rebuild_version_with_its_snapshot_entries(
        &version.image,
        &root_with_ten,
        Some(record.clone()),
    )
    .expect("恰好 10 对：重建照常建得出");
    assert_eq!(
        rebuilt_with_ten.snapshot_entries,
        version.pairs[..SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION].to_vec(),
        "恰好 10 对：10 对都交出来"
    );

    let root_with_eleven = version.root_carrying(&every_snapshot_entry);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&version.image, &root_with_eleven, &record),
        every_reader_refuses_the_tree_table_with(
            TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS
        ),
        "11 对：五个读者都判树表损坏"
    );
}

/// 两对，第二个快照的码 14 头 ID 改成第一个快照码 14 的树 ID（不是活头 inode 树的树 ID）：配对照过，五个读者都判树表损坏。
#[test]
fn a_snapshot_inode_root_whose_head_is_not_the_live_inode_tree_is_refused_by_every_reader() {
    let mut version = version_with_snapshot_pairs(2);
    let record = version.first.record.clone();
    version.pairs[1].inode_root.head_identifier = version.pairs[0].inode_root.tree.0;
    let snapshot_entries = version.entries_of_every_pair();
    let root = version.root_carrying(&snapshot_entries);
    assert_eq!(
        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
        every_reader_refuses_the_tree_table_with(
            TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE
        ),
        "码 14 的头 ID 不是活头 inode 树的树 ID：五个读者都判树表损坏"
    );
}
