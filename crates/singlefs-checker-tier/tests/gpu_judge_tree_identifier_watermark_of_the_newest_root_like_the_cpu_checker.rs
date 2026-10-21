//! checker 档模块：crash、crash_state_enumeration、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器里池级 checker 那一份的 I-7.8（根记录树 ID 水位不低于全池最大树 ID）判两格，与 CPU 的池级 checker 同判（C674）：
//! 根环全部根的水位取 max 那一格，与最新那条根自己的水位那一格（`crates/singlefs-checker/src/walk.rs` 的 `check_pool_image` 末尾）。
//! 造法：新池新建文件（发布 A，发出十二棵树）再覆盖写一次（发布 B）那条流，把 B 的根（最新那条）的水位压到 mkfs 时的水位、重封，
//! B 那次的 journal 记录的新根段跟着压（同一份新根段，I-8.10（记录新根段与同代根相同） 不跟着红）。A 的根水位与 B 原样相同、
//! 照旧盖得住 A 发的树 ⇒ 根环取 max 不变、取 max 那一格不红，只有最新那条根自己那一格红（前提在取卡之前用 CPU 判；
//! 与 harness 档 `checker_known_bad_images.rs` 里 I-7.8 最新那条根那一格的坏镜像同一个坏法，那一份钉 CPU 判器、这一份钉 GPU 判器）。
//! B 的根落了的状态上 CPU 判 I-7.8 红，GPU 与 CPU 逐状态的违例位要相同；GPU 只判取 max 那一格时这些状态上少一位 I-7.8。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `gpu` 特性。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;

use common::{
    build_pool, file_content, geometry, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, CONTENT_BYTES_OF_PUBLISH_B,
};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion, CrashStateJudgement,
};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};

