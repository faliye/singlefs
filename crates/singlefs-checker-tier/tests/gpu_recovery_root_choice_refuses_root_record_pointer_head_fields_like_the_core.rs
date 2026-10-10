//! checker 档模块：crash、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 恢复内核择根与 core 同判根记录四条指针的头部（第 34 批代码轮第一轮判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`
//! 第二节 P3、第三节第 1 条）：core 读根槽经 `RootRecord::parse_slot`（`crates/singlefs-core/src/root_record.rs`），树表、实例表、中央映射树、
//! 分配记录树四条指针都经 `NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from` 读，头部五段（`pointer.rs` 的
//! `PointerHeadAsRead::judged`：MAC 与 nonce 全 0、算法类型 0、压缩算法码 0、压后长度 0、extent 偏移 0）任一段不是第一版的取值，这条根不可择；
//! GPU 恢复内核择根（`crash_judge_common.wgsl` 的 `root_record_pointer_heads_are_all_the_first_version`）要同判。extent 偏移那一段另有用例
//! （`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 根记录实例表指针那一条）；这里四条用例各把新池新建文件那条流里发布的根的一条指针
//! 头部的另一段写成非第一版的取值、重封整槽校验和：树表指针 MAC 第 0 字节、中央映射树根指针算法类型、分配记录树根指针压缩算法码、
//! 实例表指针压后长度（压缩算法码仍 0）。前提在取卡之前用 CPU 判：每个写都落了的镜像上，改之前 core 择到发布的根，改之后退到它之前那条根。
//! GPU 与 CPU 在每个状态上红位、池级 checker 违例位、两遍恢复落到的根与结局逐位相同、对拍字（`crash_judge_gpu::CrossCheckWords`：
//! 再加两遍的失败点、不看 journal 那一遍落到的根、池级 checker 判到位）逐字段相同，一个状态都不许判不了。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `verdict-store,gpu` 特性。
#![cfg(all(feature = "verdict-store", feature = "gpu"))]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, file_content, geometry};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker_tier::crash::{
    judge_layer0_state_range, layer0_plan_state_count, Layer0OracleViolationKind,
    Layer0SegmentExpansion, Layer0StateJudgement,
};
use singlefs_checker_tier::crash_amplification::{gpu_recovery_words, recovery_words_of};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_judge_gpu::{
    cross_check_words_of_cpu_judgement, cross_check_words_of_gpu_verdict,
    pool_checker_violated_bits, verdict_bits, CrossCheckWords, POOL_CHECKER_UNDECIDABLE_WORD,
    VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::{choose_root, choose_system_configuration};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;

/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
const EXPAND_UP_TO: usize = 8;
/// 根记录里四条码 2 指针的起点（D22（单元原子性怎么合成） 已定项 7 的根记录布局；`crash_judge_common.wgsl` 的 `parse_root` 同一组偏移）。
const ROOT_RECORD_TREE_TABLE_POINTER_OFFSET: usize = 36;
const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;
const ROOT_RECORD_MAPPING_ROOT_POINTER_OFFSET: usize = 256;
const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_POINTER_OFFSET: usize = 342;
/// 根记录里实例代号（4 字节）与 checkpoint_txg（8 字节）的偏移。
const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
/// 根记录整槽校验和的偏移与宽（罩整个槽宽，校验和那一段按 0 算）。
const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
const ROOT_RECORD_CHECKSUM_BYTES: usize = 32;
/// 指针头部各段的偏移（D19（块指针的结构与宽度预算） 已定项 7 的字段表：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2、extent 偏移 2）。
const POINTER_FIRST_MAC_BYTE_OFFSET: usize = 0;
const POINTER_ALGORITHM_TYPE_OFFSET: usize = 28;
const POINTER_COMPRESSION_ALGORITHM_CODE_OFFSET: usize = 29;
const POINTER_COMPRESSED_LENGTH_OFFSET: usize = 30;

/// 根记录里的哪一条指针。
#[derive(Clone, Copy, Debug)]
enum RootRecordPointer {
    TreeTable,
    InstanceTable,
    MappingRoot,
    AllocationRecordTreeRoot,
}

impl RootRecordPointer {
    fn offset_in_the_root_record(self) -> usize {
        match self {
            RootRecordPointer::TreeTable => ROOT_RECORD_TREE_TABLE_POINTER_OFFSET,
            RootRecordPointer::InstanceTable => ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET,
            RootRecordPointer::MappingRoot => ROOT_RECORD_MAPPING_ROOT_POINTER_OFFSET,
            RootRecordPointer::AllocationRecordTreeRoot => {
                ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_POINTER_OFFSET
            }
        }
    }
}

/// 指针头部里写成非第一版取值的那一段（extent 偏移之外的四类，`pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion` 前四个成员）。
#[derive(Clone, Copy, Debug)]
enum PointerHeadFieldWrittenOutsideTheFirstVersion {
    /// MAC 第 0 字节写 1（加密关着时 MAC 与 nonce 恒 0，D19 已定项 3）。
    FirstMacByteToOne,
    /// 算法类型写 1（第一版不加密，D9（加密） 已定项 10）。
    AlgorithmTypeToOne,
    /// 压缩算法码写 1（登记表第一版只有码 0，D9（加密） 已定项 14）。
    CompressionAlgorithmCodeToOne,
    /// 压缩算法码仍 0、压后长度写 1（D9（加密） 已定项 14「码 0 时它恒 0」）。
    CompressedLengthToOneWithoutCompression,
}

impl PointerHeadFieldWrittenOutsideTheFirstVersion {
    /// 这一段在指针头部里的偏移与写进去的字节。
    fn offset_in_the_pointer_and_bytes(self) -> (usize, Vec<u8>) {
        match self {
            PointerHeadFieldWrittenOutsideTheFirstVersion::FirstMacByteToOne => {
                (POINTER_FIRST_MAC_BYTE_OFFSET, vec![1])
            }
            PointerHeadFieldWrittenOutsideTheFirstVersion::AlgorithmTypeToOne => {
                (POINTER_ALGORITHM_TYPE_OFFSET, vec![1])
            }
            PointerHeadFieldWrittenOutsideTheFirstVersion::CompressionAlgorithmCodeToOne => {
                (POINTER_COMPRESSION_ALGORITHM_CODE_OFFSET, vec![1])
            }
            PointerHeadFieldWrittenOutsideTheFirstVersion::CompressedLengthToOneWithoutCompression => {
                (
                    POINTER_COMPRESSED_LENGTH_OFFSET,
                    1u16.to_le_bytes().to_vec(),
                )
            }
        }
    }
}

/// 一条流：mkfs 之后的基镜像、写表与段、判的那条根、模型认的版本。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// 重封根记录的整槽校验和。
fn reseal_root_record(bytes: &mut [u8]) {
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
        bytes,
        bytes.len(),
        ROOT_RECORD_CHECKSUM_OFFSET,
    );
    bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + ROOT_RECORD_CHECKSUM_BYTES]
        .copy_from_slice(&digest);
}

