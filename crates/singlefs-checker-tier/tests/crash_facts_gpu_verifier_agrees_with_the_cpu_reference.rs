//! checker 档模块：crash、crash_state_enumeration、crash_identity、crash_library、gpu_unit_checks、crash_facts、crash_verify_gpu
//! 崩溃放量三段流的第 ②、③ 段在新池新建文件那条流上：① 抽事实；② GPU 内核与 CPU 参照按同一张事实表判每个状态，逐状态相等、健康流全 0；
//! ③ 改坏一次单元写的一个字节之后，两份仍逐状态相等，那次写持久的每个状态都红，而且每个红状态在 CPU 判器上也红（核对红 ⇒ 判器红）。
//! 要本机一张空闲显存够的 NVIDIA 独显：本机没有 NVIDIA 卡时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功，
//! 有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）；要 `verdict-store,gpu` 特性。
//! 卡经这个进程的卡池取（`gpu_test_start::gpu_card_pool_or_report_not_run`），抽完事实表、知道要多少显存之后按量借，
//! 放不下就等别的用例与别的 GPU 测试进程还，等满了报「本次未跑」。
//! 段的展开方式（判决 `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md` 第二节 L6-2、L1-7）：核对内核拆出的持久掩码
//! （探针入口，`GpuFactsVerifier::persisted_masks_of_states`）在甲二的两种段序上、在 33 次写一段第 2³² 个状态起，与 CPU 那一份逐序号相同；
//! 甲二展开的一条录制流上 CPU 事实表的拆法与 crash.rs 枚举逐序号相同。
#![cfg(all(feature = "verdict-store", feature = "gpu"))]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::BTreeSet;

/// 这份测试文件的仓内路径：崩溃点重放取样过库的设点文件。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs";

use common::{build_pool, file_content, geometry};
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion,
};
use singlefs_checker_tier::crash_facts::FlowFactsReadBack;
use singlefs_checker_tier::crash_facts::{
    extract_flow_facts, verdict_bits, verify_states_from_facts, FlowFacts, BASE_IMAGE_SOURCE,
};
use singlefs_checker_tier::crash_identity::{judging_units_key, JudgingUnits};
use singlefs_checker_tier::crash_library::{
    facts_through_the_pairing_library, library_reads_are_skipped_in_this_process,
    pairing_conclusion_reused_or_judged, toolchain_of_this_process, CrashLibrary,
    CrashLibraryClass, PairingConclusionSource, StageOutcome,
};
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_checker_tier::crash_verify_gpu::GpuFactsVerifier;
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_checker_tier::gpu_unit_checks::GpuCard;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;

const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;

fn expansion(_segment_index: usize, segment: &[usize]) -> CrashPointReplaySegmentExpansion {
    if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}

struct Recorded {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn record_new_pool_file_creation(tag: &str) -> Recorded {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    Recorded {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        }],
    }
}

/// CPU 判器（`judge_crash_image`）在每个状态上红不红，按序号（流先过库再取样）；只录那一趟交 `None`。
fn judge_red_ordinals(recorded: &Recorded, item_label: &str) -> Option<BTreeSet<u64>> {
    let mut reds = BTreeSet::new();
    let state_count = crash_point_replay_plan_state_count(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &expansion,
    );
    let item_name = format!("facts_pairing::judge_red_ordinals::{item_label}");
    sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        &expansion,
        0..state_count,
        &mut |ordinal, _image, judgement| {
            if !judgement.red_items().is_empty() {
                reds.insert(ordinal);
            }
        },
    )
    .judged()?;
    Some(reds)
}

