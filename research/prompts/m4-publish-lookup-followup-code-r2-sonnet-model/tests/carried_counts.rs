//! 第二轮正推腿：第 11 批 harness 用例文件的原样拷贝，只在「整份重写一个八单元的文件（池里没有快照）」那条用例里多打一行三棵树各照抄了几个节点，用来看整数遍判法在三棵树上是否空真。
//! 发布路径与整理读版本按身份查（里程碑四七·6，`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行）：
//! 照抄上一版的单元（`TransactionOutput::units_by_role`）、照抄角色的映射 key（`TransactionOutput::mapped_units_by_role`）、
//! 照抄的第一个文件之外的文件（`TransactionOutput::files_other_than_the_first_by_inode`）、整理读一版时逐 key 查中央映射
//! （`transaction::CentralMappingOfAVersionReadLeafByLeaf`）由在 `Vec` 上 `iter().find` 换成按身份建一次的索引。这里钉三样：
//! - 等价：两档池（十六个四单元的文件、四十八个八单元的文件，后一档的中央映射树不止一片叶）上跑同一串操作（按 inode 号写、整份重写四分之三的文件、
//!   抬 F、整理到停、整理之后再写、unlink 一个文件），每一版的每一次按身份查都与测试里的线性查找参照逐项相同，池级 checker 一条违例都没有；
//!   映射叶解不开、条目窄于字段表、同一把 key 有两条，三种叶上批量读与逐 key 查逐项相同。整理读每一版取的映射条目
//!   （`mounted_session::central_mapping_as_the_compaction_reads_it`）与逐 key 查的参照逐项相同（不漏、不错、次序同），批量读者解的叶
//!   恰好是进映射的 key 走到的那几片。
//! - 数的是产品调用点上做的事（七·6 代码轮第二轮判决第三节）：核心层在特性 `lookup-work-counters` 下按处记下按身份查了几次、每次看过
//!   索引几项、建了几次索引、解了几次中央映射叶（`singlefs_core::transaction::lookup_work`，harness 档的 dev 依赖打开这个特性），
//!   用例只读这些计数，不在测试里另建索引来数，不读挂钟、不读 CPU 时间。每一次用户改动、每一趟整理：每次查至多看折半查找那么多项；
//!   整份重写一个文件时快照判定那一圈与释放那三处都经索引查了换下的每个数据单元；整理每发一批，计划里每一项都经按 key 的索引查过；
//!   整理读一版恰好解进映射的 key 走到的那几片叶、各解一次，只建一次单元索引。
//! - 增长：表长四倍时每次查平均看过的索引项数涨不到门槛的倍数（合成表一条快档用例；512 与 2048 个数据单元的两档池上每一趟整理，
//!   harness 慢档用例），整理每一趟建索引的次数也涨不到门槛的倍数。
//!
//! 池都在内存里的两块 4 GiB 稀疏盘上造（`SparseBlockDevice`），配方照 E185 装置的 `construct_the_pool`：每个文件整份写进去，
//! 再整份重写下标不是 4 的倍数的那几个，抬 F 到换下的落点都回收，留下每段约四分之一还活着的聚簇段。
//!
//! 七·6 后续批（`research/prompts/m4-publish-lookup-followup-code-r1-main-verification.md` 第三节第 1–4 条）另钉：整理发批与恢复把映射位置换进
//! `units`、装一次发布要写与记录点名的单元、留给快照的角色清单两处、记录点名的数据角色那一处都经建一次的索引查；按 inode 号找文件、
//! 删快照取留存条目那几处在发布路径上建索引的次数恰好等于走了几遍；三棵多层码 2 树照抄的节点按树分开数；`singlefs-core` 构建脚本
//! 交出的特性只认 cargo 这一次真开的。
mod common;

/// `singlefs-core` 的构建脚本本身（七·6 后续批代码轮第一轮判决第三节第 4 条）：在这里编进来，判它交出的特性只看
/// `cfg!(feature = …)`、不读进程环境。这个测试包不开 `lookup-work-counters`，所以那一行的 cfg 在这里恒为假；
/// 构建脚本的 `main` 在这里用不上。
#[allow(
    dead_code,
    unexpected_cfgs,
    reason = "构建脚本的 main 在测试里不调；它写的 cfg!(feature = \"lookup-work-counters\") 是 singlefs-core 的特性，这个测试包没有这个特性"
)]
#[path = "../../singlefs-core/build.rs"]
mod singlefs_core_build_script;

