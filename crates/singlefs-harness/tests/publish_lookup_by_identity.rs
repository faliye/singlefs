//! 发布路径与整理读版本按身份查（里程碑四七·6，`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行）：
//! 照抄上一版的单元（`TransactionOutput::units_by_role`）、照抄角色的映射 key（`TransactionOutput::mapped_units_by_role`）、
//! 照抄的第一个文件之外的文件（`TransactionOutput::files_other_than_the_first_by_inode`）、整理读一版时逐 key 查中央映射
//! （`transaction::CentralMappingOfAVersionReadLeafByLeaf`）由在 `Vec` 上 `iter().find` 换成按身份建一次的索引。这里钉三样：
//! - 等价：两档池（十六个四单元的文件、四十八个八单元的文件，后一档的中央映射树不止一片叶）上跑同一串操作（按 inode 号写、整份重写四分之三的文件、
//!   抬 F、整理到停、整理之后再写、unlink 一个文件），每一版的每一次按身份查都与测试里的线性查找参照逐项相同，池级 checker 一条违例都没有；
//!   映射叶解不开、条目窄于字段表、同一把 key 有两条，三种叶上批量读与逐 key 查逐项相同。整理读每一版取的映射条目
//!   （`mounted_session::central_mapping_as_the_compaction_reads_it`）与逐 key 查的参照逐项相同（不漏、不错、次序同），批量读者解的叶
//!   恰好是进映射的 key 走到的那几片。
//! - 增长：数次数，不读挂钟、不读 CPU 时间（七·6 代码轮第一轮判决第三节 T4）。按身份查一次比几次身份：表长四倍时每次查平均比较次数
//!   涨不到门槛的倍数（合成表一条快档用例；512 与 2048 个数据单元的两档池上整理读的那几版，harness 慢档用例）；整理读一版解几片映射叶：
//!   恰好是进映射的 key 走到的那几片（每一版都判，慢档用例的两档池上也判）。
//!
//! 池都在内存里的两块 4 GiB 稀疏盘上造（`SparseBlockDevice`），配方照 E185 装置的 `construct_the_pool`：每个文件整份写进去，
//! 再整份重写下标不是 4 的倍数的那几个，抬 F 到换下的落点都回收，留下每段约四分之一还活着的聚簇段。
mod common;

use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::BTreeSet;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DataUnitIndexInFile, DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::code_two_tree::{
    leaf_position_routed_to, CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodePosition,
};
use singlefs_core::compaction_plan::{
    CompactionBatchSize, CompactionBatchUnits, CompactionSwitches,
};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable_with_compaction_switches, raise_rollback_floor_to_the_admission_ceiling,
};
use singlefs_core::mounted_session::{
    central_mapping_as_the_compaction_reads_it, CompactionRunOutcome, MountedSession, UserChange,
};
use singlefs_core::records::parse_mapping_entry;
use singlefs_core::region_scan::CentralMappingEntry;
use singlefs_core::transaction::{
    acquire_instance, central_mapping_locations_in_the_version, publish_first_file, warm_up,
    CentralMappingOfAVersionReadLeafByLeaf, FirstEntryOfEachIdentity, FirstFile, MappingLookup,
    PoolVersion, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

/// 512 字节的物理块：与 `common::parameters` 的几何同。
const PHYSICAL_BLOCK_BYTES: u32 = 512;
/// 映射 key 的宽（D19（块指针的结构与宽度预算） 已定项 10）：测试里造一把哪一版都没有的 key 用。
const MAPPING_KEY_BYTES: usize = 27;
/// 抬 F 那一圈至多推几次：每一次不够就多发一次建 inode 的发布，再抬（照 E185 装置 `construct_the_pool` 的上限）。
const FLOOR_RAISES_AT_MOST: u64 = 16;
/// 整理推到停时至多调几次显式入口：一次调用至少发一版才接着调，停了就出圈。
const COMPACTION_RUNS_AT_MOST: u64 = 64;
/// 等价那两档池里整理的批大小（只供测试的强制批大小）。
const COMPACTION_BATCH_UNITS_OF_THE_EQUIVALENCE_POOLS: usize = 2;

/// 一块 4 GiB 的稀疏盘。
fn sparse_device() -> SparseBlockDevice {
    SparseBlockDevice::new(
        common::IMAGE_BYTES,
        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
    )
}

/// 两块稀疏盘上 mkfs → 取号 → 暖机 → 新池新建文件，再按 `switches` 可写挂载（同 E185 装置的 `formatted_pool`）。
fn formatted_and_mounted(switches: CompactionSwitches) -> (Devices, MountedSession) {
    let parameters = common::parameters();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, sparse_device()))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &parameters,
        &genesis,
    ));
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file = common::file_content();
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_file,
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let mounted = mount_writable_with_compaction_switches(&parameters, &mut devices, switches)
        .expect("可写挂载");
    let (_, session) = MountedSession::of_the_mount(mounted);
    (devices, session)
}

/// 产品开关，批大小换成只供测试的强制 `batch_units`。
fn product_switches_with_the_batch_size(batch_units: usize) -> CompactionSwitches {
    let CompactionSwitches {
        batch_size: _,
        forced_region,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    } = CompactionSwitches::PRODUCT;
    CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(batch_units).expect("批大小至少 1"),
        ),
        forced_region,
        run_forcing,
        region_exclusion,
        destination_policy,
        release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor,
        relocation_publish_order,
    }
}

