//! 扫描里的校验和核对那两份 WGSL 核在真显卡上与 CPU 参照（核心层的 `crc32_castagnoli`）对拍、两份核彼此对拍
//! （D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现……两份实现在测试里对拍」）。
//! 要本机至少一张独立显卡：一张都没有时每条打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、返回成功；
//! 有卡而开不了判红。只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `scripts/crash/gpu-quiesce-run.sh` 起，带 `--test-threads 1`。
//!
//! 长度取 0、1、7、8、9、4095、4096（空、单字节、不满一字、正好一字、一字多一字节、一块少一字节、正好一块），每种长度又各放在
//! 起点模 4 余 0、1、2、3 的位置上（前面垫 0 到 3 字节的单元），期望值一半给对的、一半给错的，判定两边都走到。

use singlefs_background_gpu::adapter_inventory::BackgroundGpuSelection;
use singlefs_background_gpu::background_gpu::{
    open_background_gpu, BackgroundGpu, BackgroundGpuOpenRefused,
};
use singlefs_background_gpu::checksum_kernels::{
    cpu_reference_outcome, ChecksumKernelOutcome, UnitChecksumToVerify,
};
use singlefs_background_gpu::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, EntryDecision,
};
use singlefs_core::background_compute::{BackgroundComputeSettingsRefused, GpuMemoryInBytes};

const UNIT_LENGTHS_UNDER_TEST: [usize; 7] = [0, 1, 7, 8, 9, 4095, 4096];
const STARTING_OFFSET_RESIDUES: usize = 4;
/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: std::time::Duration = std::time::Duration::from_secs(60);

/// 开本机全部独显；一张都没有时打 `NOT_RUN_NO_GPU_ADAPTER` 那一行、交回 `None`。
fn background_gpu_or_report_not_run(
    test_name: &str,
    selection: &BackgroundGpuSelection,
) -> Option<BackgroundGpu> {
    match open_background_gpu(selection) {
        Ok(background_gpu) => Some(background_gpu),
        Err(BackgroundGpuOpenRefused::SettingsRefused(
            BackgroundComputeSettingsRefused::GpuNotPresent,
        )) => {
            println!(
                "NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=本机的 Vulkan 适配器里一张独立显卡都没有"
            );
            None
        }
        Err(refused) => panic!("本机有独立显卡而开不了后台 GPU：{refused:?}"),
    }
}

fn every_discrete_gpu_without_a_memory_limit() -> BackgroundGpuSelection {
    BackgroundGpuSelection {
        background_gpu_devices: None,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
    }
}

