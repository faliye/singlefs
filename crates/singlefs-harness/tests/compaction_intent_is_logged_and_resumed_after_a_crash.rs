//! 整理意图上盘与可写挂载续做（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3、G4；D26（后台整理与放置回收） 已定项 2；
//! D8（核心索引结构） 已定项 5 三态表）：签发那次发布把意图条目（字段表 44 字节）写进 logged_ops 树，可写挂载重建时读回活意图、恢复 R 排他，
//! 照「删除续做 → 抬 F → 整理」的次序续做到最后一批同发布删意图；意图条目的 R 不对齐，可写挂载在任何写之前拒。
//!
//! 「崩溃」取在整理的发布边界上：会话的整理一趟按发布预算（`B_gc` = 3S − 4 − 已经过去的发布数，S = 8）只发到第 k 次就停，丢掉会话不卸载、
//! 重开镜像再可写挂载——第 k 次发布已整个落盘、之后的一个都没发，就是第 k 次发布之后、下一次之前崩溃的盘面。
//! 池按 `common::build_pool` 起：第一个文件那次发布的提交内生块在段 50240（[`SEGMENT_OF_THE_FIRST_PUBLISH`]），强制选它当 R，一批一个。
mod common;
mod common_pointer_head_corruption;

use common::{build_pool, file_content, parameters, BuiltPool};
use common_pointer_head_corruption::{
    node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at, writable_mount_refused_before_any_write,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::{
    region_is_empty_by_the_records, CompactionBatchSize, CompactionBatchUnits,
    CompactionRegionExclusion, CompactionRunForcing, CompactionRunStop, CompactionSwitches,
    EmptyClusterSegmentsFloor, ForcedReleaseChecksumMismatchAfterIssue,
    RelocationDestinationPolicy, RelocationPublishOrder,
};
use singlefs_core::logged_operations::{
    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion, LoggedOperation,
    LoggedOperationEntry, LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES,
};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_compaction_switches, MountError, Mounted,
};
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession};
use singlefs_core::records::TREE_KIND_LOGGED_OPERATIONS;
use singlefs_core::recovery::{
    walk_to_file, RecoveryFailure, LOGGED_OPERATIONS_ROOT_ENTRIES_REFUSED_BY_THE_FIELD_TABLE,
};
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::{index_node_header_bytes, parse_index_node, seal_header_checksum};

/// 第一个文件那次发布的提交内生块落在这一段（默认几何，D3（空间分配） 已定项 10 ⑤ 的开放段 [50240, 50304)）。
const SEGMENT_OF_THE_FIRST_PUBLISH: u64 = 50240;
/// 一次用户改动之后整理一趟的发布预算（判决 G8 `B_gc`）：3 × 8 − 4，S = 8 是 mkfs 写的每区根槽数。
const PUBLISH_BUDGET_RIGHT_AFTER_A_USER_CHANGE: u64 = 3 * 8 - 4;
/// 整理意图条目里 R 的起始槽在条目里的偏移（判决 G1 字段表：key 10 之后设备 4）。
const COMPACTION_INTENT_REGION_START_SLOT_OFFSET_IN_THE_ENTRY: usize = 14;

fn region_of_the_first_publish() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(SEGMENT_OF_THE_FIRST_PUBLISH),
    )
    .expect("64 的倍数")
}

/// 强制选段 50240、一批一个，别的字段取产品值。
fn switches_moving_one_unit_per_batch_out_of_the_region_of_the_first_publish() -> CompactionSwitches
{
    CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(1).expect("至少 1"),
        ),
        forced_region: Some(region_of_the_first_publish()),
        run_forcing: CompactionRunForcing::FollowTheWatermarks,
        region_exclusion: CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
        destination_policy:
            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
        release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue::NotForced,
        empty_cluster_segments_floor: EmptyClusterSegmentsFloor::OfTheFirstVersion,
        relocation_publish_order:
            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
    }
}