/// GPU 核对器在卡上要占多少显存（MiB，向上取整）：事实表每一项上卡展成 u32 表，展开之后不超过它在事实表 KV 字节
/// （[`FlowFacts::to_bytes`]）里那一项的 [`LARGEST_EXPANSION_OF_A_FACTS_BYTE_ON_THE_CARD`] 倍；加一次核完整条流的结论与回读缓冲
/// （每个状态各 4 字节）；再加 1 MiB 给派活的头与空表各占的一个字。
fn mebibytes_a_facts_verifier_needs(facts: &FlowFacts) -> u64 {
    let table_bytes = u64::try_from(facts.to_bytes().len()).expect("装得进 u64")
        * LARGEST_EXPANSION_OF_A_FACTS_BYTE_ON_THE_CARD;
    let verdict_and_readback_bytes = facts.state_count * 2 * 4;
    (table_bytes + verdict_and_readback_bytes).div_ceil(1024 * 1024) + 1
}

/// 事实表 KV 里一个字节上卡之后最多占几个字节：每次写一个字节的可撕标志上卡是一行 4 个字（16 字节），别的项展开得都比它窄。
const LARGEST_EXPANSION_OF_A_FACTS_BYTE_ON_THE_CARD: u64 = 16;

/// 同一张卡上同时活着几个核对器：健康流那一个一直留到用例末尾，改坏一份的那一个、两份都改坏的那一个各再来一个；
/// 三条流是同一条流改了几个字节，事实表一样大。
const FACTS_VERIFIERS_ALIVE_AT_ONCE: u64 = 3;

/// 对拍的比法：CPU 参照与 GPU 内核逐状态相等、GPU 分两块核与一次核完逐位相同（块边界不改结论），交回要存的结论（每个状态的核对位，
/// 小端四字节一个）。这个函数自成一组判法单元（`gpu_and_cpu_facts_verifier_pairing_comparison`），它的键拼进对拍结论的输入摘要。
fn verifier_bits_agreeing_on_every_state(
    cpu: &[u32],
    gpu: &[u32],
    gpu_in_two_blocks: &[u32],
) -> Vec<u8> {
    assert_eq!(cpu, gpu, "GPU 内核与 CPU 参照逐状态相等");
    assert_eq!(
        gpu_in_two_blocks, gpu,
        "分两块核与一次核完逐位相同（块边界不改结论）"
    );
    cpu.iter().flat_map(|bits| bits.to_le_bytes()).collect()
}

/// 一张事实表上 GPU 核对内核与 CPU 参照逐状态对拍，交回每个状态的核对位。对拍结论入库（用户 2026-10-15 定「对拍结论入库」，
/// `crash_library::pairing_conclusion_reused_or_judged`）：键是事实表摘要与两边核对那一组判法单元的键（CPU 参照、GPU 核对内核），
/// 库里有就复用、不再上卡；没有就两边都核（GPU 那一侧另分两块核一遍，块边界不改结论）、逐状态相等才存（不等当场判红、不入库）。
/// 两边哪一边的判法单元拒算：照核、不入库。交回核对位与这一份对拍结论从哪来。
fn verifier_bits_paired_through_the_library(
    card: &GpuCard,
    facts: &FlowFactsReadBack,
) -> (Vec<u32>, PairingConclusionSource) {
    let library =
        CrashLibrary::open(CrashLibraryClass::FactsPairing).expect("对拍那一类的库开得了");
    let judging_units = |units: JudgingUnits| {
        judging_units_key(&units, toolchain_of_this_process()).map_err(|refused| refused.sentence())
    };
    // 比法那一组的键拼进输入摘要；它拒算时整份对拍照核、不入库（拒算交进一边）
    let comparison_units = judging_units(JudgingUnits::PairingComparison {
        group_name: "gpu_and_cpu_facts_verifier_pairing_comparison",
    });
    let mut inputs = facts.digest().to_vec();
    inputs.extend_from_slice(
        comparison_units
            .as_ref()
            .map_or(&[0u8; 32], |units| units.fingerprint()),
    );
    let (conclusion, source) = pairing_conclusion_reused_or_judged(
        &library,
        &singlefs_harness::sha256::sha256_digest(&inputs),
        &comparison_units.and_then(|_| judging_units(JudgingUnits::CpuFactsVerifier)),
        &judging_units(JudgingUnits::GpuFactsVerifier),
        || {
            let cpu = verify_states_from_facts(facts, 0..facts.state_count);
            let verifier = GpuFactsVerifier::new(card, facts);
            let gpu = verifier.verify_states(0..facts.state_count);
            let half = facts.state_count / 2;
            let mut in_two_blocks = verifier.verify_states(0..half);
            in_two_blocks.extend(verifier.verify_states(half..facts.state_count));
            verifier_bits_agreeing_on_every_state(&cpu, &gpu, &in_two_blocks)
        },
    )
    .expect("对拍结论读写库");
    let bits = conclusion
        .as_chunks::<4>()
        .0
        .iter()
        .map(|word| u32::from_le_bytes(*word))
        .collect();
    (bits, source)
}

