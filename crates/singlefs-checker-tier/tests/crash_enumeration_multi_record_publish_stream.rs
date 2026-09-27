//! checker 档模块：crash、layer0_progress
//! 里程碑「覆盖写、释放、回退与复用」并行线一的层 0 小负载（验收第 3 条）：多条记录的发布边界在小单元数上造出来——
//! 「取号 → 暖机 × 2 → A（第一个文件，一个数据单元：12 个单元）→ B（顺序写两个数据单元：14 个单元、两条记录）
//! → C（顺序写三个数据单元：15 个单元、三条记录）」整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，
//! 与新池新建文件同一种切法（上一次发布的系统配置槽写与下一次发布的单元写落在同一段）。每个状态跑恢复 + 多版本 oracle、
//! 池级 checker、记录核对器；另逐状态核发布边界（D23（journal 的角色与格式） 已定项 14 第六条）：
//! 一次发布的第一条记录落了而末条一份都没落 ⇒ 那次发布整体不施加，读回上一版、文件是旧长度（I-4.3（提交原子））。
//!
//! **流按今天的形状钉：B 14 个单元、C 15 个，验收第 3 条写的「一次发布的单元数不超过 10」这条流做不到。**
//! 分配记录树按位置寻址（D8（核心索引结构） 已定项 14），4 GiB × 2 的池上一次发布带着它的五个节点（两盘各叶 61、两盘各第 1 层、根），
//! 加上 extent 根、inode 叶容器与根、记账树、中央映射树、树表，数据单元之外的固定开销是 11 个单元，两个数据单元起 extent
//! 多一片下段节点、12 个。一次发布的记录条数等于它的数据单元数，要两条记录就至少两个数据单元：B 最少 14 个、C（三条记录）最少 15 个，
//! 挑内容大小压不下来（按内容大小逐档量的数在 `research/prompts/m2-rev-b3a3c-implementer-report.md` 第五节）。全量照这个形状跑。
//!
//! **比已有的流多罩了什么**：已有的流上每次发布恰一条记录，记录段恒是 2 写；这里 B、C 的记录段是 4 写与 6 写，
//! 崩在一次发布的记录之间的状态（B 3 个、C 12 个）第一次被枚举到；一次发布写两个、三个数据单元，
//! extent 下段一片根兼叶装两条、三条记录（上段那条条目换成下段根指针，D8（核心索引结构） 已定项 14），
//! 中央映射里两条、三条码 1 条目。多跑的一步只有恢复本身（这条流上没有重开、挂载）。
//!
//! 快的那条只展开写数小于 10 的段（A、B、C 三个单元段 26 写、30 写与 32 写不展开，只以整段持久进入后面的状态）；
//! 全量标 ignored（层 0 的枚举域 12230590578 个状态，B、C 两个单元段占 12079595518；每次写只取两态时的闭式 5435818083、
//! 两段占 5368709118），登记成崩溃枚举用例，门禁 54 号 --full 在 release 下跑，带断点续跑、可双机分片（与第一、第二条流同一个入口）。
//! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker_tier::crash::{
    enumerate_layer0_in_state_slices_or_one_shard,
    enumerate_layer0_selecting_versions_observing_each_state, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites, Layer0EnumerationOutcome, Layer0Parallelism,
    Layer0SegmentExpansion, Layer0Tally,
};
use singlefs_checker_tier::layer0_progress::Layer0Resume;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::journal::record_offset;
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    publish_sequential_write, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 全量那一趟的流名：进断点续跑的进度文件名与双机分片的账本名（`singlefs_checker_tier::layer0_progress`）。
const FULL_ENUMERATION_STREAM_NAME: &str = "file_overwrite_parallel_line_one";

/// 恰好要 `data_units` 个数据单元的内容：最后一个单元装一半。
fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 11 + seed * 3 + 7) % 251).expect("小于 256"))
        .collect()
}

fn sequential_write(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_sequential_write(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .expect("顺序写");
    pool.output = output.clone();
    output
}

/// 一次多条记录的发布在写表里的样子：它每一条记录的两份写各在写表的哪一格，它的根在哪一版、上一版是哪一版。
struct MultiRecordPublish {
    /// 这次发布的记录按 jsn 升序，每条两盘各一份：写表下标。
    record_writes_in_order: Vec<Vec<usize>>,
    version: (InstanceGeneration, CheckpointTxg),
    version_before: (InstanceGeneration, CheckpointTxg),
    content_before: Vec<u8>,
}

impl MultiRecordPublish {
    fn is_any_copy_persisted(persisted: &[bool], copies: &[usize]) -> bool {
        copies.iter().any(|copy| persisted[*copy])
    }

    /// 这一状态里这次发布的第一条记录至少落了一份，而末条一份都没落：前缀停在这次发布中间。
    fn is_cut_before_its_last_record(&self, persisted: &[bool]) -> bool {
        let first = self.record_writes_in_order.first().expect("至少两条记录");
        let last = self.record_writes_in_order.last().expect("至少两条记录");
        Self::is_any_copy_persisted(persisted, first)
            && !Self::is_any_copy_persisted(persisted, last)
    }
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    multi_record_publishes: Vec<MultiRecordPublish>,
}

/// 写表里写到某个 journal 记录槽（两盘同偏移）的那几格。
fn writes_to_record(writes: &[RetainedWrite], counter: u64) -> Vec<usize> {
    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
    writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::JournalRecord && write.offset == offset)
        .map(|(index, _)| index)
        .collect()
}

