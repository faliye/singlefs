//! 攻方原型（m3-batch60-61-code-r1 opus）：只用判定库的公开接口造库，量按计划导入（`VerdictStore::import_the_planned_rows_from`）。
//! P5 代价：对方库里累计的别的流、别的版本的行数 N 从 0 往上取，点名的项不变，逐步数这一次导入在 RocksDB 里做了几次点读、几次前缀扫
//! （同一份 librocksdb 的按线程 PerfContext：GetFromMemtableCount 是 Get 次数、SeekOnMemtableCount 是迭代器 seek 次数，
//! BlockReadCount / UserKeyComparisonCount 是每次读在 LSM 里走了多深），连同开对方库的挂钟、整库导入的挂钟。
//! P4 断了重跑：一个子进程（这个测试二进制自己，`--exact opus_resume_child`）做按计划导入，父进程在导入开始之后的不同时刻 SIGKILL 它，
//! 再开库、逐项读（读得出的与不断那一份逐项相同、读不出的不报错），再跑同一次导入补齐，与不断那一份、整库导入那一份逐项比。
//! 库都建在 OPUS_STORE_DIR（rerun.sh 给工作目录下的 stores/）；建过的库（目录里有 built 标记）下一次直接用。
use std::path::{Path, PathBuf};
use std::time::Instant;

use rocksdb::perf::{set_perf_stats, PerfContext, PerfMetric, PerfStatsLevel};
use singlefs_checker_tier::crash_identity::{ConclusionKind, ConclusionVersion, ObserverName};
use singlefs_checker_tier::verdict_store::{
    BlockCountVector, BlockStartStateIndex, ConclusionBlockRecord, EncodedVerdictVector, EnumerationPlanHash,
    InputFingerprint, JudgingDevice, PlannedImportKeys, PlannedImportTally, RecoveryWordsOfState, StateOffsetInBlock,
    StreamOrNodeName, VerdictBlock, VerdictBlockKey, VerdictStore, VerdictVectorNumber,
};

const CPU_VERSION: ConclusionVersion = ConclusionVersion([0xC1; 32]);
const GPU_VERSION: ConclusionVersion = ConclusionVersion([0xC2; 32]);
const OBSERVER_VERSION: ConclusionVersion = ConclusionVersion([0xC3; 32]);
const OLD_CPU_VERSION: ConclusionVersion = ConclusionVersion([0xB1; 32]);
const CPU_VERIFIER: [u8; 32] = [0x81; 32];
const GPU_VERIFIER: [u8; 32] = [0x82; 32];
const FLOW_FACTS: [u8; 32] = [0x72; 32];

fn store_dir() -> PathBuf {
    PathBuf::from(std::env::var("OPUS_STORE_DIR").expect("rerun.sh 设 OPUS_STORE_DIR"))
}

fn key(node: &str, block_index: u64) -> VerdictBlockKey {
    VerdictBlockKey {
        input_fingerprint: InputFingerprint([0x11; 32]),
        stream_or_node_name: StreamOrNodeName::new(node).expect("名字不长"),
        enumeration_plan_hash: EnumerationPlanHash([0x22; 32]),
        block_start: BlockStartStateIndex(block_index * 65536),
    }
}

fn observer_kind() -> ConclusionKind {
    ConclusionKind::Observer { observer_name: ObserverName::new("probe").expect("名字") }
}