fn facts_of(recorded: &Recorded) -> FlowFacts {
    extract_flow_facts(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &expansion,
    )
}

#[test]
fn gpu_and_cpu_verifiers_agree_on_every_state_and_a_flipped_unit_byte_is_red_on_both_and_on_the_judge(
) {
    const TEST_NAME: &str = "gpu_and_cpu_verifiers_agree_on_every_state_and_a_flipped_unit_byte_is_red_on_both_and_on_the_judge";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let healthy = record_new_pool_file_creation("crash-facts-gpu-healthy");
    let facts = facts_of(&healthy);
    // 事实表先入库（对拍那一类的库）、再读库，CPU 参照与 GPU 核对器都只拿读回的那一份
    let StageOutcome::Judged(facts) = facts_through_the_pairing_library(TEST_NAME, &facts) else {
        return;
    };
    let Some(lease) = cards.lease_card_or_report_not_run(
        TEST_NAME,
        FACTS_VERIFIERS_ALIVE_AT_ONCE * mebibytes_a_facts_verifier_needs(&facts),
    ) else {
        return;
    };
    let card = lease.card();
    assert!(
        facts
            .units
            .iter()
            .any(|unit| unit.source == BASE_IMAGE_SOURCE),
        "基线里有 mkfs 写的单元（实例表、树表）"
    );
    assert!(
        facts
            .roots
            .iter()
            .any(|root| root.source == BASE_IMAGE_SOURCE),
        "基线里有 mkfs 写的根"
    );
    assert!(facts.newest_base_root.is_some());
    assert_eq!(
        FlowFacts::from_bytes(&facts.to_bytes()).as_ref(),
        Some(&*facts),
        "事实表往返 KV 的字节不丢"
    );
    // 健康流：GPU 内核与 CPU 参照逐状态相等（对拍结论经库复用）
    let (cpu, _source) = verifier_bits_paired_through_the_library(card, &facts);
    assert_eq!(
        cpu.len(),
        usize::try_from(facts.state_count).expect("装得进")
    );
    // 同一张事实表再对拍一次：键（事实表摘要、两边核对的判法单元键）相同，结论从库里复用、不再上卡（只写不读那一趟照判）
    let (cpu_again, source_again) = verifier_bits_paired_through_the_library(card, &facts);
    assert_eq!(cpu_again, cpu, "复用的结论与判的那一份逐状态相同");
    if !library_reads_are_skipped_in_this_process() {
        assert_eq!(
            source_again,
            PairingConclusionSource::ReusedFromTheLibrary,
            "第二趟同一键的对拍结论从库里复用"
        );
    }
    assert!(
        cpu.iter().all(|bits| *bits == 0),
        "健康的新池新建文件一个状态都不该红：{:?}",
        cpu.iter()
            .enumerate()
            .filter(|(_, bits)| **bits != 0)
            .take(5)
            .collect::<Vec<_>>()
    );
    let Some(healthy_reds) = judge_red_ordinals(&healthy, "healthy") else {
        return;
    };
    assert!(healthy_reds.is_empty(), "CPU 判器在健康流上也全绿");

    // 改坏发布的根直接指着的那个单元（树表）载荷里的一个字节：整单元 CRC 与根指针里的对不上；没被任何根引用的半截单元
    // 校验和本来就可能对不上，池级 checker 不判它们，核对也不判（第一版对拍打中的）
    let mut broken = record_new_pool_file_creation("crash-facts-gpu-broken");
    let publish_root = facts
        .roots
        .iter()
        .filter(|root| root.source != BASE_IMAGE_SOURCE)
        .max_by_key(|root| root.source)
        .expect("发布写了根");
    // 树表指针带两份位置条目（两块盘各一份）：只改坏一份，另一份仍对得上，不红（镜像的冗余，判器也这么看）——两份都改坏才红
    let tree_table_writes: Vec<usize> = publish_root.pointers[0]
        .locations
        .iter()
        .map(|location| {
            facts
                .units
                .iter()
                .filter(|unit| unit.source != BASE_IMAGE_SOURCE)
                .find(|unit| unit.device == location.device && unit.slot == location.slot)
                .map(|unit| unit.source as usize)
                .expect("发布的根指着这次发布写的树表单元（每块盘一份）")
        })
        .collect();
    assert_eq!(tree_table_writes.len(), 2);
    assert_ne!(
        tree_table_writes[0], tree_table_writes[1],
        "两份在两块盘上、两次写"
    );
    let root_write = publish_root.source as usize;
    for write_index in &tree_table_writes {
        assert_eq!(broken.writes[*write_index].kind, StepKind::UnitWrite);
        match &mut broken.writes[*write_index].contents {
            WrittenContents::Bytes(bytes) => {
                let index = bytes.len() - 1;
                bytes[index] ^= 0x01;
            }
            WrittenContents::Zeros { .. } => panic!("单元写是普通写"),
        }
    }
    // 只改坏一份：不红（对拍与判器都认镜像的冗余）
    let mut half_broken = record_new_pool_file_creation("crash-facts-gpu-half-broken");
    match &mut half_broken.writes[tree_table_writes[0]].contents {
        WrittenContents::Bytes(bytes) => {
            let index = bytes.len() - 1;
            bytes[index] ^= 0x01;
        }
        WrittenContents::Zeros { .. } => panic!("单元写是普通写"),
    }
    let half_facts = facts_of(&half_broken);
    // 事实表先入库（对拍那一类的库）、再读库，CPU 参照与 GPU 核对器都只拿读回的那一份
    let StageOutcome::Judged(half_facts) =
        facts_through_the_pairing_library(&format!("{TEST_NAME}::half_broken"), &half_facts)
    else {
        return;
    };
    let (half_cpu, _half_source) = verifier_bits_paired_through_the_library(card, &half_facts);
    assert!(
        half_cpu.iter().all(|bits| *bits == 0),
        "另一份位置条目还对得上，不红"
    );
    let broken_facts = facts_of(&broken);
    // 事实表先入库（对拍那一类的库）、再读库，CPU 参照与 GPU 核对器都只拿读回的那一份
    let StageOutcome::Judged(broken_facts) =
        facts_through_the_pairing_library(&format!("{TEST_NAME}::broken"), &broken_facts)
    else {
        return;
    };
    // 改坏之后两份仍逐状态相等（对拍里判）
    let (broken_cpu, _broken_source) =
        verifier_bits_paired_through_the_library(card, &broken_facts);
    let red_states: Vec<usize> = broken_cpu
        .iter()
        .enumerate()
        .filter(|(_, bits)| **bits != 0)
        .map(|(ordinal, _)| ordinal)
        .collect();
    assert!(
        !red_states.is_empty(),
        "根落了、它指的树表单元被改坏的状态必须红"
    );
    for (ordinal, bits) in broken_cpu.iter().enumerate() {
        let persisted = broken_facts.persisted_writes_of_state(ordinal as u64);
        // 这次发布的根持久 ⇒ 它是最新的根，它的树表指针指的单元 CRC 对不上 ⇒ 红（落点那一位）
        if persisted[root_write] {
            assert_ne!(
                *bits & verdict_bits::NEWEST_ROOT_POINTER_TARGET,
                0,
                "根落了而它指的单元改坏了，落点那一位要红（序号 {ordinal}，位 {bits:#x}）"
            );
        } else {
            assert_eq!(
                *bits, 0,
                "这次发布的根没落，最新的根是暖机那一条、指的单元没改：不红（序号 {ordinal}）"
            );
        }
    }
    // 第 ③ 段的判据：核对红 ⇒ CPU 判器红（判器覆盖得更宽，反过来不要求）
    let Some(judge_reds) = judge_red_ordinals(&broken, "broken") else {
        return;
    };
    for ordinal in &red_states {
        assert!(
            judge_reds.contains(&(*ordinal as u64)),
            "核对判红的状态 {ordinal} 在 CPU 判器上也要红"
        );
    }
    println!(
        "CRASH_FACTS_GPU states={} healthy_red=0 broken_red={} judge_red={} units={} roots={}",
        facts.state_count,
        red_states.len(),
        judge_reds.len(),
        facts.units.len(),
        facts.roots.len()
    );
}

