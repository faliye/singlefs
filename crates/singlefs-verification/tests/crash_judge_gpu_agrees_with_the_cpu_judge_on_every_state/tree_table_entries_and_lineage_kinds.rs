//! 树表（I-9 类）：树表条数不超过扫描内核的上限，谱系树容器记录宽、谱系树根的树 ID 与谱系树、logged_ops 两条的发号次序。

use super::*;

// ───────────── 树表 11 条与种类 9–11（删除与并发那条线批 6、批 8 的盘面）：GPU 判器跟上之后各有一条分得出的流 ─────────────

/// 一个树表单元（码 2、key 宽 8、条目宽 200）的条目数（索引节点头里 key 区间之后那两字节）。
pub(super) fn entry_count_of_the_tree_table_read_by_its_key_width(bytes: &[u8]) -> usize {
    let header_end = 86 + 2 * usize::from(index_node_key_width(bytes));
    usize::from(u16::from_le_bytes(
        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
    ))
}

/// 扫描内核一张树表最多记几棵树：池级 checker 共用那一半里 `const C_MAX_TREES_PER_TABLE: u32 = Nu;` 那一行的 N。
fn tree_count_the_gpu_scan_kernel_records_per_tree_table() -> usize {
    const DECLARATION_PREFIX: &str = "const C_MAX_TREES_PER_TABLE: u32 = ";
    let source = singlefs_verification::crash_judge_gpu::GPU_JUDGE_CHECKER_COMMON_SOURCE;
    let declarations: Vec<&str> = source
        .lines()
        .filter_map(|line| line.strip_prefix(DECLARATION_PREFIX))
        .collect();
    assert_eq!(
        declarations.len(),
        1,
        "池级 checker 共用那一半里恰好一行 `{DECLARATION_PREFIX}…`"
    );
    declarations[0]
        .strip_suffix("u;")
        .expect("声明以 `u;` 结尾")
        .parse()
        .expect("上限是十进制数")
}

/// 一条流写下的每一张树表，条目数都不超过 GPU 扫描内核一张树表能记的树数：核心层的树表比扫描内核记的树数长时
/// （新池第一个文件版本的树表 11 条，扫描内核只记 8 棵就从第 9 棵起判不了），这条在取卡之前就红，不等对拍跑到那条流上第一个判不了的状态。
/// 没卡的机器上也判到。
fn assert_every_tree_table_of_the_flow_fits_the_tree_count_the_gpu_scan_kernel_records(
    flow: &RecordedFlow,
) {
    let capacity = tree_count_the_gpu_scan_kernel_records_per_tree_table();
    let is_a_tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    let entry_counts: Vec<usize> = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter_map(|write| write.bytes())
        .filter(|bytes| is_a_tree_table(bytes))
        .map(entry_count_of_the_tree_table_read_by_its_key_width)
        .collect();
    assert!(!entry_counts.is_empty(), "流 {} 写了树表单元", flow.name);
    let largest = entry_counts.iter().copied().max().expect("至少一张");
    assert!(
        largest <= capacity,
        "流 {} 写下的树表最多 {largest} 条，GPU 扫描内核一张树表只记 {capacity} 棵树：先把 C_MAX_TREES_PER_TABLE 跟上（草稿区顺移、SCRATCH_WORDS 重算）",
        flow.name
    );
}

/// 场景：新池新建文件那条流写下的每一张树表。预期：条目数都不超过 GPU 扫描内核一张树表能记的树数（[`assert_every_tree_table_of_the_flow_fits_the_tree_count_the_gpu_scan_kernel_records`]）。
#[test]
fn every_tree_table_the_new_pool_file_creation_writes_fits_the_tree_count_the_gpu_scan_kernel_records(
) {
    assert_every_tree_table_of_the_flow_fits_the_tree_count_the_gpu_scan_kernel_records(
        &record_new_pool_file_creation("crash-judge-gpu-tree-count-new-pool", "new-pool"),
    );
}