/// 每个写都落了的镜像上 core 择到的根的 checkpoint_txg（core 只读恢复与挂载择的是它）。
fn checkpoint_txg_of_the_root_the_core_chooses_with_every_write_landed(
    base: &MemoryPool,
    writes: &[RetainedWrite],
) -> u64 {
    let mut image = base.clone();
    image.apply_writes(writes);
    let system_configuration = choose_system_configuration(&image).expect("系统配置择得到");
    choose_root(&image, &system_configuration)
        .expect("择得到根")
        .checkpoint_txg
        .0
}

/// 新池新建文件那条流，发布的根里 `pointer` 那条指针头部的 `field` 那一段（本来是第一版的取值）写成非第一版的取值、重封。
/// 改之前每个写都落了的镜像上 core 择到发布的根、改之后退到更早的根，在这里当场断言（取卡之前、CPU 上判）。
fn new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
    tag: &str,
    pointer: RootRecordPointer,
    field: PointerHeadFieldWrittenOutsideTheFirstVersion,
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
    let base = pool.memory_pool_after_mkfs();
    let WrittenContents::Bytes(publish_root) = &writes[judged_root_index].contents else {
        panic!("根槽写是普通写");
    };
    assert_eq!(
        u32::from_le_bytes(
            publish_root[ROOT_RECORD_INSTANCE_OFFSET..ROOT_RECORD_INSTANCE_OFFSET + 4]
                .try_into()
                .expect("4 字节")
        ),
        1,
        "发布的根是实例 1 的"
    );
    let publish_checkpoint_txg = u64::from_le_bytes(
        publish_root[ROOT_RECORD_CHECKPOINT_TXG_OFFSET..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
            .try_into()
            .expect("8 字节"),
    );
    let (field_offset, field_bytes) = field.offset_in_the_pointer_and_bytes();
    let field_start = pointer.offset_in_the_root_record() + field_offset;
    let field_range = field_start..field_start + field_bytes.len();
    assert!(
        publish_root[field_range.clone()]
            .iter()
            .all(|byte| *byte == 0),
        "第一版写者把根记录 {pointer:?} 指针头部 {field:?} 那一段写成 0"
    );
    let mut rewritten_root = publish_root.clone();
    rewritten_root[field_range].copy_from_slice(&field_bytes);
    reseal_root_record(&mut rewritten_root);
    let writes_before_the_rewrite = writes.clone();
    writes[judged_root_index].contents = WrittenContents::Bytes(rewritten_root);
    assert_eq!(
        checkpoint_txg_of_the_root_the_core_chooses_with_every_write_landed(
            &base,
            &writes_before_the_rewrite
        ),
        publish_checkpoint_txg,
        "改之前每个写都落了的镜像上 core 择到发布的根"
    );
    let chosen_checkpoint_txg_after_the_rewrite =
        checkpoint_txg_of_the_root_the_core_chooses_with_every_write_landed(&base, &writes);
    assert!(
        chosen_checkpoint_txg_after_the_rewrite < publish_checkpoint_txg,
        "根记录 {pointer:?} 指针头部 {field:?} 那一段不是第一版的取值，core 择根要退到发布之前那条根（择到 txg {chosen_checkpoint_txg_after_the_rewrite}，发布的根 txg {publish_checkpoint_txg}）"
    );
    RecordedFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(publish_checkpoint_txg),
            content: file_content(),
        }],
    }
}

