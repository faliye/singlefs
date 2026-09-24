//! 里程碑「第二个事务」并行线一的层 0 小负载（验收第 3 条）：一次发布的单元数不超过 10，多条记录的发布边界就在小单元数上造出来——
//! 「取号 → 暖机 × 2 → A（第一个文件，一个数据单元）→ B（顺序写两个数据单元：9 个单元、两条记录）
//! → C（顺序写三个数据单元：10 个单元、三条记录）」整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，
//! 与第一个事务同一种切法（上一次发布的系统配置槽写与下一次发布的单元写落在同一段）。每个状态跑恢复 + 多版本 oracle、
//! 池级 checker、记录核对器；另逐状态核发布边界（D23（journal 的角色与格式） 已定项 14 第六条）：
//! 一次发布的第一条记录落了而末条一份都没落 ⇒ 那次发布整体不施加，读回上一版、文件是旧长度（I-4.3（提交原子））。
//!
//! **比已有的流多罩了什么**：已有的流上每次发布恰一条记录，记录段恒是 2 写；这里 B、C 的记录段是 4 写与 6 写，
//! 崩在一次发布的记录之间的状态（B 3 个、C 12 个）第一次被枚举到；一次发布写两个、三个数据单元，
//! extent 根兼叶装两条、三条记录，中央映射里两条、三条码 1 条目。多跑的一步只有恢复本身（这条流上没有重开、挂载）。
//!
//! 快的那条只展开写数小于 10 的段（B、C 两个单元段 20 写与 22 写不展开，只以整段持久进入后面的状态）；
//! 全量标 ignored（闭式 5505123 个状态，B、C 两个单元段占 524 万），不在本轮跑。

mod common;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::journal::record_offset;
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    publish_sequential_write, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions_observing_each_state,
    enumerate_layer0_versions, writes_and_segments, CrashImage, Layer0Tally, MemoryPool,
    PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

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
    assert_eq!(second.rewritten.len(), 9, "B：9 个单元（≤ 10）");
    let third_content = content_needing(3, 2);
    let third = sequential_write(&mut pool, &third_content);
    assert_eq!(
        third.earlier_records_of_this_publish.len() + 1,
        3,
        "C：三个数据单元三条记录"
    );
    assert_eq!(third.rewritten.len(), 10, "C：10 个单元（≤ 10）");

    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 20, 4, 1, 22, 6, 1, 2],
        "取号 2、暖机两次、A 的 16 个单元写并上一次的系统配置槽写 18；B 的 18 个单元写并 A 的轮换 20、两条记录 4 写；\
         C 的 20 个单元写并 B 的轮换 22、三条记录 6 写"
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

/// 平时跑的那一份：B、C 两个单元段（20 写、22 写）与 A 那一段（18 写）不展开，其余每段任意子集——B 的两条记录段（4 写）、
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
        tally.states,
        closed_form_state_count(&expanded),
        "展开的段按闭式数"
    );
    assert_eq!(
        tally.states, 102,
        "1 + 六个 2 写段各 3 + 五个 1 写段各 1 + 4 写段 15 + 6 写段 63"
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

/// 全量：16 段、闭式见下。B、C 两个单元段各 2^20 − 1 与 2^22 − 1 个状态，每个状态两遍恢复 + checker。
#[test]
#[ignore = "全量 5505123 个状态、每个两遍恢复 + checker，debug 下数小时；交 crash-verifier 在 release 下跑"]
fn full_enumeration_of_the_parallel_line_one_stream_is_exhaustive_and_clean() {
    let prepared = prepare("parallel-line-one-layer0-full");
    let closed_form = closed_form_state_count(&prepared.segments);
    assert_eq!(
        closed_form,
        1 + 6 * 3 + 5 + ((1 << 18) - 1) + ((1 << 20) - 1) + 15 + ((1 << 22) - 1) + 63,
        "闭式：1 + Σ(2^|段| − 1)，十六段"
    );
    assert_eq!(closed_form, 5_505_123);
    let tally = enumerate_layer0_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
    );
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