/// 这份测试文件的仓内路径：崩溃点重放取样过库的设点文件。
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs";
use singlefs_checker_tier::crash_judge_gpu::{
    pool_checker_violated_and_judged_bits, pool_checker_violated_bits, verdict_bits,
    POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    gpu_card_pool_or_report_not_run,
    image_with_every_write_landed_read_back_from_the_pairing_library,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::journal::JournalRecord;
use singlefs_core::root_record::RootRecord;
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{JOURNAL_RECORD_BYTES, TREE_IDENTIFIER_WATERMARK_AT_MKFS};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;

/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
const EXPAND_UP_TO: usize = 8;
/// B 的根压到的水位：mkfs 时的水位，不高于 A 发出的树 ID。
const LOWERED_TREE_IDENTIFIER_WATERMARK: u64 = TREE_IDENTIFIER_WATERMARK_AT_MKFS;
/// 池级 checker 判最新那条根自己的水位那一格的违例说明开头（`walk.rs` 的 `check_pool_image` 末尾）；取 max 那一格的说明以「根环水位最大」开头。
/// 同一条不变量只留第一格违例的说明，取 max 那一格先判：说明以这一句开头 ⟺ 取 max 那一格没红、最新那条根那一格红了。
const NEWEST_ROOT_WATERMARK_VIOLATION_PREFIX: &str = "最新那条根";

/// 一条流：mkfs 之后的基镜像、写表与段、判的那条根、模型认的版本。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// B 的根（流里最后那条根槽写）的水位怎么放。
enum NewestRootWatermark {
    AsPublished,
    LoweredToTheMkfsWatermark,
}

fn root_record_of_the_write(write: &RetainedWrite) -> RootRecord {
    let bytes = write.bytes().expect("根槽写带字节");
    let root = RootRecord::parse_slot(bytes, &parameters().filesystem_identifier)
        .expect("流里的根槽写自证得过");
    assert_eq!(
        root.to_slot(bytes.len()),
        bytes,
        "根记录解出再写回逐字节相同：改水位之后重写的槽只差水位与校验和"
    );
    root
}

/// 覆盖写 B 那一版的内容：与 A 的不同。
fn content_of_publish_b() -> Vec<u8> {
    (0..CONTENT_BYTES_OF_PUBLISH_B)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 新池新建文件（A）再覆盖写一次（B）那条流，从 mkfs 之后起；`newest_root_watermark` 是压低时，B 的根的水位换成
/// [`LOWERED_TREE_IDENTIFIER_WATERMARK`]、重封，流里 B 那次（同 (实例, txg)）的每条 journal 记录的新根段水位跟着换、重封。
fn overwrite_flow(tag: &str, newest_root_watermark: NewestRootWatermark) -> RecordedFlow {
    let mut pool = build_pool(tag);
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content_of_publish_b());
    let operations = pool.retained_operations();
    let (mut writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    let publish_a_root_index = writes[..judged_root_index]
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("B 之前有 A 的根槽写");
    let publish_root = root_record_of_the_write(&writes[judged_root_index]);
    let publish_a_root = root_record_of_the_write(&writes[publish_a_root_index]);
    match newest_root_watermark {
        NewestRootWatermark::AsPublished => {}
        NewestRootWatermark::LoweredToTheMkfsWatermark => {
            assert_eq!(
                publish_a_root.tree_identifier_watermark, publish_root.tree_identifier_watermark,
                "A 的根的水位与 B 的相同：只压 B 的根，根环里全部根取 max 不变"
            );
            assert!(
                LOWERED_TREE_IDENTIFIER_WATERMARK < publish_root.tree_identifier_watermark,
                "压到的水位 {LOWERED_TREE_IDENTIFIER_WATERMARK} 低于 B 的根原来的 {}",
                publish_root.tree_identifier_watermark
            );
            let slot_bytes = writes[judged_root_index]
                .bytes()
                .expect("根槽写带字节")
                .len();
            let mut lowered_root = publish_root;
            lowered_root.tree_identifier_watermark = LOWERED_TREE_IDENTIFIER_WATERMARK;
            writes[judged_root_index].contents =
                WrittenContents::Bytes(lowered_root.to_slot(slot_bytes));
            let journal_filesystem_identifier =
                unit_filesystem_identifier(&parameters().filesystem_identifier);
            // 每块盘上 B 那次记录里最大的计数器，与流里同实例的记录里最大的计数器：后者更大就有记录要按改后的记录头重算反向链。
            let mut highest_rewritten_counter_on_each_device = BTreeMap::new();
            let mut highest_counter_of_the_instance_on_each_device = BTreeMap::new();
            for write in writes
                .iter_mut()
                .filter(|write| write.kind == StepKind::JournalRecord)
            {
                let bytes = write.bytes().expect("记录写带字节");
                assert_eq!(
                    u64::try_from(bytes.len()).expect("记录写的长度装得进 u64"),
                    JOURNAL_RECORD_BYTES,
                    "一次记录写恰好一条记录"
                );
                let mut record = JournalRecord::parse(bytes, journal_filesystem_identifier)
                    .expect("流里的记录写自证得过");
                assert_eq!(
                    record.to_bytes(),
                    bytes,
                    "记录解出再写回逐字节相同：改水位之后重写的记录只差新根段水位与校验和"
                );
                if record.instance != publish_root.instance {
                    continue;
                }
                let highest_of_the_instance = highest_counter_of_the_instance_on_each_device
                    .entry(write.device)
                    .or_insert(record.counter);
                *highest_of_the_instance = (*highest_of_the_instance).max(record.counter);
                if record.checkpoint_txg != publish_root.checkpoint_txg {
                    continue;
                }
                assert_eq!(
                    record.new_tree_identifier_watermark, publish_root.tree_identifier_watermark,
                    "B 那次记录的新根段水位与 B 的根相同"
                );
                record.new_tree_identifier_watermark = LOWERED_TREE_IDENTIFIER_WATERMARK;
                write.contents = WrittenContents::Bytes(record.to_bytes());
                let highest_rewritten = highest_rewritten_counter_on_each_device
                    .entry(write.device)
                    .or_insert(record.counter);
                *highest_rewritten = (*highest_rewritten).max(record.counter);
            }
            assert_eq!(
                highest_rewritten_counter_on_each_device.len(),
                2,
                "两块盘上各有 B 那次的记录"
            );
            assert_eq!(
                highest_rewritten_counter_on_each_device,
                highest_counter_of_the_instance_on_each_device,
                "B 那次的记录是各盘上本实例最后的记录：后面没有记录的反向链要跟着重算"
            );
        }
    }
    RecordedFlow {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions: vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: content_of_publish_b(),
            },
        ],
    }
}

/// 全部写都落了的那个镜像上，CPU 池级 checker 判红的不变量与各自第一格违例的说明（GPU 判器对拍的 CPU 参照那一侧）。
// cpu-reference-for-gpu-comparison
fn violations_with_every_write_landed(
    flow: &RecordedFlow,
) -> Option<BTreeMap<&'static str, String>> {
    // 镜像的输入先经对拍那一类的库（写进去再读回，用户 2026-10-13 定：CPU 参照也从库读），判的是读回的那一份；只录那一趟交 None
    let image =
        image_with_every_write_landed_read_back_from_the_pairing_library(&flow.base, &flow.writes)?;
    let violations = check_pool_image(&image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    Some(violations)
}

