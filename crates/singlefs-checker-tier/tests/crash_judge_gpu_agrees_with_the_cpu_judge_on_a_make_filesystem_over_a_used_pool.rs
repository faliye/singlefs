//! checker 档模块：crash、crash_state_enumeration、crash_identity、crash_library、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器与 CPU 判器在带清零写的流上逐状态对拍（C656（mkfs 中途崩溃的语义没有条款） 的崩溃枚举用例上卡之前，判表照旧拒清零写）：
//! 用过的盘（旧池：mkfs → 取号 1 → 暖机 → 第一个文件）上经内层入口拿前 8 字节不同的 fsid 重做 mkfs，录下的 27 步按甲二展开，
//! 每个状态 GPU 判出的对拍字（`crash_judge_gpu::CrossCheckWords`：红项、两遍恢复各自落到的根、结局与失败点、池级 checker 逐条违例位与判到位、
//! I-1.13 的结局码、GPU 判不了）与 CPU 判器从判定库读回的同一条流上判出的逐字段相同，一个状态都不许判不了。
//! 这条流前两段十次全是清零写（两块盘的 [0, 根环基址)、三个根环区域与整个 journal 环），罩住旧池的系统配置槽、根槽与 journal 记录槽，
//! 判表照落点模型把它们（连同撕裂镜像）记进罩住的落点的写；漏记一处，作废之后那一槽的版本照旧是旧池的字节，GPU 读回旧池、CPU 整池拒，逐状态比那一句红。
//! 两边的输入都经对拍那一类的库（判表入库再读回上卡；CPU 那一侧流先入库、再读回判）；两边判出的东西作一份对拍结论入库
//! （`crash_library::pairing_conclusion_reused_or_judged`，键是这条流的输入摘要与两边判法单元的键），比法与断言每趟照读回的那一份现比、不进键。
//! 要本机一张 NVIDIA 独显：一张适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。
//! 这是 GPU 判器（加速路径）与 CPU 判器（参照路径）的对拍，照 `.claude/rules/test-script-discipline.md`「分快慢两档」一律进快档，不标 `#[ignore]`。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::num::NonZeroUsize;
use std::sync::Mutex;

use common::{build_pool, crash_state_devices, file_content, geometry, parameters};
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, quick_tier_expansion, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_identity::{judging_units_key, JudgingUnits, SectorImage};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_judge_gpu::{
    cross_check_words_compared_by_the_cross_check, cross_check_words_of_cpu_judgement,
    cross_check_words_of_gpu_verdict, CrossCheckWords, LandedRootWords, VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{
    build_judge_tables_against_the_version_table, JudgeFlow, JudgeTables,
};
use singlefs_checker_tier::crash_library::{
    pairing_conclusion_reused_or_judged, toolchain_of_this_process, CrashLibrary, CrashLibraryClass,
};
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu_with_threads_against_the_version_table, CrashPointReplayItem,
};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    gpu_card_pool_or_report_not_run, GpuCardPool,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_harness::heavy_threads::{
    own_variable_or_default_heavy_threads, registered_heavy_threads,
    THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, PublishedVersion, RetainedWrite, WrittenContents,
};
use singlefs_harness::numeric_environment_variable::process_environment_variable;
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_harness::SharedStream;

/// 这份测试文件的仓内路径：CPU 那一侧取样的设点文件。
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_a_make_filesystem_over_a_used_pool.rs";

/// 两边过库的项名（判表那一侧与 CPU 流那一侧各在自己那一类库里用它）。
const ITEM_NAME: &str = "make_filesystem_over_a_used_pool_gpu_and_cpu_pairing";

/// 一份对拍字按对拍比的那几个字段摊成的字节数（[`compared_cross_check_bytes`]）：一个 u32 红位、两组 u128 位、两遍落到的根各四个 u32
/// （`LandedRootWords::words`：标记、实例代号、checkpoint_txg 低 32 位与高 32 位）与两遍的结局、失败点四个 u32、一个 u32 结局码、一个字节的判不了。
const COMPARED_CROSS_CHECK_BYTES: usize = 4 + 16 + 4 * (2 * 4 + 4) + 16 + 4 + 1;

/// 一个字几个字节（GPU 结论按 u32 字排）。
const BYTES_PER_WORD: usize = 4;