fn current_of(session: &MountedSession) -> &TransactionOutput {
    session.current.version_of_trees().expect("有树的一版")
}

/// R 那块盘上起点在 R 里、未释放的记录条数：一批一个时，这就是 R 里还要搬几批。
fn unreleased_records_in_the_region(version: &TransactionOutput) -> usize {
    let region = region_of_the_first_publish();
    version
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && region.contains_slot(record.slot)
        })
        .count()
}

/// 在可写挂载开的会话上跑整理一趟，预算压到 `publishes` 次发布：停在「预算用完」，交回这一趟发成的每一版。
fn run_stopping_after(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, common::Recorded)>,
    publishes: u64,
) -> Vec<TransactionOutput> {
    session.compaction.publishes_since_the_last_user_change =
        PUBLISH_BUDGET_RIGHT_AFTER_A_USER_CHANGE - publishes;
    let run = session.run_compaction(devices).expect("整理一趟");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget })
                if budget == publishes
        ),
        "预算压到 {publishes} 次发布，停在预算用完：{:?}",
        run.outcome
    );
    assert_eq!(
        u64::try_from(run.versions_published.len()).expect("发布数"),
        publishes,
        "发了 {publishes} 次"
    );
    run.versions_published
}

/// 重开、可写挂载（整理开关同崩溃之前那一趟），开会话：交回挂载的结果（续做在它里面）。
fn mount_after_the_crash(pool: &mut BuiltPool) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_compaction_switches(
        &parameters(),
        &mut devices,
        switches_moving_one_unit_per_batch_out_of_the_region_of_the_first_publish(),
    )
    .expect("崩溃之后可写挂载");
    pool.devices = Some(devices);
    mounted
}

/// 一个池：可写挂载、跑整理一趟只发到第 `publishes` 次就丢掉会话（崩溃）。交回那时活着的意图（签发那次发布的 txg 与意图）与崩溃之前 R 里还剩几条未释放的记录。
fn pool_crashed_after_the_compaction_publish(
    tag: &str,
    publishes: u64,
) -> (BuiltPool, (CheckpointTxg, CompactionIntent), usize) {
    let mut pool = build_pool(tag);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session.allocator.set_compaction_switches(
        switches_moving_one_unit_per_batch_out_of_the_region_of_the_first_publish(),
    );
    let versions = run_stopping_after(&mut session, &mut devices, publishes);
    let last = versions.last().expect("至少一次发布");
    let live = live_compaction_intent_of_the_entries(&last.logged_operations)
        .expect("崩溃那一刻意图还在 logged_ops 里");
    let records_left = unreleased_records_in_the_region(last);
    pool.devices = Some(devices);
    (pool, live, records_left)
}

/// 崩溃之后的可写挂载续做到底：续做那一趟停在「没有活意图可续」（意图删了）、搬了 `records_left` 批（一批一个）、最后那一版 logged_ops 空、
/// R 按记录全空、分配器这一侧撤了 R 排他；第一个文件读得回；会话接着用挂载那一份观测（一条意图完成）。
fn assert_the_mount_continued_the_intent_to_its_end(
    pool: &BuiltPool,
    mounted: Mounted,
    records_left: usize,
) {
    let run = mounted
        .output
        .compaction_continuation
        .run
        .as_ref()
        .expect("盘上有一条未完成的意图：挂载续做它");
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::NoLiveIntentLeftToContinue)
        ),
        "续做到意图删了：{:?}",
        run.outcome
    );
    assert_eq!(
        run.batches_published,
        u64::try_from(records_left).expect("条数"),
        "一批一个：R 里剩几条就搬几批"
    );
    assert!(
        run.versions_published.last().is_some_and(|version| !version
            .units_relocated_by_this_publish
            .is_empty()
            && version.logged_operations.is_empty()),
        "最后一批同一次发布删意图"
    );
    let batches_published_by_the_continuation = run.batches_published;
    let (_output, session) = MountedSession::of_the_mount(mounted);
    let current = current_of(&session);
    assert!(
        current.logged_operations.is_empty(),
        "续做之后 logged_ops 空了"
    );
    assert!(
        region_is_empty_by_the_records(&current.allocation_records, region_of_the_first_publish()),
        "R 按记录全空（停机谓词，判决 G4）"
    );
    assert_eq!(
        session.allocator.region_excluded_from_placement(),
        None,
        "R 排他跟着撤了"
    );
    assert_eq!(
        session.compaction.observations.intents_completed, 1,
        "会话接着用挂载续做那一份观测：一条意图完成"
    );
    assert_eq!(
        session.compaction.observations.batches_published,
        batches_published_by_the_continuation
    );
    let mut mapping_fallbacks = 0;
    assert_eq!(
        walk_to_file(&pool.memory_pool(), &current.root, &mut mapping_fallbacks),
        Ok(Some(file_content())),
        "第一个文件读得回"
    );
}

