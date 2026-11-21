//! 里程碑四：快照、deadlist 与删快照，整理搬单元与 discard，自建 GC 新判的那几条（I-3.15、I-5.6、I-4.4、I-3.16、I-1.14）。

use super::*;

/// 快照下覆盖写的内容：与第一个文件（`common::file_content`）不同的一份。
pub(super) fn content_overwritten_under_the_snapshot() -> Vec<u8> {
    (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + 5) % 239).expect("小于 256"))
        .collect()
}

/// 一次发布交回的那一版在版本表里那一行：(实例代号, checkpoint_txg) 取它的根，内容由调用方按条款给。
pub(super) fn published_version_of(
    output: &TransactionOutput,
    content: Vec<u8>,
) -> PublishedVersion {
    PublishedVersion {
        instance: output.root.instance,
        checkpoint_txg: output.root.checkpoint_txg,
        content,
    }
}

/// 同一个进程里接着 `pool.output` 建一个快照，池接到它那一版上，交回那一版。
pub(super) fn create_a_snapshot_and_advance(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let created = publish_create_snapshot(
        &mut writer,
        &mut pool.allocator,
        &previous,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("建快照");
    pool.output = created.clone();
    created
}

/// 建的那个快照的树号（码 14 条目点名它）。
///
/// # Panics
/// 这一版的树表里码 14 条目不是恰好一条：这几条流只建一个快照。
fn the_only_snapshot_of(version: &TransactionOutput) -> TreeIdentifier {
    let snapshots: Vec<TreeIdentifier> = version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| entry.tree)
        .collect();
    let [snapshot] = snapshots.as_slice() else {
        panic!("这一版的树表里恰好一条码 14 条目，实际 {snapshots:?}");
    };
    *snapshot
}

/// 同一个进程里接着 `pool.output` 在快照下覆盖写一次（内容 [`content_overwritten_under_the_snapshot`]），池接到它那一版上，交回那一版。
pub(super) fn overwrite_under_the_snapshot_and_advance(pool: &mut BuiltPool) -> TransactionOutput {
    let previous = pool.output.clone();
    let overwritten = publish_overwrite_in_process(
        pool,
        &previous,
        &content_overwritten_under_the_snapshot(),
        FIXED_WRITE_TIME_SECONDS + 60,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("快照下覆盖写");
    pool.output = overwritten.clone();
    overwritten
}

/// 删快照：意图那次发布，再按产品批大小走空销毁中的快照，池接到最后一版上；交回意图那一版与之后的每一批。
fn destroy_the_snapshot_and_advance(
    pool: &mut BuiltPool,
    snapshot: TreeIdentifier,
) -> Vec<TransactionOutput> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let intent = publish_snapshot_destruction_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        snapshot,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("删快照的意图");
    let mut current = Arc::new(intent.clone());
    let walk = walk_the_deletion_queue_until_it_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    );
    walk.stopped_by.expect("销毁中的快照走空走到底");
    pool.output = Arc::unwrap_or_clone(current);
    std::iter::once(intent)
        .chain(walk.steps_published.into_iter().map(Arc::unwrap_or_clone))
        .collect()
}

