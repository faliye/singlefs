//! harness 用例与给定状态用例共用场景；池级 checker 镜像交给给定状态引擎两段判。
//! 发布路径与整理读版本按身份查（里程碑四七·6，`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行）：
//! 照抄上一版的单元（`TransactionOutput::units_by_role`）、照抄角色的映射 key（`TransactionOutput::mapped_units_by_role`）、
//! 照抄的第一个文件之外的文件（`TransactionOutput::files_other_than_the_first_by_inode`）、整理读一版时逐 key 查中央映射
//! （`transaction::CentralMappingOfAVersionReadLeafByLeaf`）由在 `Vec` 上 `iter().find` 换成按身份建一次的索引。这里钉三样：
//! - 等价：两档池（十六个四单元的文件、四十八个八单元的文件，后一档的中央映射树不止一片叶）上跑同一串操作（按 inode 号写、整份重写四分之三的文件、
//!   抬 F、整理到停、整理之后再写、unlink 一个文件），每一版的每一次按身份查都与测试里的线性查找参照逐项相同，池级 checker 一条违例都没有；
//!   映射叶解不开、条目窄于字段表、同一把 key 有两条，三种叶上批量读与逐 key 查逐项相同。整理读每一版取的映射条目
//!   （`mounted_session::central_mapping_as_the_compaction_reads_it`）与逐 key 查的参照逐项相同（不漏、不错、次序同），批量读者解的叶
//!   恰好是进映射的 key 走到的那几片。
//! - 数的是产品调用点上做的事：核心层在特性 `lookup-work-counters` 下按处记下按身份查了几次、每次看过
//!   索引几项、建了几次索引、解了几次中央映射叶（`singlefs_core::transaction::lookup_work`，验证包的 dev 依赖打开这个特性），
//!   用例只读这些计数，不在测试里另建索引来数，不读挂钟、不读 CPU 时间。每一次用户改动、每一趟整理：每次查至多看折半查找那么多项；
//!   整份重写一个文件时快照判定那一圈与释放那三处都经索引查了换下的每个数据单元；整理每发一批，计划里每一项都经按 key 的索引查过；
//!   整理读一版恰好解进映射的 key 走到的那几片叶、各解一次，只建一次单元索引。
//! - 增长：表长四倍时每次查平均看过的索引项数涨不到门槛的倍数（合成表一条用例；512 与 2048 个数据单元的两种池上每一趟整理，
//!   各一条用例），整理每一趟建索引的次数也涨不到门槛的倍数。
//!
//! 池都在内存里的两块 4 GiB 稀疏盘上造（`SparseBlockDevice`），配方照 E185 装置的 `construct_the_pool`：每个文件整份写进去，
//! 再整份重写下标不是 4 的倍数的那几个，抬 F 到换下的落点都回收，留下每段约四分之一还活着的聚簇段。
//!
//! 七·6 后续批（`research/prompts/m4-publish-lookup-followup-code-r1-main-verification.md` 第三节第 1–4 条）另钉：整理发批与恢复把映射位置换进
//! `units`、装一次发布要写与记录点名的单元、留给快照的角色清单两处、记录点名的数据角色那一处都经建一次的索引查；按 inode 号找文件、
//! 删快照取留存条目那几处在发布路径上建索引的次数恰好等于走了几遍；三棵多层码 2 树照抄的节点按树分开数；`singlefs-core` 构建脚本
//! 交出的特性只认 cargo 这一次真开的。
//!
//! 另钉：
//! 恢复重建一版把映射位置换进 `units` 每次重挂建一份、查有位置的进映射角色那么多次；恢复按 inode 号找 inode 记录与照抄第一个文件时问数据单元
//! 在不在这次重写的角色里各建一次索引；点名的几处之外（`None`）建的索引不随一次写的单元数涨；记账树照抄到不止一个节点时按树分开数；
//! 一把有位置的 key 都没有就不建；构建脚本真开着时报开着，`RUSTFLAGS` 的 `--cfg feature=…` 与它自己的 cfg 不符时停下。
//!
//! 已知边界（不判）：计数只数索引上的动作（建、查、走），「索引照用、旁边再加一遍线性找」在计数里看不见——把映射位置换进
//! `units`、装要写的单元、装记录点名的单元、留给快照的角色清单两处、记录点名的数据角色那一处都是这样。判它要数产品里每一次身份比较
//! （给 `TransactionUnit` 的相等比较计数，碰全部调用点），或量挂钟（这份文件不读挂钟）；这一类写法是对抗写法，正常改代码不会在用了索引之后
//! 再逐个比一遍，所以登记在这里、不进 `crates/mutations.tsv`。攻方 NA4（记录点名的数据角色那一处索引之外再逐个 `contains`）、正推 ALT4
//! （留给快照的角色清单那一处同样）、正推 ALT5（映射位置换进 `units` 时索引之外再线性 `position` 一遍）三条都落在这一类，跑这份文件全绿。

