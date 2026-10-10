//! 小写流上的崩溃状态：从某一次发布之后起取录制流，按屏障与 FUA 切段，每一段都展开（整写的任意真子集，段末再加全落），每个状态恢复之后
//! 跑池级 checker（枚举器自己跑一遍、记违例），观察者再跑一遍分类删除线几条的判定种类与 GPU 恢复内核会拒的形状。
//! 原型里的流自己造，不从名字带 layer0 的用例里拷。每次跑之前先算全量状态数，超过 10^6 不跑。
use std::collections::BTreeMap;

use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::RecoveryOutcome;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments_with_stream_indexes, PublishedVersion, RetainedWrite};
use singlefs_harness::segments::StepKind;

use crate::judge::{gpu_shape_divergence, judge, outcome_class};
use crate::pool::{content_of, geometry, SparsePool, INSTANCE_ONE};

pub const STATE_LIMIT: u64 = 1_000_000;

pub struct CrashFlow {
    pub name: &'static str,
    pub pool: SparsePool,
    /// 流从 `published[base_index]` 那一版之后起。
    pub base_index: usize,
}

fn three_units() -> Vec<u8> {
    let payload = data_unit_payload_capacity();
    content_of(2 * payload + payload / 2, 1)
}

pub fn flows() -> Vec<CrashFlow> {
    let mut out = Vec::new();
    // c-del3：三个单元的文件，意图 + 批大小 1 删三批（删到最后一批，extent 树只剩零条目根）。
    {
        let mut pool = SparsePool::build(4 << 30, &content_of(3000, 0));
        pool.sequential_write(&three_units());
        let base_index = pool.published.len() - 1;
        pool.intent(1).expect("意图");
        while !pool.queue_is_empty() {
            pool.step(1).expect("批");
        }
        out.push(CrashFlow { name: "c-del3", pool, base_index });
    }
    // c-del1：一个单元的第一个文件，意图 + 最后一批。
    {
        let mut pool = SparsePool::build(4 << 30, &content_of(3000, 0));
        let base_index = pool.published.len() - 1;
        pool.intent(1).expect("意图");
        while !pool.queue_is_empty() {
            pool.step(144).expect("批");
        }
        out.push(CrashFlow { name: "c-del1", pool, base_index });
    }
    // c-empty：建一个空 inode（2 号）之后删它。
    {
        let mut pool = SparsePool::build(4 << 30, &content_of(3000, 0));
        pool.new_inodes(1);
        let base_index = pool.published.len() - 1;
        pool.intent(2).expect("意图");
        while !pool.queue_is_empty() {
            pool.step(1).expect("批");
        }
        out.push(CrashFlow { name: "c-empty", pool, base_index });
    }
    // c-midcreate：三个单元，意图、第一批之后建一个 inode，再删完。
    {
        let mut pool = SparsePool::build(4 << 30, &content_of(3000, 0));
        pool.sequential_write(&three_units());
        let base_index = pool.published.len() - 1;
        pool.intent(1).expect("意图");
        pool.step(1).expect("第一批");
        pool.new_inodes(1);
        while !pool.queue_is_empty() {
            pool.step(1).expect("批");
        }
        out.push(CrashFlow { name: "c-midcreate", pool, base_index });
    }
    // c-post：第一个文件删完之后建 2 个、删 3 号。
    {
        let mut pool = SparsePool::build(4 << 30, &content_of(3000, 0));
        pool.intent(1).expect("意图");
        while !pool.queue_is_empty() {
            pool.step(144).expect("批");
        }
        let base_index = pool.published.len() - 1;
        pool.new_inodes(2);
        pool.intent(3).expect("意图 3");
        while !pool.queue_is_empty() {
            pool.step(1).expect("批");
        }
        out.push(CrashFlow { name: "c-post", pool, base_index });
    }
    out
}

pub struct Prepared {
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
    pub states: u64,
}