/// 快照、deadlist 与删快照的流：新池新建文件（txg 3）、建快照（txg 4：树表多一对码 14 / 15 条目）、快照下覆盖写（txg 5：换下的
/// 数据单元、extent 树、inode 叶容器与根进 deadlist、映射条目留存）、删快照的意图（码 14 条目 flags 位 0）与之后走空的每一批
/// （最后一批删掉那一对条目）。每一版都带文件：建快照那一版是第一个文件的内容，覆盖写那一版起是覆盖写的内容。
///
/// # Panics
/// 覆盖写那一版的 deadlist 是空的、或删到最后那一版的树表里还有码 14 条目：那这条流就没走到要罩的那几种形态。
fn record_new_pool_through_a_snapshot_its_deadlist_and_its_destruction(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![published_version_of(&pool.output, file_content())];
    let created = create_a_snapshot_and_advance(&mut pool);
    versions.push(published_version_of(&created, file_content()));
    let snapshot = the_only_snapshot_of(&created);
    let overwritten = overwrite_under_the_snapshot_and_advance(&mut pool);
    assert!(
        !overwritten.deadlist_entries.is_empty(),
        "快照下覆盖写换下的用户可见单元进 deadlist"
    );
    versions.push(published_version_of(
        &overwritten,
        content_overwritten_under_the_snapshot(),
    ));
    for destroyed in destroy_the_snapshot_and_advance(&mut pool, snapshot) {
        versions.push(published_version_of(
            &destroyed,
            content_overwritten_under_the_snapshot(),
        ));
    }
    assert!(
        pool.output
            .tree_table_entries
            .iter()
            .all(|entry| entry.kind != TREE_KIND_SNAPSHOT_INODE_ROOT),
        "删到最后那一版的树表里没有码 14 条目"
    );
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 快照、deadlist 与删快照那条流的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位与判到位、两遍恢复落到的根与结局）：
/// 走读内核按快照树走（码 14 / 15 条目的树、I-9.16 ③ 成对、I-9.27、I-9.25 跳过销毁中的快照），引用与快照那一截判 I-3.5 ③ ④ 与逆向、
/// I-3.6、I-3.7（判决 m4-snapshot-r1 第三节 S5，D13（验证路线） 已定项 15）。CPU 池级 checker 在这条流上判过 I-3.5、I-3.6、I-3.7 而成立。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_snapshot_its_deadlist_and_its_destruction()
{
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_snapshot_its_deadlist_and_its_destruction";
    let flow = record_new_pool_through_a_snapshot_its_deadlist_and_its_destruction(
        "crash-judge-gpu-snapshot-deadlist-destruction",
        "snapshot-deadlist-and-destruction",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-3.5", "I-3.6", "I-3.7"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "快照、deadlist 与删快照的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
    }
}

/// 同一个进程里接着 `pool.output` 发整理的一批（搬 `planned` 里那几个单元），池接到它那一版上，交回那一版。
fn relocate_and_advance(pool: &mut BuiltPool, planned: &[PlannedRelocation]) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let relocated = publish_relocation_batch(
        &mut writer,
        &mut pool.allocator,
        &previous,
        planned,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("整理的一批");
    pool.output = relocated.clone();
    relocated
}

/// 搬 `version` 里 `identity` 这个角色的那一项计划：映射 key 与它按映射位置的源落点。
///
/// # Panics
/// 这个角色不进映射、或在这一版的映射里查不到。
fn planned_relocation_of_the_role(
    version: &TransactionOutput,
    identity: TransactionUnit,
) -> PlannedRelocation {
    let (_, mapping_key) = version
        .mapped_units
        .iter()
        .find(|(role, _)| *role == identity)
        .expect("这个角色进映射");
    let locations = match central_mapping_locations_in_the_version(version, mapping_key) {
        MappingLookup::Found(locations) => locations,
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => {
            panic!("{identity:?} 在这一版的映射里查得到")
        }
    };
    PlannedRelocation {
        mapping_key: mapping_key.clone(),
        source: Placement {
            slot: locations[0].slot,
            span: identity.span_slots(),
        },
    }
}

/// 搬迁的流：新池新建文件（txg 3）之后整理的一批搬第一个文件唯一的数据单元（txg 4：原样字节写到目的地、映射条目换到目的地、
/// 源落点进 defer；extent 树里那条指针的位置条目不动，成了过期的提示）。两版都是第一个文件的内容。
fn record_new_pool_with_the_data_unit_relocated(tag: &str, name: &'static str) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let created = pool.output.clone();
    let planned =
        planned_relocation_of_the_role(&created, TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    let batch = relocate_and_advance(&mut pool, std::slice::from_ref(&planned));
    assert_eq!(
        batch.units_relocated_by_this_publish.len(),
        1,
        "这一批搬了一个单元"
    );
    recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            published_version_of(&created, file_content()),
            published_version_of(&batch, file_content()),
        ],
    )
}

/// 搬迁的流（数据单元搬走、提示过期）每个状态上 GPU 与 CPU 整份相同：两个检查内核把码 1 / 2 / 3 的位置条目当提示——先按提示读、
/// 两份都对不上再按映射位置读，引用记映射位置（判决 m4-gc-r1 第三节 G3）；恢复内核与 core 的冷走读同样经映射回退。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_data_unit_relocation() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_data_unit_relocation";
    let flow = record_new_pool_with_the_data_unit_relocated(
        "crash-judge-gpu-data-unit-relocation",
        "data-unit-relocation",
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(TEST_NAME, &flow);
}