fn oracle_index(kind: Layer0OracleViolationKind) -> u32 {
    u32::try_from(
        Layer0OracleViolationKind::EVERY_KIND
            .iter()
            .position(|candidate| *candidate == kind)
            .expect("七类之一"),
    )
    .expect("装得进 u32")
}

/// CPU 判定 → GPU 结论第 0 字那一套红位（两遍 oracle 各一类、记录核对器两条）。
fn red_bits_on_the_cpu(judgement: &Layer0StateJudgement) -> u32 {
    let mut bits = 0u32;
    if let Some(violation) = &judgement.consulted_oracle {
        bits |= 1 << (verdict_bits::CONSULTED_ORACLE_FIRST + oracle_index(violation.kind));
    }
    if let Some(violation) = &judgement.ignored_oracle {
        bits |= 1 << (verdict_bits::IGNORED_ORACLE_FIRST + oracle_index(violation.kind));
    }
    if judgement.records.root_without_record {
        bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
    }
    if judgement.records.claimed_state_missing_unit {
        bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
    }
    bits
}

/// CPU 池级 checker 的违例位：位 i ↔ `IMPLEMENTED_INVARIANTS[i]`（与 GPU 判器结论里那一段同一个编号）。
fn pool_checker_bits_on_the_cpu(judgement: &Layer0StateJudgement) -> u128 {
    judgement
        .pool_checker
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(
                IMPLEMENTED_INVARIANTS
                    .iter()
                    .position(|candidate| candidate == invariant)
                    .expect("池级 checker 报的不变量在清单里"),
            ),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .fold(0u128, |bits, index| bits | (1u128 << index))
}

fn invariant_names(bits: u128) -> Vec<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(index, _)| bits & (1u128 << index) != 0)
        .map(|(_, name)| *name)
        .collect()
}

/// 一个状态上 CPU 判出的三样：红位、池级 checker 违例位、两遍恢复落到的根与结局（`recovery_words_of` 的口径）。
struct JudgedOnTheCpu {
    red_bits: u32,
    pool_checker_bits: u128,
    recovery_words: [u32; 4],
    cross_check_words: CrossCheckWords,
}