use std::collections::BTreeSet;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, InodeNumber, SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::code_two_tree::{
    leaf_position_routed_to, CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodePosition,
};
use singlefs_core::compaction_plan::{
    CompactionBatchSize, CompactionBatchUnits, CompactionSwitches, PlannedRelocation,
};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable_with_compaction_switches, raise_rollback_floor_to_the_admission_ceiling,
};
use singlefs_core::mounted_session::{
    central_mapping_as_the_compaction_reads_it, CompactionRun, CompactionRunOutcome,
    MountedSession, UserChange,
};
use singlefs_core::records::{parse_mapping_entry, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::region_scan::CentralMappingEntry;
use singlefs_core::transaction::lookup_work::{
    lookup_work_while, LookupSite, LookupWork, LookupWorkOfEachSite,
};
use singlefs_core::transaction::{
    acquire_instance, asks_the_snapshot_lifetime_judgement,
    central_mapping_locations_in_the_version, copies_failing_the_release_checksum_check,
    placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots, publish_first_file,
    publish_relocation_batch, publish_sequential_write_of_an_inode,
    refuse_mapping_entries_that_do_not_name_two_pool_devices, warm_up,
    CentralMappingOfAVersionReadLeafByLeaf, FileContentOfThePublish, FirstEntryOfEachIdentity,
    FirstFile, MappedUnitsOfAVersionReadByRole, MappingLookup, MultiLevelCodeTwoTree, PoolVersion,
    PoolWriter, RelocatedUnitHolder, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{
    build_index_node, data_unit_payload_capacity, parse_index_node, UNIT_CLASS_DATA,
};
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

/// 一个数的二进制位数：折半查一份这么多项的索引，一次至多看这么多项。
fn binary_digits_of(entries: u64) -> u64 {
    u64::from(u64::BITS - entries.leading_zeros())
}

/// 每一处（连同不在点名几处里的）按身份查了几次，看过的索引项数不超过「每次查都是一次折半查找」那么多：
/// 在索引上逐项走（退回线性找、查之前先把索引走一遍）看过的项数按表长涨，查的次数不涨，过不了这一条。
fn assert_every_lookup_examines_at_most_a_binary_search(
    lookup_work: &LookupWorkOfEachSite,
    what_was_done: &str,
) {
    // 迭代上界是点名的几处加一。
    for (site, work) in &lookup_work.work_of_each_site {
        let examined_at_most = work.index_lookups * binary_digits_of(work.largest_index_looked_up);
        assert!(
            work.index_entries_examined <= examined_at_most,
            "{what_was_done}：{site:?} 那一处按身份查了 {} 次、看过索引 {} 项，每次折半查至多看 {} 项（最大的索引 {} 项），合计至多 {examined_at_most} 项：{work:?}",
            work.index_lookups,
            work.index_entries_examined,
            binary_digits_of(work.largest_index_looked_up),
            work.largest_index_looked_up
        );
    }
}

/// 整理推一趟（量它在产品里做的按身份查）：每次查至多一次折半查找；每发一批，计划里每一项都经按 key 的索引查过
/// （整理发批核源落点退回在 `mapped_units` 上按 key 线性找，这一处一次都不查）。交回这一趟与它做的查找。
fn run_compaction_and_count_its_lookups(
    session: &mut MountedSession,
    devices: &mut Devices,
) -> (CompactionRun, LookupWorkOfEachSite) {
    let (run, lookup_work) = lookup_work_while(|| session.run_compaction(devices));
    let run = run.expect("整理一趟");
    assert_every_lookup_examines_at_most_a_binary_search(&lookup_work, "整理一趟");
    let units_relocated: u64 = run
        .versions_published
        .iter()
        .map(|version| {
            u64::try_from(version.units_relocated_by_this_publish.len()).expect("单元数")
        })
        .sum();
    let relocation_sources = lookup_work.of_the_site(LookupSite::RelocationSources);
    assert!(
        relocation_sources.index_lookups >= units_relocated,
        "整理这一趟搬了 {units_relocated} 个单元，核源落点那一处只经索引查了 {} 次（计划里每一项按映射 key 查一次）：{relocation_sources:?}",
        relocation_sources.index_lookups
    );
    (run, lookup_work)
}

/// 一次用户改动发成，把这次发成的每一版（连同改动之后的那一趟整理）交给 `on_every_version`；交回这次发布在产品里做的按身份查
/// （每次查至多一次折半查找，这里就判）。量的只是发布那一次调用，`on_every_version` 里测试自己做的查找不算进去。
fn publish_and_look_at_every_version(
    session: &mut MountedSession,
    devices: &mut Devices,
    change: UserChange<'_>,
    on_every_version: &mut OnEveryVersion<'_>,
) -> LookupWorkOfEachSite {
    let (published, lookup_work) =
        lookup_work_while(|| session.publish_user_change(devices, change));
    let published =
        published.unwrap_or_else(|refusal| panic!("用户改动 {change:?} 发成：{refusal:?}"));
    assert_every_lookup_examines_at_most_a_binary_search(
        &lookup_work,
        &format!("用户改动 {change:?}"),
    );
    for version in &published.versions_published {
        on_every_version(version);
    }
    if let Some(compaction) = &published.compaction_after_the_change {
        for version in &compaction.versions_published {
            on_every_version(version);
        }
    }
    lookup_work
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
        let lookup_work = publish_and_look_at_every_version(
            &mut session,
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: *inode,
                content: &content,
                write_time_seconds,
            },
            on_every_version,
        );
        // 整份重写换下这个文件的每个数据单元：快照判定那一圈逐个取映射 key，释放的三处各自逐个取映射条目（三处分开记、各判各的：
        // 一处改回线性，另两处补不上它的数），每个换下的角色按 inode 号找上一版的文件，都经索引。
        let data_units_replaced =
            u64::try_from(shape.data_units_per_file).expect("一个文件的数据单元数装得进 u64");
        for site in [
            LookupSite::SnapshotJudgement,
            LookupSite::MappingEntriesOfTheReleaseJudgement,
            LookupSite::MappingEntriesCheckedForTwoPoolDevices,
            LookupSite::MappingEntriesCheckedAgainstTheReleaseChecksums,
            LookupSite::FilesOfTheReplacedRolesInThePreviousVersion,
        ] {
            let work = lookup_work.of_the_site(site);
            assert!(
                work.index_lookups >= data_units_replaced,
                "整份重写 inode {} 换下 {data_units_replaced} 个数据单元，{site:?} 那一处只经索引查了 {} 次（换下的每个单元经索引查一次）：{work:?}",
                inode.0,
                work.index_lookups
            );
        }
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

/// 整理读这一版（`central_mapping_as_the_compaction_reads_it`）在产品里解叶的次数恰好是进映射的 key 走到的叶的片数：
/// 每片只解一次（同一片叶每把 key 重解、退回逐 key 查都多解），只建一次单元索引（每把 key 现建一个批量读者就建了 key 那么多次），
/// 每次查至多一次折半查找。
fn assert_the_compaction_read_decodes_exactly_the_routed_leaves(version: &TransactionOutput) {
    let (_, lookup_work) =
        lookup_work_while(|| central_mapping_as_the_compaction_reads_it(version));
    let compaction_read = lookup_work.of_the_site(LookupSite::CompactionReadsTheCentralMapping);
    let routed_leaves =
        u64::try_from(mapping_leaves_the_mapped_keys_route_to(version).len()).expect("叶的片数");
    assert_eq!(
        compaction_read.mapping_leaf_decodes,
        routed_leaves,
        "txg {}：整理读这一版解了 {} 次映射叶，进映射的 {} 把 key 走到 {routed_leaves} 片（每片解一次、每把 key 都经批量读者查）：{compaction_read:?}",
        version.root.checkpoint_txg.0,
        compaction_read.mapping_leaf_decodes,
        version.mapped_units.len()
    );
    assert_eq!(
        compaction_read.index_builds, 1,
        "txg {}：整理读这一版建了 {} 次索引（一个批量读者、一份单元索引）：{compaction_read:?}",
        version.root.checkpoint_txg.0, compaction_read.index_builds
    );
    assert_every_lookup_examines_at_most_a_binary_search(
        &lookup_work,
        &format!("txg {} 整理读这一版", version.root.checkpoint_txg.0),
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

/// 索引按身份升序交出的每一项都是表里那个身份的最先那一项（比地址），表里每个不同的身份恰好出现一次。
fn assert_the_index_lists_the_first_entry_of_every_identity_once<'table, Identity, Entry>(
    index: &FirstEntryOfEachIdentity<'table, Identity, Entry>,
    table: &'table [Entry],
    identity_of: impl Fn(&'table Entry) -> Identity,
    which_index: &str,
) where
    Identity: Ord + std::fmt::Debug,
{
    let mut identities_listed = Vec::new();
    // 迭代上界是索引项数。
    for (identity, entry) in index.first_entries() {
        let found_linearly = table
            .iter()
            .find(|candidate| identity_of(*candidate) == *identity);
        assert!(
            is_the_same_entry(Some(entry), found_linearly),
            "{which_index}：索引里身份 {identity:?} 那一项不是表里它的最先那一项"
        );
        identities_listed.push(identity);
    }
    let distinct_identities: BTreeSet<Identity> = table.iter().map(&identity_of).collect();
    assert!(
        identities_listed.windows(2).all(|pair| pair[0] < pair[1])
            && identities_listed.len() == distinct_identities.len(),
        "{which_index}：索引按身份升序列出表里 {} 个不同的身份各一次，列出了 {} 项",
        distinct_identities.len(),
        identities_listed.len()
    );
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
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &units_by_role,
        &version.units,
        |unit| unit.identity,
        &format!("txg {} 的 units 按角色", version.root.checkpoint_txg.0),
    );
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
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &mapped_units_by_role,
        &version.mapped_units,
        |(role, _)| *role,
        &format!(
            "txg {} 的 mapped_units 按角色",
            version.root.checkpoint_txg.0
        ),
    );
    // 整理发批按映射 key 找角色走的索引：每一把 key（连同一把哪一版都没有的）查到的与在 `mapped_units` 上按 key 线性找的是同一项。
    let mapped_units_by_key = version.mapped_units_by_key();
    let absent_key = vec![0xff_u8; MAPPING_KEY_BYTES];
    for key in version
        .mapped_units
        .iter()
        .map(|(_, key)| key)
        .chain([&absent_key])
    {
        let found_linearly = version
            .mapped_units
            .iter()
            .find(|(_, mapped_key)| mapped_key == key);
        assert!(
            is_the_same_entry(
                mapped_units_by_key.first_entry_of(&key.as_slice()),
                found_linearly
            ),
            "txg {}：mapped_units 按 key {key:02x?} 查到的与线性找的不是同一项",
            version.root.checkpoint_txg.0
        );
        compared.mapping_key_lookups += 1;
    }
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &mapped_units_by_key,
        &version.mapped_units,
        |(_, key)| key.as_slice(),
        &format!(
            "txg {} 的 mapped_units 按 key",
            version.root.checkpoint_txg.0
        ),
    );
    // 释放的三处与快照判定那一圈进 deadlist 那一支按角色取映射条目走的批量形态：每个进映射的角色与「线性找 key、逐 key 查」逐项相同。
    let mut read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(version);
    for (role, _) in &version.mapped_units {
        let (_, key_found_linearly) = version
            .mapped_units
            .iter()
            .find(|(mapped, _)| mapped == role)
            .expect("角色就取自 mapped_units");
        assert_eq!(
            read_by_role.locations_of_role(*role),
            central_mapping_locations_in_the_version(version, key_found_linearly),
            "txg {}：角色 {role:?} 按角色批量取的映射条目与线性找 key、逐 key 查的不同",
            version.root.checkpoint_txg.0
        );
        compared.central_mapping_lookups += 1;
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
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &files_by_inode,
        &version.files_other_than_the_first,
        |file| file.inode,
        &format!("txg {} 的文件按 inode 号", version.root.checkpoint_txg.0),
    );
    let mut read_leaf_by_leaf = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
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
        central_mapping_as_the_compaction_reads_it(version),
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
        let (run, _) = run_compaction_and_count_its_lookups(&mut session, &mut devices);
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

/// 在 `table` 上建按身份的索引（`FirstEntryOfEachIdentity`，发布与整理用的同一个），把表里每一项的身份连同一个表里没有的
/// `absent_identity` 各查一次：表里的查得到、`absent_identity` 查不到。交回这几次查在产品里记下的计数（建索引那一次不在其内）。
fn lookup_work_of_looking_up_every_identity<Entry, Identity: Ord + Copy>(
    table: &[Entry],
    identity_of: impl Fn(&Entry) -> Identity,
    absent_identity: Identity,
) -> LookupWork {
    let index = FirstEntryOfEachIdentity::of_the_table(table, &identity_of);
    let ((), lookup_work) = lookup_work_while(|| {
        // 迭代上界是表长加一。
        for identity in table.iter().map(&identity_of).chain([absent_identity]) {
            assert_eq!(
                index.first_entry_of(&identity).is_some(),
                identity != absent_identity,
                "表里的身份查得到，表里没有的查不到"
            );
        }
    });
    assert_every_lookup_examines_at_most_a_binary_search(&lookup_work, "合成表上每个身份查一次");
    lookup_work.of_every_site()
}

/// 表长四倍时，查一次平均看过的索引项数（与整理每一趟建索引的次数）至多涨到几倍（百分数）：折半查按层数涨（512 到 2048 多两层），
/// 在表上逐项找按表长涨（四倍）；门槛取两者之间、离逐项找远的一边。
const HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE: u64 = 200;

/// 两档每次查平均看过的索引项数之比（百分数）。
fn growth_in_percent_of_the_entries_examined_per_lookup(
    smaller: LookupWork,
    larger: LookupWork,
) -> u64 {
    larger.index_entries_examined * smaller.index_lookups * 100
        / (smaller.index_entries_examined * larger.index_lookups)
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

/// 512 个与 2048 个身份的合成表上，按身份的索引每次查平均看过的索引项数涨不到门槛的倍数（读产品里的计数，不读挂钟与 CPU 时间）。
/// 判别力：`crates/mutations.tsv` 里「七·6 按身份查：索引从大到小逐项找」那一行。
#[test]
fn first_entry_of_each_identity_examines_less_than_twice_as_many_entries_per_lookup_on_a_table_four_times_as_long(
) {
    let smaller = lookup_work_of_looking_up_every_identity(
        &synthetic_table_of_identities(IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE),
        |identity| *identity,
        u64::MAX,
    );
    let larger = lookup_work_of_looking_up_every_identity(
        &synthetic_table_of_identities(IDENTITIES_OF_THE_LARGER_SYNTHETIC_TABLE),
        |identity| *identity,
        u64::MAX,
    );
    let growth_in_percent = growth_in_percent_of_the_entries_examined_per_lookup(smaller, larger);
    println!(
        "publish_lookup_by_identity synthetic_table smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent}"
    );
    assert!(
        growth_in_percent < HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE,
        "表长四倍，每次查平均看过的索引项数涨了 {growth_in_percent}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（小表 {smaller:?}，大表 {larger:?}）"
    );
}

/// 增长那一条量的两档：每个文件八个数据单元，512 个与 2048 个数据单元。
const DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS: usize = 8;
const FILES_OF_THE_SMALLER_GROWTH_POOL: u64 = 64;
const FILES_OF_THE_LARGER_GROWTH_POOL: u64 = 256;
/// 一批整理的批大小（E185 的 K = 8）。
const COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS: usize = 8;
/// 每档量几次（一次调用至多发一版）。
const MEASURED_CALLS_PER_POOL: usize = 8;
/// 每档至少要有几次真发了一版才算数。
const PUBLISHING_CALLS_REQUIRED_PER_POOL: usize = 5;

/// 一次调用推一次整理：调用之前把这次挂载数过的发布数压到上限，一次调用至多发一版（照 E185 装置 `measure_one_intent`）。
/// 调用之前判整理读这一批要读的那一版恰好解进映射的 key 走到的那几片叶；交回这一次整理在产品里做的查找（每一处合起来）；
/// 一版都没发的交回 `None`。
fn lookup_work_of_one_compaction_call(
    session: &mut MountedSession,
    devices: &mut Devices,
) -> Option<LookupWork> {
    session.compaction.publishes_since_the_last_user_change = 1000;
    assert_the_compaction_read_decodes_exactly_the_routed_leaves(version_of(session));
    let (run, lookup_work) = run_compaction_and_count_its_lookups(session, devices);
    match run.outcome {
        CompactionRunOutcome::Stopped(_) => {}
        CompactionRunOutcome::Failed(failure) => panic!("整理的一次发布没发成：{failure:?}"),
    }
    (!run.versions_published.is_empty()).then(|| lookup_work.of_every_site())
}

/// 造一档池（`files` 个八单元的文件），批大小强制 K 重挂，推 `MEASURED_CALLS_PER_POOL` 次整理：交回真发了一版的那几次各自做的查找，
/// 按调用的次序。
fn lookup_work_of_the_compaction_calls_on_a_pool_of(files: u64) -> Vec<LookupWork> {
    let mut devices = build_a_pool_of_files(
        PoolOfFiles {
            files,
            data_units_per_file: DATA_UNITS_PER_FILE_OF_THE_GROWTH_POOLS,
        },
        &mut |_| {},
    )
    .devices;
    let parameters = common::parameters();
    let switches = product_switches_with_the_batch_size(COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS);
    let (_, mut session) = MountedSession::of_the_mount(
        mount_writable_with_compaction_switches(&parameters, &mut devices, switches)
            .unwrap_or_else(|refusal| panic!("重挂 {files} 个文件的池：{refusal:?}")),
    );
    // 迭代上界是 MEASURED_CALLS_PER_POOL；跨轮携带的是会话与盘。
    (0..MEASURED_CALLS_PER_POOL)
        .filter_map(|_| lookup_work_of_one_compaction_call(&mut session, &mut devices))
        .collect()
}

/// 几次调用做的查找合起来。
fn sum_of_the_lookup_work(readings: &[LookupWork]) -> LookupWork {
    readings
        .iter()
        .fold(LookupWork::default(), |sum, reading| sum.plus(*reading))
}

/// 512 个数据单元的池与 2048 个的池（每个文件八个单元），各推 8 次整理（K = 8、一次至多一版），读产品里记下的计数，不读挂钟、
/// 不读 CPU 时间：每一批读的那一版上，整理读它恰好解进映射的 key 走到的那几片叶；每一趟整理每次查至多一次折半查找、计划里每一项都经
/// 按 key 的索引查过；查一次平均看过的索引项数与每一趟建索引的次数从小池到大池涨不到门槛的倍数。两档互不依赖：各造各的池、各挂各的会话，
/// 计数记在各自的线程上，各在一个线程里量，合并按档的次序。
/// 判别力：`crates/mutations.tsv` 里以「七·6 增长：」起头的几行（按身份查的索引退回逐项找、整理读版本退回逐 key 查映射）。
// harness-test-granularity:one-scenario 两档池同一个场景：比的是同一串操作在两档上的比，单造一档量不出比。
#[test]
#[ignore = "harness 慢档用例：造 2048 个数据单元的池要发约 460 次发布，debug 下单条超过 60 秒"]
fn compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units(
) {
    let (smaller_readings, larger_readings) = std::thread::scope(|scope| {
        let smaller = scope.spawn(|| {
            lookup_work_of_the_compaction_calls_on_a_pool_of(FILES_OF_THE_SMALLER_GROWTH_POOL)
        });
        let larger = scope.spawn(|| {
            lookup_work_of_the_compaction_calls_on_a_pool_of(FILES_OF_THE_LARGER_GROWTH_POOL)
        });
        (
            smaller.join().expect("量小池的线程没 panic"),
            larger.join().expect("量大池的线程没 panic"),
        )
    });
    println!(
        "publish_lookup_by_identity growth smaller_readings={smaller_readings:?} larger_readings={larger_readings:?}"
    );
    assert!(
        smaller_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL
            && larger_readings.len() >= PUBLISHING_CALLS_REQUIRED_PER_POOL,
        "两档各至少 {PUBLISHING_CALLS_REQUIRED_PER_POOL} 次真发了一版：小池 {smaller_readings:?}，大池 {larger_readings:?}"
    );
    let smaller = sum_of_the_lookup_work(&smaller_readings);
    let larger = sum_of_the_lookup_work(&larger_readings);
    let smaller_calls = u64::try_from(smaller_readings.len()).expect("次数");
    let larger_calls = u64::try_from(larger_readings.len()).expect("次数");
    let growth_in_percent_of_the_entries_examined =
        growth_in_percent_of_the_entries_examined_per_lookup(smaller, larger);
    let growth_in_percent_of_the_index_builds =
        larger.index_builds * smaller_calls * 100 / (smaller.index_builds * larger_calls);
    println!(
        "publish_lookup_by_identity growth smaller={smaller:?} larger={larger:?} growth_in_percent={growth_in_percent_of_the_entries_examined} index_builds_growth_in_percent={growth_in_percent_of_the_index_builds}"
    );
    assert!(
        growth_in_percent_of_the_entries_examined
            < HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE,
        "数据单元四倍，整理每次按身份查平均看过的索引项数涨了 {growth_in_percent_of_the_entries_examined}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（小池 {smaller:?}，大池 {larger:?}）"
    );
    assert!(
        growth_in_percent_of_the_index_builds
            < HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE,
        "数据单元四倍，整理每一趟建索引的次数涨了 {growth_in_percent_of_the_index_builds}%，门槛 {HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE}%（每查一把 key 现建一个索引就按 key 数涨；小池 {smaller:?} 共 {smaller_calls} 次，大池 {larger:?} 共 {larger_calls} 次）"
    );
}

/// 第一个文件之外 `files` 个文件，各整份写 `data_units_per_file` 个数据单元（经会话一个个写，不抬 F）：交回盘、会话与文件的号。
fn pool_with_files_written_once(files: u64, data_units_per_file: usize) -> PoolBuilt {
    pool_with_files_of_the_sizes(&vec![
        data_units_per_file;
        usize::try_from(files).expect("文件数装得进 usize")
    ])
}

/// 第一个文件之外每个文件各整份写一次，第 k 个文件 `data_units_of_each_file[k]` 个数据单元（经会话一个个写，不抬 F）：交回盘、会话与文件的号。
fn pool_with_files_of_the_sizes(data_units_of_each_file: &[usize]) -> PoolBuilt {
    let files = u64::try_from(data_units_of_each_file.len()).expect("文件数装得进 u64");
    let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
    let first_inode = version_of(&session).inode_number_watermark();
    let write_time_of_the_new_inodes = common::FIXED_WRITE_TIME_SECONDS + 600;
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: files,
                write_time_seconds: write_time_of_the_new_inodes,
            },
        )
        .unwrap_or_else(|refusal| panic!("建 {files} 个 inode：{refusal:?}"));
    let inodes: Vec<InodeNumber> = (first_inode..first_inode + files)
        .map(InodeNumber)
        .collect();
    // 每写一个文件写入时间往后推一秒。
    for ((write_time_seconds, inode), data_units) in (write_time_of_the_new_inodes + 1..)
        .zip(&inodes)
        .zip(data_units_of_each_file)
    {
        let content = content_of_data_units(*data_units, inode.0);
        session
            .publish_user_change(
                &mut devices,
                UserChange::SequentialWriteOfAnInode {
                    inode: *inode,
                    content: &content,
                    write_time_seconds,
                },
            )
            .unwrap_or_else(|refusal| panic!("写 inode {}：{refusal:?}", inode.0));
    }
    PoolBuilt {
        devices,
        session,
        inodes,
    }
}

/// 不经会话、直接在会话现行那一版上整份重写 `inode`（一次发布，`transaction::publish_sequential_write_of_an_inode`），会话接到这一版上：
/// 交回上一版、这一版与这一次发布在产品里做的按身份查（每次查至多一次折半查找，这里就判）。一次调用恰好一次发布，下面的用例按这一版逐处算该查几次。
fn whole_rewrite_published_once(
    pool: &mut PoolBuilt,
    inode: InodeNumber,
    seed: u64,
    data_units: usize,
    write_time_seconds: u64,
) -> (TransactionOutput, TransactionOutput, LookupWorkOfEachSite) {
    let parameters = common::parameters();
    let previous = version_of(&pool.session).clone();
    let content = content_of_data_units(data_units, seed);
    let session = &mut pool.session;
    let devices = &mut pool.devices;
    let (published, lookup_work) = lookup_work_while(|| {
        let mut writer = PoolWriter::new(&parameters, devices);
        publish_sequential_write_of_an_inode(
            &mut writer,
            &mut session.allocator,
            &previous,
            inode,
            &content,
            write_time_seconds,
            session.instance,
        )
    });
    let published =
        published.unwrap_or_else(|refusal| panic!("整份重写 inode {}：{refusal:?}", inode.0));
    assert_every_lookup_examines_at_most_a_binary_search(
        &lookup_work,
        &format!("整份重写 inode {}", inode.0),
    );
    session.current = PoolVersion::of_a_version_of_trees(published.clone());
    (previous, published, lookup_work)
}

/// `keys` 按 `version` 中央映射树的形状走到的叶有几片（去重）：批量读者查这几把 key 该解的就是这几片。
fn distinct_mapping_leaves_routed_to_by<'key>(
    version: &TransactionOutput,
    keys: impl Iterator<Item = &'key [u8]>,
) -> u64 {
    let leaves: BTreeSet<CodeTwoTreeNodePosition> = keys
        .filter_map(|key| {
            leaf_position_routed_to(
                &version.central_mapping_tree.shape,
                &CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
            )
        })
        .collect();
    u64::try_from(leaves.len()).expect("叶的片数")
}

