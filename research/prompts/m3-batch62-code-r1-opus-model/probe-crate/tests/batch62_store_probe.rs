//! 攻方原型（m3-batch62-code-r1 opus）：只用判定库的公开接口（加 expose.py 在仓副本里加的三个包装）照 verdict_store.rs 单测的形状造库 A、B，
//! 量第 62 批 Q3 的按计划导入：mirror_* 三条照被判的三条单测逐条断言（一字节一批 28 行 7 批、事实行补写、21 行一批），
//! attack_* 两条是这一条腿加的：一批里登记两种以上新判定向量时，写下去之后内存表与盘上对不对得上、下一批与导入之后再登记的编号撞不撞。
//! 每条先打一行 OPUS_<名> 开头的读数再断言；变异副本上照样跑，读 rc 与读数分辨红在哪一句。
use std::collections::BTreeMap;
use std::path::PathBuf;

use singlefs_checker_tier::crash_identity::{ConclusionKind, ConclusionVersion, ObserverName};
use singlefs_checker_tier::verdict_store::{
    count_vector_bytes, BlockCountVector, BlockStartStateIndex, ConclusionBlockRecord, EncodedVerdictVector,
    EnumerationPlanHash, InputFingerprint, JudgingDevice, PlannedImportKeys, PlannedImportTally, RecoveryWordsOfState,
    StateOffsetInBlock, StreamOrNodeName, VerdictBlock, VerdictBlockKey, VerdictStore, VerdictVectorNumber,
};

const GPU_V: ConclusionVersion = ConclusionVersion([0x51; 32]);
const CPU_V: ConclusionVersion = ConclusionVersion([0x52; 32]);
const OBS_V: ConclusionVersion = ConclusionVersion([0x53; 32]);
const OLD_GPU_V: ConclusionVersion = ConclusionVersion([0x54; 32]);
const CPU_VERIFIER: [u8; 32] = [0x81; 32];
const GPU_VERIFIER: [u8; 32] = [0x82; 32];
const OLD_FACTS: [u8; 32] = [0x71; 32];
const F_FACTS: [u8; 32] = [0x72; 32];
const G_FACTS: [u8; 32] = [0x73; 32];
const F_NODE: &str = "flow-f::node-f::0011";
const G_NODE: &str = "flow-g::node-g::0022";
const STEP: u64 = 4;
const PRODUCT_BATCH: usize = 64 << 20;