fn multi_record_publish(
    writes: &[RetainedWrite],
    output: &TransactionOutput,
    before: &TransactionOutput,
    content_before: Vec<u8>,
) -> MultiRecordPublish {
    let mut record_writes_in_order: Vec<Vec<usize>> = output
        .earlier_records_of_this_publish
        .iter()
        .map(|earlier| writes_to_record(writes, earlier.record.counter))
        .collect();
    record_writes_in_order.push(writes_to_record(writes, output.record.counter));
    for copies in &record_writes_in_order {
        assert_eq!(copies.len(), 2, "每条记录两盘各一份");
    }
    MultiRecordPublish {
        record_writes_in_order,
        version: (output.root.instance, output.root.checkpoint_txg),
        version_before: (before.root.instance, before.root.checkpoint_txg),
        content_before,
    }
}

fn prepare(tag: &str) -> Prepared {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content = content_needing(2, 1);
    let second = sequential_write(&mut pool, &second_content);
    assert_eq!(
        second.earlier_records_of_this_publish.len() + 1,
        2,
        "B：两个数据单元两条记录"
    );
    assert_eq!(
        second.rewritten.len(),
        2 + 2 + 2 + 5 + 3,
        "B：两个数据单元 + extent 下段根兼叶与上段根 + inode 叶容器与根 + 分配记录树五个节点 + 记账树、中央映射树、树表 = 14 个单元\
         （验收第 3 条的「≤ 10」今天不成立，见文件头）：{:?}",
        second.rewritten
    );
    let third_content = content_needing(3, 2);
    let third = sequential_write(&mut pool, &third_content);
    assert_eq!(
        third.earlier_records_of_this_publish.len() + 1,
        3,
        "C：三个数据单元三条记录"
    );
    assert_eq!(
        third.rewritten.len(),
        3 + 2 + 2 + 5 + 3,
        "C：三个数据单元 + 与 B 同样的 12 个单元 = 15 个单元（「≤ 10」今天不成立，见文件头）：{:?}",
        third.rewritten
    );

    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        vec![2, 2, 1, 2, 2, 1, 26, 2, 1, 30, 4, 1, 32, 6, 1, 2],
        "取号 2、暖机两次、A 的 24 个单元写并上一次的系统配置槽写 26；B 的 28 个单元写并 A 的轮换 30、两条记录 4 写；\
         C 的 30 个单元写并 B 的轮换 32、三条记录 6 写"
    );
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 5, "暖机两次、A、B、C 各一条根槽写");
    let versions = vec![
        PublishedVersion {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
            content: file_content(),
        },
        PublishedVersion {
            instance: second.root.instance,
            checkpoint_txg: second.root.checkpoint_txg,
            content: second_content.clone(),
        },
        PublishedVersion {
            instance: third.root.instance,
            checkpoint_txg: third.root.checkpoint_txg,
            content: third_content,
        },
    ];
    let multi_record_publishes = vec![
        multi_record_publish(&writes, &second, &first, file_content()),
        multi_record_publish(&writes, &third, &second, second_content),
    ];
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        versions,
        multi_record_publishes,
    }
}

/// 每个状态上核发布边界：一次发布的第一条记录落了、末条没落 ⇒ 实际走的根不是这次发布的，读回上一版、文件是旧长度。
/// 这一条 oracle 判不出来：层 0 按屏障切段，记录段里的状态上这次发布的单元全都落了，半次发布施加上去读回的也是完整的新内容。
fn observe_the_publish_boundary(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    cut_states_by_publish: &mut [u64],
) {
    for (publish_index, publish) in prepared.multi_record_publishes.iter().enumerate() {
        if !publish.is_cut_before_its_last_record(&image.persisted) {
            continue;
        }
        cut_states_by_publish[publish_index] += 1;
        assert_eq!(
            report.effective_root,
            Some(publish.version_before),
            "第 {} 次多记录发布的末条没落：那次发布整体不施加，走的是上一版的根（{:?}）",
            publish_index,
            publish.version
        );
        let RecoveryOutcome::FileRead { content, .. } = &report.outcome else {
            panic!("要读回上一版，实际 {:?}", report.outcome);
        };
        assert!(
            *content == publish.content_before,
            "读回上一版：文件是旧长度 {} 字节，实际 {} 字节",
            publish.content_before.len(),
            content.len()
        );
    }
}

fn assert_clean(tally: &Layer0Tally) {
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
}