/// 这一版照抄（没重写）的单元里角色合 `is_of_the_kind` 的有几个。
fn carried_units_of_the_kind(
    published: &TransactionOutput,
    is_of_the_kind: impl Fn(TransactionUnit) -> bool,
) -> u64 {
    let carried = published
        .units
        .iter()
        .filter(|unit| {
            is_of_the_kind(unit.identity) && !published.rewritten.contains(&unit.identity)
        })
        .count();
    u64::try_from(carried).expect("单元数")
}

/// extent 树的节点（`build_extent_tree` 照抄的那几种）。
fn is_an_extent_tree_node(identity: TransactionUnit) -> bool {
    matches!(
        identity,
        TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
    )
}

/// 第一个文件之外一个文件的下段节点。
fn is_a_lower_node_of_a_file_other_than_the_first(identity: TransactionUnit) -> bool {
    matches!(
        identity,
        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
    )
}

/// `tree` 这棵多层码 2 树的节点（`build_multi_level_tree` 照抄的那几种）。
fn is_a_node_of_the_multi_level_tree(
    tree: MultiLevelCodeTwoTree,
    identity: TransactionUnit,
) -> bool {
    match tree {
        MultiLevelCodeTwoTree::Accounting => matches!(
            identity,
            TransactionUnit::AccountingTree | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        ),
        MultiLevelCodeTwoTree::Deadlist => matches!(
            identity,
            TransactionUnit::DeadlistTree | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
        ),
        MultiLevelCodeTwoTree::CentralMapping => matches!(
            identity,
            TransactionUnit::MappingTree | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        ),
    }
}