#![allow(dead_code, reason = "引它的测试目标各只调用其中一部分场景与帮手")]

use crate::common_images_for_the_given_states_engine::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_shared_by_harness_scenarios::{
    pseudo_random_content_five_bytes_short_of_data_units, SparseDevices,
};
use std::sync::Arc;
/// `singlefs-core` 的构建脚本本身：在这里编进来，判它交出的特性只看
/// `cfg!(feature = …)`、不读进程环境。这个测试包不开 `lookup-work-counters`，所以那一行的 cfg 在这里恒为假；
/// 构建脚本的 `main` 在这里用不上。
#[allow(
    dead_code,
    unexpected_cfgs,
    reason = "构建脚本的 main 在测试里不调；它写的 cfg!(feature = \"lookup-work-counters\") 是 singlefs-core 的特性，这个测试包没有这个特性"
)]
#[path = "../../../singlefs-core/build.rs"]
mod singlefs_core_build_script;

use std::collections::BTreeSet;

use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, InodeNumber, SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::code_two_tree::{
    leaf_position_routed_to, CodeTwoKeyFieldWidths, CodeTwoTreeKey, CodeTwoTreeNodeCapacity,
    CodeTwoTreeNodePosition,
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
    ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT,
};
use singlefs_core::transaction::{
    acquire_instance, asks_the_snapshot_lifetime_judgement,
    central_mapping_locations_in_the_version, copies_failing_the_release_checksum_check,
    placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots, publish_first_file,
    publish_new_inodes, publish_relocation_batch, publish_sequential_write_of_an_inode,
    refuse_mapping_entries_that_do_not_name_two_pool_devices,
    take_the_mapping_positions_over_the_location_hints, warm_up,
    CentralMappingOfAVersionReadLeafByLeaf, CodeTwoTreeNodeCapacities, FileContentOfThePublish,
    FirstEntryOfEachIdentity, FirstFile, MappedUnitsOfAVersionReadByRole, MappingLookup,
    MultiLevelCodeTwoTree, PoolVersion, PoolWriter, RelocatedUnitHolder, TransactionOutput,
    TransactionUnit,
};
use singlefs_core::unit::{build_index_node, parse_index_node, UNIT_CLASS_DATA};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

#[path = "publish_lookup_by_identity/compaction_lookup_work_from_512_to_2048_data_units.rs"]
mod compaction_lookup_work_from_512_to_2048_data_units;
#[path = "publish_lookup_by_identity/core_build_script_features.rs"]
mod core_build_script_features;
#[path = "publish_lookup_by_identity/deadlist_carried_first_files_and_remounts.rs"]
mod deadlist_carried_first_files_and_remounts;
#[path = "publish_lookup_by_identity/every_lookup_agrees_with_a_linear_find.rs"]
mod every_lookup_agrees_with_a_linear_find;
#[path = "publish_lookup_by_identity/first_entry_of_each_identity_and_the_leaf_by_leaf_mapping_read.rs"]
mod first_entry_of_each_identity_and_the_leaf_by_leaf_mapping_read;
#[path = "publish_lookup_by_identity/snapshots_and_relocation_batches.rs"]
mod snapshots_and_relocation_batches;
#[path = "publish_lookup_by_identity/whole_rewrites_looked_up_through_one_index.rs"]
mod whole_rewrites_looked_up_through_one_index;