/// CPU 那一侧判器的线程数变量（`.claude/rules/verification.md`「两档：轻型与重型」里「GPU 对拍与内存对拍不压并发」那一条：
/// GPU 对拍测试的 CPU 一侧读它，不读崩溃点重放判段的 `SINGLEFS_CRASH_POINT_REPLAY_THREADS`）。
const CPU_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CPU_THREADS";

/// CPU 判器线程数变量的值（`None` 是没设）：设了照它，没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP` 的值，
/// 设了照它，再没设取 `.claude/heavy-thread-default` 的登记值，判法在 `singlefs_harness::heavy_threads`）；设了而不是正整数（0 也算）
/// 就 panic、消息带变量名与原值，不悄悄换成至少 1 或默认。
///
/// # Panics
/// 设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
fn cpu_threads_from_the_value_of_the_variable(
    value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> NonZeroUsize {
    own_variable_or_default_heavy_threads(CPU_THREADS_ENVIRONMENT_VARIABLE, value, thread_cap_value)
}

/// 这一趟进程环境里 CPU 判器的线程数（[`cpu_threads_from_the_value_of_the_variable`] 读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS` 与
/// `SINGLEFS_THREAD_CAP` 两个变量）。
fn cpu_threads_of_this_process() -> NonZeroUsize {
    cpu_threads_from_the_value_of_the_variable(
        process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// CPU 判器线程数变量没设：取登记的每台默认重活线程数，不取本机核数；只设了 `SINGLEFS_THREAD_CAP` 照它；两个都设了照 CPU 判器自己的变量。
#[test]
fn crash_judge_cpu_threads_take_the_registered_heavy_thread_default_when_unset() {
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(None, None),
        registered_heavy_threads(),
        "SINGLEFS_CRASH_JUDGE_CPU_THREADS 与 SINGLEFS_THREAD_CAP 都没设取 .claude/heavy-thread-default 的登记值"
    );
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(None, Some("5")).get(),
        5,
        "SINGLEFS_CRASH_JUDGE_CPU_THREADS 没设、SINGLEFS_THREAD_CAP=5：照 SINGLEFS_THREAD_CAP"
    );
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(Some("3"), Some("5")).get(),
        3,
        "SINGLEFS_CRASH_JUDGE_CPU_THREADS=3、SINGLEFS_THREAD_CAP=5：照 CPU 判器自己的变量"
    );
}

/// CPU 判器线程数变量写成 0：panic，消息点名的是 `SINGLEFS_CRASH_JUDGE_CPU_THREADS` 与原值（读的是这个变量，不是崩溃点重放判段的）。
#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_CPU_THREADS=\"0\" 认不出")]
fn a_zero_crash_judge_cpu_threads_value_panics_naming_the_variable_and_the_value() {
    let _threads = cpu_threads_from_the_value_of_the_variable(Some("0"), None);
}

/// CPU 那一侧取线程不走崩溃点重放判段的读法（那一个读 `SINGLEFS_CRASH_POINT_REPLAY_THREADS`，GPU 对拍的 CPU 一侧不读它）：
/// 这份文件里一处都不调它。读源文件判：取线程的调用点在要卡的那条对拍用例里，不上卡的用例够不着它。
#[test]
fn the_cpu_side_does_not_take_its_threads_from_the_crash_point_replay_reading() {
    let source_path = format!("{}/../../{THIS_FILE}", env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|error| panic!("读不了这份测试自己的源文件 {source_path}：{error}"));
    let crash_point_replay_reading = ["crash_point_replay", "worker_threads"].join("_");
    assert!(
        !source.contains(&crash_point_replay_reading),
        "这份文件调了 {crash_point_replay_reading}：GPU 对拍的 CPU 一侧要读 SINGLEFS_CRASH_JUDGE_CPU_THREADS（cpu_threads_of_this_process）"
    );
}

struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    /// 旧池最后确认的一版（oracle 的版本表里只有它一版有文件）。
    versions: Vec<PublishedVersion>,
}

