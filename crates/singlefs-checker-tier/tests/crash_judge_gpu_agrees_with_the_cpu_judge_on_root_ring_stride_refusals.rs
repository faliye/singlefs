//! checker 档模块：crash、crash_state_enumeration、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器照 CPU 判根环区域间距那三判（D22（单元原子性怎么合成） 已定项 16 第 1 句、I-7.13，m3-close-c506 三轮判决第三节，用户 2026-10-22 定）：
//! 系统配置里的区域步长 P（偏移 363）、chunk（偏移 367）等于第一版常量，S × 固定结构槽距 ≤ P × chunk。GPU 两份读者各写一份
//! （恢复内核 `crash_judge_common.wgsl` 的 `parse_system_configuration`、走读内核 `crash_judge_checker.wgsl` 的
//! `checker_read_system_configuration_slot`，WGSL 只有 u32，S × 槽距那一判写成槽距 ≤ ⌊P × chunk ÷ S⌋），这里在三盘上逐状态核它们与
//! CPU 判器（core 的 `system_configuration_values_this_reader_accepts`、池级 checker 的 `geometry_of`）对拍的那几个字相同：
//! 新池新建文件那条流，最后一次系统配置槽写改坏一处、整槽校验和重封——第一轮那一盘（S 写 16、槽距写 512 KiB）、P 写 0、chunk 写 4 MiB。
//! 那一次写落了的状态上 CPU 两遍恢复整池拒（失败点「系统配置值不收」）、池级 checker 红 I-7.13，GPU 要逐位跟上。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。要 `gpu` 特性。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, file_content, geometry};
use singlefs_checker::image::IMPLEMENTED_INVARIANTS;
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion,
};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_judge_gpu::{
    cross_check_words_agree_in_the_cross_check, cross_check_words_of_cpu_judgement,
    cross_check_words_of_gpu_verdict, CrossCheckWords,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED, VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;

/// 这份测试文件的仓内路径：崩溃点重放取样过库的设点文件。
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_root_ring_stride_refusals.rs";
/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
const EXPAND_UP_TO: usize = 8;
/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）：S 在 362、P 在 363、chunk 在 367、固定结构槽距在 429，用例自己写死。
const SLOTS_PER_REGION_OFFSET: usize = 362;
const PRIME_STEP_OFFSET: usize = 363;
const CHUNK_BYTES_OFFSET: usize = 367;
const FIXED_STRUCTURE_SLOT_SPACING_OFFSET: usize = 429;

/// 一条流：mkfs 之后的基镜像、写表与段、判的那条根、模型认的版本。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// 新池新建文件那条流，最后一次系统配置槽写照 `rewrite` 改几个字节、整槽校验和重封（那一槽自证得过，拒它的只能是读者收不收那几判）。
fn new_pool_with_the_last_system_configuration_slot_rewritten(
    tag: &str,
    rewrite: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (mut writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    let last_system_configuration_write = writes
        .iter()
        .rposition(|write| write.kind == StepKind::SystemConfigurationSlot)
        .expect("流里有系统配置槽写");
    let mut slot = writes[last_system_configuration_write]
        .bytes()
        .expect("系统配置槽写带字节")
        .to_vec();
    rewrite(&mut slot);
    let slot_bytes = slot.len();
    let digest =
        wide_checksum_with_field_zeroed(&slot, slot_bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
        .copy_from_slice(&digest);
    writes[last_system_configuration_write].contents = WrittenContents::Bytes(slot);
    RecordedFlow {
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

/// I-7.13 在池级 checker 违例位里的那一位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
fn bit_of_invariant_seven_thirteen() -> u128 {
    1u128
        << IMPLEMENTED_INVARIANTS
            .iter()
            .position(|invariant| *invariant == "I-7.13")
            .expect("I-7.13 在池级 checker 的清单里")
}

/// 逐状态比 GPU 与 CPU 对拍的那几个字；先要 CPU 在至少一个状态上两遍恢复报「系统配置值不收」、池级 checker 红 I-7.13（这条流才判得出这几判）。
fn assert_the_gpu_judge_agrees_with_the_cpu_refusing_the_pool(
    test_name: &str,
    flow: &RecordedFlow,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
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
        judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)
    else {
        return;
    };
    let mut cpu: Vec<CrossCheckWords> = Vec::new();
    let Some(judged_on_the_cpu) = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: test_name,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| cpu.push(cross_check_words_of_cpu_judgement(judgement)),
    )
    .judged() else {
        return;
    };
    assert_eq!(judged_on_the_cpu, state_count);
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    let invariant_seven_thirteen = bit_of_invariant_seven_thirteen();
    assert!(
        cpu.iter().any(|words| {
            words.pool_checker_violated_bits & invariant_seven_thirteen != 0
                && words.consulted_pass_failure_point
                    == GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED
                && words.ignored_pass_failure_point
                    == GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED
        }),
        "改坏的那一槽落了的状态上，CPU 两遍恢复都报系统配置值不收、池级 checker 红 I-7.13"
    );
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(&gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_words, gpu_verdict))| {
            let gpu_words = cross_check_words_of_gpu_verdict(gpu_verdict);
            (!cross_check_words_agree_in_the_cross_check(&gpu_words, cpu_words))
                .then(|| format!("状态 {ordinal}：CPU {cpu_words:?}；GPU {gpu_words:?}"))
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} 个状态 GPU 与 CPU 对拍的字不同（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 第一轮那一盘：S 写 16、槽距写 512 KiB（16 × 512 KiB = 8 MiB 大于区域间距 3 MiB）；P、chunk 照旧是第一版常量。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_sixteen_slots_at_a_512_kibibyte_spacing_overrun_the_region_stride(
) {
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_when_sixteen_slots_at_a_512_kibibyte_spacing_overrun_the_region_stride";
    let flow = new_pool_with_the_last_system_configuration_slot_rewritten(
        "crash-judge-gpu-i713-region-longer-than-the-stride",
        &|slot| {
            slot[SLOTS_PER_REGION_OFFSET] = 16;
            slot[FIXED_STRUCTURE_SLOT_SPACING_OFFSET..FIXED_STRUCTURE_SLOT_SPACING_OFFSET + 4]
                .copy_from_slice(&(512u32 << 10).to_le_bytes());
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_refusing_the_pool(TEST_NAME, &flow);
}

/// P 写 0：不等于第一版常量 3。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_a_prime_step_of_zero(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_a_prime_step_of_zero";
    let flow = new_pool_with_the_last_system_configuration_slot_rewritten(
        "crash-judge-gpu-i713-prime-step-zero",
        &|slot| {
            slot[PRIME_STEP_OFFSET..PRIME_STEP_OFFSET + 4].copy_from_slice(&0u32.to_le_bytes());
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_refusing_the_pool(TEST_NAME, &flow);
}

/// chunk 写 4 MiB：不等于第一版常量 1 MiB。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_a_chunk_of_four_mebibytes(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_a_chunk_of_four_mebibytes";
    let flow = new_pool_with_the_last_system_configuration_slot_rewritten(
        "crash-judge-gpu-i713-chunk-four-mebibytes",
        &|slot| {
            slot[CHUNK_BYTES_OFFSET..CHUNK_BYTES_OFFSET + 4]
                .copy_from_slice(&(4u32 << 20).to_le_bytes());
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_refusing_the_pool(TEST_NAME, &flow);
}