/// 库目录：在 TMPDIR 下（rerun.sh 设成工作目录的 tmp/），名字带用例名与进程号，丢掉时删。
struct Dir(PathBuf);
impl Dir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!("opus-b62-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self(path)
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn key(node: &str, index: u64) -> VerdictBlockKey {
    VerdictBlockKey {
        input_fingerprint: InputFingerprint([0x11; 32]),
        stream_or_node_name: StreamOrNodeName::new(node).expect("名字"),
        enumeration_plan_hash: EnumerationPlanHash([0x22; 32]),
        block_start: BlockStartStateIndex(index * STEP),
    }
}
fn f(index: u64) -> VerdictBlockKey {
    key(F_NODE, index)
}
fn g(index: u64) -> VerdictBlockKey {
    key(G_NODE, index)
}
fn vv(bytes: &[u8]) -> EncodedVerdictVector {
    EncodedVerdictVector(bytes.to_vec())
}
fn observer() -> ConclusionKind {
    ConclusionKind::Observer { observer_name: ObserverName::new("mount").expect("名字") }
}
fn reg(store: &mut VerdictStore, bytes: &[u8]) -> VerdictVectorNumber {
    store.register_verdict_vector(&vv(bytes)).expect("登记")
}
/// GPU 判器那样的一块：违例给几个、恢复字逐状态给（被判单测的 conclusion_record 同形：块、违例、计数、属性、恢复字各一行）。
fn gpu_record(numbers: &[VerdictVectorNumber], violating: &[u64]) -> ConclusionBlockRecord {
    let states = u32::try_from(numbers.len()).expect("短");
    ConclusionBlockRecord {
        block: VerdictBlock::new(numbers.to_vec()).expect("至少一个状态"),
        violating_states: violating.iter().copied().map(StateOffsetInBlock).collect(),
        count_vector: BlockCountVector::default(),
        judged_on: JudgingDevice::Gpu { card_type: "probe card".to_string(), driver_version: "probe driver 1".to_string() },
        recovery_words: Some((0..states).map(|s| RecoveryWordsOfState([2, 5 + s, 1, 0, 2, 4, 1, 100 + s])).collect()),
    }
}
/// 观察者那样的一块：不带恢复字（块、违例、计数、属性各一行）。
fn observer_record(numbers: &[VerdictVectorNumber], violating: &[u64]) -> ConclusionBlockRecord {
    ConclusionBlockRecord {
        block: VerdictBlock::new(numbers.to_vec()).expect("至少一个状态"),
        violating_states: violating.iter().copied().map(StateOffsetInBlock).collect(),
        count_vector: BlockCountVector::default(),
        judged_on: JudgingDevice::Cpu,
        recovery_words: None,
    }
}
fn put(store: &mut VerdictStore, k: &VerdictBlockKey, kind: &ConclusionKind, v: &ConclusionVersion, r: &ConclusionBlockRecord) {
    store.store_conclusion_block(k, kind, v, r).expect("存结论");
}

/// 本库 A：照被判单测的 build_the_local_library_a（向量 0 干净、1 red-f；流 F 四块录入；F0 两种判器、F1 的 CPU、F0/F1 旧 GPU 版本、G0；三条路径；旧事实）。
fn build_a(dir: &Dir) -> VerdictStore {
    let mut s = VerdictStore::create_empty(&dir.0).expect("建库");
    let clean = reg(&mut s, b"");
    let red = reg(&mut s, b"red-f");
    for (index, states) in [(0, 2), (1, 2), (2, 3), (3, 2)] {
        s.record_block(&f(index), states).expect("录入");
    }
    s.record_block(&g(0), 2).expect("录入");
    for (kind, v) in [(ConclusionKind::GpuJudge, GPU_V), (ConclusionKind::CpuJudge, CPU_V)] {
        put(&mut s, &f(0), &kind, &v, &gpu_record(&[clean, red], &[1]));
    }
    put(&mut s, &f(1), &ConclusionKind::CpuJudge, &CPU_V, &gpu_record(&[clean, red], &[1]));
    put(&mut s, &f(0), &ConclusionKind::GpuJudge, &OLD_GPU_V, &gpu_record(&[clean, clean], &[]));
    put(&mut s, &f(1), &ConclusionKind::GpuJudge, &OLD_GPU_V, &gpu_record(&[red, clean], &[0]));
    put(&mut s, &g(0), &ConclusionKind::GpuJudge, &GPU_V, &gpu_record(&[clean, clean], &[]));
    for (path, identity) in [("f.0", "flow-f-identity-0"), ("f.0/f.1", "flow-f-identity-1-before-the-fork"), ("g.0", "flow-g-identity-0")] {
        s.record_path(path, identity).expect("路径");
    }
    s.store_flow_facts(&OLD_FACTS, b"facts of an older extractor").expect("事实");
    s
}

/// 对方库 B：照 build_the_other_library_b（A 加上 F1 GPU、F2 GPU 与观察者（新向量 2）、G1；F1、F2、G1 的 CPU 核对行；F、G 的事实；三条路径）。
fn build_b(dir: &Dir) -> VerdictStore {
    let mut s = build_a(dir);
    let clean = reg(&mut s, b"");
    let red = reg(&mut s, b"red-f");
    let only_other = reg(&mut s, b"red-f-judged-on-the-other-host");
    put(&mut s, &f(1), &ConclusionKind::GpuJudge, &GPU_V, &gpu_record(&[clean, red], &[1]));
    put(&mut s, &f(2), &ConclusionKind::GpuJudge, &GPU_V, &gpu_record(&[only_other, clean, clean], &[0]));
    put(&mut s, &f(2), &observer(), &OBS_V, &observer_record(&[only_other, clean, clean], &[0]));
    put(&mut s, &g(1), &ConclusionKind::GpuJudge, &GPU_V, &gpu_record(&[red, clean], &[0]));
    for (k, facts, bits) in [(f(1), F_FACTS, vec![0, 1]), (f(2), F_FACTS, vec![1, 0, 0]), (g(1), G_FACTS, vec![1, 0])] {
        s.store_verifier_block(&k, &CPU_VERIFIER, &facts, &bits).expect("核对行");
    }
    s.store_flow_facts(&F_FACTS, b"facts of flow f").expect("事实");
    s.store_flow_facts(&G_FACTS, b"facts of flow g").expect("事实");
    for (path, identity) in [("f.0/f.1", "flow-f-identity-1-after-the-fork"), ("f.0/f.1/f.2", "flow-f-identity-2"), ("g.0/g.1", "flow-g-identity-1")] {
        s.record_path(path, identity).expect("路径");
    }
    s
}

/// B 再加 F3 的 GPU 结论、用到第二种新向量（照 build_the_other_library_b_with_a_second_new_verdict_vector）。
fn build_b_second(dir: &Dir) -> VerdictStore {
    let mut s = build_b(dir);
    let clean = reg(&mut s, b"");
    let red_three = reg(&mut s, b"red-f-block-three-on-the-other-host");
    put(&mut s, &f(3), &ConclusionKind::GpuJudge, &GPU_V, &gpu_record(&[red_three, clean], &[0]));
    s
}

/// 这一条腿加的形状：B 的 F2 GPU 那一块一块里用到两种 A 没有的向量（x、y），F3 GPU 再用第三种（z）。按计划导入时 x、y 落在同一批，
/// 一字节一批时 z 在下一批：z 的编号要接在 x、y 两种后面。
fn build_b_two_in_one(dir: &Dir) -> VerdictStore {
    let mut s = build_a(dir);
    let clean = reg(&mut s, b"");
    let red = reg(&mut s, b"red-f");
    let x = reg(&mut s, b"red-x-only-on-the-other-host");
    let y = reg(&mut s, b"red-y-only-on-the-other-host");
    let z = reg(&mut s, b"red-z-only-on-the-other-host");
    put(&mut s, &f(1), &ConclusionKind::GpuJudge, &GPU_V, &gpu_record(&[clean, red], &[1]));
    put(&mut s, &f(2), &ConclusionKind::GpuJudge, &GPU_V, &gpu_record(&[x, y, clean], &[0, 1]));
    put(&mut s, &f(3), &ConclusionKind::GpuJudge, &GPU_V, &gpu_record(&[z, clean], &[0]));
    s
}

/// 照 planned_keys_of_flow_f_beyond_this_host_s_share：F1–F3 每块两种判器与观察者、两种核对代码；路径 F 的三个点。
fn planned_f() -> PlannedImportKeys {
    let mut p = PlannedImportKeys {
        conclusions: Vec::new(),
        paths: vec!["f.0".to_string(), "f.0/f.1".to_string(), "f.0/f.1/f.2".to_string()],
        verifier_blocks: Vec::new(),
    };
    for index in 1..4 {
        for (kind, v) in [(ConclusionKind::GpuJudge, GPU_V), (ConclusionKind::CpuJudge, CPU_V), (observer(), OBS_V)] {
            p.conclusions.push((f(index), kind, v));
        }
        for digest in [CPU_VERIFIER, GPU_VERIFIER] {
            p.verifier_blocks.push((f(index), digest));
        }
    }
    p
}

fn kinds() -> [(ConclusionKind, ConclusionVersion); 3] {
    [(ConclusionKind::GpuJudge, GPU_V), (ConclusionKind::CpuJudge, CPU_V), (observer(), OBS_V)]
}

/// 一块一种结论按内容读回（编号换成内存表里的内容；内存表里没有的编号记成 `<未登记 n>`，不 panic，好让读数打出来）。
fn by_content(s: &VerdictStore, k: &VerdictBlockKey, kind: &ConclusionKind, v: &ConclusionVersion) -> Option<String> {
    let content = |n: VerdictVectorNumber| match s.verdict_vector_of_number(n) {
        Some(vector) => String::from_utf8_lossy(&vector.0).to_string(),
        None => format!("<未登记 {}>", n.as_u8()),
    };
    s.read_conclusion_block(k, kind, v).expect("读结论").map(|stored| {
        let states: Vec<String> = stored.block.verdict_vector_number_of_each_state().iter().map(|n| content(*n)).collect();
        let violations: Vec<(u64, String)> =
            stored.violations.iter().map(|vi| (vi.offset_in_block.0, content(vi.verdict_vector_number))).collect();
        format!(
            "states={states:?} violations={violations:?} count={:?} on={:?} recovery={:?}",
            count_vector_bytes(&stored.count_vector),
            stored.judged_on,
            stored.recovery_words
        )
    })
}

fn entries(s: &VerdictStore) -> BTreeMap<Vec<u8>, Vec<u8>> {
    s.opus_every_stored_entry().into_iter().collect()
}

/// 导入前后盘上的差：新出现的键几个、值变了的键几个（与账里的 rows_written 走不同的路数同一件事）。
fn diff(before: &BTreeMap<Vec<u8>, Vec<u8>>, after: &BTreeMap<Vec<u8>, Vec<u8>>) -> (usize, usize) {
    let new_keys = after.keys().filter(|k| !before.contains_key(*k)).count();
    let changed = after.iter().filter(|(k, v)| before.get(*k).is_some_and(|old| old != *v)).count();
    (new_keys, changed)
}

fn tally(
    written: u64, identical: u64, missing: u64, paths: u64, verifiers: u64, facts: u64, rows: u64, synced: u64,
) -> PlannedImportTally {
    PlannedImportTally {
        conclusion_blocks_written: written,
        conclusion_blocks_already_identical: identical,
        conclusion_blocks_missing_in_the_other_library: missing,
        paths_written: paths,
        verifier_blocks_written: verifiers,
        flow_facts_written: facts,
        rows_written: rows,
        synced_writes: synced,
    }
}

/// 被判单测 planned_import_in_one_byte_batches_numbers_new_verdict_vectors_on_across_batches_and_writes_what_one_batch_writes 的逐条照抄
/// （断言次序与绝对值同原文）；另打盘上的差（新键几个、改值几个）与 OPUS_BATCH 逐批的行数（设了 OPUS_TRACE_BATCHES 时）。
#[test]
fn mirror_q3_one_byte_batches() {
    let od = Dir::new("q3-other");
    let other = build_b_second(&od);
    let d1 = Dir::new("q3-one");
    let mut one = build_a(&d1);
    let dm = Dir::new("q3-many");
    let mut many = build_a(&dm);
    let before = entries(&one);
    assert_eq!(before, entries(&many), "两份 A 开头逐字节相同");
    let planned = planned_f();
    eprintln!("OPUS_IMPORT q3 batch=product");
    let t1 = one.import_the_planned_rows_from(&other, &planned).expect("照产品的批长导入");
    eprintln!("OPUS_IMPORT q3 batch=one-byte");
    let tm = many.opus_import_in_batches_of(&other, &planned, 1).expect("一字节一批导入");
    let (new_keys, changed) = diff(&before, &entries(&many));
    println!("OPUS_Q3 product={t1:?}");
    println!("OPUS_Q3 one_byte={tm:?}");
    println!("OPUS_Q3 disk new_keys={new_keys} changed_values={changed} same_as_one_batch={}", entries(&many) == entries(&one));
    assert_eq!(t1, tally(4, 1, 4, 2, 2, 1, 28, 1), "照产品的批长：一批写完");
    assert_eq!(tm, tally(4, 1, 4, 2, 2, 1, 28, 7), "一字节一批：同样的 28 行分 7 批写");
    assert_eq!(entries(&many), entries(&one), "多批与一批写下的库逐字节相同");
    drop((one, many));
    let (reopened, _) = VerdictStore::open_existing(&dm.0).expect("重开多批那一份");
    for (n, expected) in [(2u8, &b"red-f-judged-on-the-other-host"[..]), (3, &b"red-f-block-three-on-the-other-host"[..])] {
        assert_eq!(reopened.opus_vector_of_number_u8(n).as_deref(), Some(expected), "编号 {n}");
    }
    for index in 1..4 {
        for (kind, v) in kinds() {
            assert_eq!(by_content(&reopened, &f(index), &kind, &v), by_content(&other, &f(index), &kind, &v), "F{index} {kind:?}");
        }
    }
}

/// 被判单测 planned_import_writes_the_facts_named_by_verifier_rows_this_library_already_has 的逐条照抄。
#[test]
fn mirror_q3_facts_backfill() {
    let od = Dir::new("facts-other");
    let other = build_b(&od);
    let dl = Dir::new("facts-local");
    let mut local = build_a(&dl);
    for (index, bits) in [(1, vec![0, 1]), (2, vec![1, 0, 0])] {
        local.store_verifier_block(&f(index), &CPU_VERIFIER, &F_FACTS, &bits).expect("写核对行");
        assert_eq!(
            format!("{:?}", local.read_verifier_block(&f(index), &CPU_VERIFIER).expect("读")),
            format!("{:?}", other.read_verifier_block(&f(index), &CPU_VERIFIER).expect("读")),
            "前提：F{index} 的核对行与 B 相同"
        );
    }
    assert_eq!(local.read_flow_facts(&F_FACTS).expect("读事实"), None, "前提：本库没有 F 的事实行");
    let planned = PlannedImportKeys { conclusions: Vec::new(), paths: Vec::new(), verifier_blocks: planned_f().verifier_blocks };
    let t = local.import_the_planned_rows_from(&other, &planned).expect("重导");
    println!("OPUS_FACTS tally={t:?}");
    assert_eq!(t, tally(0, 0, 0, 0, 0, 1, 1, 1), "核对行都已有且相同、一行不写；事实行补写 1 行");
    assert_eq!(local.read_flow_facts(&F_FACTS).expect("读事实"), Some(b"facts of flow f".to_vec()), "F 的事实行补上了");
}

/// 被判单测 importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows 的照抄
/// （整行替换那一行变异点名的那一条）：读回与整库导入相同、账 21 行 1 批、写下的正是 B 比 A 多出属于 F 的行。
#[test]
fn mirror_21_rows() {
    let od = Dir::new("r21-other");
    let other = build_b(&od);
    let dw = Dir::new("r21-whole");
    let mut whole = build_a(&dw);
    let dp = Dir::new("r21-plan");
    let mut by_plan = build_a(&dp);
    let before = entries(&by_plan);
    assert_eq!(before, entries(&whole), "两份 A 开头逐字节相同");
    whole.import_from(&other).expect("整库导入");
    let t = by_plan.import_the_planned_rows_from(&other, &planned_f()).expect("按计划导入");
    println!("OPUS_R21 tally={t:?}");
    for index in 0..4 {
        for (kind, v) in kinds().into_iter().chain([(ConclusionKind::GpuJudge, OLD_GPU_V)]) {
            assert_eq!(by_content(&by_plan, &f(index), &kind, &v), by_content(&whole, &f(index), &kind, &v), "F{index} {kind:?}");
        }
        for digest in [CPU_VERIFIER, GPU_VERIFIER] {
            assert_eq!(
                format!("{:?}", by_plan.read_verifier_block(&f(index), &digest).expect("读")),
                format!("{:?}", whole.read_verifier_block(&f(index), &digest).expect("读")),
                "F{index} 核对行"
            );
        }
    }
    for path in ["f.0", "f.0/f.1", "f.0/f.1/f.2"] {
        assert_eq!(by_plan.identity_of_path(path).expect("读"), whole.identity_of_path(path).expect("读"), "路径 {path}");
    }
    assert_eq!(by_plan.read_flow_facts(&F_FACTS).expect("读"), whole.read_flow_facts(&F_FACTS).expect("读"), "事实");
    assert_eq!(t, tally(3, 1, 5, 2, 2, 1, 21, 1), "按计划导入的账");
    let is_g = |k: &[u8]| {
        [G_NODE.as_bytes(), b"g.0/g.1".as_slice(), G_FACTS.as_slice()].iter().any(|needle| k.windows(needle.len()).any(|w| w == *needle))
    };
    let written: Vec<_> = entries(&by_plan).into_iter().filter(|e| before.get(&e.0) != Some(&e.1)).collect();
    let beyond: Vec<_> = entries(&other).into_iter().filter(|e| before.get(&e.0) != Some(&e.1) && !is_g(&e.0)).collect();
    println!("OPUS_R21 written={} other_beyond_a_of_f={}", written.len(), beyond.len());
    assert_eq!(written.len(), 21, "写下 21 行");
    assert_eq!(written, beyond, "写下的正是 B 比 A 多出、属于 F 的那几行");
}

/// 攻：照产品的批长从 B（加 F3）导入，A2、A3 两种新向量落在同一批。导入之后同一个进程接着用这份库（流水线里导入在第 ① 段判之前：
/// 判本机的块、观察者存结论都经内存表登记向量、按内存表读编号），不重开：按内存表读回 F1–F3 每种结论与 B 比；再登记一种新向量，
/// 编号要是 4（盘上已有 0–3）；重开之后 F1–F3 与 B 比、那一种新向量读得回。
#[test]
fn attack_in_memory_table_after_a_product_batch_import() {
    let od = Dir::new("mem-other");
    let other = build_b_second(&od);
    let dl = Dir::new("mem-local");
    let mut local = build_a(&dl);
    let t = local.import_the_planned_rows_from(&other, &planned_f()).expect("照产品的批长导入");
    let in_memory: Vec<Option<String>> =
        (0..5u8).map(|n| local.opus_vector_of_number_u8(n).map(|b| String::from_utf8_lossy(&b).to_string())).collect();
    println!("OPUS_MEM tally={t:?}");
    println!("OPUS_MEM in_memory_numbers_0_to_4={in_memory:?}");
    let mut before_reopen = 0;
    for index in 1..4 {
        for (kind, v) in kinds() {
            if by_content(&local, &f(index), &kind, &v) != by_content(&other, &f(index), &kind, &v) {
                before_reopen += 1;
                println!("OPUS_MEM differs_before_reopen F{index} {kind:?} local={:?}", by_content(&local, &f(index), &kind, &v));
            }
        }
    }
    let fresh = reg(&mut local, b"registered-after-the-import");
    println!("OPUS_MEM fresh_number={}", fresh.as_u8());
    drop(local);
    let (reopened, _) = VerdictStore::open_existing(&dl.0).expect("重开");
    let mut after_reopen = 0;
    for index in 1..4 {
        for (kind, v) in kinds() {
            if by_content(&reopened, &f(index), &kind, &v) != by_content(&other, &f(index), &kind, &v) {
                after_reopen += 1;
                println!("OPUS_MEM differs_after_reopen F{index} {kind:?} local={:?}", by_content(&reopened, &f(index), &kind, &v));
            }
        }
    }
    let fresh_back = reopened.opus_vector_of_number_u8(fresh.as_u8()).map(|b| String::from_utf8_lossy(&b).to_string());
    println!("OPUS_MEM differs_before_reopen={before_reopen} differs_after_reopen={after_reopen} fresh_read_back={fresh_back:?}");
    assert_eq!(
        (before_reopen, fresh.as_u8(), after_reopen, fresh_back.as_deref()),
        (0, 4, 0, Some("registered-after-the-import")),
        "导入之后内存表与盘上一致、再登记的编号接在后面、不盖掉导进来的向量"
    );
}

/// 攻：B 的 F2 GPU 一块里用两种 A 没有的向量（x、y，同一批登记），F3 GPU 再用第三种（z）。一字节一批时 x、y 一批、z 下一批，
/// 与照产品的批长一批写完的那份逐字节比；重开之后编号 2、3、4 是 x、y、z，F1–F3 按内容读回与 B 相同。
#[test]
fn attack_two_new_vectors_in_one_batch_then_one_more_batch() {
    let od = Dir::new("two-other");
    let other = build_b_two_in_one(&od);
    let d1 = Dir::new("two-one");
    let mut one = build_a(&d1);
    let dm = Dir::new("two-many");
    let mut many = build_a(&dm);
    eprintln!("OPUS_IMPORT two batch=product");
    let t1 = one.import_the_planned_rows_from(&other, &planned_f()).expect("照产品的批长导入");
    eprintln!("OPUS_IMPORT two batch=one-byte");
    let tm = many.opus_import_in_batches_of(&other, &planned_f(), 1).expect("一字节一批导入");
    let same = entries(&many) == entries(&one);
    println!("OPUS_TWO product={t1:?}");
    println!("OPUS_TWO one_byte={tm:?}");
    println!("OPUS_TWO same_bytes_as_one_batch={same}");
    drop((one, many));
    let (reopened, _) = VerdictStore::open_existing(&dm.0).expect("重开多批那一份");
    let numbers: Vec<Option<String>> =
        (2..5u8).map(|n| reopened.opus_vector_of_number_u8(n).map(|b| String::from_utf8_lossy(&b).to_string())).collect();
    let mut differs = 0;
    for index in 1..4 {
        for (kind, v) in kinds() {
            if by_content(&reopened, &f(index), &kind, &v) != by_content(&other, &f(index), &kind, &v) {
                differs += 1;
                println!("OPUS_TWO differs F{index} {kind:?} local={:?}", by_content(&reopened, &f(index), &kind, &v));
            }
        }
    }
    println!("OPUS_TWO numbers_2_to_4={numbers:?} differs_after_reopen={differs}");
    assert!(same, "多批与一批写下的库逐字节相同");
    assert_eq!(
        numbers,
        vec![
            Some("red-x-only-on-the-other-host".to_string()),
            Some("red-y-only-on-the-other-host".to_string()),
            Some("red-z-only-on-the-other-host".to_string())
        ],
        "编号 2、3 是同一批登记的 x、y，4 是下一批的 z"
    );
    assert_eq!(differs, 0, "F1–F3 按内容读回与 B 相同");
}

/// 攻（放开批长扫）：两份对方库（B 加 F3、一块两种新向量那一份）× 批长 1–40 字节逐个、再 64 字节到 8 KiB 每档乘 1.5，
/// 每档从同一份关好的 A（建一次、逐档拷目录再开）照这个批长导入，与照产品批长导入的那一份比盘上字节，重开之后 F1–F3 按内容读回
/// 与对方库比；打每份对方库走过几档、见过几种批数、几档不一致，不一致的档逐个打出来。库目录在 TMPDIR 下（rerun.sh 给这一条设 tmpfs）。
#[test]
fn sweep_batch_lengths_against_one_batch() {
    let mut lengths: Vec<usize> = (1..=40).collect();
    let mut next = 64.0f64;
    while next <= 8192.0 {
        lengths.push(next as usize);
        next *= 1.5;
    }
    let template = Dir::new("sweep-template-a");
    drop(build_a(&template));
    let copy_of_a = |to: &Dir| {
        std::fs::create_dir_all(&to.0).expect("建目录");
        for entry in std::fs::read_dir(&template.0).expect("读目录") {
            let entry = entry.expect("一项");
            if entry.file_type().expect("类型").is_file() {
                std::fs::copy(entry.path(), to.0.join(entry.file_name())).expect("拷");
            }
        }
        VerdictStore::open_existing(&to.0).expect("开拷来的 A").0
    };
    for (shape, build) in [("b-second", build_b_second as fn(&Dir) -> VerdictStore), ("two-in-one", build_b_two_in_one)] {
        let od = Dir::new(&format!("sweep-other-{shape}"));
        let other = build(&od);
        let dr = Dir::new(&format!("sweep-ref-{shape}"));
        let mut reference = copy_of_a(&dr);
        reference.import_the_planned_rows_from(&other, &planned_f()).expect("照产品的批长导入");
        let reference_entries = entries(&reference);
        let mut batch_counts = std::collections::BTreeSet::new();
        let mut differs = Vec::new();
        for length in &lengths {
            let dl = Dir::new(&format!("sweep-{shape}-{length}"));
            let mut local = copy_of_a(&dl);
            let t = local.opus_import_in_batches_of(&other, &planned_f(), *length).expect("照这个批长导入");
            batch_counts.insert(t.synced_writes);
            let same_bytes = entries(&local) == reference_entries;
            drop(local);
            let (reopened, _) = VerdictStore::open_existing(&dl.0).expect("重开");
            let mut read_back_differs = 0;
            for index in 1..4 {
                for (kind, v) in kinds() {
                    if by_content(&reopened, &f(index), &kind, &v) != by_content(&other, &f(index), &kind, &v) {
                        read_back_differs += 1;
                    }
                }
            }
            if !same_bytes || read_back_differs > 0 {
                differs.push((*length, t.synced_writes, same_bytes, read_back_differs));
            }
        }
        println!(
            "OPUS_SWEEP shape={shape} lengths={} batch_counts_seen={batch_counts:?} differing={}",
            lengths.len(),
            differs.len()
        );
        for row in &differs {
            println!("OPUS_SWEEP shape={shape} differs length={} synced={} same_bytes={} read_back_differs={}", row.0, row.1, row.2, row.3);
        }
        assert!(differs.is_empty(), "{shape}：有批长下与一批写完的那份不同");
    }
}