/// 快照钉住的单元被搬的流：新池新建文件（txg 3）、建快照（txg 4）、快照下覆盖写（txg 5，旧数据单元只被快照引用、映射条目留存）、
/// 整理的一批搬那个旧数据单元（txg 6：留存的映射条目换到目的地，快照树里的提示过期）。覆盖写那一版起是覆盖写的内容。
fn record_new_pool_with_a_unit_kept_for_the_snapshot_relocated(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![published_version_of(&pool.output, file_content())];
    let created = create_a_snapshot_and_advance(&mut pool);
    versions.push(published_version_of(&created, file_content()));
    let overwritten = overwrite_under_the_snapshot_and_advance(&mut pool);
    versions.push(published_version_of(
        &overwritten,
        content_overwritten_under_the_snapshot(),
    ));
    let (kept_key, kept_locations) = overwritten
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(key, _)| key.first() == Some(&UNIT_CLASS_DATA))
        .cloned()
        .expect("旧数据单元的映射条目留存着");
    let planned = PlannedRelocation {
        mapping_key: kept_key,
        source: Placement {
            slot: kept_locations[0].slot,
            span: TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots(),
        },
    };
    let batch = relocate_and_advance(&mut pool, std::slice::from_ref(&planned));
    assert_eq!(
        batch
            .units_relocated_by_this_publish
            .iter()
            .map(|relocated| relocated.holder)
            .collect::<Vec<_>>(),
        vec![RelocatedUnitHolder::KeptForTheSnapshots],
        "搬的是只被快照引用的那一个"
    );
    versions.push(published_version_of(
        &batch,
        content_overwritten_under_the_snapshot(),
    ));
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 快照钉住的单元被搬的流每个状态上 GPU 与 CPU 整份相同：走读内核走快照树时同样先按提示读、再按映射位置读，
/// 引用与快照那一截按映射位置认单元（判决 m4-gc-r1 第三节 G3「被快照引用的块照样能搬」）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_relocation_of_a_unit_kept_for_the_snapshot(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_relocation_of_a_unit_kept_for_the_snapshot";
    let flow = record_new_pool_with_a_unit_kept_for_the_snapshot_relocated(
        "crash-judge-gpu-relocation-of-a-unit-kept-for-the-snapshot",
        "relocation-of-a-unit-kept-for-the-snapshot",
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(TEST_NAME, &flow);
}

/// discard 的流：新池新建文件的流尾另起一段，放一次盘 0 上罩住第一个文件数据单元那一份的整段 discard（64 槽对齐、64 槽长，
/// 种类 `StepKind::Discard`，写表里按「范围写零」记）。这一段取 D13（验证路线） 已定项 4 的三态：没落、整段是 0、撕裂。
fn record_new_pool_with_the_segment_of_the_data_unit_discarded_on_device_zero(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-segment-of-the-data-unit-discarded-on-device-zero",
    );
    let data_unit_slot = flow
        .writes
        .iter()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device == DeviceIdentity(0)
                && write
                    .bytes()
                    .is_some_and(|bytes| unit_class(bytes) == UNIT_CLASS_DATA)
        })
        .expect("新池新建文件在盘 0 上写一份数据单元")
        .offset
        .0
        / SLOT_BYTES;
    let segment_start = data_unit_slot - data_unit_slot % singlefs_format::CLUSTER_SEGMENT_SLOTS;
    append_a_segment_of_writes(
        &mut flow,
        vec![RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::Discard,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(segment_start * SLOT_BYTES),
            contents: WrittenContents::Zeros {
                length: singlefs_format::CLUSTER_SEGMENT_SLOTS * SLOT_BYTES,
            },
        }],
    );
    flow
}

/// discard 的流建得出判表（第一版不再拒 `StepKind::Discard`）、写表里那一次 discard 不占落点，每个状态上 GPU 与 CPU 整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_discard_over_the_segment_of_a_data_unit() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_discard_over_the_segment_of_a_data_unit";
    let flow = record_new_pool_with_the_segment_of_the_data_unit_discarded_on_device_zero(
        "crash-judge-gpu-discard-over-a-data-unit",
    );
    let tables = judge_tables_of_the_flow(&flow);
    let discard = tables.writes.last().expect("写表里至少有那一次 discard");
    assert_eq!(discard.kind, StepKind::Discard, "流尾那一次写是 discard");
    assert_eq!(discard.location, None, "discard 罩多个落点，不占一个落点");
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(TEST_NAME, &flow);
}

