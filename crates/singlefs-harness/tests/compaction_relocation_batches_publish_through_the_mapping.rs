//! 整理的一批（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3，用户定「重构」取乙）：第三个发布入口
//! `transaction::publish_relocation_batch` 把单元原样字节写到目的地、映射条目换到目的地、源落点进 defer、目的地记一条分配代 = 诞生代的记录，
//! journal 点名搬迁副本；写路径以映射为准——内存里那一版记映射位置、父节点重写时写映射位置、重建时映射条目的位置覆盖提示。
//! 期望值都从被测之前那一版现算（单元的映射 key、它的落点、它的诞生代），读回一律从盘上（录制流施加到内存盘上）。
mod common;

use common::{
    build_pool, disk_snapshot, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::Placement;
use singlefs_core::compaction_plan::{
    CompactionSwitches, PlannedRelocation, RelocationPublishOrder,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::{
    birth_txg_of_a_mapping_key, TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::{
    read_the_first_file_of_a_snapshot, rebuild_version, tree_table_entry_bytes_of_root,
    walk_to_file, RebuiltVersion, SnapshotFileReading,
};
use singlefs_core::transaction::{
    central_mapping_locations_in_the_version, publish_create_snapshot, publish_overwrite,
    publish_relocation_batch, publish_sequential_write, FirstFile, MappingLookup,
    MultiLevelCodeTwoTree, PoolWriter, RelocatedUnitHolder, RelocationBatchRefused,
    TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

const INSTANCE: InstanceGeneration = InstanceGeneration(1);
const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// `version` 里 `identity` 这个角色的映射 key 与它的源落点（按映射位置）：一批搬它的计划项。
fn planned_relocation_of(
    version: &TransactionOutput,
    identity: TransactionUnit,
) -> PlannedRelocation {
    let (_, mapping_key) = version
        .mapped_units
        .iter()
        .find(|(role, _)| *role == identity)
        .expect("这个角色进映射");
    let MappingLookup::Found(locations) =
        central_mapping_locations_in_the_version(version, mapping_key)
    else {
        panic!("{identity:?} 在这一版的映射里查得到");
    };
    PlannedRelocation {
        mapping_key: mapping_key.clone(),
        source: Placement {
            slot: locations[0].slot,
            span: identity.span_slots(),
        },
    }
}

/// 同一个进程里接着 `pool.output` 发一批，池接到它那一版上。
fn relocate_and_advance(
    pool: &mut BuiltPool,
    planned: &[PlannedRelocation],
) -> Result<TransactionOutput, RelocationBatchRefused> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let relocated = publish_relocation_batch(
        &mut writer,
        &mut pool.allocator,
        &previous,
        planned,
        INSTANCE,
    )?;
    pool.output = relocated.clone();
    Ok(relocated)
}

/// 把盘上 `slot` 起 `span` 个槽两块盘都清零（模拟源落点已被别的单元复用或 discard 过）。
fn zero_the_slots_on_both_devices(image: &mut MemoryPool, slot: SlotNumber, span: u64) {
    for device in BOTH_DEVICES {
        image
            .devices
            .get_mut(&device)
            .expect("两块盘")
            .zero_fill(DeviceOffsetInBytes(slot.0 * SLOT_BYTES), span * SLOT_BYTES);
    }
}

fn mapping_locations_of(version: &TransactionOutput, mapping_key: &[u8]) -> [LocationEntry; 2] {
    match central_mapping_locations_in_the_version(version, mapping_key) {
        MappingLookup::Found(locations) => locations,
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => {
            panic!("这把 key 在这一版的映射里")
        }
    }
}

/// 搬第一个文件的数据单元：映射条目换到目的地、目的地两盘各一条分配代 = 诞生代的已分配记录、源落点两盘都改成已释放（释放代 = 这一批的 txg）
/// 且在这次释放清单里；末条记录点名搬迁副本（目的地位置项、诞生代）。
#[test]
fn a_batch_moves_the_mapping_entry_to_the_destination_and_releases_the_source() {
    let mut pool = build_pool("compaction-batch-moves-the-mapping");
    let before = pool.output.clone();
    let planned = planned_relocation_of(&before, TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    let batch = relocate_and_advance(&mut pool, std::slice::from_ref(&planned)).expect("一批");
    let destination = mapping_locations_of(&batch, &planned.mapping_key);
    assert_ne!(destination[0].slot, planned.source.slot, "目的地不是源落点");
    assert_eq!(
        batch.units_relocated_by_this_publish.len(),
        1,
        "这一批搬了一个单元"
    );
    let relocated = &batch.units_relocated_by_this_publish[0];
    assert_eq!(
        relocated.destination, destination,
        "映射条目就是搬迁记下的目的地"
    );
    assert_eq!(
        relocated.holder,
        RelocatedUnitHolder::RoleOfThisVersion(TransactionUnit::Data(DataUnitIndexInFile::FIRST)),
        "这一版的树引用着它"
    );
    let birth = birth_txg_of_a_mapping_key(&planned.mapping_key).expect("27 字节的 key");
    for device in BOTH_DEVICES {
        let destination_record = pool
            .allocator
            .record_for(device, destination[0].slot)
            .expect("目的地在册");
        assert!(!destination_record.is_released, "{device:?}：目的地已分配");
        assert_eq!(
            destination_record.generation, birth,
            "{device:?}：目的地的分配代写单元的诞生代（D26 已定项 5 第 2 条）"
        );
        let source_record = pool
            .allocator
            .record_for(device, planned.source.slot)
            .expect("源落点在册");
        assert!(source_record.is_released, "{device:?}：源落点已释放");
        assert_eq!(
            source_record.generation, batch.root.checkpoint_txg,
            "{device:?}：释放代是这一批的 txg"
        );
    }
    assert!(
        batch.released.contains(&planned.source),
        "源落点在这一批的释放清单里"
    );
    assert!(
        batch
            .record
            .named
            .iter()
            .any(|named| named.locations == destination && named.birth_txg == birth),
        "末条记录点名搬迁副本：目的地位置项、诞生代不变"
    );
}

/// 搬一批之后读出的内容逐字不变：源落点两盘都清零，冷走读与重建都经映射位置读回第一个文件，重建出来那一版的数据指针就是映射位置。
#[test]
fn after_a_batch_the_cold_walk_and_the_rebuild_read_the_content_at_the_mapping_position() {
    let mut pool = build_pool("compaction-batch-read-at-the-mapping-position");
    let planned = planned_relocation_of(
        &pool.output.clone(),
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
    );
    let batch = relocate_and_advance(&mut pool, std::slice::from_ref(&planned)).expect("一批");
    let destination = mapping_locations_of(&batch, &planned.mapping_key);
    let mut image = pool.memory_pool();
    zero_the_slots_on_both_devices(&mut image, planned.source.slot, planned.source.span);
    let mut mapping_fallbacks = 0;
    assert_eq!(
        walk_to_file(&image, &batch.root, &mut mapping_fallbacks),
        Ok(Some(file_content())),
        "冷走读读回搬之前的内容"
    );
    assert!(mapping_fallbacks >= 1, "提示那一槽清零了，经映射回退读到的");
    let rebuilt =
        match rebuild_version(&image, &batch.root, Some(batch.record.clone())).expect("重建") {
            RebuiltVersion::WithFile(version) => version,
            RebuiltVersion::WithoutFile | RebuiltVersion::WithTreesAndNoFile(_) => {
                panic!("带文件的一版")
            }
        };
    assert_eq!(
        rebuilt.data_pointers[0].locations, destination,
        "重建时映射条目的位置覆盖提示"
    );
    assert_eq!(
        rebuilt
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .slot,
        destination[0].slot,
        "重建那一版的单元落点就是映射位置"
    );
}

/// 搬了数据单元之后进程退出、可写挂载：挂载接着那一版发的写行与暖机照抄这个单元，映射条目仍在目的地（写路径以映射为准，不退回取提示）。
#[test]
fn a_writable_mount_after_a_batch_keeps_the_mapping_entry_at_the_destination() {
    let mut pool = build_pool("compaction-batch-then-mount");
    let planned = planned_relocation_of(
        &pool.output.clone(),
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
    );
    let batch = relocate_and_advance(&mut pool, std::slice::from_ref(&planned)).expect("一批");
    let destination = mapping_locations_of(&batch, &planned.mapping_key);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    let current = mounted.current.version_of_trees().expect("有树的一版");
    assert_eq!(
        mapping_locations_of(current, &planned.mapping_key),
        destination,
        "挂载发的那几版照抄这个单元：映射条目仍在目的地"
    );
    assert_eq!(
        current.data_pointers[0].locations, destination,
        "挂载交回的那一版记着映射位置"
    );
}

/// 搬 extent 树的根：这一批重写的树表里 extent 树那一条写的是映射位置（父节点重写时写映射位置，C417 第一版「随重写刷新」）；盘上读回的同。
#[test]
fn the_tree_table_rewritten_by_the_batch_names_the_relocated_root_at_its_mapping_position() {
    let mut pool = build_pool("compaction-batch-tree-table-names-the-root");
    let planned = planned_relocation_of(&pool.output.clone(), TransactionUnit::ExtentRoot);
    let batch = relocate_and_advance(&mut pool, std::slice::from_ref(&planned)).expect("一批");
    let destination = batch.units_relocated_by_this_publish[0].destination;
    assert_ne!(destination[0].slot, planned.source.slot, "搬走了");
    let image = pool.memory_pool();
    let extent_entry_on_disk = tree_table_entry_bytes_of_root(&image, &batch.root)
        .expect("树表读得出")
        .iter()
        .filter_map(|bytes| TreeTableEntry::parse(bytes.as_slice()))
        .find(|entry| entry.kind == TREE_KIND_EXTENT)
        .expect("树表里有 extent 树那一条");
    assert_eq!(
        extent_entry_on_disk.root.locations, destination,
        "盘上树表里 extent 树的根指针写的是映射位置"
    );
    let mut mapping_fallbacks = 0;
    assert_eq!(
        walk_to_file(&image, &batch.root, &mut mapping_fallbacks),
        Ok(Some(file_content())),
        "读得回第一个文件"
    );
    assert_eq!(mapping_fallbacks, 0, "树表直接指着目的地，不回退");
}

/// 被快照引用的块照样能搬（判决 G3，映射条目留存是前提）：建快照、覆盖写之后旧数据单元只被快照引用；搬它之后留存的映射条目在目的地，
/// 源落点清零，快照照样读出建它那一刻的内容。
#[test]
fn a_unit_kept_only_for_a_snapshot_is_relocated_and_the_snapshot_still_reads_it() {
    let mut pool = build_pool("compaction-batch-moves-a-snapshot-unit");
    let publish_parameters = parameters();
    let created = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE)
            .expect("建快照")
    };
    pool.output = created.clone();
    let snapshot = created
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("建了一个快照")
        .tree;
    let overwrite_content: Vec<u8> = (0..2600_u32)
        .map(|index| u8::try_from((index * 7 + 3) % 241).expect("小于 256"))
        .collect();
    let overwritten = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &overwrite_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            INSTANCE,
        )
        .expect("快照下覆盖写")
    };
    pool.output = overwritten.clone();
    let (kept_key, kept_locations) = overwritten
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(key, _)| key.first() == Some(&singlefs_core::unit::UNIT_CLASS_DATA))
        .cloned()
        .expect("旧数据单元的映射条目留存着");
    let planned = PlannedRelocation {
        mapping_key: kept_key.clone(),
        source: Placement {
            slot: kept_locations[0].slot,
            span: 2,
        },
    };
    let batch = relocate_and_advance(&mut pool, std::slice::from_ref(&planned)).expect("一批");
    let (_, kept_after) = batch
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(key, _)| *key == kept_key)
        .expect("留存的映射条目还在");
    assert_ne!(
        kept_after[0].slot, kept_locations[0].slot,
        "留存的映射条目换到了目的地"
    );
    assert_eq!(
        batch.units_relocated_by_this_publish[0].holder,
        RelocatedUnitHolder::KeptForTheSnapshots,
        "搬的是只被快照引用的那一种"
    );
    let mut image = pool.memory_pool();
    zero_the_slots_on_both_devices(&mut image, planned.source.slot, planned.source.span);
    assert_eq!(
        read_the_first_file_of_a_snapshot(&image, &batch.root, snapshot),
        Ok(SnapshotFileReading::File(file_content())),
        "快照照样读出建它那一刻的内容"
    );
}