/// 只有掩码计划那几样的事实表：一段 `segment_writes`（录制流 `recorded_write_count` 次写），序号区间 [0, `states`)，最后全部持久那一个。
fn facts_of_one_segment(
    recorded_write_count: u32,
    segment_writes: Vec<u32>,
    states: u64,
) -> FlowFacts {
    FlowFacts {
        recorded_write_count,
        enumerated_write_count: recorded_write_count,
        is_tearable: vec![false; usize::try_from(recorded_write_count).expect("装得进 usize")],
        torn_images: vec![],
        segments: vec![singlefs_checker_tier::crash_facts::SegmentFact {
            writes: segment_writes,
            states: 0..states,
            slice_start: None,
        }],
        state_count: states + 1,
        units: vec![],
        roots: vec![],
        newest_base_root: None,
    }
}

/// 甲二一段两次原地写 0、1 与三次单元写 2、3、4（攻方 r2 模型那一形），段序是原地写在前与单元写在前两种：核对内核拆出的 8 个掩码与 CPU 拆法逐序号相同。
#[test]
fn the_gpu_verifier_decodes_in_place_subsets_with_copy_on_write_none_or_all_like_the_cpu_in_both_segment_orders(
) {
    const TEST_NAME: &str =
        "the_gpu_verifier_decodes_in_place_subsets_with_copy_on_write_none_or_all_like_the_cpu_in_both_segment_orders";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let expansions = vec![
        singlefs_checker_tier::crash_facts::SegmentExpansionFact::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
            in_place_writes: vec![0, 1],
            copy_on_write_writes: vec![2, 3, 4],
        },
    ];
    for segment_writes in [vec![0, 1, 2, 3, 4], vec![2, 3, 4, 0, 1]] {
        let facts = facts_of_one_segment(5, segment_writes.clone(), 7);
        // 事实表先入库（对拍那一类的库）、再读库，CPU 参照与 GPU 核对器都只拿读回的那一份
        let StageOutcome::Judged(facts) = facts_through_the_pairing_library(
            &format!(
                "{TEST_NAME}::{}",
                segment_writes
                    .iter()
                    .map(|write| write.to_string())
                    .collect::<Vec<String>>()
                    .join("_")
            ),
            &facts,
        ) else {
            return;
        };
        let Some(lease) =
            cards.lease_card_or_report_not_run(TEST_NAME, mebibytes_a_facts_verifier_needs(&facts))
        else {
            return;
        };
        let masks =
            GpuFactsVerifier::new_by_the_segment_expansions(lease.card(), &facts, &expansions)
                .persisted_masks_of_states(0..facts.state_count);
        drop(lease);
        for (ordinal, words) in masks.iter().enumerate() {
            let ordinal = u64::try_from(ordinal).expect("装得进 u64");
            assert_eq!(
                singlefs_checker_tier::crash_verify_gpu::persisted_set_of_the_mask_words(words, 5),
                facts.persisted_writes_of_state_by_the_segment_expansions(&expansions, ordinal),
                "段序 {segment_writes:?} 序号 {ordinal}"
            );
        }
    }
}