// ───────── 里程碑四自建 GC 新判的那几条上 GPU（判决 m4-gc-r1 第三节 G1、G3 的 N2–N6）─────────

/// 同一个进程里接着 `pool.output` 发整理的一次发布（搬 `planned` 里那几个单元、对意图做 `intent_step`），池接到它那一版上，交回那一版。
fn compaction_step_and_advance(
    pool: &mut BuiltPool,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let published = publish_compaction_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        planned,
        intent_step,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("整理的一次发布");
    pool.output = published.clone();
    published
}

/// 整理意图的流：新池新建文件（txg 3）、签发整理意图（txg 4：logged_ops 写一条，R 是第一个文件数据单元所在的那一段，c0 取新建文件那一版的 txg；
/// 分配器这一侧同时装上 R 排他）、整理的一批搬那个数据单元（txg 5：意图照抄）、丢意图（txg 6：logged_ops 删掉那一条，树表条目根指针回到全零）。
/// 四版都是第一个文件的内容。
fn record_new_pool_through_a_compaction_intent_a_relocation_and_its_drop(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let created = pool.output.clone();
    let data_unit_slot = created
        .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .slot;
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(data_unit_slot.0 - data_unit_slot.0 % singlefs_format::CLUSTER_SEGMENT_SLOTS),
    )
    .expect("段起点是 64 的倍数、装得进 6 字节");
    let intent = CompactionIntent {
        region,
        allocation_generation_at_issue: created.root.checkpoint_txg,
        empty_cluster_segments_at_issue: 0,
    };
    pool.allocator.compaction_mut().issue(intent);
    let issued = compaction_step_and_advance(&mut pool, &[], CompactionIntentStep::Issue(intent));
    assert_eq!(
        issued.logged_operations.len(),
        1,
        "签发那一版的 logged_ops 里一条意图"
    );
    let planned =
        planned_relocation_of_the_role(&issued, TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    let batch = compaction_step_and_advance(
        &mut pool,
        std::slice::from_ref(&planned),
        CompactionIntentStep::CarryTheIntent,
    );
    assert_eq!(
        batch.units_relocated_by_this_publish.len(),
        1,
        "这一批搬了一个单元"
    );
    let dropped = compaction_step_and_advance(&mut pool, &[], CompactionIntentStep::Drop);
    pool.allocator.compaction_mut().withdraw_the_live_intent();
    assert!(
        dropped.logged_operations.is_empty(),
        "丢意图那一版的 logged_ops 空了"
    );
    let versions = [&created, &issued, &batch, &dropped]
        .into_iter()
        .map(|version| published_version_of(version, file_content()))
        .collect();
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 整理意图那条流的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位与判到位、两遍恢复落到的根与结局）：走读内核与引用集合那一遍
/// 把 logged_ops 树当多层码 2 树走、节点记进引用，恢复内核把 logged_ops 根数进映射条目数，扫描内核判 I-3.16，引用与快照那一截之后判 I-4.4、I-5.6、
/// I-3.15、I-1.14（D13（验证路线） 已定项 15）。CPU 池级 checker 在这条流上判过那五条而成立。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_compaction_intent_a_relocation_and_its_drop(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_compaction_intent_a_relocation_and_its_drop";
    let flow = record_new_pool_through_a_compaction_intent_a_relocation_and_its_drop(
        "crash-judge-gpu-compaction-intent-relocation-drop",
        "compaction-intent-relocation-and-drop",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-4.4", "I-5.6", "I-3.15", "I-3.16", "I-1.14"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "整理意图那条流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
    }
}

/// 流里每一次命中 `select` 的单元写的下标（至少一个单元的两份副本）。
fn every_unit_write_where(
    flow: &RecordedFlow,
    what: &str,
    select: &dyn Fn(&[u8]) -> bool,
) -> Vec<usize> {
    let matching: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::UnitWrite && write.bytes().is_some_and(select))
        .map(|(index, _)| index)
        .collect();
    assert!(
        matching.len() >= 2,
        "流 {} 里命中「{what}」的单元写至少两条（一个单元的两份副本），只有 {} 条",
        flow.name,
        matching.len()
    );
    matching
}

/// 码 2 节点的条目从头的末尾再过 29 字节起（同 `only_tree_table_entry_of_kind` 的 131 = 86 + 2 × 8 + 29）。
pub(super) fn first_index_node_entry_offset(bytes: &[u8]) -> usize {
    86 + 2 * usize::from(index_node_key_width(bytes)) + 29
}