/// 照抄三棵多层码 2 树的节点各记在哪一处。
const CARRIED_NODES_OF_EACH_MULTI_LEVEL_TREE: [(MultiLevelCodeTwoTree, LookupSite); 3] = [
    (
        MultiLevelCodeTwoTree::Accounting,
        LookupSite::CarriedNodesOfTheAccountingTree,
    ),
    (
        MultiLevelCodeTwoTree::Deadlist,
        LookupSite::CarriedNodesOfTheDeadlistTree,
    ),
    (
        MultiLevelCodeTwoTree::CentralMapping,
        LookupSite::CarriedNodesOfTheCentralMappingTree,
    ),
];

/// 三棵多层码 2 树照抄的节点按树分开数（七·6 后续批代码轮第一轮判决第三节第 3 条）：照抄了节点的那棵树，那一处走一遍每个照抄的节点
/// 经上一版建一次的索引查一次（不另建索引、不解叶，判整数遍）；一个都没照抄的那棵，那一处不建、不查、不解叶。一棵退回在 `units` 上
/// 线性找，它那一处查的次数掉到 0，另两棵补不上它的数。交回每棵树这一版照抄了几个节点（按 [`CARRIED_NODES_OF_EACH_MULTI_LEVEL_TREE`] 的次序）。
fn assert_the_carried_nodes_of_each_multi_level_tree_are_looked_up_through_one_index(
    published: &TransactionOutput,
    lookup_work: &LookupWorkOfEachSite,
) -> [u64; 3] {
    CARRIED_NODES_OF_EACH_MULTI_LEVEL_TREE.map(|(tree, site)| {
        let carried =
            carried_units_of_the_kind(published, |identity| is_a_node_of_the_multi_level_tree(tree, identity));
        let work = lookup_work.of_the_site(site);
        if carried == 0 {
            assert_eq!(
                (work.index_builds, work.mapping_leaf_decodes, work.index_lookups),
                (0, 0, 0),
                "{site:?}：这一版 {tree:?} 一个节点都没照抄，这一处不建索引、不解叶、不查：{work:?}"
            );
        } else {
            assert_the_work_is_whole_passes_of(
                work,
                (0, 0, carried),
                &format!(
                    "{site:?}：这一版 {tree:?} 照抄了 {carried} 个节点，走一遍每个经上一版建一次的索引查一次（这一处不另建索引、不解叶）"
                ),
            );
        }
        carried
    })
}

/// 一处按「走一遍进一次」进的（[`LookupWork::times_entered`]）：至少进过一次，建索引次数恰好等于进的次数（每走一遍建一份，
/// 不随这一遍查的项数涨；每查一项现建一份的按查的次数涨）。交回进了几次（走了几遍）。
fn assert_one_index_built_per_entry(work: LookupWork, what: &str) -> u64 {
    assert!(
        work.times_entered >= 1 && work.index_builds == work.times_entered,
        "{what}：进了 {} 次（走了几遍），建索引 {} 次，每走一遍恰好建一份：{work:?}",
        work.times_entered,
        work.index_builds
    );
    work.times_entered
}

/// 一次带文件的发布（不搬迁）在产品里做的、七·6 后续批管的那几处（判决第三节第 1–3 条），`data_units_rewritten` 是这次重写的数据单元数：
/// - 装要写的单元与记录点名的单元：每装一次建一份这一版 `units` 按角色的索引，重写的每个角色经它取一次要写的单元，记录点名的每个角色经它取一次单元；
/// - 按 inode 号找文件：释放判定路径每走一遍建一份（建的次数等于释放判定路径进的次数），经映射释放的角色清单与快照判定那一圈各每走一遍建一份；
/// - 留给快照的角色清单两处、记录点名的数据角色那一处：每走一遍建一份按角色的索引，至少这次重写的每个数据单元查一次。
fn assert_the_lookups_of_one_publish_build_one_index_per_pass(
    published: &TransactionOutput,
    lookup_work: &LookupWorkOfEachSite,
    data_units_rewritten: u64,
    what: &str,
) {
    assert_the_units_of_the_published_version_are_looked_up_through_one_index(
        published,
        lookup_work,
        what,
    );
    let release_judgement_passes = lookup_work
        .of_the_site(LookupSite::MappingEntriesOfTheReleaseJudgement)
        .times_entered;
    let files_of_the_release_judgement =
        lookup_work.of_the_site(LookupSite::FilesOfTheReplacedRolesInThePreviousVersion);
    // 快照下换下的数据单元留给快照、不进释放判定路径：一个第一个文件之外的角色都不问的那几遍不建，所以是「至多」。
    assert!(
        release_judgement_passes >= 1
            && files_of_the_release_judgement.index_builds <= release_judgement_passes,
        "{what}：释放判定路径走了 {release_judgement_passes} 遍，按 inode 号找文件每遍至多建一份：{files_of_the_release_judgement:?}"
    );
    let snapshot_judgement_passes = lookup_work
        .of_the_site(LookupSite::SnapshotJudgement)
        .times_entered;
    let files_of_the_snapshot_judgement =
        lookup_work.of_the_site(LookupSite::FilesOfTheUserVisibleRolesOfTheSnapshotJudgement);
    assert_eq!(
        assert_one_index_built_per_entry(
            files_of_the_snapshot_judgement,
            &format!("{what}：快照判定那一圈按 inode 号找文件"),
        ),
        snapshot_judgement_passes,
        "{what}：快照判定那一圈走了 {snapshot_judgement_passes} 遍，每遍问一次换下的角色有没有落点：{files_of_the_snapshot_judgement:?}"
    );
    // 经映射释放的角色清单同样先去掉留给快照的角色：快照下一个第一个文件之外的角色都不剩的那几遍不建，所以是「至多」；
    // 每问一个角色现建一份的，建的次数按问的角色数涨、超过走的遍数。
    let files_of_the_mapped_roles_released =
        lookup_work.of_the_site(LookupSite::FilesOfTheMappedRolesReleasedByThisPublish);
    assert!(
        files_of_the_mapped_roles_released.times_entered >= 1
            && files_of_the_mapped_roles_released.index_builds
                <= files_of_the_mapped_roles_released.times_entered,
        "{what}：经映射释放的角色清单走了 {} 遍，按 inode 号找文件每遍至多建一份：{files_of_the_mapped_roles_released:?}",
        files_of_the_mapped_roles_released.times_entered
    );
    for site in [
        LookupSite::RolesKeptForTheSnapshotsInTheReleaseJudgement,
        LookupSite::RolesKeptForTheSnapshotsAmongTheMappedRolesReleased,
        LookupSite::DataRolesNamedBeforeTheLastRecord,
    ] {
        let work = lookup_work.of_the_site(site);
        let passes = assert_one_index_built_per_entry(work, &format!("{what}：{site:?}"));
        assert!(
            work.index_lookups >= passes * data_units_rewritten,
            "{what}：{site:?} 走了 {passes} 遍，每遍至少这次重写的 {data_units_rewritten} 个数据单元各查一次：{work:?}"
        );
    }
}

/// 装这一版（`publish_admitted`）时取要写的单元与记录点名的单元（七·6 后续批代码轮第一轮判决第三节第 1 条）：每装一次建一份这一版
/// `units` 按角色的索引（记在要写的那一处），重写的每个角色经它取一次要写的单元，记录点名的每个角色（搬迁副本不按角色取，不算）经同一份取一次单元。
/// 改回在 `units` 上线性找的那一处查的次数掉到 0；每取一个角色现建索引的建的次数按角色数涨。
fn assert_the_units_of_the_published_version_are_looked_up_through_one_index(
    published: &TransactionOutput,
    lookup_work: &LookupWorkOfEachSite,
    what: &str,
) {
    let written = lookup_work.of_the_site(LookupSite::UnitsOfTheRolesWrittenByThisPublish);
    let assemblies = assert_one_index_built_per_entry(written, &format!("{what}：装要写的单元"));
    let rewritten = u64::try_from(published.rewritten.len()).expect("角色数");
    let named_entries: usize = published
        .earlier_records_of_this_publish
        .iter()
        .map(|earlier| earlier.record.named.len())
        .sum::<usize>()
        + published.record.named.len();
    let named_roles =
        u64::try_from(named_entries - published.units_relocated_by_this_publish.len())
            .expect("点名项数");
    let named = lookup_work.of_the_site(LookupSite::UnitsOfTheRolesNamedByTheRecords);
    assert_eq!(
        (
            written.index_lookups,
            named.index_builds,
            named.index_lookups
        ),
        (assemblies * rewritten, 0, assemblies * named_roles),
        "{what}：装了 {assemblies} 次，每次 {rewritten} 个重写的角色各取一次要写的单元、{named_roles} 个点名的角色各取一次单元，\
         都经装这一次建的那一份索引（要写的查的次数、点名的建索引次数、点名的查的次数）：written={written:?} named={named:?}"
    );
}