/// 一个确定的伪随机序列（xorshift），造块的内容。
struct Pseudo(u64);
impl Pseudo {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// 一块结论：`states` 个状态，编号在 `numbers` 里按伪随机取，违例取不是第 0 个编号的那几个状态（最多 8 个），恢复字给了就逐状态伪随机。
fn record(states: usize, numbers: &[VerdictVectorNumber], seed: u64, recovery: bool) -> ConclusionBlockRecord {
    let mut pseudo = Pseudo(seed | 1);
    let each: Vec<VerdictVectorNumber> =
        (0..states).map(|_| numbers[usize::try_from(pseudo.next() % numbers.len() as u64).expect("小")]).collect();
    let violating_states = each
        .iter()
        .enumerate()
        .filter(|(_, number)| **number != numbers[0])
        .take(8)
        .map(|(offset, _)| StateOffsetInBlock(offset as u64))
        .collect();
    let recovery_words = recovery.then(|| {
        (0..states)
            .map(|_| {
                let mut words = [0u32; 8];
                for word in &mut words {
                    *word = (pseudo.next() >> 16) as u32;
                }
                RecoveryWordsOfState(words)
            })
            .collect()
    });
    ConclusionBlockRecord {
        block: VerdictBlock::new(each).expect("至少一个状态"),
        violating_states,
        count_vector: BlockCountVector::default(),
        judged_on: JudgingDevice::Cpu,
        recovery_words,
    }
}

fn vectors(store: &mut VerdictStore, names: &[&str]) -> Vec<VerdictVectorNumber> {
    names
        .iter()
        .map(|name| store.register_verdict_vector(&EncodedVerdictVector(name.as_bytes().to_vec())).expect("登记"))
        .collect()
}

fn copy_library(from: &Path, to: &Path) {
    let _ = std::fs::remove_dir_all(to);
    std::fs::create_dir_all(to).expect("建目录");
    for entry in std::fs::read_dir(from).expect("读目录") {
        let entry = entry.expect("一项");
        if entry.file_type().expect("类型").is_file() {
            std::fs::copy(entry.path(), to.join(entry.file_name())).expect("拷");
        }
    }
}

fn library_bytes_and_files(directory: &Path) -> (u64, usize, u64) {
    let mut bytes = 0;
    let mut sst = 0;
    let mut wal = 0;
    for entry in std::fs::read_dir(directory).expect("读目录") {
        let entry = entry.expect("一项");
        let length = entry.metadata().expect("长度").len();
        bytes += length;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".sst") {
            sst += 1;
        }
        if name.ends_with(".log") && !name.starts_with("LOG") {
            wal += length;
        }
    }
    (bytes, sst, wal)
}

// ───────────── P5：导入的读次数与对方库里累计的行数 ─────────────

const FLOW_F_NODE: &str = "flow-f::node-f::0011";
const COST_STATES_PER_BLOCK: usize = 4096;
const COST_FLOW_F_BLOCKS: u64 = 64;

fn cost_sizes() -> Vec<u64> {
    std::env::var("OPUS_COST_SIZES")
        .unwrap_or_else(|_| "0,3000".to_string())
        .split(',')
        .map(|text| text.parse().expect("OPUS_COST_SIZES 是逗号分开的整数"))
        .collect()
}

/// 流 F 第 k 块的结论（两边相同的内容）：本机这一份是偶数块，第二台那一份是奇数块；第 4j+1 块挂观察者。
fn flow_f_record(numbers: &[VerdictVectorNumber], block_index: u64) -> ConclusionBlockRecord {
    record(COST_STATES_PER_BLOCK, numbers, 1000 + block_index, false)
}

/// 流水线导入那一段照计划排的键：不归本机的（奇数）块，两种判器各一把当前版本、挂观察者的块观察者一把，两种核对代码各一行；路径是流 F 的每一条。
fn cost_planned_keys() -> PlannedImportKeys {
    let mut planned = PlannedImportKeys {
        conclusions: Vec::new(),
        paths: (0..COST_FLOW_F_BLOCKS).map(|block_index| format!("f.{block_index}")).collect(),
        verifier_blocks: Vec::new(),
    };
    for block_index in (0..COST_FLOW_F_BLOCKS).filter(|block_index| block_index % 2 == 1) {
        let block_key = key(FLOW_F_NODE, block_index);
        planned.conclusions.push((block_key.clone(), ConclusionKind::GpuJudge, GPU_VERSION));
        planned.conclusions.push((block_key.clone(), ConclusionKind::CpuJudge, CPU_VERSION));
        if block_index % 4 == 1 {
            planned.conclusions.push((block_key.clone(), observer_kind(), OBSERVER_VERSION));
        }
        planned.verifier_blocks.push((block_key.clone(), CPU_VERIFIER));
        planned.verifier_blocks.push((block_key, GPU_VERIFIER));
    }
    planned
}