pub fn prepare(flow: &CrashFlow) -> Prepared {
    let operations = flow.pool.stream.retained_operations();
    let from = flow.pool.published[flow.base_index].operations;
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&operations[from..], &geometry());
    let judged_root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("有根槽写");
    let versions = flow
        .pool
        .published
        .iter()
        .filter_map(|published| {
            published.content.clone().map(|content| PublishedVersion { instance: INSTANCE_ONE, checkpoint_txg: CheckpointTxg(published.txg), content })
        })
        .collect();
    let states = closed_form_state_count(&segments);
    Prepared { writes, segments, judged_root_index, versions, states }
}

pub fn count() {
    for flow in flows() {
        let prepared = prepare(&flow);
        let sizes: Vec<usize> = prepared.segments.iter().map(Vec::len).collect();
        println!(
            "CRASH_COUNT flow={} base={} publishes_in_stream={} writes={} segments={} largest_segment={} states={}",
            flow.name,
            flow.pool.published[flow.base_index].name,
            flow.pool.published.len() - 1 - flow.base_index,
            prepared.writes.len(),
            sizes.len(),
            sizes.iter().max().copied().unwrap_or(0),
            prepared.states
        );
    }
}

pub fn run(name: &str) {
    let flow = flows().into_iter().find(|flow| flow.name == name).unwrap_or_else(|| panic!("没有这条流：{name}"));
    let prepared = prepare(&flow);
    if prepared.states > STATE_LIMIT {
        println!("CRASH_SKIPPED flow={name} states={} limit={STATE_LIMIT}", prepared.states);
        return;
    }
    let base = flow.pool.published[flow.base_index].image.clone();
    let started = std::time::Instant::now();
    let mut kinds: BTreeMap<String, u64> = BTreeMap::new();
    let mut flagged: BTreeMap<String, (u64, String)> = BTreeMap::new();
    let mut observed = 0u64;
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &|_segment_index, _segment| true,
        &mut |image, report| {
            observed += 1;
            let judged = judge(image);
            for (invariant, kind, text) in &judged.watched {
                *kinds.entry(format!("{invariant}{kind}")).or_default() += 1;
                if matches!(kind, 'G' | 'P' | 'V') {
                    let key = format!("{invariant}={kind}:{}", text.chars().take(70).collect::<String>());
                    let entry = flagged.entry(key).or_insert((0, String::new()));
                    entry.0 += 1;
                }
            }
            let class = outcome_class(&report.outcome);
            *kinds.entry(format!("cpu_consult_{}", class.chars().next().unwrap_or('?'))).or_default() += 1;
            if matches!(report.outcome, RecoveryOutcome::NoFile { .. } | RecoveryOutcome::FileRead { .. }) {
                let shape = gpu_shape_divergence(image, report.effective_root.map(|(instance, txg)| (instance.0, txg.0)));
                if !shape.is_empty() {
                    let entry = flagged.entry(format!("gpu:{}:{shape}", class)).or_insert((0, String::new()));
                    entry.0 += 1;
                }
            }
        },
    );
    println!(
        "CRASH_RUN flow={name} states={} observed={observed} enumerator_violations={} enumerator_failed_states={} no_file_states={} file_read_states={} seconds={:.1}",
        tally.states,
        tally.violations,
        tally.failed_states,
        tally.no_file_states,
        tally.file_read_states,
        started.elapsed().as_secs_f64()
    );
    println!("CRASH_CHECKER_VIOLATED flow={name} {:?}", tally.checker_violated_states);
    println!("CRASH_KINDS flow={name} {:?}", kinds);
    for (key, (count, _)) in &flagged {
        println!("CRASH_FLAGGED flow={name} count={count} {key}");
    }
    if let Some(first) = &tally.first_violation {
        println!("CRASH_FIRST_VIOLATION flow={name} {}", first.chars().take(300).collect::<String>());
    }
}