/// 一处做的查找（建索引次数、解叶次数、查的次数）恰好是那一处走一遍该做的 `one_pass` 的正整数倍：发布算到定点时同一处可以走几遍
/// （每一遍对同一个上一版、同一串角色做一样多的事），遍数从建索引次数（建索引不为 0 时）或查的次数现取。交回遍数。
fn assert_the_work_is_whole_passes_of(
    work: LookupWork,
    one_pass: (u64, u64, u64),
    what: &str,
) -> u64 {
    let (builds_of_one_pass, decodes_of_one_pass, lookups_of_one_pass) = one_pass;
    assert!(
        lookups_of_one_pass >= 1,
        "{what}：走一遍至少查一次，这一格才判得出（走一遍 {one_pass:?}）"
    );
    // 走一遍要建索引的，遍数按建索引次数算；走一遍不建索引的，按查的次数算（上面判过走一遍至少查一次）。
    let passes = work
        .index_builds
        .checked_div(builds_of_one_pass)
        .unwrap_or(work.index_lookups / lookups_of_one_pass);
    assert!(
        passes >= 1
            && (
                work.index_builds,
                work.mapping_leaf_decodes,
                work.index_lookups
            ) == (
                passes * builds_of_one_pass,
                passes * decodes_of_one_pass,
                passes * lookups_of_one_pass
            ),
        "{what}：（建索引次数、解叶次数、查的次数）是走一遍 {one_pass:?} 的整数倍（{passes} 遍）：{work:?}"
    );
    passes
}

/// 第一个文件之外 48 个八单元的文件（中央映射树长到不止一片叶）上整份重写其中一个（七·6 代码轮第三轮判决第三节第 1、2 条）。
/// 重写之前，拿那个文件的数据单元与下段节点各调一次释放的三处（`transaction` 的三个公开入口，只读）：每一处各自建两份索引（按角色的 key 索引、
/// 批量读者的单元索引）、那几把 key 走到的叶各解一次、每个角色查一次 key、每片叶查一次单元；释放判定路径那一处每个角色按 inode 号经索引找一次上一版的文件。
/// 再整份重写它（一次发布）：快照判定那一圈（池里没有快照）每建一份索引至少查换下的数据单元那么多次；照抄的 extent 树节点、第一个文件之外文件的
/// 下段节点、多层码 2 树的节点各按角色（按 inode 号）经上一版建一次的索引查一次（发布算到定点时同一处可以走几遍，判整数遍）。
/// 改回线性的那一处查的次数掉到 0，每查一个角色现建索引的那一处建的次数按角色数涨，落点改回逐 key 查的那一处解叶次数按 key 数涨。
/// 七·6 后续批另判：核两条位置项与读盘核校验和经「经映射释放的角色清单」每走一遍各建一份按 inode 号找文件的读者与留给快照的角色清单索引；
/// 那一次发布里三棵多层码 2 树照抄的节点按树分开数，装要写与点名的单元、按 inode 号找文件、留给快照的角色清单与记录点名的数据角色都每走一遍建一份
/// （`assert_the_lookups_of_one_publish_build_one_index_per_pass`）；整棵 extent 树装的时候都在照抄那一处里，每个节点现建一份索引的建的次数记在那里。
/// 判别力：`crates/mutations.tsv` 里以「七·6 第三轮」起头、点名这一条的几行（攻方 RB、RD、RE、RF，正推 M3、M4，释放与照抄那几处改回线性）；
/// 以「七·6 后续批」起头、点名这一条的几行（装要写与点名的单元改回线性找，攻方 MA2、MA3、MA5，正推自打的经映射释放的角色清单那一行）。
#[test]
fn a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site(
) {
    let mut pool = pool_with_files_written_once(48, 8);
    let rewritten_inode = pool.inodes[7];
    let before_the_rewrite = version_of(&pool.session).clone();
    assert!(
        before_the_rewrite.central_mapping_tree.shape.nodes().len() > 1,
        "中央映射树不止一个节点：照抄多层码 2 树的节点走得到"
    );
    let file = before_the_rewrite
        .file_other_than_the_first(rewritten_inode)
        .expect("写过的文件在这一版里");
    let roles: Vec<TransactionUnit> = file
        .data_roles()
        .into_iter()
        .chain(file.extent_lower_node_roles())
        .collect();
    let replaced_roles = u64::try_from(roles.len()).expect("角色数");
    let routed_leaves = distinct_mapping_leaves_routed_to_by(
        &before_the_rewrite,
        before_the_rewrite
            .mapped_units
            .iter()
            .filter(|(role, _)| roles.contains(role))
            .map(|(_, key)| key.as_slice()),
    );
    assert!(
        replaced_roles >= 9,
        "八个数据单元连同下段节点：{replaced_roles}"
    );
    let one_pass_of_a_release_site = (2, routed_leaves, replaced_roles + routed_leaves);
    let (_, release_judgement) = lookup_work_while(|| {
        placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(
            &before_the_rewrite,
            &pool.session.allocator,
            FileContentOfThePublish::NotRewrittenWhole,
            &roles,
            &[],
        )
        .expect("这个文件的单元在上一版里都分配着、映射里查得到")
    });
    let pool_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let (_, two_pool_devices_check) = lookup_work_while(|| {
        refuse_mapping_entries_that_do_not_name_two_pool_devices(
            &before_the_rewrite,
            &roles,
            &[],
            &pool_devices,
            FileContentOfThePublish::NotRewrittenWhole,
        )
        .expect("每条映射条目的两条位置项各指池里一块不同的盘")
    });
    let (_, release_checksum_check) = lookup_work_while(|| {
        copies_failing_the_release_checksum_check(
            &before_the_rewrite,
            &roles,
            &[],
            pool.devices.as_slice(),
            FileContentOfThePublish::NotRewrittenWhole,
        )
        .expect("位置项都指池里的盘")
    });
    println!(
        "publish_lookup_by_identity release_sites replaced_roles={replaced_roles} routed_leaves={routed_leaves} release_judgement={release_judgement:?} two_pool_devices_check={two_pool_devices_check:?} release_checksum_check={release_checksum_check:?}"
    );
    for (site, lookup_work) in [
        (
            LookupSite::MappingEntriesOfTheReleaseJudgement,
            &release_judgement,
        ),
        (
            LookupSite::MappingEntriesCheckedForTwoPoolDevices,
            &two_pool_devices_check,
        ),
        (
            LookupSite::MappingEntriesCheckedAgainstTheReleaseChecksums,
            &release_checksum_check,
        ),
    ] {
        let work = lookup_work.of_the_site(site);
        assert_eq!(
            (work.index_builds, work.mapping_leaf_decodes, work.index_lookups),
            one_pass_of_a_release_site,
            "{site:?}：建两份索引（按角色的 key 索引、批量读者的单元索引）、{replaced_roles} 个角色的 key 走到的 {routed_leaves} 片叶各解一次、\
             每个角色查一次 key、每片叶查一次单元（建索引次数、解叶次数、查的次数）：{work:?}"
        );
    }
    let files_of_the_replaced_roles =
        release_judgement.of_the_site(LookupSite::FilesOfTheReplacedRolesInThePreviousVersion);
    assert_eq!(
        (
            files_of_the_replaced_roles.index_builds,
            files_of_the_replaced_roles.index_lookups
        ),
        (1, replaced_roles),
        "释放判定路径：{replaced_roles} 个角色在上一版里有没有落点，各按 inode 号经一份建一次的索引找一次文件：{files_of_the_replaced_roles:?}"
    );
    // 留给快照的角色清单（这里是空的）在释放判定路径与经映射释放的角色清单那一处各建一次按角色的索引，换下的每个角色查一次。
    let kept_roles_of_the_release_judgement =
        release_judgement.of_the_site(LookupSite::RolesKeptForTheSnapshotsInTheReleaseJudgement);
    assert_eq!(
        (
            kept_roles_of_the_release_judgement.index_builds,
            kept_roles_of_the_release_judgement.index_lookups,
            kept_roles_of_the_release_judgement.times_entered
        ),
        (1, replaced_roles, 1),
        "释放判定路径：留给快照的角色清单建一次按角色的索引，{replaced_roles} 个换下的角色各查一次（建索引次数、查的次数、进的次数）：\
         {kept_roles_of_the_release_judgement:?}"
    );
    // 核两条位置项与读盘核校验和都经「经映射释放的角色清单」（读盘核校验和那一处先调核两条位置项，清单走两遍）：
    // 每走一遍按 inode 号找文件的读者建一份、留给快照的角色清单建一份，换下的每个角色各查一次。
    for (which, lookup_work) in [
        ("核两条位置项", &two_pool_devices_check),
        ("读盘核校验和", &release_checksum_check),
    ] {
        for site in [
            LookupSite::FilesOfTheMappedRolesReleasedByThisPublish,
            LookupSite::RolesKeptForTheSnapshotsAmongTheMappedRolesReleased,
        ] {
            let work = lookup_work.of_the_site(site);
            let passes = assert_one_index_built_per_entry(work, &format!("{which}：{site:?}"));
            assert_eq!(
                work.index_lookups,
                passes * replaced_roles,
                "{which}：{site:?} 走了 {passes} 遍，每遍 {replaced_roles} 个换下的角色各查一次：{work:?}"
            );
        }
    }

    let (_, published, lookup_work) = whole_rewrite_published_once(
        &mut pool,
        rewritten_inode,
        rewritten_inode.0 + 1_000_003,
        8,
        common::FIXED_WRITE_TIME_SECONDS + 5_000,
    );
    println!("publish_lookup_by_identity whole_rewrite lookup_work={lookup_work:?}");
    let snapshot_judgement = lookup_work.of_the_site(LookupSite::SnapshotJudgement);
    assert!(
        snapshot_judgement.index_builds >= 1
            && snapshot_judgement.index_lookups >= 8 * snapshot_judgement.index_builds,
        "快照判定那一圈（池里没有快照）：每走一遍建一份按角色的索引，换下的用户可见角色（至少八个数据单元）各经它查一次映射 key：{snapshot_judgement:?}"
    );
    for (site, carried, which) in [
        (
            LookupSite::CarriedExtentTreeNodes,
            carried_units_of_the_kind(&published, is_an_extent_tree_node),
            "照抄的 extent 树节点",
        ),
        (
            LookupSite::FilesOfTheCarriedExtentLowerNodes,
            carried_units_of_the_kind(&published, is_a_lower_node_of_a_file_other_than_the_first),
            "照抄的第一个文件之外文件的下段节点",
        ),
    ] {
        assert_the_work_is_whole_passes_of(
            lookup_work.of_the_site(site),
            (0, 0, carried),
            &format!(
                "{site:?}：这一版{which}有 {carried} 个，走一遍每个经上一版建一次的索引查一次（这一处不另建索引、不解叶）"
            ),
        );
    }
    let carried_by_tree = assert_the_carried_nodes_of_each_multi_level_tree_are_looked_up_through_one_index(
        &published,
        &lookup_work,
    );
    println!("name=carried test=whole_rewrite_no_snapshot carried_accounting_deadlist_mapping={carried_by_tree:?} none_site={:?}", lookup_work.work_of_each_site.get(&None));
    assert_the_lookups_of_one_publish_build_one_index_per_pass(
        &published,
        &lookup_work,
        8,
        "整份重写一个八单元的文件（池里没有快照）",
    );
}

