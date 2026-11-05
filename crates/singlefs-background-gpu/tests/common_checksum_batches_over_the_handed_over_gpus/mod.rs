//! 校验和核对那两份核的真卡测试目标共用：造单元与期望值、CPU 参照、按卡的顺序填满服务交来的几张卡的那一批（用户 2026-11-02 定「按卡的顺序填满」，测饱和）。
//! 一张卡的在 `checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference`，两张、三张的各是一个测试目标（源头声明 `// task-server-gpu-cards: <张数>`，
//! 服务照它分同一台的几张）：七·3 验收「一张卡1gb 2张卡 三张卡 每张卡都能用来分片gc」（用户 2026-11-01 定，原话在
//! `records/2026-10-19-里程碑四GPU垃圾回收开工.md`）。要先 `mod common_discrete_gpus_of_this_host;` 再 mod 进这一份。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::time::Duration;

use crate::common_discrete_gpus_of_this_host::{
    handed_over_gpus_of_the_declared_count_or_report_not_run, identifiers_of_the_handed_over_gpus,
    opened_background_gpu, selection_over_the_handed_over_gpus,
};
use singlefs_background_gpu::background_gpu::BackgroundGpu;
use singlefs_background_gpu::checksum_kernels::{
    cpu_reference_outcome, ChecksumKernelOutcome, UnitChecksumToVerify, CHECKSUM_CHUNK_FOOTPRINT,
};
use singlefs_core::background_compute::{GpuDeviceIdentifier, GpuMemoryInBytes};

/// 一个单元的字节：按长度与起点余数现算的伪随机字节，每个单元不同。
#[must_use]
pub fn unit_bytes_of(unit_length: usize, starting_offset_residue: usize) -> Vec<u8> {
    let mut generator_state = 0x9E37_79B9_7F4A_7C15_u64
        ^ (u64::try_from(unit_length * 8 + starting_offset_residue).expect("小数"));
    (0..unit_length)
        .map(|_byte_position| {
            generator_state = generator_state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            generator_state.to_le_bytes()[7]
        })
        .collect()
}

#[must_use]
pub fn units_to_verify(
    unit_bytes_and_expected: &[(Vec<u8>, u32)],
) -> Vec<UnitChecksumToVerify<'_, usize>> {
    unit_bytes_and_expected
        .iter()
        .enumerate()
        .map(
            |(entry_key, (unit_bytes, expected_crc32c))| UnitChecksumToVerify {
                entry_key,
                unit_bytes,
                expected_crc32c: *expected_crc32c,
            },
        )
        .collect()
}

#[must_use]
pub fn cpu_reference_outcomes(
    unit_bytes_and_expected: &[(Vec<u8>, u32)],
) -> Vec<ChecksumKernelOutcome> {
    unit_bytes_and_expected
        .iter()
        .map(|(unit_bytes, expected_crc32c)| cpu_reference_outcome(unit_bytes, *expected_crc32c))
        .collect()
}

/// `unit_count` 个 4096 字节、期望值照 CPU 参照算对的单元。
#[must_use]
pub fn units_of_four_kibibytes(unit_count: u64) -> Vec<(Vec<u8>, u32)> {
    (0..usize::try_from(unit_count).expect("单元数装得进 usize"))
        .map(|unit| {
            let unit_bytes = unit_bytes_of(4096, unit);
            let expected_crc32c = singlefs_core::checksum::crc32_castagnoli(&unit_bytes);
            (unit_bytes, expected_crc32c)
        })
        .collect()
}

/// 一张、两张、三张卡那几条里每张卡的上限一次交得下几个 4096 字节的单元（上限照校验和那一段的占用现算，远大于最小的一片）。
pub const UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS: u64 = 64;

/// 一次交得下 [`UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS`] 个 4096 字节的单元、多一个就交不下的显存上限。
#[must_use]
pub fn memory_limit_of_the_fill_over_the_gpus() -> GpuMemoryInBytes {
    GpuMemoryInBytes(CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
        4096 * UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS,
        UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS,
    ))
}

