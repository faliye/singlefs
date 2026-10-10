//! checker 档模块：crash、gpu_unit_checks、crash_facts、crash_verify_gpu
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

use common::{build_pool, file_content, geometry};
use singlefs_checker_tier::crash::{
    judge_layer0_state_range, layer0_plan_state_count, Layer0SegmentExpansion,
};
use singlefs_checker_tier::crash_facts::{
    extract_flow_facts, verdict_bits, verify_states_from_facts, FlowFacts, BASE_IMAGE_SOURCE,
};
use singlefs_checker_tier::crash_verify_gpu::{gpu_verifier_digest, GpuFactsVerifier};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;

const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;

fn expansion(_segment_index: usize, segment: &[usize]) -> Layer0SegmentExpansion {
    if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
        Layer0SegmentExpansion::EveryProperSubset
    } else {
        Layer0SegmentExpansion::NotExpanded
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

/// CPU 判器（`judge_crash_image`）在每个状态上红不红，按序号。
fn judge_red_ordinals(recorded: &Recorded) -> BTreeSet<u64> {
    let mut reds = BTreeSet::new();
    let state_count = layer0_plan_state_count(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &expansion,
    );
    let _state_count = judge_layer0_state_range(
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
    );
    reds
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
        Some(&facts),
        "事实表往返 KV 的字节不丢"
    );
    let cpu = verify_states_from_facts(&facts, 0..facts.state_count);
    let verifier = GpuFactsVerifier::new(card, &facts);
    let gpu = verifier.verify_states(0..facts.state_count);
    assert_eq!(cpu.len(), gpu.len());
    assert_eq!(cpu, gpu, "GPU 内核与 CPU 参照逐状态相等（健康流）");
    assert!(
        cpu.iter().all(|bits| *bits == 0),
        "健康的新池新建文件一个状态都不该红：{:?}",
        cpu.iter()
            .enumerate()
            .filter(|(_, bits)| **bits != 0)
            .take(5)
            .collect::<Vec<_>>()
    );
    assert!(
        judge_red_ordinals(&healthy).is_empty(),
        "CPU 判器在健康流上也全绿"
    );
    // 分两块核，与一次核完逐位相同（块边界不改结论）
    let half = facts.state_count / 2;
    let mut in_two_blocks = verifier.verify_states(0..half);
    in_two_blocks.extend(verifier.verify_states(half..facts.state_count));
    assert_eq!(in_two_blocks, gpu);
    assert_ne!(gpu_verifier_digest(), [0u8; 32]);

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
    let half_cpu = verify_states_from_facts(&half_facts, 0..half_facts.state_count);
    assert!(
        half_cpu.iter().all(|bits| *bits == 0),
        "另一份位置条目还对得上，不红"
    );
    assert_eq!(
        GpuFactsVerifier::new(card, &half_facts).verify_states(0..half_facts.state_count),
        half_cpu
    );
    let broken_facts = facts_of(&broken);
    let broken_cpu = verify_states_from_facts(&broken_facts, 0..broken_facts.state_count);
    let broken_verifier = GpuFactsVerifier::new(card, &broken_facts);
    let broken_gpu = broken_verifier.verify_states(0..broken_facts.state_count);
    assert_eq!(broken_cpu, broken_gpu, "改坏之后两份仍逐状态相等");
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
    let judge_reds = judge_red_ordinals(&broken);
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
    let judged = judge_layer0_state_range(
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
    );
    assert_eq!(judged, facts.state_count);
    assert_eq!(states, facts.state_count);
}