/// 场景：固定脚本到回退 D 那条流写下的每一张树表。预期：条目数都不超过 GPU 扫描内核一张树表能记的树数（[`assert_every_tree_table_of_the_flow_fits_the_tree_count_the_gpu_scan_kernel_records`]）。
#[test]
fn every_tree_table_the_fixed_script_through_a_rollback_writes_fits_the_tree_count_the_gpu_scan_kernel_records(
) {
    assert_every_tree_table_of_the_flow_fits_the_tree_count_the_gpu_scan_kernel_records(
        &record_fixed_script_through_rollback("crash-judge-gpu-tree-count-rollback"),
    );
}

/// 第一版能出现的最大树表那条流：新池新建文件（txg 3）之后接连建 `SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`（10）个快照，每一版都是第一个文件的内容。
/// 最后那一版的树表是进发号次序的 12 条加 10 对码 14 / 15 条目，共 32 条。
///
/// # Panics
/// 最后那一版的树表里码 14 条目不是 10 条：那这条流就没走到第一版的上限。
fn record_new_pool_through_the_most_snapshots_of_the_first_version(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![published_version_of(&pool.output, file_content())];
    for _snapshot in 0..SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
        let created = create_a_snapshot_and_advance(&mut pool);
        versions.push(published_version_of(&created, file_content()));
    }
    assert_eq!(
        pool.output
            .tree_table_entries
            .iter()
            .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
            .count(),
        SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION,
        "最后那一版的树表里码 14 条目是第一版的快照上限那么多条"
    );
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 第一版能出现的最大树表（10 个快照：12 + 2 × 10 = 32 条）装得进 GPU 扫描内核一张树表能记的树数，而且恰好装满（上限少一棵这条就红），
/// CPU 参照一侧照这条流建得出判表（`build_judge_tables`）：快照六条流在 GPU 上判不了（站点 28：扫描内核一张树表只记 12 棵）那一次，
/// 上限只按发号次序的 12 种取、没把快照条目算进去。取卡之前就判，没卡的机器上也判到。
#[test]
fn the_tree_table_of_the_most_snapshots_of_the_first_version_fills_the_tree_count_the_gpu_scan_kernel_records_and_its_judge_tables_build(
) {
    const TREE_KINDS_IN_THE_ISSUING_ORDER: usize = 12;
    let capacity = tree_count_the_gpu_scan_kernel_records_per_tree_table();
    let flow = record_new_pool_through_the_most_snapshots_of_the_first_version(
        "crash-judge-gpu-tree-count-most-snapshots",
        "most-snapshots-of-the-first-version",
    );
    let is_a_tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    let largest = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter_map(|write| write.bytes())
        .filter(|bytes| is_a_tree_table(bytes))
        .map(entry_count_of_the_tree_table_read_by_its_key_width)
        .max()
        .expect("这条流写了树表单元");
    assert_eq!(
        largest,
        TREE_KINDS_IN_THE_ISSUING_ORDER + 2 * SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION,
        "流 {} 最大的那张树表是发号次序的 12 条加每个快照一对",
        flow.name
    );
    assert_eq!(
        capacity, largest,
        "GPU 扫描内核一张树表能记的树数（C_MAX_TREES_PER_TABLE）就是第一版能出现的最大树表的条数：少了快照流判不了（站点 28），多了是没有来源的数"
    );
    let expansion =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow);
    assert!(
        tables.is_ok(),
        "流 {} 上 CPU 参照一侧建得出判表：{:?}",
        flow.name,
        tables.err()
    );
}