/// 33 次两态写的一段（判决 m3-crash-enumeration-merge-r2 第二节 L1-7，攻方模型 decode_alias.py 那一形）：从第 2³² 个状态起拆 8 个，
/// 核对内核按 lo / hi 两个字算段内余数，拆出的掩码与 CPU 拆法逐序号相同。
#[test]
fn the_gpu_verifier_decodes_an_ordinal_of_a_thirty_three_write_segment_past_thirty_two_bits_like_the_cpu(
) {
    const TEST_NAME: &str =
        "the_gpu_verifier_decodes_an_ordinal_of_a_thirty_three_write_segment_past_thirty_two_bits_like_the_cpu";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let facts = facts_of_one_segment(33, (0..33).collect(), (1u64 << 33) - 1);
    // 事实表先入库（对拍那一类的库）、再读库，CPU 参照与 GPU 核对器都只拿读回的那一份
    let StageOutcome::Judged(facts) = facts_through_the_pairing_library(TEST_NAME, &facts) else {
        return;
    };
    let Some(lease) = cards.lease_card_or_report_not_run(TEST_NAME, 16) else {
        return;
    };
    let first = 1u64 << 32;
    let masks =
        GpuFactsVerifier::new(lease.card(), &facts).persisted_masks_of_states(first..first + 8);
    drop(lease);
    for (offset, words) in masks.iter().enumerate() {
        let ordinal = first + u64::try_from(offset).expect("装得进 u64");
        assert_eq!(
            singlefs_checker_tier::crash_verify_gpu::persisted_set_of_the_mask_words(words, 33),
            facts.persisted_writes_of_state(ordinal),
            "序号 {ordinal}"
        );
    }
}