/// 必红世界（`.claude/kb/milestone/04-fourth-txn.md` 二·3：源落点在映射发布之前就被释放）：只供测试的开关让这一批释放源落点、写目的地，
/// 映射条目与指针却留在源落点上。池级 checker 判红。
#[test]
fn a_batch_that_releases_the_source_while_the_mapping_still_names_it_is_judged_red() {
    let mut pool = build_pool("compaction-batch-source-released-before-the-mapping");
    let product = CompactionSwitches::PRODUCT;
    pool.allocator.set_compaction_switches(CompactionSwitches {
        batch_size: product.batch_size,
        forced_region: product.forced_region,
        run_forcing: product.run_forcing,
        region_exclusion: product.region_exclusion,
        destination_policy: product.destination_policy,
        release_checksum_mismatch_after_issue: product.release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: product.empty_cluster_segments_floor,
        relocation_publish_order:
            RelocationPublishOrder::SourceReleasedWhileTheMappingStillNamesItForTests,
    });
    let planned = planned_relocation_of(
        &pool.output.clone(),
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
    );
    let batch =
        relocate_and_advance(&mut pool, std::slice::from_ref(&planned)).expect("一批照样发出去");
    assert_eq!(
        mapping_locations_of(&batch, &planned.mapping_key)[0].slot,
        planned.source.slot,
        "映射条目留在源落点上"
    );
    let image = pool.memory_pool();
    let violated: Vec<&'static str> = check_pool_image(&image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    assert!(!violated.is_empty(), "源落点先于映射释放的镜像判红");
}