#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use compaction_lookup_work_from_512_to_2048_data_units::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use core_build_script_features::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use deadlist_carried_first_files_and_remounts::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use every_lookup_agrees_with_a_linear_find::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use first_entry_of_each_identity_and_the_leaf_by_leaf_mapping_read::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use snapshots_and_relocation_batches::*;
#[allow(
    unused_imports,
    reason = "引这一份的测试目标各只用其中一部分，用不到这一块的测试二进制里转出去没人用"
)]
pub use whole_rewrites_looked_up_through_one_index::*;

/// 512 字节的物理块：与 `crate::common::parameters` 的几何同。
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
        crate::common::IMAGE_BYTES,
        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
    )
}

/// 两块稀疏盘上 mkfs → 取号 → 暖机 → 新池新建文件，再按 `switches` 可写挂载（同 E185 装置的 `formatted_pool`）。
fn formatted_and_mounted(switches: CompactionSwitches) -> (SparseDevices, MountedSession) {
    let parameters = crate::common::parameters();
    let mut devices: SparseDevices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, sparse_device()))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), crate::common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), crate::common::IMAGE_BYTES),
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
        let first_file = crate::common::file_content();
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_file,
                write_time_seconds: crate::common::FIXED_WRITE_TIME_SECONDS,
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

fn version_of_the_session(session: &MountedSession) -> &TransactionOutput {
    session
        .current
        .version_of_trees()
        .expect("新池新建文件之后每一版都有树")
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
    devices: &mut SparseDevices,
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
    devices: &mut SparseDevices,
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
    devices: SparseDevices,
    session: MountedSession,
    inodes: Vec<InodeNumber>,
}

/// 照 E185 装置 `construct_the_pool` 的配方造池：建 `files` 个 inode、每个整份写 `data_units_per_file` 个数据单元；整份重写下标不是 4 的倍数的
/// 那几个（换下的落点在原来那几段里留下空洞）；抬 F 到换下的落点都回收（每段约四分之一还活着）。
fn build_a_pool_of_files(
    shape: PoolOfFiles,
    on_every_version: &mut OnEveryVersion<'_>,
) -> PoolBuilt {
    let parameters = crate::common::parameters();
    let (mut devices, mut session) = formatted_and_mounted(CompactionSwitches::PRODUCT);
    let mut write_time_seconds = crate::common::FIXED_WRITE_TIME_SECONDS + 600;
    let first_inode = version_of_the_session(&session).inode_number_watermark();
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
        let content = pseudo_random_content_five_bytes_short_of_data_units(
            shape.data_units_per_file,
            inode.0,
        );
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
        replaced_slots.extend(data_slots_of_the_file(
            version_of_the_session(&session),
            *inode,
        ));
        let content = pseudo_random_content_five_bytes_short_of_data_units(
            shape.data_units_per_file,
            inode.0 + 1_000_003,
        );
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
        let mut current = version_of_the_session(&session).clone();
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
            slots_under_unreleased_records_on_device_zero(version_of_the_session(&session));
        // 换下的落点回收之后可能又被后面的整份重写取去装新的数据单元：那样的槽算回收过（同 E185 装置 `construct_the_pool` 的判法）。
        let slots_of_live_data_units: std::collections::BTreeSet<SlotNumber> = inodes
            .iter()
            .flat_map(|inode| data_slots_of_the_file(version_of_the_session(&session), *inode))
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
    let first_inode = version_of_the_session(&session).inode_number_watermark();
    let write_time_of_the_new_inodes = crate::common::FIXED_WRITE_TIME_SECONDS + 600;
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
        let content = pseudo_random_content_five_bytes_short_of_data_units(*data_units, inode.0);
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
    whole_rewrite_published_once_with_node_capacities(
        pool,
        inode,
        seed,
        data_units,
        write_time_seconds,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    )
}