/// logged_ops 条目里 value 的起点（key 10 之后）与 value 里 c0、补零两段的偏移（判决 G1 字段表；walk.rs 的
/// `COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE`、`COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE`）。
pub(super) const LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY: usize = 10;

const COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE: usize = 12;

const COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE: usize = 28;

/// 整理意图那条流上改坏 logged_ops 根（码 2、key 宽 10、条目宽 44）的每一份——签发那次写的与之后照抄时写的都改——里第一条条目，重封到根。
fn record_the_compaction_intent_flow_with_its_logged_operation_changed(
    tag: &str,
    name: &'static str,
    mutate_the_first_entry: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_a_compaction_intent_a_relocation_and_its_drop(tag, name);
    let logged_operations_root = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 10
            && index_node_entry_width(bytes) == 44
    };
    let targets = every_unit_write_where(&flow, "logged_ops 根", &logged_operations_root);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let entry = first_index_node_entry_offset(bytes);
        mutate_the_first_entry(&mut bytes[entry..entry + 44]);
    });
    flow
}

/// I-4.4（意图只描述目标态，N4）：整理意图 value 末尾的补零第一个字节写 1（「再搬 N 个」这样的增量只能落在补零里），每个状态 GPU 与 CPU 整份相同、
/// CPU 红 I-4.4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_compaction_intent_carries_an_increment_in_its_padding()
{
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_compaction_intent_carries_an_increment_in_its_padding";
    let flow = record_the_compaction_intent_flow_with_its_logged_operation_changed(
        "crash-judge-gpu-compaction-intent-padding",
        "compaction-intent-with-an-increment-in-its-padding",
        &|entry| {
            entry[LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY
                + COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE] = 1;
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-4.4");
}

/// I-5.6（活整理意图的区域里没有签发之后分配的记录，N3）：整理意图的 c0 改成 0，R 里新建文件那一版分配的数据单元的记录分配代大于它，
/// 每个状态 GPU 与 CPU 整份相同、CPU 红 I-5.6。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_region_of_a_compaction_intent_holds_records_allocated_after_its_c0(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_region_of_a_compaction_intent_holds_records_allocated_after_its_c0";
    let flow = record_the_compaction_intent_flow_with_its_logged_operation_changed(
        "crash-judge-gpu-compaction-intent-c0-zero",
        "compaction-intent-with-c0-zero",
        &|entry| {
            let at = LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY
                + COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE;
            entry[at..at + 8].copy_from_slice(&0u64.to_le_bytes());
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-5.6");
}

/// 没人分配也没写副本的一个槽（新池新建文件之后，同 `checker_compaction_intent_and_relocation_must_red_worlds.rs` 的 `SLOT_NOBODY_ALLOCATED`）。
const SLOT_NOBODY_ALLOCATED: u64 = 50_400;

/// 中央映射叶条目 55 字节里两条位置条目的槽号（6 字节）在条目里的偏移：key 27 之后，各条 盘 4 + 槽 6 + 整单元校验和 4。
const MAPPING_ENTRY_LOCATION_SLOT_OFFSETS: [usize; 2] = [27 + 4, 27 + 14 + 4];

/// I-3.15（N2 反查）「只改映射不写副本」：新池新建文件之后，中央映射叶里数据单元那一条的两条位置条目改指一个没人分配、也没写副本的槽，
/// 重封到根。源槽那条未释放记录罩住的数据单元在映射里那把相符的 key 落在别处：每个状态 GPU 与 CPU 整份相同、CPU 红 I-3.15。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_mapping_entry_moved_without_a_copy() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_mapping_entry_moved_without_a_copy";
    let mut flow = record_new_pool_file_creation(
        "crash-judge-gpu-mapping-entry-moved-without-a-copy",
        "new-pool-mapping-entry-moved-without-a-copy",
    );
    let data_unit_slot = flow
        .writes
        .iter()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device == DeviceIdentity(0)
                && write.bytes().is_some_and(|bytes| unit_class(bytes) == 1)
        })
        .expect("新池新建文件在盘 0 上写一份数据单元")
        .offset
        .0
        / singlefs_format::SLOT_BYTES;
    let mapping_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 27
            && index_node_entry_width(bytes) == 55
    };
    let targets = last_unit_copies(&flow, "中央映射叶", &mapping_leaf, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let header_end = 86 + 2 * usize::from(index_node_key_width(bytes));
        let count = usize::from(u16::from_le_bytes(
            bytes[header_end - 4..header_end - 2].try_into().expect("2"),
        ));
        let entry = (0..count)
            .map(|position| first_index_node_entry_offset(bytes) + position * 55)
            .find(|entry| {
                let at = entry + MAPPING_ENTRY_LOCATION_SLOT_OFFSETS[0];
                let mut slot = [0u8; 8];
                slot[..6].copy_from_slice(&bytes[at..at + 6]);
                u64::from_le_bytes(slot) == data_unit_slot
            })
            .expect("映射叶里有数据单元那一条");
        for offset in MAPPING_ENTRY_LOCATION_SLOT_OFFSETS {
            bytes[entry + offset..entry + offset + 6]
                .copy_from_slice(&SLOT_NOBODY_ALLOCATED.to_le_bytes()[..6]);
        }
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-3.15");
}