/// 池里有一个快照的样子：快照之后整份重写过的文件的旧单元进了 deadlist、映射条目留存。
struct PoolWithASnapshot {
    pool: PoolBuilt,
    snapshot: TreeIdentifier,
}

/// 第一个文件之外 `files` 个四单元的文件，建一个快照（经会话）。
fn pool_with_a_snapshot_of_files(files: u64) -> PoolWithASnapshot {
    let mut pool = pool_with_files_written_once(files, 4);
    let created = pool
        .session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .unwrap_or_else(|refusal| panic!("建快照：{refusal:?}"));
    drop(created);
    let snapshot = version_of(&pool.session)
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("建了一个快照")
        .tree;
    PoolWithASnapshot { pool, snapshot }
}

/// 一个快照下整份重写一个文件（一次发布），再删这个快照（经会话）：
/// 重写那一次，快照判定那一圈每走一遍换下的用户可见角色各查一次映射 key，进 deadlist 的各经批量读者查一次映射条目（建三份索引：按角色的 key 索引两份、
/// 批量读者的单元索引一份；进 deadlist 的 key 走到的叶各解一次）；删快照那几批，放掉的每条条目按 key 经索引取一次它留存的映射条目。
/// 七·6 后续批另判（判决第三节第 3 条 ①）：删快照放条目的每一遍恰好建一份留存条目按 key 的索引（建索引次数等于进那一处的次数，
/// 查的次数是遍数乘放掉的条数）；重写那一次按 inode 号找文件等几处每走一遍建一份（`assert_the_lookups_of_one_publish_build_one_index_per_pass`）。
/// 判别力：`crates/mutations.tsv` 里正推 M2（进 deadlist 那一支退回线性找 key、逐 key 查）与「七·6 第三轮 删快照：」起头的那一行；
/// 「七·6 后续批」起头、点名这一条的几行（攻方 MA1，正推自打的删快照与快照判定那一圈两行）。
#[test]
fn a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once(
) {
    let PoolWithASnapshot { mut pool, snapshot } = pool_with_a_snapshot_of_files(8);
    let rewritten_inode = pool.inodes[2];
    let (previous, published, lookup_work) = whole_rewrite_published_once(
        &mut pool,
        rewritten_inode,
        rewritten_inode.0 + 3_000_017,
        4,
        common::FIXED_WRITE_TIME_SECONDS + 7_000,
    );
    let killed_keys: Vec<&[u8]> = published
        .deadlist_entries
        .iter()
        .filter(|entry| {
            !previous
                .deadlist_entries
                .iter()
                .any(|before| before.mapping_key == entry.mapping_key)
        })
        .map(|entry| entry.mapping_key.as_slice())
        .collect();
    let killed = u64::try_from(killed_keys.len()).expect("条数");
    let routed_leaves =
        distinct_mapping_leaves_routed_to_by(&previous, killed_keys.iter().copied());
    let replaced_user_visible_roles = u64::try_from(
        previous
            .mapped_units
            .iter()
            .filter(|(role, _)| {
                published.rewritten.contains(role) && asks_the_snapshot_lifetime_judgement(*role)
            })
            .count(),
    )
    .expect("角色数");
    let snapshot_judgement = lookup_work.of_the_site(LookupSite::SnapshotJudgement);
    println!(
        "publish_lookup_by_identity rewrite_under_a_snapshot killed={killed} routed_leaves={routed_leaves} replaced_user_visible_roles={replaced_user_visible_roles} snapshot_judgement={snapshot_judgement:?}"
    );
    assert!(
        killed >= 4,
        "快照下整份重写：旧的四个数据单元都进了 deadlist（{killed} 条）"
    );
    assert_the_work_is_whole_passes_of(
        snapshot_judgement,
        (
            3,
            routed_leaves,
            replaced_user_visible_roles + killed + routed_leaves,
        ),
        &format!(
            "快照判定那一圈：走一遍建三份索引、进 deadlist 的 {killed} 把 key 走到的 {routed_leaves} 片叶各解一次；换下的 {replaced_user_visible_roles} 个用户可见角色各查一次 key，\
             进 deadlist 的各再查一次 key、每片叶查一次单元"
        ),
    );
    assert_the_lookups_of_one_publish_build_one_index_per_pass(
        &published,
        &lookup_work,
        4,
        "快照下整份重写一个四单元的文件",
    );

    let kept_before = version_of(&pool.session)
        .mapping_entries_kept_for_the_snapshots
        .len();
    let (destroyed, destruction_work) = lookup_work_while(|| {
        pool.session
            .publish_user_change(&mut pool.devices, UserChange::DestroySnapshot { snapshot })
    });
    destroyed.unwrap_or_else(|refusal| panic!("删快照：{refusal:?}"));
    assert_every_lookup_examines_at_most_a_binary_search(&destruction_work, "删快照");
    let kept_after = version_of(&pool.session)
        .mapping_entries_kept_for_the_snapshots
        .len();
    let released = u64::try_from(kept_before - kept_after).expect("条数");
    let kept_entries_released = destruction_work
        .of_the_site(LookupSite::KeptMappingEntriesReleasedByTheSnapshotDestruction);
    println!(
        "publish_lookup_by_identity snapshot_destruction released={released} kept_entries_released={kept_entries_released:?} files_of_the_snapshot_judgement={:?}",
        destruction_work.of_the_site(LookupSite::FilesOfTheUserVisibleRolesOfTheSnapshotJudgement)
    );
    // 八个四单元的文件、重写两个：留存的条目一批就放完（最后一批取全部），放条目的每一遍都放这同一批。
    let passes_that_release = assert_one_index_built_per_entry(
        kept_entries_released,
        "删快照：放条目的每一遍建一份留存条目按 key 的索引",
    );
    assert!(
        released >= 4 && kept_entries_released.index_lookups == passes_that_release * released,
        "删快照放掉 {released} 条留存的映射条目（之前 {kept_before} 条、之后 {kept_after} 条）：发布算到定点时放条目的每一遍（{passes_that_release} 遍），\
         放掉的每条按 key 经那一遍建一次的索引取一次：{kept_entries_released:?}"
    );
}

/// `version` 里第一个文件之外 `inode` 那个文件前 `data_units` 个数据单元的搬迁计划项：映射 key、映射位置上的源落点。
fn planned_relocations_of_the_data_units(
    version: &TransactionOutput,
    inode: InodeNumber,
    data_units: u64,
) -> Vec<PlannedRelocation> {
    (0..data_units)
        .map(|unit| {
            let role = TransactionUnit::DataOfAFileOtherThanTheFirst {
                inode,
                unit: DataUnitIndexInFile(unit),
            };
            let (_, mapping_key) = version
                .mapped_units
                .iter()
                .find(|(mapped, _)| *mapped == role)
                .expect("写过的数据单元进映射");
            let MappingLookup::Found(locations) =
                central_mapping_locations_in_the_version(version, mapping_key)
            else {
                panic!("{role:?} 在这一版的映射里查得到");
            };
            PlannedRelocation {
                mapping_key: mapping_key.clone(),
                source: Placement {
                    slot: locations[0].slot,
                    span: role.span_slots(),
                },
            }
        })
        .collect()
}

/// `version` 里留存的、数据单元那一类的映射条目，从第 `skipped` 条起取 `count` 条的搬迁计划项。
fn planned_relocations_of_kept_data_units(
    version: &TransactionOutput,
    skipped: usize,
    count: usize,
) -> Vec<PlannedRelocation> {
    let planned: Vec<PlannedRelocation> = version
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .filter(|(key, _)| key.first() == Some(&UNIT_CLASS_DATA))
        .skip(skipped)
        .take(count)
        .map(|(key, locations)| PlannedRelocation {
            mapping_key: key.clone(),
            source: Placement {
                slot: locations[0].slot,
                span: 2,
            },
        })
        .collect();
    assert_eq!(planned.len(), count, "留存的数据单元映射条目够 {count} 条");
    planned
}