/// 对方库 B_N：流 F 的 64 块（当前版本 CPU 结论、观察者、核对行、事实行、路径）、F 在一把旧版本下的 64 块，再加 N 块别的流（结论、核对行，
/// 每 64 块一条路径）。按进度文件续建：建到一半被前台时限打断的，下一次接着建。
fn build_the_other_library(noise_blocks: u64) -> PathBuf {
    let directory = store_dir().join(format!("cost-other-{noise_blocks}"));
    let built = store_dir().join(format!("cost-other-{noise_blocks}.built"));
    let progress = store_dir().join(format!("cost-other-{noise_blocks}.progress"));
    if built.exists() {
        return directory;
    }
    let started = Instant::now();
    let (mut store, mut written) = if directory.join("CURRENT").exists() {
        let written: u64 = std::fs::read_to_string(&progress).expect("有进度").trim().parse().expect("数");
        (VerdictStore::open_existing(&directory).expect("开").0, written)
    } else {
        let _ = std::fs::remove_dir_all(&directory);
        let mut store = VerdictStore::create_empty(&directory).expect("建");
        let numbers = vectors(&mut store, &["", "red-a", "red-b"]);
        for block_index in 0..COST_FLOW_F_BLOCKS {
            let block_key = key(FLOW_F_NODE, block_index);
            store.store_conclusion_block(&block_key, &ConclusionKind::CpuJudge, &CPU_VERSION, &flow_f_record(&numbers, block_index)).expect("存");
            store.store_conclusion_block(&block_key, &ConclusionKind::CpuJudge, &OLD_CPU_VERSION, &record(COST_STATES_PER_BLOCK, &numbers, 5000 + block_index, false)).expect("存");
            if block_index % 4 == 1 {
                store.store_conclusion_block(&block_key, &observer_kind(), &OBSERVER_VERSION, &record(COST_STATES_PER_BLOCK, &numbers[..1], 7, false)).expect("存");
            }
            store.store_verifier_block(&block_key, &CPU_VERIFIER, &FLOW_FACTS, &vec![0; COST_STATES_PER_BLOCK]).expect("核对行");
            store.record_path(&format!("f.{block_index}"), &format!("flow-f-identity-{block_index}")).expect("路径");
        }
        store.store_flow_facts(&FLOW_FACTS, &[0x5a; 1024]).expect("事实");
        std::fs::write(&progress, "0").expect("进度");
        (store, 0)
    };
    let numbers: Vec<VerdictVectorNumber> =
        ["", "red-a", "red-b"].iter().map(|name| store.register_verdict_vector(&EncodedVerdictVector(name.as_bytes().to_vec())).expect("登记")).collect();
    let deadline = std::env::var("OPUS_BUILD_SECONDS").ok().and_then(|text| text.parse::<u64>().ok()).unwrap_or(200);
    while written < noise_blocks {
        let node = format!("flow-g{}::node-g::{:04}", written / 64, written / 64);
        let block_key = key(&node, written % 64);
        store.store_conclusion_block(&block_key, &ConclusionKind::CpuJudge, &CPU_VERSION, &record(COST_STATES_PER_BLOCK, &numbers, 90_000 + written, false)).expect("存");
        store.store_verifier_block(&block_key, &CPU_VERIFIER, &FLOW_FACTS, &vec![0; COST_STATES_PER_BLOCK]).expect("核对行");
        if written % 64 == 0 {
            store.record_path(&format!("g{}.0", written / 64), &node).expect("路径");
        }
        written += 1;
        if written % 500 == 0 {
            std::fs::write(&progress, written.to_string()).expect("进度");
            if started.elapsed().as_secs() > deadline {
                println!("OPUS_P5 build noise={noise_blocks} written={written} seconds={:.1} (接着建)", started.elapsed().as_secs_f64());
                return directory;
            }
        }
    }
    drop(store);
    std::fs::write(&progress, written.to_string()).expect("进度");
    std::fs::write(&built, "").expect("标记");
    println!("OPUS_P5 build noise={noise_blocks} written={written} seconds_this_part={:.1} done", started.elapsed().as_secs_f64());
    directory
}