/// 甲二展开（平时快档，`crash::quick_tier_expansion`）的新池新建文件那条流：事实表按段的展开方式拆出的持久集合与 crash.rs 枚举交给回调的镜像逐序号相同
/// （不要卡）。
#[test]
fn crash_facts_decode_in_place_subsets_with_copy_on_write_none_or_all_like_the_crash_enumeration() {
    let recorded = record_new_pool_file_creation("crash-facts-in-place-subsets");
    let expansion = singlefs_checker_tier::crash::quick_tier_expansion;
    let facts = extract_flow_facts(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &expansion,
    );
    let expansions = singlefs_checker_tier::crash_facts::segment_expansion_facts(
        &recorded.writes,
        &recorded.segments,
        &expansion,
    );
    let mut states = 0u64;
    let Some(judged) = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: "facts_pairing::in_place_subsets_like_the_crash_enumeration",
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        &expansion,
        0..facts.state_count,
        &mut |ordinal, image, _judgement| {
            states += 1;
            assert_eq!(
                facts.persisted_writes_of_state_by_the_segment_expansions(&expansions, ordinal),
                image.persisted,
                "序号 {ordinal}"
            );
        },
    )
    .judged() else {
        return;
    };
    assert_eq!(judged, facts.state_count);
    assert_eq!(states, facts.state_count);
}