/// 连发两批、各搬一个数据单元，进程不卸载就退出、可写挂载：两个单元的映射条目都在各自那一批的目的地（判决 G3「映射条目换位置」）。
/// 文件比中央映射树一片叶装得下的条目多一个数据单元，映射树至少两片叶：数据单元的码 1 key 排在最前，装它们的那片叶里一批换号的那几把 key
/// 一把都没有（换号的是这一版新写的节点，码 2）——只按 key 进出定重写哪几片叶，这片叶就照抄上一版、条目的位置留在源落点上，
/// 挂载按映射覆盖提示时把单元指回已经释放的源落点。期望值从被测之前那一版现算（单元的映射 key、源落点），挂载之后从盘上重建的那一版读。
#[test]
fn consecutive_batches_rewrite_the_mapping_leaf_holding_each_moved_entry_and_a_writable_mount_keeps_both(
) {
    let mut pool = build_pool("compaction-batches-rewrite-the-mapping-leaf");
    let leaf_entries = MultiLevelCodeTwoTree::CentralMapping
        .node_capacity_of_the_node_format()
        .leaf_entries;
    let content: Vec<u8> = (0..(leaf_entries + 1) * data_unit_payload_capacity())
        .map(|index| u8::try_from(index % 241).expect("小于 256"))
        .collect();
    let written = {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            INSTANCE,
        )
        .expect("顺序写一个跨多个数据单元的文件")
    };
    pool.output = written.clone();
    assert_eq!(
        written.data_pointers.len(),
        leaf_entries + 1,
        "文件比映射树一片叶的条目多一个数据单元"
    );
    assert!(
        written
            .central_mapping_tree
            .shape
            .nodes()
            .iter()
            .filter(|node| node.position.level == 0)
            .count()
            >= 2,
        "中央映射树至少两片叶：装数据单元条目的那片叶在一批里没有 key 进出"
    );
    let first_unit = TransactionUnit::Data(DataUnitIndexInFile::FIRST);
    let second_unit = TransactionUnit::Data(DataUnitIndexInFile(1));
    let planned_first = planned_relocation_of(&written, first_unit);
    let first_batch =
        relocate_and_advance(&mut pool, std::slice::from_ref(&planned_first)).expect("第一批");
    let planned_second = planned_relocation_of(&first_batch, second_unit);
    let second_batch =
        relocate_and_advance(&mut pool, std::slice::from_ref(&planned_second)).expect("第二批");
    let first_destination = first_batch.units_relocated_by_this_publish[0].destination;
    let second_destination = second_batch.units_relocated_by_this_publish[0].destination;
    assert_ne!(
        first_destination[0].slot, planned_first.source.slot,
        "第一批搬走了"
    );
    assert_ne!(
        second_destination[0].slot, planned_second.source.slot,
        "第二批搬走了"
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    let current = mounted.current.version_of_trees().expect("有树的一版");
    assert_eq!(
        mapping_locations_of(current, &planned_first.mapping_key),
        first_destination,
        "挂载之后第一批搬的单元的映射条目仍在目的地，不回源落点"
    );
    assert_eq!(
        mapping_locations_of(current, &planned_second.mapping_key),
        second_destination,
        "挂载之后第二批搬的单元的映射条目在目的地"
    );
    assert_eq!(
        current.data_pointers[0].locations, first_destination,
        "挂载交回的那一版第一个数据指针是映射位置"
    );
    assert_eq!(
        current.data_pointers[1].locations, second_destination,
        "挂载交回的那一版第二个数据指针是映射位置"
    );
    assert_eq!(
        mapping_locations_of(&first_batch, &planned_first.mapping_key),
        first_destination,
        "第一批交回的那一版里映射条目就在目的地"
    );
    assert_eq!(
        mapping_locations_of(&second_batch, &planned_first.mapping_key),
        first_destination,
        "第二批照抄第一批搬的单元：映射条目取上一版的映射位置"
    );
}

/// 计划的源落点不是映射位置：在任何写之前拒成 `PlannedSourceIsNotTheMappingPosition`，盘上逐项不变。
#[test]
fn a_planned_source_off_the_mapping_position_is_refused_before_any_write() {
    let mut pool = build_pool("compaction-batch-plan-off-the-mapping");
    let mut planned = planned_relocation_of(
        &pool.output.clone(),
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
    );
    planned.source.slot = SlotNumber(planned.source.slot.0 + 2);
    let disk_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = relocate_and_advance(&mut pool, std::slice::from_ref(&planned));
    assert!(
        matches!(
            refused,
            Err(RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition { .. })
        ),
        "拒成 PlannedSourceIsNotTheMappingPosition：{refused:?}"
    );
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        disk_before,
        "在任何写之前拒：盘上逐项不变"
    );
}