/// 发一批整理（`transaction::publish_relocation_batch`，搬 `planned` 那几项），会话接到这一版上：判核源落点那一处的计数——
/// 建索引：按 key 的索引、批量读者的单元索引各一份，计划里有留存的映射条目时再建一份留存条目按 key 的索引，不随批大小涨；
/// 解叶：进映射那几项的 key 走到的叶各解一次；查：每一项按 key 查一次，留存的那几项再按 key 在留存条目里查一次，进映射的那几项按角色取一次字节，
/// 每片解的叶按角色取一次单元，合起来是项数的两倍加解叶次数。
fn relocate_a_batch_and_check_the_relocation_sources(
    pool: &mut PoolBuilt,
    planned: &[PlannedRelocation],
    kept_items: u64,
) {
    let parameters = common::parameters();
    let previous = version_of(&pool.session).clone();
    let items = u64::try_from(planned.len()).expect("项数");
    let role_keys: Vec<&[u8]> = planned
        .iter()
        .map(|item| item.mapping_key.as_slice())
        .filter(|key| {
            previous
                .mapped_units
                .iter()
                .any(|(_, mapped)| mapped.as_slice() == *key)
        })
        .collect();
    let routed_leaves = distinct_mapping_leaves_routed_to_by(&previous, role_keys.iter().copied());
    let session = &mut pool.session;
    let devices = &mut pool.devices;
    let (relocated, lookup_work) = lookup_work_while(|| {
        let mut writer = PoolWriter::new(&parameters, devices);
        publish_relocation_batch(
            &mut writer,
            &mut session.allocator,
            &previous,
            planned,
            session.instance,
        )
    });
    let relocated = relocated.unwrap_or_else(|refusal| panic!("搬 {items} 项的一批：{refusal:?}"));
    assert_every_lookup_examines_at_most_a_binary_search(
        &lookup_work,
        &format!("搬 {items} 项的一批"),
    );
    let relocated_kept = relocated
        .units_relocated_by_this_publish
        .iter()
        .filter(|unit| unit.holder == RelocatedUnitHolder::KeptForTheSnapshots)
        .count();
    assert_eq!(
        (
            relocated.units_relocated_by_this_publish.len(),
            u64::try_from(relocated_kept).expect("项数")
        ),
        (planned.len(), kept_items),
        "计划里每一项都搬了，留存的那几项按留存的搬"
    );
    session.current = PoolVersion::of_a_version_of_trees(relocated);
    let relocation_sources = lookup_work.of_the_site(LookupSite::RelocationSources);
    println!(
        "publish_lookup_by_identity relocation_batch items={items} kept_items={kept_items} routed_leaves={routed_leaves} relocation_sources={relocation_sources:?}"
    );
    let index_builds_of_the_batch = if kept_items == 0 { 2 } else { 3 };
    assert_eq!(
        (
            relocation_sources.index_builds,
            relocation_sources.mapping_leaf_decodes,
            relocation_sources.index_lookups
        ),
        (
            index_builds_of_the_batch,
            routed_leaves,
            2 * items + routed_leaves
        ),
        "搬 {items} 项（其中留存的 {kept_items} 项）的一批：建 {index_builds_of_the_batch} 份索引、进映射那几项的 key 走到的 {routed_leaves} 片叶各解一次、\
         查项数两倍加解叶次数那么多次（建索引次数、解叶次数、查的次数）：{relocation_sources:?}"
    );
    // 搬走的、这一版的树引用着的那几项（映射 key 是这一版某个角色的），在上一版的拷贝里换成目的地时按角色找 `units` 里那一项：
    // 建一份按角色的索引，每一项查一次（七·6 后续批代码轮第一轮判决第三节第 1 条）。
    let mapping_positions =
        lookup_work.of_the_site(LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions);
    let roles_relocated = u64::try_from(role_keys.len()).expect("项数");
    assert_eq!(
        (
            mapping_positions.index_builds,
            mapping_positions.index_lookups,
            mapping_positions.times_entered
        ),
        (1, roles_relocated, 1),
        "搬 {items} 项的一批，其中 {roles_relocated} 项是这一版的角色：把映射位置换进 `units` 时建一份按角色的索引、每个这样的角色查一次\
         （建索引次数、查的次数、进的次数）：{mapping_positions:?}"
    );
    assert_the_units_of_the_published_version_are_looked_up_through_one_index(
        version_of(&pool.session),
        &lookup_work,
        &format!("搬 {items} 项的一批"),
    );
}

/// 快照下整份重写过两个文件的池上，整理发三批：两项（一项进映射、一项留存）、八项（四项进映射、四项留存）与三项（都进映射、没有留存的）。
/// 有留存项的两批建的索引都是那三份、没有留存项的那批两份，不随批大小涨（每项现建索引的那一处按项数涨，留存条目的索引不等要找就建的多一份）；
/// 查的次数是项数的两倍加解叶次数（找留存条目或取字节改回线性的少了那几次）；解叶恰好是进映射那几项的 key 走到的叶（落点改回逐 key 查的按项数解）。
/// 每批把映射位置换进上一版的拷贝时，按角色找 `units` 那一项走建一次的索引；装这一版要写与点名的单元走同一份这一版的索引。
/// 判别力：`crates/mutations.tsv` 里攻方 RA、RC，正推 M7，与「七·6 第三轮 核源落点：」起头的两行；「七·6 后续批」起头、点名这一条的几行
/// （映射位置换进 `units`、装要写与点名的单元改回线性找，留存条目索引不等要找就建）。
#[test]
fn relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice(
) {
    let PoolWithASnapshot { mut pool, .. } = pool_with_a_snapshot_of_files(8);
    for (position, rewritten_inode) in [pool.inodes[1], pool.inodes[2]].into_iter().enumerate() {
        whole_rewrite_published_once(
            &mut pool,
            rewritten_inode,
            rewritten_inode.0 + 5_000_011,
            4,
            common::FIXED_WRITE_TIME_SECONDS + 9_000 + u64::try_from(position).expect("序号"),
        );
    }
    let mut planned_batch_of_two =
        planned_relocations_of_the_data_units(version_of(&pool.session), pool.inodes[3], 1);
    planned_batch_of_two.extend(planned_relocations_of_kept_data_units(
        version_of(&pool.session),
        0,
        1,
    ));
    relocate_a_batch_and_check_the_relocation_sources(&mut pool, &planned_batch_of_two, 1);
    let mut planned_batch_of_eight =
        planned_relocations_of_the_data_units(version_of(&pool.session), pool.inodes[4], 4);
    planned_batch_of_eight.extend(planned_relocations_of_kept_data_units(
        version_of(&pool.session),
        1,
        4,
    ));
    relocate_a_batch_and_check_the_relocation_sources(&mut pool, &planned_batch_of_eight, 4);
    // 一项留存的都没有的一批（七·6 后续批代码轮第一轮判决第三节第 3 条 ④）：留存条目按 key 的索引不建，只建两份。
    let planned_batch_without_kept_units =
        planned_relocations_of_the_data_units(version_of(&pool.session), pool.inodes[5], 3);
    relocate_a_batch_and_check_the_relocation_sources(
        &mut pool,
        &planned_batch_without_kept_units,
        0,
    );
}

/// 4096 项、64 个身份各出现 64 次（次序按与表长互素的步长打乱）的长表上，索引对每个身份取的是表里它最先那一项（比地址，同 `iter().find`），
/// 按身份升序交出的每一项也是。短表上不稳定排序碰巧也保序，长表上同一身份的几项会被它换了次序。
/// 判别力：`crates/mutations.tsv` 里正推 M1（稳定排序换成不稳定排序）。
#[test]
fn first_entry_of_each_identity_keeps_the_first_entry_of_identities_repeated_across_a_long_shuffled_table(
) {
    const ENTRIES: u64 = 4096;
    const DISTINCT_IDENTITIES: u64 = 64;
    let table: Vec<(u64, u64)> = (0..ENTRIES)
        .map(|position| {
            (
                position * STRIDE_THAT_SHUFFLES_THE_SYNTHETIC_TABLE % ENTRIES % DISTINCT_IDENTITIES,
                position,
            )
        })
        .collect();
    let index = FirstEntryOfEachIdentity::of_the_table(&table, |(identity, _)| *identity);
    for identity in (0..DISTINCT_IDENTITIES).chain([DISTINCT_IDENTITIES]) {
        let found_linearly = table.iter().find(|(candidate, _)| *candidate == identity);
        assert!(
            is_the_same_entry(index.first_entry_of(&identity), found_linearly),
            "身份 {identity}：索引查到的 {:?} 不是表里它最先那一项 {found_linearly:?}",
            index.first_entry_of(&identity)
        );
    }
    assert_the_index_lists_the_first_entry_of_every_identity_once(
        &index,
        &table,
        |(identity, _)| *identity,
        "4096 项、64 个身份的长表",
    );
}

/// 计数器自己会红（七·6 代码轮第三轮判决第三节第 4 条）：在 512 个身份的合成表上建一次索引记一次建、从前往后与从后往前各走一遍各记 512 项看过、
/// 每个身份连同一个表里没有的各查一次记 513 次查、每次看过的项数在 1 与折半查找的层数之间。
/// 判别力：`crates/mutations.tsv` 里攻方 RG（删掉从前往后走那一句计数）与「七·6 第三轮 计数：」起头的那一行。
#[test]
fn lookup_work_counts_one_entry_per_step_of_a_walk_from_either_end_and_one_lookup_per_search() {
    let table = synthetic_table_of_identities(IDENTITIES_OF_THE_SMALLER_SYNTHETIC_TABLE);
    let (index, building) =
        lookup_work_while(|| FirstEntryOfEachIdentity::of_the_table(&table, |identity| *identity));
    assert_eq!(
        building.of_every_site(),
        LookupWork {
            index_builds: 1,
            ..LookupWork::default()
        },
        "建一次索引记一次建、别的都不记"
    );
    let (walked_forward, forward) = lookup_work_while(|| index.first_entries().count());
    let (walked_backward, backward) = lookup_work_while(|| index.first_entries().rev().count());
    let entries = u64::try_from(table.len()).expect("项数");
    for (walked, work, direction) in [
        (walked_forward, forward, "从前往后"),
        (walked_backward, backward, "从后往前"),
    ] {
        assert_eq!(
            (u64::try_from(walked).expect("项数"), work.of_every_site()),
            (
                entries,
                LookupWork {
                    index_entries_examined: entries,
                    ..LookupWork::default()
                }
            ),
            "{direction}走一遍：走过 {entries} 项、记 {entries} 项看过，不记查与建"
        );
    }
    let (_, searching) = lookup_work_while(|| {
        // 迭代上界是表长加一。
        for identity in table.iter().copied().chain([u64::MAX]) {
            let _ = index.first_entry_of(&identity);
        }
    });
    let searched = searching.of_every_site();
    assert!(
        searched.index_lookups == entries + 1
            && searched.index_builds == 0
            && searched.index_entries_examined > entries
            && searched.index_entries_examined
                <= (entries + 1) * binary_digits_of(entries)
            && searched.largest_index_looked_up == entries,
        "每个身份连同一个表里没有的各查一次：记 {} 次查，每次看过的项在 1 与 {} 之间，最大的索引 {entries} 项：{searched:?}",
        entries + 1,
        binary_digits_of(entries)
    );
}

/// deadlist 树长到几片叶的那个池：第一个文件之外一个小文件，后面跟一串大文件；快照之后整份重写前面那一段大文件，换下的单元进 deadlist。
/// 每个大文件换下约三十几条 deadlist 条目，二十四个约八百条；一片 deadlist 叶装约三百七十条（16 KiB 节点、43 字节条目，推的），按 key 递增追加时
/// 叶分裂成各约一半，于是长到四五片叶（用例开头现判至少三片叶）。
const DATA_UNITS_OF_THE_SMALL_FILE_OF_THE_DEADLIST_POOL: usize = 8;
const DATA_UNITS_OF_A_LARGE_FILE_OF_THE_DEADLIST_POOL: usize = 32;
const LARGE_FILES_OF_THE_DEADLIST_POOL: usize = 26;
const LARGE_FILES_REWRITTEN_UNDER_THE_SNAPSHOT: usize = 24;
/// 这个池里 deadlist 树至少几个节点（根加至少三片叶）：整份重写一个文件只改一两片叶与根，照抄的就不止一个。
const DEADLIST_TREE_NODES_AT_LEAST: usize = 4;