/// 本机库 A：流 F 的偶数块（与 B 里逐项相同）与 F 的路径。
fn build_the_local_library(directory: &Path) -> VerdictStore {
    let _ = std::fs::remove_dir_all(directory);
    let mut store = VerdictStore::create_empty(directory).expect("建");
    let numbers = vectors(&mut store, &["", "red-a", "red-b"]);
    for block_index in (0..COST_FLOW_F_BLOCKS).filter(|block_index| block_index % 2 == 0) {
        store.store_conclusion_block(&key(FLOW_F_NODE, block_index), &ConclusionKind::CpuJudge, &CPU_VERSION, &flow_f_record(&numbers, block_index)).expect("存");
    }
    for block_index in 0..COST_FLOW_F_BLOCKS {
        store.record_path(&format!("f.{block_index}"), &format!("flow-f-identity-{block_index}")).expect("路径");
    }
    store
}

const COUNTED: [(&str, PerfMetric); 9] = [
    ("gets", PerfMetric::GetFromMemtableCount),
    ("seeks", PerfMetric::SeekOnMemtableCount),
    ("nexts", PerfMetric::NextOnMemtableCount),
    ("block_reads", PerfMetric::BlockReadCount),
    ("block_read_bytes", PerfMetric::BlockReadByte),
    ("cache_hits", PerfMetric::BlockCacheHitCount),
    ("key_comparisons", PerfMetric::UserKeyComparisonCount),
    ("keys_skipped", PerfMetric::InternalKeySkippedCount),
    ("get_read_bytes", PerfMetric::GetReadBytes),
];

fn counted(context: &PerfContext) -> String {
    COUNTED.iter().map(|(name, metric)| format!("{name}={}", context.metric(*metric))).collect::<Vec<_>>().join(" ")
}

fn tally_text(tally: &PlannedImportTally) -> String {
    format!(
        "written={} identical={} missing={} paths={} verifier={} facts={} rows={} synced={}",
        tally.conclusion_blocks_written,
        tally.conclusion_blocks_already_identical,
        tally.conclusion_blocks_missing_in_the_other_library,
        tally.paths_written,
        tally.verifier_blocks_written,
        tally.flow_facts_written,
        tally.rows_written,
        tally.synced_writes
    )
}

/// 建（续建）每一档 N 的对方库；建完的跳过。每次调用最多建 OPUS_BUILD_SECONDS 秒。
#[test]
fn opus_cost_build() {
    if std::env::var_os("OPUS_STORE_DIR").is_none() {
        return;
    }
    for noise_blocks in cost_sizes() {
        build_the_other_library(noise_blocks);
        if !store_dir().join(format!("cost-other-{noise_blocks}.built")).exists() {
            return;
        }
    }
}