/// 同 [`whole_rewrite_published_once`]，写入口装上 `node_capacities`（只供测试的记账树与中央映射树节点容量）。
fn whole_rewrite_published_once_with_node_capacities(
    pool: &mut PoolBuilt,
    inode: InodeNumber,
    seed: u64,
    data_units: usize,
    write_time_seconds: u64,
    node_capacities: CodeTwoTreeNodeCapacities,
) -> (TransactionOutput, TransactionOutput, LookupWorkOfEachSite) {
    let parameters = crate::common::parameters();
    let previous = version_of_the_session(&pool.session).clone();
    let content = pseudo_random_content_five_bytes_short_of_data_units(data_units, seed);
    let session = &mut pool.session;
    let devices = &mut pool.devices;
    let (published, lookup_work) = lookup_work_while(|| {
        let mut writer = PoolWriter::new(&parameters, devices);
        writer.set_code_two_tree_node_capacities(node_capacities);
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

/// 不经会话、直接在会话现行那一版上经写入口建一个 size 0 的 inode（`transaction::publish_new_inodes`，一次发布），写入口装上 `node_capacities`，
/// 会话接到这一版上。inode 号水位每变一次留一级台阶行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」），下一次发布照抄它：
/// 记账树一次长一行。
fn one_inode_created_with_node_capacities(
    pool: &mut PoolBuilt,
    write_time_seconds: u64,
    node_capacities: CodeTwoTreeNodeCapacities,
) {
    let parameters = crate::common::parameters();
    let previous = version_of_the_session(&pool.session).clone();
    let mut writer = PoolWriter::new(&parameters, &mut pool.devices);
    writer.set_code_two_tree_node_capacities(node_capacities);
    let created = publish_new_inodes(
        &mut writer,
        &mut pool.session.allocator,
        &previous,
        1,
        write_time_seconds,
        pool.session.instance,
    )
    .unwrap_or_else(|refusal| panic!("建一个 inode：{refusal:?}"));
    pool.session.current = PoolVersion::of_a_version_of_trees(created);
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

/// 三棵多层码 2 树照抄的节点按树分开数：照抄了节点的那棵树，那一处走一遍每个照抄的节点
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

/// 一次带文件的发布（不搬迁）在产品里做的那几处，`data_units_rewritten` 是这次重写的数据单元数：
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
    // 建索引次数等于进的次数分不开「每走一遍建一份」与「每问一个角色进一次、建一份」：再判每遍查的次数，
    // 这次重写的每个数据单元在上一版里有没有落点各按 inode 号查一次文件。
    assert!(
        files_of_the_snapshot_judgement.index_lookups
            >= snapshot_judgement_passes * data_units_rewritten,
        "{what}：快照判定那一圈走了 {snapshot_judgement_passes} 遍，每遍这次重写的 {data_units_rewritten} 个数据单元各按 inode 号查一次文件：\
         {files_of_the_snapshot_judgement:?}"
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

/// 装这一版（`publish_admitted`）时取要写的单元与记录点名的单元：每装一次建一份这一版
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

/// 一次装发布在点名的几处之外（`None` 那一格）至多建几份索引。量到的：
/// 这份用例文件里没有快照时整份重写一个八单元的文件是 5（照抄第一个文件时整份重写两单元与十六单元的四次一样多），快照下 deadlist 几片叶的池上
/// 整份重写八单元与三十二单元的文件是 5 与 6；64 个文件的池上量到 6（
/// T2.2）。份数是常数，不随这次写的单元数涨；
/// 每查一项现建一份、又建在点名的那一处之外的，按项数涨。
const INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST: u64 = 6;

/// 一次发布在点名的几处之外（`None` 那一格）建的索引：每装一次至多 [`INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST`] 份
/// （装的次数取装要写的单元那一处进的次数）。交回（装的次数，`None` 那一格建索引的次数），给两档单元数比「不涨」用。
fn assert_the_index_builds_outside_the_named_sites_stay_bounded(
    lookup_work: &LookupWorkOfEachSite,
    what: &str,
) -> (u64, u64) {
    let assemblies = lookup_work
        .of_the_site(LookupSite::UnitsOfTheRolesWrittenByThisPublish)
        .times_entered;
    let outside_the_named_sites = lookup_work
        .work_of_each_site
        .get(&None)
        .copied()
        .unwrap_or_default();
    assert!(
        assemblies >= 1
            && outside_the_named_sites.index_builds
                <= assemblies * INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST,
        "{what}：装了 {assemblies} 次，点名的几处之外每装一次至多建 {INDEX_BUILDS_OUTSIDE_THE_NAMED_SITES_PER_ASSEMBLY_AT_MOST} 份索引，\
         建了 {} 份：{outside_the_named_sites:?}",
        outside_the_named_sites.index_builds
    );
    (assemblies, outside_the_named_sites.index_builds)
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