fn version_of(session: &MountedSession) -> &TransactionOutput {
    session
        .current
        .version_of_trees()
        .expect("新池新建文件之后每一版都有树")
}

/// 定长的伪随机字节流（xorshift64，种子取奇数）：`data_units` 个数据单元差 5 个字节写满。
fn content_of_data_units(data_units: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..data_units * data_unit_payload_capacity() - 5)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

/// 这一版里 `inode` 那个文件每个数据单元落的槽（头一槽；一个数据单元占两槽，第二槽是它加一）。
fn data_slots_of_the_file(version: &TransactionOutput, inode: InodeNumber) -> Vec<SlotNumber> {
    let file = version
        .files_other_than_the_first
        .iter()
        .find(|file| file.inode == inode)
        .expect("写过的文件在这一版里有数据单元");
    (0..file.data_pointers.len())
        .map(|position| version.unit(file.data_role_at(position)).slot)
        .collect()
}

/// 0 号盘上这一版还没释放的分配记录罩着的槽。
fn slots_under_unreleased_records_on_device_zero(
    version: &TransactionOutput,
) -> std::collections::BTreeSet<u64> {
    version
        .allocation_records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && !record.is_released)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// 造池那一串操作里每一版都交给它看一眼（等价那两条交比对，增长那一条什么都不做）。
type OnEveryVersion<'callback> = dyn FnMut(&TransactionOutput) + 'callback;

/// 一次用户改动发成，把这次发成的每一版（连同改动之后的那一趟整理）交给 `on_every_version`。
fn publish_and_look_at_every_version(
    session: &mut MountedSession,
    devices: &mut Devices,
    change: UserChange<'_>,
    on_every_version: &mut OnEveryVersion<'_>,
) {
    let published = session
        .publish_user_change(devices, change)
        .unwrap_or_else(|refusal| panic!("用户改动 {change:?} 发成：{refusal:?}"));
    for version in &published.versions_published {
        on_every_version(version);
    }
    if let Some(compaction) = &published.compaction_after_the_change {
        for version in &compaction.versions_published {
            on_every_version(version);
        }
    }
}

/// 一个池的样子：哪几个文件、每个几个数据单元。
#[derive(Clone, Copy, Debug)]
struct PoolOfFiles {
    files: u64,
    data_units_per_file: usize,
}

/// 造好的池：盘、还挂着的会话、文件的号（按写的次序）。
struct PoolBuilt {
    devices: Devices,
    session: MountedSession,
    inodes: Vec<InodeNumber>,
}

/// 照 E185 装置 `construct_the_pool` 的配方造池：建 `files` 个 inode、每个整份写 `data_units_per_file` 个数据单元；整份重写下标不是 4 的倍数的
/// 那几个（换下的落点在原来那几段里留下空洞）；抬 F 到换下的落点都回收（每段约四分之一还活着）。
fn build_a_pool_of_files(
    shape: PoolOfFiles,
    on_every_version: &mut OnEveryVersion<'_>,
) -> PoolBuilt {
    let parameters = common::parameters();
    let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
    let mut write_time_seconds = common::FIXED_WRITE_TIME_SECONDS + 600;
    let first_inode = version_of(&session).inode_number_watermark();
    write_time_seconds += 1;
    publish_and_look_at_every_version(
        &mut session,
        &mut devices,
        UserChange::NewInodes {
            count: shape.files,
            write_time_seconds,
        },
        on_every_version,
    );
    let inodes: Vec<InodeNumber> = (first_inode..first_inode + shape.files)
        .map(InodeNumber)
        .collect();
    for inode in &inodes {
        let content = content_of_data_units(shape.data_units_per_file, inode.0);
        write_time_seconds += 1;
        publish_and_look_at_every_version(
            &mut session,
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: *inode,
                content: &content,
                write_time_seconds,
            },
            on_every_version,
        );
    }
    let mut replaced_slots: Vec<SlotNumber> = Vec::new();
    for (position, inode) in inodes.iter().enumerate() {
        if position % 4 == 0 {
            continue;
        }
        replaced_slots.extend(data_slots_of_the_file(version_of(&session), *inode));
        let content = content_of_data_units(shape.data_units_per_file, inode.0 + 1_000_003);
        write_time_seconds += 1;
        publish_and_look_at_every_version(
            &mut session,
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: *inode,
                content: &content,
                write_time_seconds,
            },
            on_every_version,
        );
    }
    // 迭代上界是 FLOOR_RAISES_AT_MOST；跨轮携带的是会话（每一轮抬一次 F，不够就多发一次建 inode 的发布再抬）。
    let mut is_every_replaced_slot_reclaimed = false;
    for _ in 0..FLOOR_RAISES_AT_MOST {
        let mut current = version_of(&session).clone();
        raise_rollback_floor_to_the_admission_ceiling(
            &parameters,
            &mut devices,
            &mut session.allocator,
            &mut current,
            session.shadow_ledger,
        )
        .expect("抬 F");
        on_every_version(&current);
        session.current = PoolVersion::of_a_version_of_trees(current);
        let under_unreleased_records =
            slots_under_unreleased_records_on_device_zero(version_of(&session));
        // 换下的落点回收之后可能又被后面的整份重写取去装新的数据单元：那样的槽算回收过（同 E185 装置 `construct_the_pool` 的判法）。
        let slots_of_live_data_units: std::collections::BTreeSet<SlotNumber> = inodes
            .iter()
            .flat_map(|inode| data_slots_of_the_file(version_of(&session), *inode))
            .collect();
        is_every_replaced_slot_reclaimed = replaced_slots.iter().all(|slot| {
            slots_of_live_data_units.contains(slot)
                || (!under_unreleased_records.contains(&slot.0)
                    && session.allocator.devices[0].is_free(*slot))
        });
        if is_every_replaced_slot_reclaimed {
            break;
        }
        write_time_seconds += 1;
        publish_and_look_at_every_version(
            &mut session,
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds,
            },
            on_every_version,
        );
    }
    assert!(
        is_every_replaced_slot_reclaimed,
        "抬 F {FLOOR_RAISES_AT_MOST} 次之内整份重写换下的落点都回收了（{shape:?}）"
    );
    PoolBuilt {
        devices,
        session,
        inodes,
    }
}