/// 回退流上回退那次发布 D 往谱系树追加的那一片容器（码 3、记录类型 5）：记录宽从 24 改成 25、重封到根。
/// CPU 池级 checker 走谱系树走到这片容器、按 I-1.7 判它的记录宽不是登记的 24 → 红；恢复不读容器，两遍恢复照常读到文件。
/// GPU 走读内核不走谱系树、或不认谱系记录的登记宽时这一位对不上。
fn record_rollback_with_the_lineage_container_record_width_off_its_registered_width(
    tag: &str,
) -> RecordedFlow {
    const LINEAGE_RECORD_TYPE: u16 = 5;
    const PACKED_RECORD_WIDTH_OFFSET: usize = 71;
    let lineage_container =
        |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == LINEAGE_RECORD_TYPE;
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-container-record-width-seventeen",
        &lineage_container,
        &|bytes| {
            let width = u16::from_le_bytes(
                bytes[PACKED_RECORD_WIDTH_OFFSET..PACKED_RECORD_WIDTH_OFFSET + 2]
                    .try_into()
                    .expect("2"),
            );
            assert_eq!(width, 24, "谱系记录宽 24");
            bytes[PACKED_RECORD_WIDTH_OFFSET..PACKED_RECORD_WIDTH_OFFSET + 2]
                .copy_from_slice(&(width + 1).to_le_bytes());
        },
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width",
        &record_rollback_with_the_lineage_container_record_width_off_its_registered_width(
            "crash-judge-gpu-lineage-container",
        ),
        "I-1.7",
    );
}

/// 回退流上回退那次发布 D 往谱系树追加的那一片容器（码 3、记录类型 5、记录宽 24）：第一条谱系记录的 H_cur（偏移 16，C 的 inode 号水位 2）
/// 改成 3、重封到根。落在 D 上的状态 CPU 池级 checker 判 I-9.30 ④：D 记账里 T_rb − 1 之前最大的台阶是 A 留的 (3, 2)，不等于 3 → 红；
/// 恢复不读谱系记录的内容，两遍恢复照常读到文件。GPU 扫描内核不读 H_cur、不判 I-9.30 ④，或谱系记录宽仍按 16 读时这一位对不上。
fn record_rollback_with_the_lineage_record_carrying_another_watermark_rolled_back_from(
    tag: &str,
) -> RecordedFlow {
    const LINEAGE_RECORD_TYPE: u16 = 5;
    const PACKED_UNIT_FIRST_RECORD_OFFSET: usize = 136;
    const LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: usize = 16;
    let lineage_container =
        |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == LINEAGE_RECORD_TYPE;
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-record-watermark-rolled-back-from-three",
        &lineage_container,
        &|bytes| {
            let offset = PACKED_UNIT_FIRST_RECORD_OFFSET
                + LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET;
            let watermark_rolled_back_from =
                u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"));
            assert_eq!(
                watermark_rolled_back_from, 2,
                "D 的谱系记录带 C 的 inode 号水位 2"
            );
            bytes[offset..offset + 8].copy_from_slice(&3u64.to_le_bytes());
        },
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from",
        &record_rollback_with_the_lineage_record_carrying_another_watermark_rolled_back_from(
            "crash-judge-gpu-lineage-watermark-rolled-back-from",
        ),
        "I-9.30",
    );
}

/// 回退流上回退那次发布 D（实例 2、txg 9，回退目标 A 的 txg 3，前一版 C 的 txg 8）那片谱系容器里唯一那条记录：(T_rb, T_old) 两个字段改成给定的值、重封到根。
/// 坏镜像的造法同 `checker_judges_reference_intervals_and_every_new_root_segment_field.rs` 的
/// `rollback_whose_lineage_record_is_later_than_the_rollback_version_reddens_only_the_reference_interval_invariant`。
/// D 照旧整棵继承 A 的 extent 树与 inode 树（根指针出生 txg 3），(b) 只差谱系那一半。
fn record_rollback_with_the_lineage_record_interval_changed(
    tag: &str,
    name: &'static str,
    rollback_txg_of_the_record: u64,
    rollback_target_txg_of_the_record: u64,
) -> RecordedFlow {
    const LINEAGE_RECORD_TYPE: u16 = 5;
    const PACKED_UNIT_FIRST_RECORD_OFFSET: usize = 136;
    const LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: usize = 0;
    const LINEAGE_RECORD_ROLLBACK_TARGET_TXG_OFFSET: usize = 8;
    const TXG_OF_THE_ROLLBACK_PUBLISH: u64 = 9;
    const TXG_OF_THE_ROLLBACK_TARGET: u64 = 3;
    let lineage_container =
        |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == LINEAGE_RECORD_TYPE;
    record_rollback_with_a_resealed_field(tag, name, &lineage_container, &|bytes| {
        let rollback_txg = PACKED_UNIT_FIRST_RECORD_OFFSET + LINEAGE_RECORD_ROLLBACK_TXG_OFFSET;
        let rollback_target_txg =
            PACKED_UNIT_FIRST_RECORD_OFFSET + LINEAGE_RECORD_ROLLBACK_TARGET_TXG_OFFSET;
        assert_eq!(
            u64::from_le_bytes(bytes[rollback_txg..rollback_txg + 8].try_into().expect("8")),
            TXG_OF_THE_ROLLBACK_PUBLISH,
            "回退那次发布写的谱系记录 T_rb 是它自己的 txg"
        );
        assert_eq!(
            u64::from_le_bytes(
                bytes[rollback_target_txg..rollback_target_txg + 8]
                    .try_into()
                    .expect("8")
            ),
            TXG_OF_THE_ROLLBACK_TARGET,
            "谱系记录 T_old 是回退目标 A 的 txg"
        );
        bytes[rollback_txg..rollback_txg + 8]
            .copy_from_slice(&rollback_txg_of_the_record.to_le_bytes());
        bytes[rollback_target_txg..rollback_target_txg + 8]
            .copy_from_slice(&rollback_target_txg_of_the_record.to_le_bytes());
    })
}