/// CPU 池级 checker 的违例位：位 i ↔ `IMPLEMENTED_INVARIANTS[i]`（与 GPU 判器结论里那一段同一个编号）；经 `pool_checker_violated_and_judged_bits` 取
/// （与 CPU 一侧的对拍字同一个函数），与 GPU 那一份逐位整份比（I-1.13 由扫描重建那一截判、置位，内核不置位的表是空的，C821）。
fn pool_checker_bits_on_the_cpu(judgement: &CrashStateJudgement) -> u128 {
    let (violated, _judged) = pool_checker_violated_and_judged_bits(&judgement.pool_checker);
    violated
}

fn invariant_names(bits: u128) -> Vec<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(index, _)| bits & (1u128 << index) != 0)
        .map(|(_, name)| *name)
        .collect()
}

#[test]
fn gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_like_the_cpu_checker()
{
    // 前提（取卡之前、CPU 上判）：原样的流全部落盘时 I-7.8 不红；压低之后只红 I-7.8，而且红的是最新那条根自己那一格、
    // 取 max 那一格没红——这条流只在这一格上分得出 GPU 判没判这一格。
    let Some(as_published) = violations_with_every_write_landed(&overwrite_flow(
        "gpu-judge-newest-root-watermark-as-published",
        NewestRootWatermark::AsPublished,
    )) else {
        return;
    };
    assert!(
        !as_published.contains_key("I-7.8"),
        "原样的流全部落盘时 I-7.8 不红，实际 {as_published:?}"
    );
    let flow = overwrite_flow(
        "gpu-judge-newest-root-watermark-lowered",
        NewestRootWatermark::LoweredToTheMkfsWatermark,
    );
    let Some(lowered) = violations_with_every_write_landed(&flow) else {
        return;
    };
    assert_eq!(
        lowered.keys().copied().collect::<Vec<_>>(),
        vec!["I-7.8"],
        "压低最新那条根的水位之后只红 I-7.8，实际 {lowered:?}"
    );
    assert!(
        lowered["I-7.8"].starts_with(NEWEST_ROOT_WATERMARK_VIOLATION_PREFIX),
        "红的是最新那条根自己那一格、取 max 那一格没红，实际说明「{}」",
        lowered["I-7.8"]
    );
    const TEST_NAME: &str =
        "gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_like_the_cpu_checker";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    let state_count =
        crash_point_replay_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count);
    let Some(gpu): Option<Vec<[u32; VERDICT_WORDS]>> =
        judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
    else {
        return;
    };
    let mut cpu: Vec<u128> = Vec::new();
    let Some(judged_on_the_cpu) = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: TEST_NAME,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| cpu.push(pool_checker_bits_on_the_cpu(judgement)),
    )
    .judged() else {
        return;
    };
    assert_eq!(judged_on_the_cpu, state_count);
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    let tree_identifier_watermark_bit = 1u128
        << IMPLEMENTED_INVARIANTS
            .iter()
            .position(|invariant| *invariant == "I-7.8")
            .expect("I-7.8 在清单里");
    let states_red_on_the_tree_identifier_watermark_on_the_cpu = cpu
        .iter()
        .filter(|bits| *bits & tree_identifier_watermark_bit != 0)
        .count();
    assert!(
        states_red_on_the_tree_identifier_watermark_on_the_cpu > 0,
        "展开到的状态里要有 B 的根落了、CPU 判 I-7.8 红的，这条流才分得出 GPU 判没判最新那条根那一格"
    );
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(&gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_bits, gpu_words))| {
            let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
                || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0;
            let gpu_bits = pool_checker_violated_bits(gpu_words);
            (undecidable || gpu_bits != *cpu_bits).then(|| {
                format!(
                    "状态 {ordinal}：CPU {:?}；GPU {:?}（判不了：{undecidable}）",
                    invariant_names(*cpu_bits),
                    invariant_names(gpu_bits)
                )
            })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} 个状态 GPU 与 CPU 的池级 checker 违例位不同（CPU 判 I-7.8 红的状态 {states_red_on_the_tree_identifier_watermark_on_the_cpu} 个；\
         GPU 少判最新那条根自己的水位那一格时这些状态上少一位 I-7.8）：\n{}",
        mismatches.len(),
        mismatches.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
}