/// 按身份查的几样与测试里的线性查找参照比过多少次。
#[derive(Debug, Default)]
struct LookupsCompared {
    versions: u64,
    unit_lookups: u64,
    mapping_key_lookups: u64,
    file_lookups: u64,
    central_mapping_lookups: u64,
    compaction_reads_of_the_central_mapping: u64,
    versions_whose_central_mapping_tree_has_several_nodes: u64,
}

/// 一版里进映射单元的 key 按这一版中央映射树的形状走到的叶（去重）：整理读这一版时批量读者该解的就是这几片。
fn mapping_leaves_the_mapped_keys_route_to(
    version: &TransactionOutput,
) -> BTreeSet<CodeTwoTreeNodePosition> {
    version
        .mapped_units
        .iter()
        .filter_map(|(_, key)| {
            leaf_position_routed_to(
                &version.central_mapping_tree.shape,
                &CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
            )
        })
        .collect()
}

/// 整理读一版该取到的映射条目，用逐 key 查（`central_mapping_locations_in_the_version`，每把 key 线性找叶、整片重解）当参照：
/// 每个进映射单元按 `mapped_units` 的次序查，查到的收，后面接这一版留存的映射条目。
fn mapping_entries_found_key_by_key(version: &TransactionOutput) -> Vec<CentralMappingEntry> {
    version
        .mapped_units
        .iter()
        .filter_map(
            |(_, key)| match central_mapping_locations_in_the_version(version, key) {
                MappingLookup::Found(locations) => Some(CentralMappingEntry {
                    key: key.clone(),
                    locations,
                }),
                MappingLookup::NodeMalformedOrNoEntryWithThisKey
                | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
            },
        )
        .chain(
            version
                .mapping_entries_kept_for_the_snapshots
                .iter()
                .map(|(key, locations)| CentralMappingEntry {
                    key: key.clone(),
                    locations: *locations,
                }),
        )
        .collect()
}

/// 整理读这一版时批量读者解过的叶恰好是进映射的 key 走到的那几片（少了是有 key 没经它查，例如退回逐 key 查；
/// 读者按叶记、一片只解一次）。
fn assert_the_compaction_read_decodes_exactly_the_routed_leaves(version: &TransactionOutput) {
    let leaves_decoded = central_mapping_as_the_compaction_reads_it(version).mapping_leaves_decoded;
    let routed_leaves = mapping_leaves_the_mapped_keys_route_to(version).len();
    assert_eq!(
        leaves_decoded,
        routed_leaves,
        "txg {}：整理读这一版解了 {leaves_decoded} 片映射叶，进映射的 {} 把 key 走到 {routed_leaves} 片（每片解一次、每把 key 都经批量读者查）",
        version.root.checkpoint_txg.0,
        version.mapped_units.len()
    );
}

/// 在两个 `Option<&Entry>` 上判「是表里同一项（或都没有）」：比地址，不比值（取的必须是最先那一项，不只是值相等的一项）。
fn is_the_same_entry<Entry>(indexed: Option<&Entry>, found_linearly: Option<&Entry>) -> bool {
    match (indexed, found_linearly) {
        (Some(indexed_entry), Some(found_entry)) => std::ptr::eq(indexed_entry, found_entry),
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => false,
    }
}