/// I-3.5 谱系区间 R⁻.txg < T_rb 那一格：T_rb 改成 C 的 txg 8，恰好等于前一版的 txg，不落在 (C.txg, D.txg] 里，落在 D 上的状态 CPU 红 I-3.5。
/// GPU 那一格的「严格小于」写成「不大于」时这一位对不上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_names_a_rollback_at_the_previous_version(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_names_a_rollback_at_the_previous_version",
        &record_rollback_with_the_lineage_record_interval_changed(
            "crash-judge-gpu-i35-lineage-rollback-at-the-previous-version",
            "rollback-lineage-record-rollback-txg-at-the-previous-version",
            8,
            3,
        ),
        "I-3.5",
    );
}

/// I-3.5 谱系区间 T_rb ≤ R.txg 那一格：T_rb 改成 D 的 txg 加一，不落在 (C.txg, D.txg] 里，落在 D 上的状态 CPU 红 I-3.5。
/// GPU 不判这一格时这一位对不上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_names_a_rollback_later_than_itself(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_names_a_rollback_later_than_itself",
        &record_rollback_with_the_lineage_record_interval_changed(
            "crash-judge-gpu-i35-lineage-rollback-later-than-itself",
            "rollback-lineage-record-rollback-txg-later-than-the-publish",
            10,
            3,
        ),
        "I-3.5",
    );
}

/// I-3.5 谱系区间「根指针出生 txg ≤ T_old」那一格：T_old 改成 2，比整棵继承下来那两棵树的根指针出生 txg 3 小，落在 D 上的状态 CPU 红 I-3.5。
/// GPU 不判这一格时这一位对不上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_names_a_target_older_than_the_inherited_roots(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_names_a_target_older_than_the_inherited_roots",
        &record_rollback_with_the_lineage_record_interval_changed(
            "crash-judge-gpu-i35-lineage-target-older-than-the-inherited-roots",
            "rollback-lineage-record-target-older-than-the-inherited-roots",
            9,
            2,
        ),
        "I-3.5",
    );
}