/// 分片的段（C790）：一段 70 次写（68 次原地写，其中写 3、写 10 取三态，加一组两次单元写），整域 3^2 · 2^66 · 2 − 1 个态、装不进 u64；
/// 只判它域里从 2^64 + 12345 起的 300 个态。CPU 参照拆出的持久集合就是 2^64 + 12345 + i 按混合进位（最低一位单元写那一组、再原地写各一位）
/// 拆出的那一个；核对内核（探针入口）从片起点的各位起加片内序号、逐位进位拆出的掩码与 CPU 逐序号相同，两份核对位逐状态相同。
#[test]
fn the_gpu_verifier_decodes_a_sliced_segment_from_its_slice_start_like_the_cpu() {
    const TEST_NAME: &str =
        "the_gpu_verifier_decodes_a_sliced_segment_from_its_slice_start_like_the_cpu";
    const WRITES: u32 = 70;
    const SLICE_STATES: u64 = 300;
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let in_place_writes: Vec<u32> = (0..68).collect();
    let expansions = vec![
        singlefs_checker_tier::crash_facts::SegmentExpansionFact::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
            in_place_writes: in_place_writes.clone(),
            copy_on_write_writes: vec![68, 69],
        },
    ];
    let mut facts = facts_of_one_segment(WRITES, (0..WRITES).collect(), SLICE_STATES);
    facts.is_tearable[3] = true;
    facts.is_tearable[10] = true;
    let radices = expansions[0].digit_radices(&facts.segments[0], &facts.is_tearable);
    let slice_start_value: u128 = (1u128 << 64) + 12345;
    let slice_start = singlefs_checker_tier::crash::StateCount1280Bits::from_u64(u64::MAX)
        .checked_add_u64(12346)
        .expect("2^64 + 12345 装得进 1280 位");
    facts.segments[0].slice_start =
        Some(singlefs_checker_tier::crash_facts::SegmentSliceStartFact::of(slice_start, &radices));
    // 事实表先入库（对拍那一类的库）、再读库，CPU 参照与 GPU 核对器都只拿读回的那一份
    let StageOutcome::Judged(facts) = facts_through_the_pairing_library(TEST_NAME, &facts) else {
        return;
    };
    for ordinal_within_the_slice in 0..SLICE_STATES {
        let number = slice_start_value + u128::from(ordinal_within_the_slice);
        let mut expected = vec![false; 70];
        expected[68] = number % 2 == 1;
        expected[69] = number % 2 == 1;
        let mut rest = number / 2;
        for write in &in_place_writes {
            let index = usize::try_from(*write).expect("装得进 usize");
            let choices = if facts.is_tearable[index] { 3 } else { 2 };
            expected[index] = rest % choices == choices - 1;
            rest /= choices;
        }
        assert_eq!(
            facts.persisted_writes_of_state_by_the_segment_expansions(&expansions, ordinal_within_the_slice),
            expected,
            "片内第 {ordinal_within_the_slice} 个是段内编号 2^64 + 12345 + {ordinal_within_the_slice}（三态的写撕裂那一态记成没持久）"
        );
    }
    let Some(lease) =
        cards.lease_card_or_report_not_run(TEST_NAME, mebibytes_a_facts_verifier_needs(&facts))
    else {
        return;
    };
    let verifier =
        GpuFactsVerifier::new_by_the_segment_expansions(lease.card(), &facts, &expansions);
    let masks = verifier.persisted_masks_of_states(0..facts.state_count);
    for (ordinal, words) in masks.iter().enumerate() {
        let ordinal = u64::try_from(ordinal).expect("装得进 u64");
        assert_eq!(
            singlefs_checker_tier::crash_verify_gpu::persisted_set_of_the_mask_words(words, 70),
            facts.persisted_writes_of_state_by_the_segment_expansions(&expansions, ordinal),
            "序号 {ordinal}"
        );
    }
    assert_eq!(
        verifier.verify_states(0..facts.state_count),
        singlefs_checker_tier::crash_facts::verify_states_from_facts_by_the_segment_expansions(
            &facts,
            &expansions,
            0..facts.state_count
        ),
        "两份核对位逐状态相同"
    );
    drop(verifier);
    drop(lease);
}