/// 一版里每一种按身份查都与线性查找参照（`iter().find`、逐 key 的 `central_mapping_locations_in_the_version`）比一遍：表里的每个身份，
/// 再加一个哪一版都没有的身份；整理读这一版取的映射条目与逐 key 查的参照逐项比，批量读者解的叶数与 key 走到的叶数比。
fn compare_every_lookup_with_a_linear_find(
    version: &TransactionOutput,
    compared: &mut LookupsCompared,
) {
    compared.versions += 1;
    let absent_role = TransactionUnit::Data(DataUnitIndexInFile(u64::MAX));
    let units_by_role = version.units_by_role();
    for role in version
        .units
        .iter()
        .map(|unit| unit.identity)
        .chain([absent_role])
    {
        let found_linearly = version.units.iter().find(|unit| unit.identity == role);
        assert!(
            is_the_same_entry(units_by_role.first_entry_of(&role), found_linearly),
            "txg {}：units 按角色 {role:?} 查到的与线性找的不是同一项",
            version.root.checkpoint_txg.0
        );
        compared.unit_lookups += 1;
    }
    let mapped_units_by_role = version.mapped_units_by_role();
    for role in version
        .mapped_units
        .iter()
        .map(|(role, _)| *role)
        .chain([absent_role])
    {
        let found_linearly = version
            .mapped_units
            .iter()
            .find(|(mapped, _)| *mapped == role);
        assert!(
            is_the_same_entry(mapped_units_by_role.first_entry_of(&role), found_linearly),
            "txg {}：mapped_units 按角色 {role:?} 查到的与线性找的不是同一项",
            version.root.checkpoint_txg.0
        );
        if let Some((_, key)) = found_linearly {
            assert_eq!(
                mapped_units_by_role.mapping_key_of(role),
                key.as_slice(),
                "txg {}：角色 {role:?} 的映射 key 与线性找的那一把相同",
                version.root.checkpoint_txg.0
            );
        }
        compared.mapping_key_lookups += 1;
    }
    let files_by_inode = version.files_other_than_the_first_by_inode();
    for inode in version
        .files_other_than_the_first
        .iter()
        .map(|file| file.inode)
        .chain([InodeNumber(u64::MAX)])
    {
        assert!(
            is_the_same_entry(
                files_by_inode.first_entry_of(&inode),
                version.file_other_than_the_first(inode)
            ),
            "txg {}：按 inode 号 {} 查到的文件与线性找的不是同一项",
            version.root.checkpoint_txg.0,
            inode.0
        );
        compared.file_lookups += 1;
    }
    let mut read_leaf_by_leaf = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    let absent_key = vec![0xff_u8; MAPPING_KEY_BYTES];
    for key in version
        .mapped_units
        .iter()
        .map(|(_, key)| key)
        .chain(
            version
                .mapping_entries_kept_for_the_snapshots
                .iter()
                .map(|(key, _)| key),
        )
        .chain([&absent_key])
    {
        assert_eq!(
            read_leaf_by_leaf.locations_of_key(key),
            central_mapping_locations_in_the_version(version, key),
            "txg {}：key {key:02x?} 批量读与逐 key 查不同",
            version.root.checkpoint_txg.0
        );
        compared.central_mapping_lookups += 1;
    }
    assert_eq!(
        central_mapping_as_the_compaction_reads_it(version).mapping_entries,
        mapping_entries_found_key_by_key(version),
        "txg {}：整理读这一版取的映射条目与逐 key 查的参照逐项相同（不漏、不错、次序同）",
        version.root.checkpoint_txg.0
    );
    assert_the_compaction_read_decodes_exactly_the_routed_leaves(version);
    compared.compaction_reads_of_the_central_mapping += 1;
    if version.central_mapping_tree.shape.nodes().len() > 1 {
        compared.versions_whose_central_mapping_tree_has_several_nodes += 1;
    }
}

/// 这几块盘此刻的整份镜像（交池级 checker）。
fn memory_pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: common::IMAGE_BYTES,
    }
}