/// runs 那一统计量的标签（D5（快照 / 空间记账机制） 已定项 4 第 10 项；walk.rs 的 `STATISTIC_FRAGMENTATION_RUNS`）。记账行里设备、值两段的偏移
/// 用这份文件里记账那一节的 `ACCOUNTING_ROW_DEVICE_OFFSET`、`ACCOUNTING_ROW_VALUE_OFFSET`。
const STATISTIC_FRAGMENTATION_RUNS: u16 = 10;

/// I-3.16（N5 = C413）「增量少加一次」：新池新建文件那一版记账叶（根兼叶）里盘 0 的 runs 那一行减一，重封到根。全扫重算的 runs 与它对不上：
/// 每个状态 GPU 与 CPU 整份相同、CPU 红 I-3.16。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_runs_row_of_a_device_is_one_short() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_runs_row_of_a_device_is_one_short";
    let accounting_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 22
            && index_node_entry_width(bytes) == 34
    };
    let flow = record_new_pool_with_a_resealed_field(
        "crash-judge-gpu-runs-row-one-short",
        "new-pool-runs-row-of-device-zero-one-short",
        &accounting_leaf,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(index_node_key_width(bytes));
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            let row = (0..count)
                .map(|position| first_index_node_entry_offset(bytes) + position * 34)
                .find(|row| {
                    u16::from_le_bytes(bytes[*row..*row + 2].try_into().expect("2"))
                        == STATISTIC_FRAGMENTATION_RUNS
                        && u32::from_le_bytes(
                            bytes[row + ACCOUNTING_ROW_DEVICE_OFFSET
                                ..row + ACCOUNTING_ROW_DEVICE_OFFSET + 4]
                                .try_into()
                                .expect("4"),
                        ) == 0
                })
                .expect("记账叶里有盘 0 的 runs 那一行");
            let value_at = row + ACCOUNTING_ROW_VALUE_OFFSET;
            let runs = u64::from_le_bytes(bytes[value_at..value_at + 8].try_into().expect("8"));
            bytes[value_at..value_at + 8].copy_from_slice(
                &runs
                    .checked_sub(1)
                    .expect("新池新建文件那一版盘 0 的 runs 至少 1")
                    .to_le_bytes(),
            );
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-3.16");
}

/// 数据单元载荷起点（码 1 明文头 105 字节之后）往后第 100 个字节：改它不碰头里的任何字段。
const A_PAYLOAD_BYTE_OF_THE_DATA_UNIT: usize = 105 + 100;

/// I-1.14（N6）：搬迁的流上搬到目的地的那一份数据单元（流里最后一次码 1 单元写，两份副本）载荷改一个字节、重封，映射条目的校验和跟着补。
/// 数据单元那把 key 在新建文件那一版指源槽那一份、在这一批指目的槽那一份，两份字节不同：每个状态 GPU 与 CPU 整份相同、CPU 红 I-1.14。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_byte_changed_in_the_relocated_copy() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_byte_changed_in_the_relocated_copy";
    let mut flow = record_new_pool_with_the_data_unit_relocated(
        "crash-judge-gpu-relocated-copy-changed",
        "relocated-copy-with-a-byte-changed",
    );
    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    let targets = last_unit_copies(&flow, "搬到目的地的那份数据单元", &data_unit, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        bytes[A_PAYLOAD_BYTE_OF_THE_DATA_UNIT] ^= 0xff;
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-1.14");
}
