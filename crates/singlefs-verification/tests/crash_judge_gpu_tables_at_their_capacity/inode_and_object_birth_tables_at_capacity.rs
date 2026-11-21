//! 主走读 inode → 对象出生代、一版的 inode 记录、I-9.29 跨版本那张表：抬上限之后照样判得了。

use super::*;

// ───────── 主走读 inode → 对象出生代、一版的 inode 记录、I-9.29 跨版本那张表：抬上限之后照样判得了 ─────────

/// 主走读 inode → 对象出生代（`C_MAX_INODE_BIRTHS`，站点 13）与扫描内核一版的 inode 记录（`D_MAX_INODE_RECORDS`，站点 49）抬到 16384 之前装几条。
const INODES_THE_TWO_KERNELS_HELD_BEFORE_THE_RAISE: u64 = 1024;

/// I-9.29 跨版本那张表（`D_MAX_OBJECT_BIRTHS`，站点 59）抬到 16384 之前装几个 (inode 树, 号)。
const OBJECT_BIRTHS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE: u64 = 2048;

/// 那条流第一次发布建几个空 inode：连第一个文件，之后每一版的号都多于上面两个数（CPU 一侧在全落的镜像上数到各版合起来 2502 个号）。
const NEW_INODES_PAST_THE_OLD_CAPACITIES: u64 = 2500;

/// 被判那次发布建几个空 inode。
const NEW_INODES_OF_THE_JUDGED_PUBLISH: u64 = 1;

const _: () = assert!(
    NEW_INODES_PAST_THE_OLD_CAPACITIES > INODES_THE_TWO_KERNELS_HELD_BEFORE_THE_RAISE
        && NEW_INODES_PAST_THE_OLD_CAPACITIES > OBJECT_BIRTHS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE,
    "建的 inode 要多于那三张表抬之前的条数"
);

/// 新池新建文件（txg 3）→ 一次建 [`NEW_INODES_PAST_THE_OLD_CAPACITIES`] 个空 inode（txg 4）→ 被判的是再建 [`NEW_INODES_OF_THE_JUDGED_PUBLISH`] 个的那一次（txg 5）：
/// 起点镜像是 mkfs 到它之前的整段录制流施加出来的那一份，写表只录它。每一版第一个文件都不变。
fn record_the_inode_flow(tag: &str) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let mut counts = vec![(
        pool.output.root.checkpoint_txg,
        allocation_record_counts_of(&pool.output),
    )];
    let mut examined = HashSet::new();
    note_the_unreleased_records(&pool.output, &mut examined);
    publish_new_inodes_and_note_the_version(
        &mut pool,
        NEW_INODES_PAST_THE_OLD_CAPACITIES,
        &mut versions,
        &mut counts,
        &mut examined,
    );
    let operations_before_the_judged_publish = pool.stream.operation_count();
    publish_new_inodes_and_note_the_version(
        &mut pool,
        NEW_INODES_OF_THE_JUDGED_PUBLISH,
        &mut versions,
        &mut counts,
        &mut examined,
    );
    let judged_txg = pool.output.root.checkpoint_txg;
    let operations = pool.retained_operations();
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&operations[pool.mkfs_operation_count..operations_before_the_judged_publish]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_judged_publish..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("被判那次发布有根槽写");
    RecordedFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        judged_txg,
        allocation_record_counts_of_each_version: counts,
        unreleased_records_examined_across_the_versions: examined,
        journal_record_writes_on_device_zero_after_mkfs: journal_record_writes_on_device_zero(
            &operations[pool.mkfs_operation_count..],
        ),
    }
}

/// 一次建 2500 个空 inode、再建一个：每一版两千五百来个号，多于主走读 inode → 对象出生代与一版 inode 记录那两张表抬之前的 1024 条、
/// I-9.29 跨版本那张表抬之前的 2048 个。每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同；三张表里有一张改回原来的条数，
/// 走读或扫描内核在站点 13、49、59 记判不了，这条红（几张表的散列索引在这条流上建起来：两千多条时按桶表找）。
#[test]
fn the_gpu_judge_judges_versions_of_two_thousand_five_hundred_inodes_past_the_old_capacities_like_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_judges_versions_of_two_thousand_five_hundred_inodes_past_the_old_capacities_like_the_cpu_judge";
    const FLOW_NAME: &str = "inodes-past-the-old-capacities-of-the-gpu-tables";
    let flow = record_the_inode_flow(FLOW_NAME);
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(FLOW_NAME, &cpu);
    let cpu_cross_check_words: Vec<CrossCheckWords> = cpu
        .iter()
        .map(|judgement| judgement.cross_check_words)
        .collect();
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(
        FLOW_NAME,
        &gpu,
        &cpu_cross_check_words,
    );
}