fn violations_of_the_pool_checker(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 两块盘镜像（写过的扇区）的 sha256，十六进制：改前与改后同一串操作写出的镜像逐字节相同，在仓副本里比这一行（七·6 的验收）。
fn digest_of_the_images(devices: &Devices) -> String {
    let mut written = Vec::new();
    for (identity, device) in devices {
        written.extend_from_slice(&identity.0.to_le_bytes());
        for (sector, bytes) in device.image.written_sectors() {
            written.extend_from_slice(&sector.to_le_bytes());
            written.extend_from_slice(bytes);
        }
    }
    singlefs_harness::sha256::sha256_digest(&written)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// 等价那一串操作：造池（每一版都比）、整理推到停（每一版都比）、整理之后整份重写一个留着的文件、unlink 一个整份重写过的文件，
/// 最后池级 checker 判镜像。交回比过的次数与整理发了几批。
fn compare_every_lookup_along_a_history_on(shape: PoolOfFiles) -> (LookupsCompared, u64) {
    let mut compared = LookupsCompared::default();
    let PoolBuilt {
        mut devices,
        mut session,
        inodes,
    } = build_a_pool_of_files(shape, &mut |version| {
        compare_every_lookup_with_a_linear_find(version, &mut compared);
    });
    let mut on_every_version = |version: &TransactionOutput| {
        compare_every_lookup_with_a_linear_find(version, &mut compared);
    };
    session
        .allocator
        .set_compaction_switches(product_switches_with_the_batch_size(
            COMPACTION_BATCH_UNITS_OF_THE_EQUIVALENCE_POOLS,
        ));
    let mut batches_published = 0;
    // 迭代上界是 COMPACTION_RUNS_AT_MOST；跨轮携带的是会话与发了几批；一次调用一版都没发就出圈。
    for _ in 0..COMPACTION_RUNS_AT_MOST {
        let run = session.run_compaction(&mut devices).expect("整理一趟");
        for version in &run.versions_published {
            on_every_version(version);
        }
        batches_published += run.batches_published;
        match run.outcome {
            CompactionRunOutcome::Stopped(_) => {}
            CompactionRunOutcome::Failed(failure) => {
                panic!("整理的一次发布没发成（{shape:?}）：{failure:?}")
            }
        }
        if run.versions_published.is_empty() {
            break;
        }
    }
    let mut write_time_seconds = common::FIXED_WRITE_TIME_SECONDS + 1_000_000;
    let kept_file = inodes[0];
    let content = content_of_data_units(shape.data_units_per_file, kept_file.0 + 2_000_003);
    write_time_seconds += 1;
    publish_and_look_at_every_version(
        &mut session,
        &mut devices,
        UserChange::SequentialWriteOfAnInode {
            inode: kept_file,
            content: &content,
            write_time_seconds,
        },
        &mut on_every_version,
    );
    write_time_seconds += 1;
    publish_and_look_at_every_version(
        &mut session,
        &mut devices,
        UserChange::Unlink {
            inode: inodes[1],
            write_time_seconds,
        },
        &mut on_every_version,
    );
    drop(session);
    let violations = violations_of_the_pool_checker(&memory_pool_of(&devices));
    assert!(
        violations.is_empty(),
        "这一串操作之后池级 checker 一条违例都没有（{shape:?}）：{violations:?}"
    );
    println!(
        "publish_lookup_by_identity shape={shape:?} batches_published={batches_published} images_sha256={}",
        digest_of_the_images(&devices)
    );
    (compared, batches_published)
}

/// 十六个四单元的文件（64 个数据单元，中央映射树一片叶）：每一版的每一次按身份查都与线性查找相同，整理真发了批，镜像池级 checker 判绿。
/// 判别力：`crates/mutations.tsv` 里以「七·6 按身份查：」起头的几行（索引漏了一类单元、照抄取错角色、文件按错的号查、批量读取最先那条），
/// 以「七·6 整理读版本：」起头的几行（整理读版本隔一把 key 才查、漏查第一把 key、绕开批量读者逐 key 查）。
#[test]
fn every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units(
) {
    let (compared, batches_published) = compare_every_lookup_along_a_history_on(PoolOfFiles {
        files: 16,
        data_units_per_file: 4,
    });
    assert!(
        batches_published >= 1,
        "整理真搬了单元：读版本走到了批量读映射（{compared:?}）"
    );
    assert!(
        compared.versions >= 40
            && compared.file_lookups >= 16
            && compared.unit_lookups >= 64
            && compared.compaction_reads_of_the_central_mapping == compared.versions,
        "每一版都比过：{compared:?}"
    );
}

/// 四十八个八单元的文件（384 个数据单元，中央映射树不止一片叶）：同上，叶不止一片时批量读每片叶各解一次、各查各的。
#[test]
fn every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_forty_eight_files_of_eight_data_units(
) {
    let (compared, batches_published) = compare_every_lookup_along_a_history_on(PoolOfFiles {
        files: 48,
        data_units_per_file: 8,
    });
    assert!(
        batches_published >= 1,
        "整理真搬了单元：读版本走到了批量读映射（{compared:?}）"
    );
    assert!(
        compared.versions_whose_central_mapping_tree_has_several_nodes >= 1,
        "这一档的中央映射树长到了不止一个节点：{compared:?}"
    );
}

/// 同一身份出现几次时索引取最先那一项（与 `iter().find` 同一项），表里没有的身份查不到。
#[test]
fn first_entry_of_each_identity_keeps_the_first_of_repeated_identities_and_misses_absent_ones() {
    let table: Vec<(u64, &str)> = vec![
        (7, "seven first"),
        (3, "three first"),
        (7, "seven second"),
        (3, "three second"),
        (9, "nine"),
    ];
    let index = FirstEntryOfEachIdentity::of_the_table(&table, |(identity, _)| *identity);
    for identity in [7_u64, 3, 9, 4] {
        let found_linearly = table.iter().find(|(candidate, _)| *candidate == identity);
        assert!(
            is_the_same_entry(index.first_entry_of(&identity), found_linearly),
            "身份 {identity}：索引查到的 {:?} 与线性找的 {found_linearly:?} 不是同一项",
            index.first_entry_of(&identity)
        );
    }
    assert_eq!(
        index.first_entry_of(&7).map(|(_, name)| *name),
        Some("seven first"),
        "重复的身份取最先那一项"
    );
    assert_eq!(index.first_entry_of(&4), None, "表里没有的身份查不到");
}

/// 换掉一版中央映射树唯一那片叶（根兼叶）的字节：照这一版那片叶的头重装，条目换成 `entries`、条目宽换成 `entry_width`。
fn version_with_the_mapping_leaf_rebuilt(
    version: &TransactionOutput,
    entry_width: u16,
    entries: &[Vec<u8>],
) -> TransactionOutput {
    let mut rebuilt = version.clone();
    let leaf = rebuilt
        .units
        .iter_mut()
        .find(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("这一版有中央映射树的根");
    let header = parse_index_node(&leaf.bytes).expect("这一版的映射叶解得开");
    leaf.bytes = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        header.birth_txg,
        &common::E142_FILESYSTEM_IDENTIFIER,
        header.instance,
        header.birth_sequence,
        entry_width,
        entries,
    );
    rebuilt
}

/// 一版里每一把进映射的 key（连同一把哪一版都没有的）批量读与逐 key 查逐项相同；交回比了几把。
fn compare_the_leaf_by_leaf_read_with_the_per_key_lookup(version: &TransactionOutput) -> usize {
    let mut read_leaf_by_leaf = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    let absent_key = vec![0xff_u8; MAPPING_KEY_BYTES];
    let keys: Vec<&Vec<u8>> = version
        .mapped_units
        .iter()
        .map(|(_, key)| key)
        .chain([&absent_key])
        .collect();
    for key in &keys {
        assert_eq!(
            read_leaf_by_leaf.locations_of_key(key),
            central_mapping_locations_in_the_version(version, key),
            "key {key:02x?}：批量读与逐 key 查不同"
        );
    }
    keys.len()
}

/// 中央映射树只有一片叶（根兼叶）的一版上，把那片叶换成三种：解不开（全零）、条目窄于字段表（每条截掉最后一个字节）、
/// 同一把 key 有两条（一条改过位置条目的放在原条目前面，一条放在后面）；三种叶上批量读与逐 key 查逐项相同，
/// 同一把 key 两条时取最后一条（放在后面的那一条改过的位置条目查得到）。
#[test]
fn leaf_by_leaf_mapping_read_agrees_with_the_per_key_lookup_on_a_malformed_leaf_a_narrow_leaf_and_repeated_keys(
) {
    let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
    let first_inode = version_of(&session).inode_number_watermark();
    let content = content_of_data_units(3, 0x7006);
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 1,
            },
        )
        .unwrap_or_else(|refusal| panic!("建一个 inode：{refusal:?}"));
    session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(first_inode),
                content: &content,
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 2,
            },
        )
        .unwrap_or_else(|refusal| panic!("给它写三个数据单元：{refusal:?}"));
    let version = version_of(&session).clone();
    assert_eq!(
        version.central_mapping_tree.shape.nodes().len(),
        1,
        "这一版的中央映射树只有根兼叶"
    );
    let leaf_header = parse_index_node(&version.unit(TransactionUnit::MappingTree).bytes)
        .expect("这一版的映射叶解得开");
    let entry_width = u16::try_from(leaf_header.entry_width).expect("条目宽装得进 u16");
    let original_entries = leaf_header.entries.clone();
    assert!(
        original_entries.len() >= 4,
        "映射叶里有好几条：三个数据单元、下段、inode 树、分配记录树……"
    );
    let compared_on_the_original = compare_the_leaf_by_leaf_read_with_the_per_key_lookup(&version);

    let mut malformed = version.clone();
    malformed
        .units
        .iter_mut()
        .find(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("这一版有中央映射树的根")
        .bytes
        .fill(0);
    compare_the_leaf_by_leaf_read_with_the_per_key_lookup(&malformed);
    assert_eq!(
        CentralMappingOfAVersionReadLeafByLeaf::of_the_version(&malformed)
            .locations_of_key(&version.mapped_units[0].1),
        MappingLookup::NodeMalformedOrNoEntryWithThisKey,
        "叶解不开：进映射的 key 也查不到"
    );

    let narrow_entries: Vec<Vec<u8>> = original_entries
        .iter()
        .map(|entry| entry[..entry.len() - 1].to_vec())
        .collect();
    let narrow = version_with_the_mapping_leaf_rebuilt(&version, entry_width - 1, &narrow_entries);
    compare_the_leaf_by_leaf_read_with_the_per_key_lookup(&narrow);
    assert_eq!(
        CentralMappingOfAVersionReadLeafByLeaf::of_the_version(&narrow)
            .locations_of_key(&version.mapped_units[0].1),
        MappingLookup::EntryNarrowerThanItsFieldTable {
            entry_bytes: usize::from(entry_width - 1)
        },
        "条目窄于字段表：报那条条目的字节数"
    );

    let repeated_position = original_entries.len() / 2;
    let mut altered_entry = original_entries[repeated_position].clone();
    let last_byte = altered_entry.len() - 1;
    altered_entry[last_byte] ^= 0xff;
    for altered_entry_goes_after_the_original in [false, true] {
        let mut entries = original_entries.clone();
        let insert_at = if altered_entry_goes_after_the_original {
            repeated_position + 1
        } else {
            repeated_position
        };
        entries.insert(insert_at, altered_entry.clone());
        let repeated = version_with_the_mapping_leaf_rebuilt(&version, entry_width, &entries);
        compare_the_leaf_by_leaf_read_with_the_per_key_lookup(&repeated);
        let key = original_entries[repeated_position][..MAPPING_KEY_BYTES].to_vec();
        let expected_last_entry = if altered_entry_goes_after_the_original {
            &altered_entry
        } else {
            &original_entries[repeated_position]
        };
        let (_, expected_locations) =
            parse_mapping_entry(expected_last_entry).expect("55 字节的映射条目切得动");
        match CentralMappingOfAVersionReadLeafByLeaf::of_the_version(&repeated)
            .locations_of_key(&key)
        {
            MappingLookup::Found(locations) => assert_eq!(
                locations,
                expected_locations,
                "同一把 key 两条：取后面那一条（改过的放在{}）",
                if altered_entry_goes_after_the_original {
                    "后面"
                } else {
                    "前面"
                }
            ),
            other @ (MappingLookup::NodeMalformedOrNoEntryWithThisKey
            | MappingLookup::EntryNarrowerThanItsFieldTable { .. }) => {
                panic!("重复的那把 key 查得到：{other:?}")
            }
        }
    }
    assert!(compared_on_the_original >= 4, "原样那一版每把 key 都比过");
}