/// 回退流上谱系树的根（码 2、key 宽 8、条目宽 120、第一条条目的类型段是谱系记录 5）：头里的树 ID 加 1、重封到根。
/// 落在 D 上的状态，CPU 恢复读谱系树根时核自描述（树 ID 不是树表条目里那棵）→ 走读失败、读不到文件；CPU 池级 checker 读这个根判 I-1.3 → 红。
/// GPU 恢复内核不读谱系树根、或读了不核树 ID，GPU 走读内核不读谱系树根时，恢复或这一位对不上。
fn record_rollback_with_the_lineage_root_naming_another_tree(tag: &str) -> RecordedFlow {
    const LINEAGE_RECORD_TYPE: u16 = 5;
    const INDEX_NODE_TREE_IDENTIFIER_OFFSET: usize = 42;
    const CONTAINER_TREE_ENTRY_TYPE_OFFSET: usize = 16;
    let lineage_root = |bytes: &[u8]| {
        if unit_class(bytes) != 2
            || index_node_key_width(bytes) != 8
            || index_node_entry_width(bytes) != 120
        {
            return false;
        }
        let first_entry = 86 + 2 * 8 + 29;
        u16::from_le_bytes(
            bytes[first_entry + CONTAINER_TREE_ENTRY_TYPE_OFFSET
                ..first_entry + CONTAINER_TREE_ENTRY_TYPE_OFFSET + 2]
                .try_into()
                .expect("2"),
        ) == LINEAGE_RECORD_TYPE
    };
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-root-names-another-tree",
        &lineage_root,
        &|bytes| {
            bytes[INDEX_NODE_TREE_IDENTIFIER_OFFSET] =
                bytes[INDEX_NODE_TREE_IDENTIFIER_OFFSET].wrapping_add(1);
        },
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree(
) {
    let flow =
        record_rollback_with_the_lineage_root_naming_another_tree("crash-judge-gpu-lineage-root");
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-1.3"),
        "流 {} 上 CPU 池级 checker 要红 I-1.3（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.cpu_red > 0,
        "流 {} 上落在 D 上的状态 CPU 恢复读谱系树根要失败、判器要有红",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 新池新建文件那张树表里谱系树（种类 11）与 logged_ops（种类 12）两条互换种类码、重封到根：盘上次序仍按树 ID 升序（I-9.16 ① 成立），
/// 按发号次序谱系树在 logged_ops 之前、树 ID 却更大（② 不成立）。两条都是零根、头 ID 都该是 0，别的判定不受影响。
/// 根落了的状态 CPU 恢复在读任何一棵树之前拒（I-9.16）、判器有红，CPU 池级 checker 红 I-9.16；GPU 两处 ② 只数到种类 8 时都对不上。
fn record_new_pool_with_the_lineage_and_logged_operations_kinds_swapped(tag: &str) -> RecordedFlow {
    const LINEAGE_KIND: u16 = 11;
    const LOGGED_OPERATIONS_KIND: u16 = 12;
    const TREE_TABLE_FIRST_ENTRY_OFFSET: usize = 86 + 2 * 8 + 29;
    const TREE_TABLE_ENTRY_BYTES: usize = 200;
    const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-tree-table-lineage-and-logged-operations-kinds-swapped",
        &tree_table,
        false,
        &|bytes| {
            let kind_offset_of = |kind: u16| {
                let offsets: Vec<usize> =
                    (0..entry_count_of_the_tree_table_read_by_its_key_width(bytes))
                        .map(|entry| {
                            TREE_TABLE_FIRST_ENTRY_OFFSET
                                + entry * TREE_TABLE_ENTRY_BYTES
                                + TREE_TABLE_ENTRY_KIND_OFFSET
                        })
                        .filter(|offset| {
                            u16::from_le_bytes(bytes[*offset..*offset + 2].try_into().expect("2"))
                                == kind
                        })
                        .collect();
                assert_eq!(offsets.len(), 1, "树表里种类 {kind} 恰好一条");
                offsets[0]
            };
            let lineage = kind_offset_of(LINEAGE_KIND);
            let logged_operations = kind_offset_of(LOGGED_OPERATIONS_KIND);
            assert!(
                lineage < logged_operations,
                "谱系树那条在 logged_ops 那条之前（树 ID 小）"
            );
            bytes[lineage..lineage + 2].copy_from_slice(&LOGGED_OPERATIONS_KIND.to_le_bytes());
            bytes[logged_operations..logged_operations + 2]
                .copy_from_slice(&LINEAGE_KIND.to_le_bytes());
        },
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order(
) {
    let flow = record_new_pool_with_the_lineage_and_logged_operations_kinds_swapped(
        "crash-judge-gpu-i916-issuing-order",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.16"),
        "流 {} 上 CPU 池级 checker 要红 I-9.16（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.cpu_red > 0,
        "流 {} 上根落了的状态 CPU 恢复要在读树之前拒、判器要有红",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}