/// 照交来的次序点名服务交来的头 `declared_gpu_count` 张卡（这一份源头声明的张数）、每张给 `memory_limit` 的上限、开卡；服务没交这一台的卡时打
/// `NOT_RUN_NO_GPU_ADAPTER` 那一行、交回 `None`（这一条没跑到）。交回开着的后台 GPU 与点名的卡（照交来的次序）。
///
/// # Panics
/// 交来的比声明的少、交来的卡对不上本机的独显、开卡拒了、点名的不是都参与（`crate::common_discrete_gpus_of_this_host` 那几种判红）。
#[must_use]
pub fn background_gpu_on_the_declared_count_of_handed_over_gpus_or_report_not_run(
    test_name: &str,
    declared_gpu_count: usize,
    memory_limit: GpuMemoryInBytes,
    wait_limit: Duration,
) -> Option<(BackgroundGpu, Vec<GpuDeviceIdentifier>)> {
    let handed_over_gpus = handed_over_gpus_of_the_declared_count_or_report_not_run(
        test_name,
        declared_gpu_count,
        wait_limit,
    )?;
    let background_gpu = opened_background_gpu(&selection_over_the_handed_over_gpus(
        &handed_over_gpus,
        memory_limit,
        wait_limit,
    ));
    assert_eq!(
        background_gpu.participating_gpus().len(),
        declared_gpu_count,
        "点名的这几张都参与（上限装得下最小的一片）"
    );
    Some((
        background_gpu,
        identifiers_of_the_handed_over_gpus(&handed_over_gpus),
    ))
}

/// 七·3 验收「真卡用例……一张、两张、三张卡各跑一遍」（用户 2026-11-01 定），按用户 2026-11-02 定「按卡的顺序填满」「你要测任务饱和的情况」：
/// 点名服务交来的头 `declared_gpu_count` 张卡、每张的上限一次只交得下 [`UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS`] 个 4096 字节的单元，交一批
/// 比每张都填满还多一次的单元（`declared_gpu_count` + 1 次的量）：前面的卡各填满一次、排在最后的那张多交一次。两份核逐条与 CPU 参照相同，
/// 这一段记下分到活的正好是点名的这几张、按点名的次序，交成 `declared_gpu_count` + 1 次、全部单元都交了。
/// 服务没交这一台的卡时打 `NOT_RUN_NO_GPU_ADAPTER` 那一行、交回成功（这一条没跑到）；交来的比声明的少判红。
pub fn every_handed_over_gpu_of_a_batch_is_given_work_and_submits_on_the_real_gpus(
    test_name: &str,
    declared_gpu_count: usize,
    wait_limit: Duration,
) {
    let Some((mut background_gpu, named_cards)) =
        background_gpu_on_the_declared_count_of_handed_over_gpus_or_report_not_run(
            test_name,
            declared_gpu_count,
            memory_limit_of_the_fill_over_the_gpus(),
            wait_limit,
        )
    else {
        return;
    };
    let card_count_wide = u64::try_from(declared_gpu_count).expect("卡数");
    let submissions_saturating_every_card = card_count_wide + 1;
    let unit_count =
        UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS * submissions_saturating_every_card;
    let unit_bytes_and_expected = units_of_four_kibibytes(unit_count);
    let outcomes = background_gpu
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("健康的卡上一批交得成");
    let submissions = background_gpu
        .take_the_submissions_of_the_batch()
        .unit_checksum_verification;
    println!(
        "FILL_OVER_THE_GPUS test={test_name} cards={declared_gpu_count} units={unit_count} submissions={} pieces={}",
        submissions.submissions_completed, submissions.pieces_submitted
    );
    assert_eq!(
        outcomes,
        cpu_reference_outcomes(&unit_bytes_and_expected)
            .iter()
            .map(|outcome| (*outcome, *outcome))
            .collect::<Vec<_>>(),
        "按卡的顺序填满几张卡，两份核照样逐条与 CPU 参照相同"
    );
    assert_eq!(
        submissions.devices_given_work, named_cards,
        "每张点名的卡都分到活、按点名的次序各列一次：{submissions:?}"
    );
    assert_eq!(
        (
            submissions.submissions_completed,
            submissions.pieces_submitted
        ),
        (submissions_saturating_every_card, unit_count),
        "前面的卡各填满交一次、排在最后的那张交两次，全部单元都交了：{submissions:?}"
    );
}