/// 包着一个身份，每被比一次（`==` 或 `cmp`）就在共用的计数上加一：数按身份的索引查一次比了几次身份
/// （七·6 的增长判据数次数，不读挂钟、不读 CPU 时间；次数只由表与查法定，不随负载与构建档漂移）。
struct IdentityCountingItsComparisons<'counter, Identity> {
    identity: Identity,
    comparisons: &'counter Cell<u64>,
}

impl<Identity: PartialEq> PartialEq for IdentityCountingItsComparisons<'_, Identity> {
    fn eq(&self, other: &Self) -> bool {
        self.comparisons.set(self.comparisons.get() + 1);
        self.identity == other.identity
    }
}

impl<Identity: Eq> Eq for IdentityCountingItsComparisons<'_, Identity> {}

impl<Identity: Ord> PartialOrd for IdentityCountingItsComparisons<'_, Identity> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Identity: Ord> Ord for IdentityCountingItsComparisons<'_, Identity> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.comparisons.set(self.comparisons.get() + 1);
        self.identity.cmp(&other.identity)
    }
}

/// 查了几次、一共比了几次身份。
#[derive(Clone, Copy, Debug, Default)]
struct ComparisonsOfLookups {
    lookups: u64,
    comparisons: u64,
}

impl ComparisonsOfLookups {
    fn plus(self, other: Self) -> Self {
        Self {
            lookups: self.lookups + other.lookups,
            comparisons: self.comparisons + other.comparisons,
        }
    }
}