/// 签发那次发布把一条整理意图写进 logged_ops 树（判决 G1）：盘上树表里 logged_ops 那一条的根指针不再是零，根是码 2 层级 0、
/// 一条 44 字节的条目，字节就是字段表写出的那样；不续做的可写挂载（产品开关：批大小条款没写，续做在任何写之前停下）重建时读回同一条意图，
/// 分配器上恢复 R 排他（D26（后台整理与放置回收） 已定项 1 第 3 条）。
#[test]
fn the_issue_publish_logs_the_compaction_intent_and_a_writable_mount_reads_it_back_with_its_region_excluded(
) {
    let (mut pool, (issued_txg, intent), _records_left) =
        pool_crashed_after_the_compaction_publish("compaction-intent-logged-by-the-issue", 1);
    assert_eq!(
        intent.region,
        region_of_the_first_publish(),
        "签的是强制选的那一段"
    );
    let image = pool.memory_pool();
    let logged_operations_root = tree_table_entry_of_kind(&image, TREE_KIND_LOGGED_OPERATIONS).root;
    let node = parse_index_node(&unit_bytes_at(
        &image,
        the_slot_of(&logged_operations_root),
        node_bytes(),
    ))
    .expect("logged_ops 根解得开");
    assert_eq!(node.level, 0, "根兼叶");
    assert_eq!(
        node.entries,
        vec![LoggedOperationEntry {
            issued_txg,
            operation: LoggedOperation::Compaction(intent),
        }
        .to_bytes()],
        "盘上一条条目，字节照字段表"
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    assert!(
        matches!(
            mounted
                .output
                .compaction_continuation
                .run
                .as_ref()
                .map(|run| &run.outcome),
            Some(CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchSizeNotDecidedByTheClauses
            ))
        ),
        "产品开关下续做在任何写之前停下：{:?}",
        mounted.output.compaction_continuation.run
    );
    let (_, session) = MountedSession::of_the_mount(mounted);
    assert_eq!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations),
        Some((issued_txg, intent)),
        "重建时读回同一条活意图"
    );
    assert_eq!(
        session.allocator.region_excluded_from_placement(),
        Some(intent.region),
        "重建时恢复 R 排他"
    );
}

/// 签发之后、第一批之前崩溃（D8（核心索引结构） 已定项 5 三态表第二行「意图还在、续做」）：可写挂载续做到 R 按记录全空、同发布删意图。
#[test]
fn a_crash_right_after_the_issue_publish_is_continued_by_the_writable_mount_to_the_end() {
    let (mut pool, _live, records_left) =
        pool_crashed_after_the_compaction_publish("compaction-intent-crash-after-the-issue", 1);
    assert!(records_left >= 2, "R 里不止一个单元：续做要搬几批");
    let mounted = mount_after_the_crash(&mut pool);
    assert_the_mount_continued_the_intent_to_its_end(&pool, mounted, records_left);
}