/// 每一档 N：拷一份建好的 B（照取回的样子，开库时重放 WAL），量开库；新建本机库 A，按计划导入一次（再导一次，全是已有且相同的），
/// 数这一次在 RocksDB 里的读；N 不超过 OPUS_WHOLE_LIMIT 的另建一份 A 整库导入（旧的 import_from）量挂钟。
#[test]
fn opus_cost_measure() {
    if std::env::var_os("OPUS_STORE_DIR").is_none() {
        return;
    }
    let whole_limit: u64 = std::env::var("OPUS_WHOLE_LIMIT").ok().and_then(|text| text.parse().ok()).unwrap_or(0);
    set_perf_stats(PerfStatsLevel::EnableTimeExceptForMutex);
    let mut context = PerfContext::default();
    for noise_blocks in cost_sizes() {
        let built = store_dir().join(format!("cost-other-{noise_blocks}"));
        assert!(store_dir().join(format!("cost-other-{noise_blocks}.built")).exists(), "先跑 opus_cost_build 建好 N={noise_blocks}");
        let fetched = store_dir().join("cost-fetched");
        copy_library(&built, &fetched);
        let (bytes, sst, wal) = library_bytes_and_files(&fetched);
        context.reset();
        let started = Instant::now();
        let (other, _) = VerdictStore::open_existing(&fetched).expect("开对方库");
        let open_seconds = started.elapsed().as_secs_f64();
        println!("OPUS_P5 noise={noise_blocks} other_bytes={bytes} sst_files={sst} wal_bytes={wal} open_seconds={open_seconds:.3} {}", counted(&context));
        let local_directory = store_dir().join("cost-local");
        let mut local = build_the_local_library(&local_directory);
        let planned = cost_planned_keys();
        for attempt in ["first", "again"] {
            context.reset();
            let started = Instant::now();
            let tally = local.import_the_planned_rows_from(&other, &planned).expect("按计划导入");
            let seconds = started.elapsed().as_secs_f64();
            println!("OPUS_P5 noise={noise_blocks} planned={attempt} seconds={seconds:.4} {} {}", tally_text(&tally), counted(&context));
        }
        drop(local);
        if noise_blocks <= whole_limit {
            let mut whole = build_the_local_library(&local_directory);
            context.reset();
            let started = Instant::now();
            let imported = whole.import_from(&other).expect("整库导入");
            let seconds = started.elapsed().as_secs_f64();
            println!("OPUS_P5 noise={noise_blocks} whole seconds={seconds:.3} imported={imported} {}", counted(&context));
        }
        drop(other);
        let _ = std::fs::remove_dir_all(&fetched);
        let _ = std::fs::remove_dir_all(&local_directory);
    }
}

// ───────────── P4：按计划导入中途被杀，同一条命令重跑补齐 ─────────────

/// 块数与每块状态数：缺省 120 块、每块 65536 个状态（照冻结副本的 64 MiB 批长约 4 批）；rerun.sh 的 resume 阶段在把批长改成 256 KiB 的副本上
/// 取 40 块、每块 4096 个状态（约 16 批），机器满载时也跑得进前台时限。
fn resume_blocks() -> u64 {
    std::env::var("OPUS_RESUME_BLOCKS").ok().and_then(|text| text.parse().ok()).unwrap_or(120)
}
fn resume_states() -> usize {
    std::env::var("OPUS_RESUME_STATES").ok().and_then(|text| text.parse().ok()).unwrap_or(65536)
}
const RESUME_IDENTICAL_IN_LOCAL: u64 = 10;
const RESUME_NODE: &str = "flow-r::node-r::0099";
const RESUME_VECTORS: usize = 40;

fn resume_vector_name(index: usize) -> Vec<u8> {
    if index == 0 { Vec::new() } else { format!("red-{index}").into_bytes() }
}

/// 第 k 块用哪三种判定向量（按名字的序号）：第一次用到某一种的块散在各批里，换编号时新登记的判定向量随不同的批写下去。
fn resume_vector_indexes(block_index: u64) -> [usize; 3] {
    let k = usize::try_from(block_index).expect("小");
    [0, k % (RESUME_VECTORS - 1) + 1, (k * 7) % (RESUME_VECTORS - 1) + 1]
}

/// `number_of_name[i]` 是第 i 种判定向量在这个库里的编号。
fn resume_record(number_of_name: &[VerdictVectorNumber], block_index: u64) -> ConclusionBlockRecord {
    let numbers: Vec<VerdictVectorNumber> = resume_vector_indexes(block_index).iter().map(|index| number_of_name[*index]).collect();
    record(resume_states(), &numbers, 3000 + block_index, true)
}

fn resume_planned_keys() -> PlannedImportKeys {
    let mut planned = PlannedImportKeys {
        conclusions: Vec::new(),
        paths: (0..resume_blocks()).map(|block_index| format!("r.{block_index}")).collect(),
        verifier_blocks: Vec::new(),
    };
    for block_index in 0..resume_blocks() {
        let block_key = key(RESUME_NODE, block_index);
        planned.conclusions.push((block_key.clone(), ConclusionKind::GpuJudge, GPU_VERSION));
        planned.conclusions.push((block_key.clone(), ConclusionKind::CpuJudge, CPU_VERSION));
        planned.verifier_blocks.push((block_key, CPU_VERIFIER));
    }
    planned
}