/// 在 `table` 上建按身份的索引（`FirstEntryOfEachIdentity`，发布与整理用的同一个），把表里每一项的身份连同一个表里没有的
/// `absent_identity` 各查一次：表里的查得到、`absent_identity` 查不到。建索引时的比较不算，只数查的那几次。
fn comparisons_of_looking_up_every_identity<Entry, Identity: Ord + Copy>(
    table: &[Entry],
    identity_of: impl Fn(&Entry) -> Identity,
    absent_identity: Identity,
) -> ComparisonsOfLookups {
    let comparisons = Cell::new(0);
    let index =
        FirstEntryOfEachIdentity::of_the_table(table, |entry| IdentityCountingItsComparisons {
            identity: identity_of(entry),
            comparisons: &comparisons,
        });
    comparisons.set(0);
    let mut lookups = 0;
    // 迭代上界是表长加一；跨轮携带的只有查了几次。
    for identity in table.iter().map(&identity_of).chain([absent_identity]) {
        let found = index.first_entry_of(&IdentityCountingItsComparisons {
            identity,
            comparisons: &comparisons,
        });
        assert_eq!(
            found.is_some(),
            identity != absent_identity,
            "表里的身份查得到，表里没有的查不到"
        );
        lookups += 1;
    }
    ComparisonsOfLookups {
        lookups,
        comparisons: comparisons.get(),
    }
}

/// 表长四倍时，按身份查一次平均比较身份的次数至多涨到几倍（百分数）：有序表查找按层数涨（512 到 2048 多一两层），
/// 在表上逐项找按表长涨（四倍）；门槛取两者之间、离逐项找远的一边。
const HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE: u64 = 200;

/// 两档每次查的平均比较次数之比（百分数）。
fn growth_in_percent_of_the_comparisons_per_lookup(
    smaller: ComparisonsOfLookups,
    larger: ComparisonsOfLookups,
) -> u64 {
    larger.comparisons * smaller.lookups * 100 / (smaller.comparisons * larger.lookups)
}

/// 合成表的两档：512 个与 2048 个不同的身份，按一个与表长互素的步长打乱次序。
const IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE: u64 = 512;
const IDENTITIES_OF_THE_LARGER_SYNTHETIC_TABLE: u64 = 2048;
/// 打乱合成表次序的步长：素数，与 2 的幂的表长互素，`(位置 × 步长) mod 表长` 走遍每个身份恰好一次。
const STRIDE_THAT_SHUFFLES_THE_SYNTHETIC_TABLE: u64 = 7919;

fn synthetic_table_of_identities(identities: u64) -> Vec<u64> {
    (0..identities)
        .map(|position| position * STRIDE_THAT_SHUFFLES_THE_SYNTHETIC_TABLE % identities)
        .collect()
}

/// 512 个与 2048 个身份的合成表上，按身份的索引每次查平均比较身份的次数涨不到门槛的倍数（数次数，不读挂钟与 CPU 时间）。
/// 判别力：`crates/mutations.tsv` 里「七·6 按身份查：索引退回逐项找（数比较次数）」那一行。
#[test]
fn first_entry_of_each_identity_compares_less_than_twice_as_often_per_lookup_on_a_table_four_times_as_long(
) {
    let smaller = comparisons_of_looking_up_every_identity(
        &synthetic_table_of_identities(IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE),
        |identity| *identity,
        u64::MAX,
    );
    let larger = comparisons_of_looking_up_every_identity(
        &synthetic_table_of_identities(IDENTITIES_OF_THE_LARGER_SYNTHETIC_TABLE),
        |identity| *identity,
        u64::MAX,
    );
    let growth_in_percent = growth_in_percent_of_the_comparisons_per_lookup(smaller, larger);
    println!(
        "publish_lookup_by_identity synthetic_table smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent}"
    );
    assert!(
        growth_in_percent
            < HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE,
        "表长四倍，每次查平均比较次数涨了 {growth_in_percent}%，门槛 {HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE}%（小表 {smaller:?}，大表 {larger:?}）"
    );
}

/// 增长那一条量的两档：每个文件八个数据单元，512 个与 2048 个数据单元。
const DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS: usize = 8;
const FILES_OF_THE_SMALLER_GROWTH_POOL: u64 = 64;
const FILES_OF_THE_LARGER_GROWTH_POOL: u64 = 256;
/// 一批整理的批大小（E185 的 K = 8）。
const COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS: usize = 8;
/// 每档量几次（两档轮流，一次调用至多发一版）。
const MEASURED_CALLS_PER_POOL: usize = 8;
/// 每档至少要有几次真发了一版才算数。
const PUBLISHING_CALLS_REQUIRED_PER_POOL: usize = 5;