/// 平时跑的那一份：B、C 两个单元段（30 写、32 写）与 A 那一段（26 写）不展开，其余每段任意子集——B 的两条记录段（4 写）、
/// C 的三条记录段（6 写）全展开，崩在一次发布的记录之间的每个状态都跑到。
#[test]
fn every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it() {
    let prepared = prepare("parallel-line-one-layer0-fast");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
    let mut cut_states_by_publish = vec![0u64; prepared.multi_record_publishes.len()];
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
        &mut |image, report| {
            observe_the_publish_boundary(&prepared, image, report, &mut cut_states_by_publish);
        },
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .filter(|segment| segment.len() < 10)
        .cloned()
        .collect();
    assert_eq!(
        closed_form_state_count(&expanded),
        102,
        "每次写只取两态时展开的段按闭式数（补第三态之前的口径）：1 + 六个 2 写段各 3 + 五个 1 写段各 1 + 4 写段 15 + 6 写段 63"
    );
    assert_eq!(
        tally.states,
        layer0_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &|segment_index, segment| {
                if expand(segment_index, segment) {
                    Layer0SegmentExpansion::EveryProperSubset
                } else {
                    Layer0SegmentExpansion::NotExpanded
                }
            },
        ),
        "展开的段按层 0 的枚举域数（原地覆写取三态）"
    );
    assert_eq!(
        tally.states,
        1 + 3 * (3 * 3 - 1) + 3 * 3 + 5 + 15 + 63,
        "1 + 三个系统配置槽 2 写段各 3² − 1 + 三个记录 2 写段各 3 + 五个 1 写段各 1 + 4 写段 15 + 6 写段 63（每次写只取两态时 102）"
    );
    // B：记录段 4 写里第一条落了（至少一份）、第二条一份没落 ⇒ 3 × 1 = 3 个状态。
    // C：记录段 6 写里第一条落了、第三条一份没落 ⇒ 第一条 3 种 × 第二条 4 种 = 12 个状态。
    assert_eq!(
        cut_states_by_publish,
        vec![3, 12],
        "崩在一次发布的记录之间的状态真跑到了"
    );
    println!(
        "LAYER0_PARALLEL_LINE_ONE_FAST states={} cut_states_by_publish={cut_states_by_publish:?} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_clean(&tally);
}

/// 全量：16 段、闭式见下。B、C 两个单元段（各带上一次发布的两次系统配置槽轮换，取三态）各 3² · 2^28 − 1 与 3² · 2^30 − 1 个状态，
/// 每个状态两遍恢复 + checker。登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv`），54 号认下面打印的
/// `LAYER0_PARALLEL_LINE_ONE` 行里 `exhaustive=true`。带断点续跑：进度目录、输入指纹与强制从头跑的开关从环境变量取，
/// 没设进度目录就不留进度文件。双机分片（`SINGLEFS_LAYER0_SHARD`，`research/scripts/layer0-shard-run.sh`）：
/// 跑一片时只写账本、不打计数行、不判；merge 那一趟拿并齐的计数照下面逐项判、逐字打同样的行。
#[test]
#[ignore = "全量 12230590578 个状态（一百二十多亿）、每个两遍恢复 + checker；门禁 54 号 --full 在 release 下跑，带断点续跑、可双机分片"]
fn full_enumeration_of_the_multi_record_publish_stream_is_exhaustive_and_clean() {
    let prepared = prepare("parallel-line-one-layer0-full");
    let closed_form_with_two_states_per_write = closed_form_state_count(&prepared.segments);
    assert_eq!(
        closed_form_with_two_states_per_write,
        1 + 6 * 3 + 5 + ((1 << 26) - 1) + ((1 << 30) - 1) + 15 + ((1 << 32) - 1) + 63,
        "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，十六段"
    );
    assert_eq!(closed_form_with_two_states_per_write, 5_435_818_083);
    // 计数行的 closed_form 报枚举域的闭式（原地覆写三态）：六段各带两次系统配置槽写，每段 3^m · 2^(n−m) − 1。
    let closed_form = layer0_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    assert_eq!(
        closed_form,
        1 + 3 * (3 * 3 - 1)
            + 3 * 3
            + 5
            + (9 * (1 << 24) - 1)
            + (9 * (1 << 28) - 1)
            + 15
            + (9 * (1 << 30) - 1)
            + 63,
        "枚举域的闭式：三个只有系统配置槽写的 2 写段各 3² − 1、三个记录 2 写段各 3、五个 1 写段、A / B / C 的单元段各 3² · 2^(n−2) − 1、4 写段 15、6 写段 63"
    );
    assert_eq!(closed_form, 12_230_590_578);
    let tally = match enumerate_layer0_in_state_slices_or_one_shard(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &full_expansion,
        Layer0Parallelism::from_environment(),
        None,
        &Layer0Resume::from_environment(FULL_ENUMERATION_STREAM_NAME),
    ) {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(_written) => return,
    };
    println!(
        "LAYER0_PARALLEL_LINE_ONE states={} closed_form={closed_form} exhaustive={} violations={} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(tally.states, closed_form);
    assert_clean(&tally);
}