/// 一个单元的字节：按长度与起点余数现算的伪随机字节，每个单元不同。
fn unit_bytes_of(unit_length: usize, starting_offset_residue: usize) -> Vec<u8> {
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

/// 这批要核的单元字节与期望值：每种起点余数先垫一个那么长的单元，再放每种长度；期望值逢单数位置故意给错。
fn unit_bytes_and_expected_checksums() -> Vec<(Vec<u8>, u32)> {
    let mut units = Vec::new();
    for starting_offset_residue in 0..STARTING_OFFSET_RESIDUES {
        units.push((vec![0x5A; starting_offset_residue], 0));
        for unit_length in UNIT_LENGTHS_UNDER_TEST {
            let unit_bytes = unit_bytes_of(unit_length, starting_offset_residue);
            let correct = singlefs_core::checksum::crc32_castagnoli(&unit_bytes);
            let expected_crc32c = if units.len() % 2 == 0 {
                correct
            } else {
                correct ^ 1
            };
            units.push((unit_bytes, expected_crc32c));
        }
    }
    units
}

fn units_to_verify(
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

fn cpu_reference_outcomes(
    unit_bytes_and_expected: &[(Vec<u8>, u32)],
) -> Vec<ChecksumKernelOutcome> {
    unit_bytes_and_expected
        .iter()
        .map(|(unit_bytes, expected_crc32c)| cpu_reference_outcome(unit_bytes, *expected_crc32c))
        .collect()
}

#[test]
fn the_table_driven_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset() {
    let Some(mut background_gpu) = background_gpu_or_report_not_run(
        "the_table_driven_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset",
        &every_discrete_gpu_without_a_memory_limit(),
    ) else {
        return;
    };
    let unit_bytes_and_expected = unit_bytes_and_expected_checksums();
    let outcomes = background_gpu
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("健康的卡上一批交得成");
    let table_driven: Vec<ChecksumKernelOutcome> = outcomes
        .iter()
        .map(|(table_driven, _bitwise)| *table_driven)
        .collect();
    assert_eq!(
        table_driven,
        cpu_reference_outcomes(&unit_bytes_and_expected),
        "查表那一份核与 CPU 参照逐条相同（算出的值与判定）"
    );
}

#[test]
fn the_bitwise_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset() {
    let Some(mut background_gpu) = background_gpu_or_report_not_run(
        "the_bitwise_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset",
        &every_discrete_gpu_without_a_memory_limit(),
    ) else {
        return;
    };
    let unit_bytes_and_expected = unit_bytes_and_expected_checksums();
    let outcomes = background_gpu
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("健康的卡上一批交得成");
    let bitwise: Vec<ChecksumKernelOutcome> = outcomes
        .iter()
        .map(|(_table_driven, bitwise)| *bitwise)
        .collect();
    assert_eq!(
        bitwise,
        cpu_reference_outcomes(&unit_bytes_and_expected),
        "逐位移那一份核与 CPU 参照逐条相同（算出的值与判定）"
    );
}

#[test]
fn the_two_kernels_confirm_every_entry_on_a_healthy_gpu() {
    let Some(mut background_gpu) = background_gpu_or_report_not_run(
        "the_two_kernels_confirm_every_entry_on_a_healthy_gpu",
        &every_discrete_gpu_without_a_memory_limit(),
    ) else {
        return;
    };
    let unit_bytes_and_expected = unit_bytes_and_expected_checksums();
    let mut uncertainty = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
    let reconciled = background_gpu
        .verify_unit_checksums(&units_to_verify(&unit_bytes_and_expected), &mut uncertainty)
        .expect("健康的卡上一批交得成");
    assert_eq!(
        (
            reconciled.confirmed_entries,
            reconciled.uncertain_entries,
            reconciled.entries_judged_by_the_cpu_reference
        ),
        (unit_bytes_and_expected.len(), 0, 0),
        "两份核彼此逐条相同：全部确认"
    );
    let confirmed: Vec<ChecksumKernelOutcome> = reconciled
        .decisions
        .iter()
        .map(|decision| match decision {
            EntryDecision::ConfirmedByBothKernels(outcome) => *outcome,
            EntryDecision::UncertainThisRound | EntryDecision::JudgedByTheCpuReference(_) => {
                panic!("健康的卡上不该有不确定的条目：{decision:?}")
            }
        })
        .collect();
    assert_eq!(confirmed, cpu_reference_outcomes(&unit_bytes_and_expected));
}

#[test]
fn a_memory_limit_smaller_than_the_batch_cuts_it_into_chunks_with_the_same_outcomes() {
    let test_name =
        "a_memory_limit_smaller_than_the_batch_cuts_it_into_chunks_with_the_same_outcomes";
    let Some(unlimited) =
        background_gpu_or_report_not_run(test_name, &every_discrete_gpu_without_a_memory_limit())
    else {
        return;
    };
    let participating_gpus = unlimited.participating_gpus().len();
    drop(unlimited);
    // 8 KiB：一块 4096 字节的单元连同描述与结果装得下，整批（约 33 KiB 的输入）装不下，要切成好几段
    let small_memory_limit = GpuMemoryInBytes(8 * 1024);
    let mut limited = background_gpu_or_report_not_run(
        test_name,
        &BackgroundGpuSelection {
            background_gpu_devices: None,
            background_gpu_memory_limit: Some(vec![small_memory_limit; participating_gpus]),
            background_gpu_wait_limit: WAIT_LIMIT_OF_THE_GPU_TESTS,
        },
    )
    .expect("同一台机器上刚开过一次");
    let unit_bytes_and_expected = unit_bytes_and_expected_checksums();
    let outcomes = limited
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("每个单元单独装得进 8 KiB");
    let reference = cpu_reference_outcomes(&unit_bytes_and_expected);
    assert_eq!(
        outcomes,
        reference
            .iter()
            .map(|outcome| (*outcome, *outcome))
            .collect::<Vec<_>>(),
        "切成几段交，两份核的结果照样逐条与 CPU 参照相同"
    );
}