/// 重挂一份造好的池（批大小强制 K），一次调用一次地推整理：每次调用之前把这次挂载数过的发布数压到上限，一次调用至多发一版
/// （照 E185 装置 `measure_one_intent`）。调用之前在这一批要读的那一版上数两样：整理读它解了几片映射叶（判恰好是 key 走到的那几片），
/// `units` 上每个角色按身份查一次比了几次身份。交回比较的次数；一版都没发的交回 `None`。
fn lookup_work_of_one_compaction_call(
    session: &mut MountedSession,
    devices: &mut Devices,
) -> Option<ComparisonsOfLookups> {
    session.compaction.publishes_since_the_last_user_change = 1000;
    let version_the_batch_reads = version_of(session);
    assert_the_compaction_read_decodes_exactly_the_routed_leaves(version_the_batch_reads);
    let comparisons = comparisons_of_looking_up_every_identity(
        &version_the_batch_reads.units,
        |unit| unit.identity,
        TransactionUnit::Data(DataUnitIndexInFile(u64::MAX)),
    );
    let run = session.run_compaction(devices).expect("整理一趟");
    match run.outcome {
        CompactionRunOutcome::Stopped(_) => {}
        CompactionRunOutcome::Failed(failure) => panic!("整理的一次发布没发成：{failure:?}"),
    }
    (!run.versions_published.is_empty()).then_some(comparisons)
}

/// 512 个数据单元的池与 2048 个的池（每个文件八个单元），同一进程里重挂、两档轮流各推 8 次整理（K = 8、一次至多一版）。
/// 数次数，不读挂钟、不读 CPU 时间：每一批读的那一版上，整理读它解的映射叶恰好是进映射的 key 走到的那几片；`units` 上按角色查一次
/// 平均比较身份的次数从小池到大池涨不到门槛的倍数。判别力：`crates/mutations.tsv` 里以「七·6 增长：」起头的几行
/// （按身份查的索引退回逐项找、整理读版本退回逐 key 查映射）。
// harness-test-granularity:one-scenario 两档池同一个场景：比的是同一串操作在两档上的比，单造一档量不出比。
#[test]
#[ignore = "harness 慢档用例：造 2048 个数据单元的池要发约 460 次发布，debug 下单条超过 60 秒"]
fn compaction_batches_decode_each_routed_mapping_leaf_once_and_compare_identities_sublinearly_from_512_to_2048_data_units(
) {
    let (mut smaller_devices, mut larger_devices) = std::thread::scope(|scope| {
        let smaller = scope.spawn(|| {
            build_a_pool_of_files(
                PoolOfFiles {
                    files: FILES_OF_THE_SMALLER_GROWTH_POOL,
                    data_units_per_file: DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS,
                },
                &mut |_| {},
            )
            .devices
        });
        let larger = scope.spawn(|| {
            build_a_pool_of_files(
                PoolOfFiles {
                    files: FILES_OF_THE_LARGER_GROWTH_POOL,
                    data_units_per_file: DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS,
                },
                &mut |_| {},
            )
            .devices
        });
        (
            smaller.join().expect("造小池的线程没 panic"),
            larger.join().expect("造大池的线程没 panic"),
        )
    });
    let parameters = common::parameters();
    let switches = product_switches_with_the_batch_size(COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS);
    let (_, mut smaller_session) = MountedSession::of_the_mount(
        mount_writable_with_compaction_switches(&parameters, &mut smaller_devices, switches)
            .expect("重挂小池"),
    );
    let (_, mut larger_session) = MountedSession::of_the_mount(
        mount_writable_with_compaction_switches(&parameters, &mut larger_devices, switches)
            .expect("重挂大池"),
    );
    let mut smaller_readings = Vec::new();
    let mut larger_readings = Vec::new();
    // 迭代上界是 MEASURED_CALLS_PER_POOL；两档轮流（与改前量 CPU 时同一个次序）。
    for _ in 0..MEASURED_CALLS_PER_POOL {
        smaller_readings.extend(lookup_work_of_one_compaction_call(
            &mut smaller_session,
            &mut smaller_devices,
        ));
        larger_readings.extend(lookup_work_of_one_compaction_call(
            &mut larger_session,
            &mut larger_devices,
        ));
    }
    println!(
        "publish_lookup_by_identity growth smaller_comparisons={smaller_readings:?} larger_comparisons={larger_readings:?}"
    );
    assert!(
        smaller_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL
            && larger_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL,
        "两档各至少 {PUBLISHING_CALLS_REQUIRED_PER_POOL} 次真发了一版：小池 {smaller_readings:?}，大池 {larger_readings:?}"
    );
    let smaller = smaller_readings
        .into_iter()
        .fold(ComparisonsOfLookups::default(), ComparisonsOfLookups::plus);
    let larger = larger_readings
        .into_iter()
        .fold(ComparisonsOfLookups::default(), ComparisonsOfLookups::plus);
    let growth_in_percent = growth_in_percent_of_the_comparisons_per_lookup(smaller, larger);
    println!(
        "publish_lookup_by_identity growth smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent}"
    );
    assert!(
        growth_in_percent
            < HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE,
        "数据单元四倍，units 上按角色查一次平均比较次数涨了 {growth_in_percent}%，门槛 {HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE}%（小池 {smaller:?}，大池 {larger:?}）"
    );
}