/// 对方库：120 块 65536 个状态带恢复字的 CPU 结论（一块约 2.2 MiB，按计划导入攒满 64 MiB 写一批，约 4 批），40 种判定向量
/// 按名字序号登记；每块一行核对行；事实行；路径。本机库模板：同样 40 种按倒序登记（编号与对方不同），前 10 块已有、内容与对方相同。
fn resume_libraries() -> (PathBuf, PathBuf) {
    let other_directory = store_dir().join("resume-other");
    let local_template = store_dir().join("resume-local-template");
    let built = store_dir().join("resume.built");
    if built.exists() {
        return (other_directory, local_template);
    }
    let _ = std::fs::remove_dir_all(&other_directory);
    let _ = std::fs::remove_dir_all(&local_template);
    let mut other = VerdictStore::create_empty(&other_directory).expect("建");
    let other_numbers: Vec<VerdictVectorNumber> = (0..RESUME_VECTORS)
        .map(|index| other.register_verdict_vector(&EncodedVerdictVector(resume_vector_name(index))).expect("登记"))
        .collect();
    for block_index in 0..resume_blocks() {
        let block_key = key(RESUME_NODE, block_index);
        other.store_conclusion_block(&block_key, &ConclusionKind::CpuJudge, &CPU_VERSION, &resume_record(&other_numbers, block_index)).expect("存");
        other.store_verifier_block(&block_key, &CPU_VERIFIER, &FLOW_FACTS, &vec![(block_index % 3) as u32; resume_states()]).expect("核对行");
        other.record_path(&format!("r.{block_index}"), &format!("flow-r-identity-{block_index}")).expect("路径");
    }
    other.store_flow_facts(&FLOW_FACTS, &[0x5a; 4096]).expect("事实");
    drop(other);
    let mut local = VerdictStore::create_empty(&local_template).expect("建");
    let mut local_numbers = vec![None; RESUME_VECTORS];
    for index in (0..RESUME_VECTORS).rev() {
        local_numbers[index] = Some(local.register_verdict_vector(&EncodedVerdictVector(resume_vector_name(index))).expect("登记"));
    }
    let local_numbers: Vec<VerdictVectorNumber> = local_numbers.into_iter().map(|number| number.expect("每种都登记了")).collect();
    for block_index in 0..RESUME_IDENTICAL_IN_LOCAL {
        local.store_conclusion_block(&key(RESUME_NODE, block_index), &ConclusionKind::CpuJudge, &CPU_VERSION, &resume_record(&local_numbers, block_index)).expect("存");
    }
    drop(local);
    std::fs::write(&built, "").expect("标记");
    (other_directory, local_template)
}

/// 一块一种结论按内容读回的摘要（编号换成判定向量的内容）；没有是 "absent"，读回报错照原样写出来。
fn resume_conclusion_text(store: &VerdictStore, block_key: &VerdictBlockKey, kind: &ConclusionKind, version: &ConclusionVersion) -> String {
    use std::hash::{Hash, Hasher};
    match store.read_conclusion_block(block_key, kind, version) {
        Err(error) => format!("error {error:?}"),
        Ok(None) => "absent".to_string(),
        Ok(Some(stored)) => {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            for number in stored.block.verdict_vector_number_of_each_state() {
                store.verdict_vector_of_number(*number).expect("登记过").0.hash(&mut hasher);
            }
            for violation in &stored.violations {
                violation.offset_in_block.0.hash(&mut hasher);
                store.verdict_vector_of_number(violation.verdict_vector_number).expect("登记过").0.hash(&mut hasher);
            }
            format!("{:?}", stored.count_vector).hash(&mut hasher);
            format!("{:?}", stored.judged_on).hash(&mut hasher);
            stored.recovery_words.as_ref().map(|words| words.iter().map(|word| word.0).collect::<Vec<_>>()).hash(&mut hasher);
            format!("present {:016x}", hasher.finish())
        }
    }
}