/// 两批之间崩溃（签发、搬了一批之后）：可写挂载接着搬剩下的，续做到底。
#[test]
fn a_crash_between_two_batches_is_continued_by_the_writable_mount_to_the_end() {
    let (mut pool, _live, records_left) =
        pool_crashed_after_the_compaction_publish("compaction-intent-crash-between-batches", 2);
    assert!(records_left >= 1, "第一批之后 R 里还有要搬的");
    let mounted = mount_after_the_crash(&mut pool);
    assert_the_mount_continued_the_intent_to_its_end(&pool, mounted, records_left);
}

/// 最后一批之前崩溃：先签发，看 R 里有几个单元（n），再在同一个会话上只发 n − 1 批就崩溃——R 里剩一条，意图还在；可写挂载发最后那一批、同发布删意图。
#[test]
fn a_crash_right_before_the_last_batch_is_continued_by_the_writable_mount_to_the_end() {
    let mut pool = build_pool("compaction-intent-crash-before-the-last-batch");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session.allocator.set_compaction_switches(
        switches_moving_one_unit_per_batch_out_of_the_region_of_the_first_publish(),
    );
    run_stopping_after(&mut session, &mut devices, 1);
    let units_in_the_region = unreleased_records_in_the_region(current_of(&session));
    assert!(units_in_the_region >= 2, "R 里不止一个单元");
    run_stopping_after(
        &mut session,
        &mut devices,
        u64::try_from(units_in_the_region - 1).expect("批数"),
    );
    assert_eq!(
        unreleased_records_in_the_region(current_of(&session)),
        1,
        "崩溃那一刻 R 里只剩最后一条"
    );
    assert!(
        live_compaction_intent_of_the_entries(&current_of(&session).logged_operations).is_some(),
        "意图还在"
    );
    pool.devices = Some(devices);
    drop(session);
    let mounted_after_the_crash = mount_after_the_crash(&mut pool);
    assert_the_mount_continued_the_intent_to_its_end(&pool, mounted_after_the_crash, 1);
}

/// 码 2 节点重封：先算载荷 CRC，再算头校验和（码 2：头 86 + 2k、CRC 在 76 + 2k，k 是 key 宽，D18（块里携带什么信息） 已定项 18 的偏移表）。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_crc_offset = 76 + 2 * key_width;
    let payload_crc = crc32_castagnoli(&node[header_end..]);
    node[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 意图条目改坏（判决 G1「R 的解码拒不对齐的值」）：签发之后把盘上那一条的 R 起始槽改成 50241（不是 64 的倍数）、重封节点、新的整单元校验和
/// 沿引用链补到根槽：可写挂载重建那一版时拒成「logged_ops 根里有条目过不了字段表」，在任何写之前（盘上逐字节不变）。
#[test]
fn a_logged_compaction_intent_whose_region_is_not_aligned_is_refused_by_the_writable_mount() {
    let (pool, _live, _records_left) =
        pool_crashed_after_the_compaction_publish("compaction-intent-region-not-aligned", 1);
    let mut image = pool.memory_pool();
    let root_slot =
        the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_LOGGED_OPERATIONS).root);
    let mut node = unit_bytes_at(&image, root_slot, node_bytes());
    let start_slot_offset = index_node_header_bytes(LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES)
        + COMPACTION_INTENT_REGION_START_SLOT_OFFSET_IN_THE_ENTRY;
    assert_eq!(
        node[start_slot_offset..start_slot_offset + 6],
        SEGMENT_OF_THE_FIRST_PUBLISH.to_le_bytes()[..6],
        "改之前是段起点 50240"
    );
    node[start_slot_offset..start_slot_offset + 6]
        .copy_from_slice(&(SEGMENT_OF_THE_FIRST_PUBLISH + 1).to_le_bytes()[..6]);
    reseal_index_node(&mut node);
    replace_the_unit_and_propagate_its_checksum(&mut image, root_slot, &node);
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::UnitMalformed { what })
                if what == LOGGED_OPERATIONS_ROOT_ENTRIES_REFUSED_BY_THE_FIELD_TABLE
        ),
        "R 不对齐的意图条目让可写挂载拒：{refusal:?}"
    );
}