/// GPU 判器与 CPU 判器在这条流的每个状态上红位、池级 checker 违例位、两遍恢复落到的根与结局逐位相同，没有判不了的状态。
/// 本机没有 NVIDIA 独显适配器、或在卡池里等满了借不到卡时报「本次未跑」返回。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(test_name: &str, flow: &RecordedFlow) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
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
    let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count);
    let Some(gpu): Option<Vec<[u32; VERDICT_WORDS]>> =
        judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)
    else {
        return;
    };
    let mut cpu: Vec<JudgedOnTheCpu> = Vec::new();
    let judged_on_the_cpu = judge_layer0_state_range(
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| {
            cpu.push(JudgedOnTheCpu {
                red_bits: red_bits_on_the_cpu(judgement),
                pool_checker_bits: pool_checker_bits_on_the_cpu(judgement),
                recovery_words: recovery_words_of(judgement),
                cross_check_words: cross_check_words_of_cpu_judgement(judgement),
            });
        },
    );
    assert_eq!(judged_on_the_cpu, state_count);
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(&gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_judgement, gpu_words))| {
            let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
                || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0;
            let gpu_red_bits = gpu_words[0] & !(1 << verdict_bits::UNDECIDABLE);
            let gpu_pool_checker_bits = pool_checker_violated_bits(gpu_words);
            let gpu_recovery = gpu_recovery_words(gpu_words);
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (undecidable
                || gpu_red_bits != cpu_judgement.red_bits
                || gpu_pool_checker_bits != cpu_judgement.pool_checker_bits
                || gpu_recovery != cpu_judgement.recovery_words
                || gpu_cross_check_words != cpu_judgement.cross_check_words)
                .then(|| {
                    format!(
                        "状态 {ordinal}：CPU 红位 {:#x} 恢复 {:?} checker {:?} 对拍字 {:?}；GPU 红位 {gpu_red_bits:#x} 恢复 {gpu_recovery:?} checker {:?} 对拍字 {gpu_cross_check_words:?}（判不了：{undecidable}）",
                        cpu_judgement.red_bits,
                        cpu_judgement.recovery_words,
                        invariant_names(cpu_judgement.pool_checker_bits),
                        cpu_judgement.cross_check_words,
                        invariant_names(gpu_pool_checker_bits)
                    )
                })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} 个状态 GPU 与 CPU 不一致（GPU 恢复内核择根与 core 判根记录指针头部不同时，落到的根与两遍的结局先对不上）：\n{}",
        mismatches.len(),
        mismatches.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn the_gpu_recovery_refuses_a_root_whose_tree_table_pointer_carries_a_non_zero_mac_byte_like_the_core(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(
        "the_gpu_recovery_refuses_a_root_whose_tree_table_pointer_carries_a_non_zero_mac_byte_like_the_core",
        &new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
            "gpu-recovery-root-tree-table-pointer-first-mac-byte",
            RootRecordPointer::TreeTable,
            PointerHeadFieldWrittenOutsideTheFirstVersion::FirstMacByteToOne,
        ),
    );
}

#[test]
fn the_gpu_recovery_refuses_a_root_whose_mapping_root_pointer_carries_an_algorithm_type_like_the_core(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(
        "the_gpu_recovery_refuses_a_root_whose_mapping_root_pointer_carries_an_algorithm_type_like_the_core",
        &new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
            "gpu-recovery-root-mapping-root-pointer-algorithm-type",
            RootRecordPointer::MappingRoot,
            PointerHeadFieldWrittenOutsideTheFirstVersion::AlgorithmTypeToOne,
        ),
    );
}

#[test]
fn the_gpu_recovery_refuses_a_root_whose_allocation_record_tree_root_pointer_carries_a_compression_algorithm_code_like_the_core(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(
        "the_gpu_recovery_refuses_a_root_whose_allocation_record_tree_root_pointer_carries_a_compression_algorithm_code_like_the_core",
        &new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
            "gpu-recovery-root-allocation-record-tree-root-pointer-compression-code",
            RootRecordPointer::AllocationRecordTreeRoot,
            PointerHeadFieldWrittenOutsideTheFirstVersion::CompressionAlgorithmCodeToOne,
        ),
    );
}

#[test]
fn the_gpu_recovery_refuses_a_root_whose_instance_table_pointer_carries_a_compressed_length_without_compression_like_the_core(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(
        "the_gpu_recovery_refuses_a_root_whose_instance_table_pointer_carries_a_compressed_length_without_compression_like_the_core",
        &new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
            "gpu-recovery-root-instance-table-pointer-compressed-length",
            RootRecordPointer::InstanceTable,
            PointerHeadFieldWrittenOutsideTheFirstVersion::CompressedLengthToOneWithoutCompression,
        ),
    );
}