fn resume_readback(directory: &Path) -> Vec<String> {
    let (store, _) = VerdictStore::open_existing(directory).expect("开得了库（判定向量两张表对得上）");
    let mut lines = Vec::new();
    for block_index in 0..resume_blocks() {
        let block_key = key(RESUME_NODE, block_index);
        for (kind, version) in [(ConclusionKind::CpuJudge, CPU_VERSION), (ConclusionKind::GpuJudge, GPU_VERSION)] {
            lines.push(format!("block={block_index} kind={kind:?} {}", resume_conclusion_text(&store, &block_key, &kind, &version)));
        }
        let verifier = store.read_verifier_block(&block_key, &CPU_VERIFIER).map(|row| row.map(|row| (row.facts_digest[0], row.bits.len(), row.bits[0])));
        lines.push(format!("block={block_index} verifier={verifier:?}"));
        lines.push(format!("path r.{block_index} {:?}", store.identity_of_path(&format!("r.{block_index}"))));
    }
    lines.push(format!("facts {:?}", store.read_flow_facts(&FLOW_FACTS).map(|facts| facts.map(|bytes| bytes.len()))));
    lines
}

/// 子进程那一半：只在父进程设了 OPUS_RESUME_CHILD_LOCAL 时做事。开两份库、打一行开始、按计划导入、打一行做完。
#[test]
fn opus_resume_child() {
    let Some(local_directory) = std::env::var_os("OPUS_RESUME_CHILD_LOCAL") else {
        return;
    };
    let (other_directory, _) = resume_libraries();
    let (other, _) = VerdictStore::open_existing(&other_directory).expect("开对方库");
    let (mut local, _) = VerdictStore::open_existing(Path::new(&local_directory)).expect("开本机库");
    let planned = resume_planned_keys();
    use std::io::Write;
    println!("OPUS_CHILD_IMPORT_BEGIN");
    std::io::stdout().flush().expect("flush");
    let tally = local.import_the_planned_rows_from(&other, &planned).expect("按计划导入");
    println!("OPUS_CHILD_IMPORT_DONE {}", tally_text(&tally));
    std::io::stdout().flush().expect("flush");
}