/// 快照下 deadlist 树长到几片叶的池上，整份重写那个八单元的小文件、再整份重写一个没重写过的三十二单元的大文件（各一次发布，一次写的单元数在两档之间变）：
/// - 三棵多层码 2 树照抄的节点按树分开数（七·6 后续批代码轮第一轮判决第三节第 3 条 ③）：deadlist 树照抄了不止一个节点，
///   每个经上一版建一次的索引查一次；
/// - 留给快照的角色清单两处、记录点名的数据角色那一处（判决第三节第 2 条）、按 inode 号找文件三处、装要写与点名的单元（第 1 条），
///   两档都是每走一遍建一份索引、查的次数至少是这次重写的数据单元数（建索引的次数不随一次写的单元数涨；退回逐个比的那一处一次都不查）。
///
/// 判别力：`crates/mutations.tsv` 里攻方 MA4（记账树与 deadlist 树照抄的节点退回在 `units` 上线性找）与「七·6 后续批」起头、
/// 点名这一条的几行（留给快照的角色清单两处、记录点名的数据角色那一处退回 `.contains`）。
#[test]
fn rewrites_of_eight_and_of_thirty_two_data_units_under_a_snapshot_with_a_deadlist_of_several_leaves_build_one_index_per_pass_and_count_each_tree_apart(
) {
    let data_units_of_each_file: Vec<usize> =
        std::iter::once(DATA_UNITS_OF_THE_SMALL_FILE_OF_THE_DEADLIST_POOL)
            .chain(std::iter::repeat_n(
                DATA_UNITS_OF_A_LARGE_FILE_OF_THE_DEADLIST_POOL,
                LARGE_FILES_OF_THE_DEADLIST_POOL,
            ))
            .collect();
    let mut pool = pool_with_files_of_the_sizes(&data_units_of_each_file);
    pool.session
        .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
        .unwrap_or_else(|refusal| panic!("建快照：{refusal:?}"));
    let large_files: Vec<InodeNumber> = pool.inodes[1..].to_vec();
    // 每重写一个大文件写入时间往后推一秒。
    for (write_time_seconds, inode) in (common::FIXED_WRITE_TIME_SECONDS + 20_000..)
        .zip(&large_files[..LARGE_FILES_REWRITTEN_UNDER_THE_SNAPSHOT])
    {
        let content = content_of_data_units(
            DATA_UNITS_OF_A_LARGE_FILE_OF_THE_DEADLIST_POOL,
            inode.0 + 7_000_001,
        );
        pool.session
            .publish_user_change(
                &mut pool.devices,
                UserChange::SequentialWriteOfAnInode {
                    inode: *inode,
                    content: &content,
                    write_time_seconds,
                },
            )
            .unwrap_or_else(|refusal| panic!("快照下整份重写 inode {}：{refusal:?}", inode.0));
    }
    let deadlist_before = version_of(&pool.session);
    println!(
        "publish_lookup_by_identity deadlist_pool deadlist_entries={} deadlist_tree_nodes={}",
        deadlist_before.deadlist_entries.len(),
        deadlist_before.deadlist_tree.shape.nodes().len()
    );
    assert!(
        deadlist_before.deadlist_tree.shape.nodes().len() >= DEADLIST_TREE_NODES_AT_LEAST,
        "deadlist 树长到至少 {DEADLIST_TREE_NODES_AT_LEAST} 个节点：{} 条 deadlist 条目，{} 个节点",
        deadlist_before.deadlist_entries.len(),
        deadlist_before.deadlist_tree.shape.nodes().len()
    );
    let small_file = pool.inodes[0];
    let large_file_not_rewritten = large_files[LARGE_FILES_REWRITTEN_UNDER_THE_SNAPSHOT];
    for (inode, data_units, write_time_seconds) in [
        (
            small_file,
            DATA_UNITS_OF_THE_SMALL_FILE_OF_THE_DEADLIST_POOL,
            common::FIXED_WRITE_TIME_SECONDS + 30_000,
        ),
        (
            large_file_not_rewritten,
            DATA_UNITS_OF_A_LARGE_FILE_OF_THE_DEADLIST_POOL,
            common::FIXED_WRITE_TIME_SECONDS + 30_001,
        ),
    ] {
        let (_, published, lookup_work) = whole_rewrite_published_once(
            &mut pool,
            inode,
            inode.0 + 9_000_011,
            data_units,
            write_time_seconds,
        );
        let what = format!("快照下整份重写一个 {data_units} 单元的文件（deadlist 树几片叶）");
        let [carried_accounting_nodes, carried_deadlist_nodes, carried_mapping_nodes] =
            assert_the_carried_nodes_of_each_multi_level_tree_are_looked_up_through_one_index(
                &published,
                &lookup_work,
            );
        println!(
            "publish_lookup_by_identity deadlist_pool_rewrite data_units={data_units} carried_accounting_nodes={carried_accounting_nodes} \
             carried_deadlist_nodes={carried_deadlist_nodes} carried_mapping_nodes={carried_mapping_nodes} lookup_work={lookup_work:?}"
        );
        assert!(
            carried_deadlist_nodes >= 2,
            "{what}：deadlist 树照抄了不止一个节点（{carried_deadlist_nodes} 个）"
        );
        assert_the_lookups_of_one_publish_build_one_index_per_pass(
            &published,
            &lookup_work,
            u64::try_from(data_units).expect("单元数"),
            &what,
        );
    }
}

/// 子进程标记：设着时这条用例是父进程起的那一趟（进程环境里带着外面设的那个特性变量）。
const CHILD_RUN_MARKER_OF_THE_BUILD_SCRIPT_CHECK: &str =
    "SINGLEFS_HARNESS_CORE_BUILD_SCRIPT_CHECK_CHILD_RUN";
/// cargo 给构建脚本设的、表示 `lookup-work-counters` 开着的那个环境变量名：这里从外面带进来、冒充开着。
const VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE: &str = "CARGO_FEATURE_LOOKUP_WORK_COUNTERS";
/// 下面那条用例的名字（子进程按它 `--exact` 只跑这一条）。
const NAME_OF_THE_BUILD_SCRIPT_CHECK: &str =
    "the_core_build_script_reports_only_the_features_cargo_enabled_even_with_the_feature_variable_brought_in_from_outside";

/// `singlefs-core` 清单 `[features]` 里的特性名（`name = …` 那几行等号左边，不含注释行），按名字排好。
fn feature_names_of_the_core_manifest() -> Vec<String> {
    let manifest_text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../singlefs-core/Cargo.toml"
    ))
    .expect("读得了核心层清单");
    let mut names: Vec<String> = manifest_text
        .lines()
        .skip_while(|line| line.trim() != "[features]")
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| {
            line.split_once('=')
                .map(|(name, _)| name.trim().to_string())
        })
        .collect();
    names.sort();
    names
}

/// `singlefs-core` 构建脚本交出的特性只认 cargo 这一次真开的（七·6 后续批代码轮第一轮判决第三节第 4 条）：
/// - 进程环境里外带 `CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1`、特性不开（这个测试包没有这个特性，编进来的构建脚本里那一行 cfg 为假）：交出空串。
///   变量要真在进程环境里才判得出「不读环境」：在子进程里跑本测试二进制的这一条（带那个变量与一个标记），父进程判子进程绿、恰好跑了一条；
/// - cfg 说开着时交出 `LOOKUP_WORK_COUNTERS`（依赖方 `singlefs_unit_graph::build_features` 按这个写法读回清单里的 `lookup-work-counters`）；
/// - 构建脚本里的特性表与它清单 `[features]` 逐个相同（漏写一个，依赖方就当它没开）。
///
/// 判别力：`crates/mutations.tsv` 里「七·6 后续批 构建脚本：」起头的那一行（交出前把进程环境里设着的 `CARGO_FEATURE_*` 也算开着）。
#[test]
fn the_core_build_script_reports_only_the_features_cargo_enabled_even_with_the_feature_variable_brought_in_from_outside(
) {
    let reported_with_the_feature_off = singlefs_core_build_script::enabled_features_metadata_value(
        &singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG,
    );
    if std::env::var_os(CHILD_RUN_MARKER_OF_THE_BUILD_SCRIPT_CHECK).is_some() {
        assert!(
            std::env::var_os(VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE).is_some(),
            "子进程里外带的 {VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE} 设着"
        );
        assert_eq!(
            reported_with_the_feature_off, "",
            "环境里外带 {VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE}=1、特性没开：构建脚本不报开着"
        );
        return;
    }
    let child = std::process::Command::new(std::env::current_exe().expect("测试二进制自己的路径"))
        .args([
            "--exact",
            NAME_OF_THE_BUILD_SCRIPT_CHECK,
            "--test-threads",
            "1",
            "--nocapture",
        ])
        .env(VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE, "1")
        .env(CHILD_RUN_MARKER_OF_THE_BUILD_SCRIPT_CHECK, "1")
        .output()
        .expect("起得了子进程");
    let child_output = format!(
        "{}{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(
        child.status.success() && child_output.contains("test result: ok. 1 passed"),
        "外带 {VARIABLE_OF_THE_LOOKUP_WORK_COUNTERS_FEATURE}=1 的子进程里这一条绿、恰好跑了一条（退出 {:?}）：{child_output}",
        child.status.code()
    );
    assert_eq!(
        reported_with_the_feature_off, "",
        "不外带变量、特性没开：构建脚本不报开着"
    );
    assert_eq!(
        singlefs_core_build_script::enabled_features_metadata_value(&[(
            "lookup-work-counters",
            true
        )]),
        "LOOKUP_WORK_COUNTERS",
        "cfg 说开着：交出 cargo 环境变量名的写法"
    );
    let mut names_in_the_table: Vec<String> =
        singlefs_core_build_script::FEATURES_OF_THIS_PACKAGE_WITH_THEIR_CFG
            .iter()
            .map(|(feature, _)| (*feature).to_string())
            .collect();
    names_in_the_table.sort();
    assert_eq!(
        names_in_the_table,
        feature_names_of_the_core_manifest(),
        "构建脚本的特性表与 singlefs-core 清单 [features] 逐个相同"
    );
}