/// 用过的盘上经内层入口重做 mkfs（fsid 前 8 字节写成 `remade-x`，与旧池不同），录下的写与段；基线是旧池用完之后的整份盘面。
fn record_make_filesystem_over_a_used_pool() -> RecordedFlow {
    let old_pool = build_pool("crash-judge-gpu-make-filesystem-over-a-used-pool");
    let versions = vec![PublishedVersion {
        instance: old_pool.output.root.instance,
        checkpoint_txg: old_pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let base = old_pool.memory_pool();
    let common_parameters = parameters();
    let mut filesystem_identifier = common_parameters.filesystem_identifier;
    filesystem_identifier[..8].copy_from_slice(b"remade-x");
    let remade_parameters = MakeFilesystemParameters {
        filesystem_identifier,
        region_devices: common_parameters.region_devices,
        geometry: common_parameters.geometry,
    };
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&base, &[], &[], &stream);
    make_filesystem(&remade_parameters, &mut devices).expect("用过的盘上重做 mkfs");
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        vec![2, 12, 1, 1, 1, 4],
        "两趟写序的段序列"
    );
    assert_eq!(
        writes
            .iter()
            .filter(|write| write.kind == StepKind::ZeroFill)
            .count(),
        10,
        "前两段里十次清零"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("mkfs 写第 0 代根");
    RecordedFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

/// 一份对拍字在对拍里比的那一份（[`cross_check_words_compared_by_the_cross_check`]）摊成字节，字段次序照 `CrossCheckWords` 的声明。
fn compared_cross_check_bytes(words: CrossCheckWords) -> Vec<u8> {
    let compared = cross_check_words_compared_by_the_cross_check(words);
    let landed = |root: LandedRootWords| root.words();
    let mut bytes = Vec::with_capacity(COMPARED_CROSS_CHECK_BYTES);
    bytes.extend_from_slice(&compared.red_bits.to_le_bytes());
    bytes.extend_from_slice(&compared.pool_checker_violated_bits.to_le_bytes());
    for word in landed(compared.consulted_pass_landed_root)
        .into_iter()
        .chain(landed(compared.ignored_pass_landed_root))
        .chain([
            compared.consulted_pass_outcome,
            compared.ignored_pass_outcome,
            compared.consulted_pass_failure_point,
            compared.ignored_pass_failure_point,
        ])
    {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes.extend_from_slice(&compared.pool_checker_judged_bits.to_le_bytes());
    bytes.extend_from_slice(&compared.scan_rebuild_outcome.to_le_bytes());
    bytes.push(u8::from(compared.gpu_could_not_judge));
    assert_eq!(bytes.len(), COMPARED_CROSS_CHECK_BYTES);
    bytes
}

/// 这条流对拍的输入摘要：基线按扇区终值、每次写（盘、种类、偏移、FUA、内容摘要）、切段、甲二、被判的根、版本表与判表的摘要。只按内容取，项名不进。
fn pairing_inputs_digest(flow: &RecordedFlow, tables: &JudgeTables) -> [u8; DIGEST_BYTES] {
    let mut text = format!(
        "gpu and cpu judge pairing on zero fills inputs\nbase={}\n",
        hexadecimal_text(&SectorImage::of_base(&flow.base).digest())
    );
    for write in &flow.writes {
        let contents = match &write.contents {
            WrittenContents::Bytes(bytes) => hexadecimal_text(&sha256_digest(bytes)),
            WrittenContents::Zeros { length } => format!("zeros:{length}"),
        };
        text.push_str(&format!(
            "write device={} kind={:?} offset={} force_unit_access={} contents={contents}\n",
            write.device.0, write.kind, write.offset.0, write.is_force_unit_access
        ));
    }
    for (segment_index, segment) in flow.segments.iter().enumerate() {
        text.push_str(&format!(
            "segment {segment:?} expansion={}\n",
            quick_tier_expansion(segment_index, segment).name()
        ));
    }
    text.push_str(&format!(
        "judged_root_index={}\nversions={:?}\ntables={}\n",
        flow.judged_root_index,
        flow.versions,
        hexadecimal_text(&tables.digest())
    ));
    sha256_digest(text.as_bytes())
}

/// 两边各判一遍，交回对拍结论的字节：状态数（u64），每个状态 GPU 的结论字，每个状态 CPU 那一侧对拍比的那一份字节。
/// 只录那一趟、卡借不到交 `None`。
// cpu-reference-for-gpu-comparison
fn judgements_of_both_judges(
    cards: &'static GpuCardPool,
    flow: &RecordedFlow,
    tables: &JudgeTables,
) -> Option<Vec<u8>> {
    let threads = cpu_threads_of_this_process();
    let cpu_bytes_by_ordinal: Mutex<Vec<(u64, Vec<u8>)>> = Mutex::new(Vec::new());
    let cpu_judged = sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
        CrashPointReplayItem {
            name: ITEM_NAME,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&flow.versions),
        &quick_tier_expansion,
        0..tables.state_count,
        threads,
        &|ordinal, _image, judgement| {
            cpu_bytes_by_ordinal
                .lock()
                .expect("判定线程不 panic")
                .push((
                    ordinal,
                    compared_cross_check_bytes(cross_check_words_of_cpu_judgement(judgement)),
                ));
        },
    )
    .judged()?;
    assert_eq!(cpu_judged, tables.state_count, "CPU 那一侧判的是整条流");
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, ITEM_NAME, tables)?;
    let mut cpu_bytes_by_ordinal = cpu_bytes_by_ordinal.into_inner().expect("判定线程不 panic");
    cpu_bytes_by_ordinal.sort_by_key(|(ordinal, _)| *ordinal);
    assert_eq!(
        cpu_bytes_by_ordinal
            .iter()
            .map(|(ordinal, _)| *ordinal)
            .collect::<Vec<u64>>(),
        (0..tables.state_count).collect::<Vec<u64>>(),
        "CPU 那一侧每个状态判一次"
    );
    assert_eq!(
        u64::try_from(gpu.len()).expect("装得进 u64"),
        tables.state_count,
        "GPU 那一侧每个状态一份结论"
    );
    let mut conclusion = tables.state_count.to_le_bytes().to_vec();
    for words in &gpu {
        for word in words {
            conclusion.extend_from_slice(&word.to_le_bytes());
        }
    }
    for (_, bytes) in cpu_bytes_by_ordinal {
        conclusion.extend_from_slice(&bytes);
    }
    Some(conclusion)
}

/// 一份对拍结论拆回的两边：每个状态 GPU 的结论字，每个状态 CPU 那一侧对拍比的那一份字节（[`compared_cross_check_bytes`]），都按序号排。
struct JudgementsOfBothJudges {
    gpu_verdict_words: Vec<[u32; VERDICT_WORDS]>,
    cpu_compared_cross_check_bytes: Vec<Vec<u8>>,
}

/// 对拍结论按 [`judgements_of_both_judges`] 的布局拆回；长度对不上交 `None`。
fn both_judgements_from_the_conclusion(conclusion: &[u8]) -> Option<JudgementsOfBothJudges> {
    let (state_count_bytes, rest) = conclusion.split_first_chunk::<8>()?;
    let state_count = usize::try_from(u64::from_le_bytes(*state_count_bytes)).ok()?;
    let gpu_bytes = state_count.checked_mul(VERDICT_WORDS * BYTES_PER_WORD)?;
    let cpu_bytes = state_count.checked_mul(COMPARED_CROSS_CHECK_BYTES)?;
    if rest.len() != gpu_bytes.checked_add(cpu_bytes)? {
        return None;
    }
    let (gpu_part, cpu_part) = rest.split_at(gpu_bytes);
    let (gpu_states, _) = gpu_part.as_chunks::<{ VERDICT_WORDS * BYTES_PER_WORD }>();
    let gpu_verdict_words = gpu_states
        .iter()
        .map(|state| {
            let (word_bytes, _) = state.as_chunks::<BYTES_PER_WORD>();
            let mut words = [0u32; VERDICT_WORDS];
            for (word, bytes) in words.iter_mut().zip(word_bytes) {
                *word = u32::from_le_bytes(*bytes);
            }
            words
        })
        .collect();
    let (cpu_states, _) = cpu_part.as_chunks::<COMPARED_CROSS_CHECK_BYTES>();
    let cpu_compared_cross_check_bytes = cpu_states.iter().map(|state| state.to_vec()).collect();
    Some(JudgementsOfBothJudges {
        gpu_verdict_words,
        cpu_compared_cross_check_bytes,
    })
}

/// 判那一侧报了「本次未跑」（卡池借不到卡、只录那一趟）：对拍结论不存进库，当场收手。
struct PairingNotRun;

/// 用过的盘上重做 mkfs 那条流按甲二展开的每个状态，GPU 判器的对拍字与 CPU 判器逐字段相同、一个状态都不判不了；
/// 两边的结局里旧池读回最后确认的一版、整池拒、第 0 代空池三种都出现（这条流真把清零写摆到了判器面前）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_quick_tier_state_of_a_make_filesystem_over_a_used_pool(
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_every_quick_tier_state_of_a_make_filesystem_over_a_used_pool",
    ) else {
        return;
    };
    let flow = record_make_filesystem_over_a_used_pool();
    let tables = build_judge_tables_against_the_version_table(
        &JudgeFlow {
            base: &flow.base,
            writes: &flow.writes,
            segments: &flow.segments,
            judged_root_index: flow.judged_root_index,
            versions: &flow.versions,
            expansion: &quick_tier_expansion,
        },
        &[],
    )
    .unwrap_or_else(|refusal| panic!("带清零写的流建不出判表（GPU 判不了）：{refusal:?}"));
    assert_eq!(
        tables.state_count,
        crash_point_replay_plan_state_count(
            &flow.base,
            &flow.writes,
            &flow.segments,
            &quick_tier_expansion
        ),
        "判表的状态数就是甲二展开的崩溃点重放枚举域"
    );
    let zero_fill_entries: Vec<u32> = tables
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::ZeroFill)
        .map(|(entry, _)| u32::try_from(entry).expect("装得进 u32"))
        .collect();
    assert!(
        tables.locations.iter().any(|location| location
            .writes
            .iter()
            .any(|entry| zero_fill_entries.contains(entry))),
        "清零写罩住旧池的落点、记进它们的写"
    );
    println!(
        "CRASH_JUDGE_TABLES flow=make_filesystem_over_a_used_pool states={} locations={} most_writes_on_one_location={}",
        tables.state_count,
        tables.locations.len(),
        tables
            .locations
            .iter()
            .map(|location| location.writes.len())
            .max()
            .unwrap_or(0)
    );
    let judging_units = |units: JudgingUnits| {
        judging_units_key(&units, toolchain_of_this_process()).map_err(|refused| refused.sentence())
    };
    let library =
        CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("对拍那一类的库开得了");
    let paired = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pairing_conclusion_reused_or_judged(
            &library,
            &pairing_inputs_digest(&flow, &tables),
            &judging_units(JudgingUnits::GpuJudge),
            &judging_units(JudgingUnits::JudgementOfTheCrashImages),
            || {
                judgements_of_both_judges(cards, &flow, &tables)
                    .unwrap_or_else(|| std::panic::panic_any(PairingNotRun))
            },
        )
    }));
    let (conclusion, source) = match paired {
        Ok(Ok(paired)) => paired,
        Ok(Err(reason)) => panic!("对拍结论读写库失败：{reason}"),
        Err(payload) if payload.is::<PairingNotRun>() => return,
        Err(payload) => std::panic::resume_unwind(payload),
    };
    let judgements = both_judgements_from_the_conclusion(&conclusion)
        .expect("库里这一键下的对拍结论是这份测试写的，拆得回");
    assert_eq!(
        (
            u64::try_from(judgements.gpu_verdict_words.len()).expect("装得进 u64"),
            u64::try_from(judgements.cpu_compared_cross_check_bytes.len()).expect("装得进 u64"),
        ),
        (tables.state_count, tables.state_count),
        "两边每个状态各一份"
    );
    let mut mismatches: Vec<String> = Vec::new();
    let mut outcomes_seen = [false; 3];
    let old_version = LandedRootWords::of_the_landed_root(
        flow.versions[0].instance,
        flow.versions[0].checkpoint_txg,
    );
    let generation_zero =
        LandedRootWords::of_the_landed_root(InstanceGeneration(0), CheckpointTxg(0));
    for (ordinal, (gpu_words, cpu_bytes)) in judgements
        .gpu_verdict_words
        .iter()
        .zip(&judgements.cpu_compared_cross_check_bytes)
        .enumerate()
    {
        let gpu_cross_check = cross_check_words_of_gpu_verdict(gpu_words);
        if compared_cross_check_bytes(gpu_cross_check) != *cpu_bytes {
            mismatches.push(format!(
                "状态 {ordinal}：GPU {gpu_cross_check:?}；CPU 字节 {}",
                hexadecimal_text(cpu_bytes)
            ));
        }
        let landed = gpu_cross_check.consulted_pass_landed_root;
        if landed == old_version {
            outcomes_seen[0] = true;
        } else if landed == LandedRootWords::NOT_LANDED {
            outcomes_seen[1] = true;
        } else if landed == generation_zero {
            outcomes_seen[2] = true;
        }
    }
    println!(
        "CRASH_JUDGE_GPU_ZERO_FILL_PAIRING states={} mismatches={} source={source:?}",
        judgements.gpu_verdict_words.len(),
        mismatches.len()
    );
    assert!(
        mismatches.is_empty(),
        "GPU 判器与 CPU 判器在 {} 个状态上对拍字不同（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    );
    assert_eq!(
        outcomes_seen,
        [true, true, true],
        "读回旧池最后确认的一版、整池拒（没落到根）、第 0 代空池三种结局都出现"
    );
}