/// 父进程那一半：不断的一趟当参照（量它的挂钟 T），整库导入那一份与它逐项比；再在开始导入之后 f·T 的时刻杀子进程，f 取 OPUS_RESUME_FRACTIONS，
/// 每一次：开库（判定向量两张表对得上才开得了）、逐项读（读得出的与参照相同、读不出的报 absent、读回报错照写）、同一次导入重跑、与参照逐项比。
#[test]
fn opus_resume_after_a_kill() {
    if std::env::var_os("OPUS_STORE_DIR").is_none() || std::env::var_os("OPUS_RESUME_CHILD_LOCAL").is_some() {
        return;
    }
    let (other_directory, local_template) = resume_libraries();
    let planned = resume_planned_keys();
    let reference_directory = store_dir().join("resume-reference");
    let whole_directory = store_dir().join("resume-whole");
    let differing = |left: &[String], right: &[String]| left.iter().zip(right).filter(|(left, right)| left != right).count();
    // 参照只算一次（库与参照都按 OPUS_STORE_DIR 落盘）：机器满载时一趟前台时限里只够杀一两次
    let reference_lines_file = store_dir().join("resume-reference.lines");
    let reference_seconds_file = store_dir().join("resume-reference.seconds");
    if !reference_lines_file.exists() {
        copy_library(&local_template, &reference_directory);
        let full_seconds = {
            let (other, _) = VerdictStore::open_existing(&other_directory).expect("开对方库");
            let (mut local, _) = VerdictStore::open_existing(&reference_directory).expect("开");
            let started = Instant::now();
            let tally = local.import_the_planned_rows_from(&other, &planned).expect("按计划导入");
            let seconds = started.elapsed().as_secs_f64();
            println!("OPUS_P4R reference seconds={seconds:.3} {}", tally_text(&tally));
            seconds
        };
        let reference = resume_readback(&reference_directory);
        copy_library(&local_template, &whole_directory);
        {
            let (other, _) = VerdictStore::open_existing(&other_directory).expect("开对方库");
            let (mut local, _) = VerdictStore::open_existing(&whole_directory).expect("开");
            local.import_from(&other).expect("整库导入");
        }
        let whole = resume_readback(&whole_directory);
        println!(
            "OPUS_P4R reference_lines={} present={} whole_vs_reference={}",
            reference.len(),
            reference.iter().filter(|line| line.contains(" present ")).count(),
            differing(&whole, &reference)
        );
        std::fs::write(&reference_seconds_file, full_seconds.to_string()).expect("写参照挂钟");
        std::fs::write(&reference_lines_file, reference.join("\n")).expect("写参照");
    }
    let full_seconds: f64 = std::fs::read_to_string(&reference_seconds_file).expect("读参照挂钟").trim().parse().expect("数");
    let reference: Vec<String> =
        std::fs::read_to_string(&reference_lines_file).expect("读参照").lines().map(str::to_string).collect();
    let fractions: Vec<f64> = std::env::var("OPUS_RESUME_FRACTIONS")
        .unwrap_or_else(|_| "0.02,0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,0.97,1.5".to_string())
        .split(',')
        .map(|text| text.parse().expect("小数"))
        .collect();
    for fraction in fractions {
        let killed_directory = store_dir().join("resume-killed");
        copy_library(&local_template, &killed_directory);
        let mut child = std::process::Command::new(std::env::current_exe().expect("这个测试二进制"))
            .args(["--exact", "opus_resume_child", "--nocapture", "--test-threads", "1"])
            .env("OPUS_RESUME_CHILD_LOCAL", &killed_directory)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("起子进程");
        let mut stdout = std::io::BufReader::new(child.stdout.take().expect("管道"));
        let mut line = String::new();
        loop {
            line.clear();
            use std::io::BufRead;
            let read = stdout.read_line(&mut line).expect("读子进程");
            assert!(read > 0, "子进程没打开始那一行就退了");
            // libtest 在 --nocapture 下先打「test opus_resume_child ... 」不换行，子进程那一行接在后面
            if line.contains("OPUS_CHILD_IMPORT_BEGIN") {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs_f64(fraction * full_seconds));
        let _ = child.kill();
        let status = child.wait().expect("收子进程");
        let mut rest = String::new();
        use std::io::Read;
        let _ = stdout.read_to_string(&mut rest);
        let finished = rest.contains("OPUS_CHILD_IMPORT_DONE");
        let after_the_kill = resume_readback(&killed_directory);
        let present = after_the_kill.iter().filter(|line| line.contains(" present ")).count();
        let errors = after_the_kill.iter().filter(|line| line.contains(" error ")).count();
        let present_but_different = after_the_kill
            .iter()
            .zip(&reference)
            .filter(|(killed, reference)| killed.contains(" present ") && killed != reference)
            .count();
        let rerun = {
            let (other, _) = VerdictStore::open_existing(&other_directory).expect("开对方库");
            let (mut local, _) = VerdictStore::open_existing(&killed_directory).expect("开被杀之后的本机库");
            local.import_the_planned_rows_from(&other, &planned).map(|tally| tally_text(&tally))
        };
        let after_the_rerun = resume_readback(&killed_directory);
        println!(
            "OPUS_P4R fraction={fraction} child_finished_before_the_kill={finished} status={status} present_after_kill={present} read_errors_after_kill={errors} present_but_different={present_but_different} rerun={rerun:?} rerun_vs_reference={}",
            differing(&after_the_rerun, &reference)
        );
        let _ = std::fs::remove_dir_all(&killed_directory);
    }
    let _ = std::fs::remove_dir_all(&reference_directory);
    let _ = std::fs::remove_dir_all(&whole_directory);
}
