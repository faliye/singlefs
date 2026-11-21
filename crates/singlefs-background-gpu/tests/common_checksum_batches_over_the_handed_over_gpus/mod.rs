//! 校验和核对那两份核的真卡测试目标共用：造单元与期望值、CPU 参照、按卡的顺序填满服务交来的几张卡的那一批（测饱和）。
//! 一张卡的在 `checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference`，两张、三张的各是一个测试目标（源头声明 `// task-server-gpu-cards: <张数>`，
//! 服务照它分同一台的几张）：验收「一张卡 1 GiB、两张卡、三张卡，每张卡都能用来分片 gc」。要先 `mod common_discrete_gpus_of_this_host;`、`mod common_independent_references;` 与 `mod common_pairing_input_through_the_library;` 再 mod 进这一份。
//! 对拍分两段、经判定库交接（[`checksum_units_concluded_on_the_cards_through_the_pairing_library`]）：一批单元连同期望值编成字节，录入段按它的摘要写进库、
//! 登清单，只录那一趟到此返回、不开卡；判段照同一批现算录入键、从库里读回，对拍结论按键入库复用：库里有两边一致的不开卡，要现判的开卡之后
//! 交给 GPU 的单元与比的 CPU 参照都是读回的那一份（[`checksum_units_and_reference_outcomes_read_back`]）。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::time::Duration;

use crate::common_discrete_gpus_of_this_host::{
    handed_over_gpus_of_the_declared_count_or_report_not_run, identifiers_of_the_handed_over_gpus,
    opened_background_gpu, selection_over_the_handed_over_gpus,
};
use crate::common_independent_references::crc32c_by_the_most_significant_bit_first_table;
use crate::common_pairing_input_through_the_library::PairingJudged;
use crate::common_pairing_input_through_the_library::{
    append_count, append_length_prefixed_bytes, append_number,
    judgement_of_the_assertions_on_the_cards,
    pairing_concluded_on_the_cards_through_the_pairing_library, ComparisonOnTheCards,
    PairingInputBeingRead,
};
use singlefs_background_gpu::background_gpu::BackgroundGpu;
use singlefs_background_gpu::checksum_kernels::{
    cpu_reference_outcome, ChecksumKernelOutcome, UnitChecksumToVerify, CHECKSUM_CHUNK_FOOTPRINT,
};
use singlefs_background_gpu::memory_limit_and_chunks::{
    EntryCountOfAChunk, InputLengthOnTheHostInBytes,
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
fn cpu_reference_outcomes(
    unit_bytes_and_expected: &[(Vec<u8>, u32)],
) -> Vec<ChecksumKernelOutcome> {
    unit_bytes_and_expected
        .iter()
        .map(|(unit_bytes, expected_crc32c)| cpu_reference_outcome(unit_bytes, *expected_crc32c))
        .collect()
}

/// 一批单元连同期望值编成对拍输入的字节：单元数（8 字节小端），逐个单元是长度（8 字节小端）、字节、期望值（4 字节小端）。
#[must_use]
pub fn pairing_input_bytes_of_the_units(unit_bytes_and_expected: &[(Vec<u8>, u32)]) -> Vec<u8> {
    let mut input_bytes = Vec::new();
    append_count(&mut input_bytes, unit_bytes_and_expected.len());
    for (unit_bytes, expected_crc32c) in unit_bytes_and_expected {
        append_length_prefixed_bytes(&mut input_bytes, unit_bytes);
        input_bytes.extend_from_slice(&expected_crc32c.to_le_bytes());
    }
    input_bytes
}

/// [`pairing_input_bytes_of_the_units`] 的反向。
///
/// # Panics
/// 字节与这份布局对不上（短了、取完还有剩）。
#[must_use]
pub fn units_of_the_pairing_input_bytes(input_bytes: &[u8]) -> Vec<(Vec<u8>, u32)> {
    let mut input = PairingInputBeingRead::from_the_start_of(input_bytes);
    let unit_count = input.take_count();
    let unit_bytes_and_expected = (0..unit_count)
        .map(|_unit| {
            let unit_bytes = input.take_length_prefixed_bytes().to_vec();
            let expected_crc32c =
                u32::from_le_bytes(input.take_bytes(4).try_into().expect("刚取的正好 4 字节"));
            (unit_bytes, expected_crc32c)
        })
        .collect();
    input.finish();
    unit_bytes_and_expected
}

/// 从库里读回的一批单元（字节与期望值）连同 CPU 参照在只录那一段算好的逐条结局。
pub type ChecksumUnitsAndTheirReferenceOutcomes = (Vec<(Vec<u8>, u32)>, Vec<ChecksumKernelOutcome>);

/// 参照单元登记表里校验和核 CPU 参照那一组的名字（`tests/pairing_units_entries.tsv` 的参照一侧那几行）。
const REFERENCE_UNITS_GROUP_OF_THE_CHECKSUM_KERNELS: &str = "cpu_reference_of_the_checksum_kernels";

/// 一批结局编成字节：条数，逐条是算出的 CRC 与对不对得上（0 或 1），都是 8 字节小端。
fn bytes_of_the_outcomes(outcomes: &[ChecksumKernelOutcome]) -> Vec<u8> {
    let mut bytes = Vec::new();
    append_count(&mut bytes, outcomes.len());
    for outcome in outcomes {
        append_number(&mut bytes, u64::from(outcome.computed_crc32c));
        append_number(&mut bytes, u64::from(outcome.matches_the_expected_crc32c));
    }
    bytes
}

/// [`bytes_of_the_outcomes`] 的反向。
fn outcomes_of_the_bytes(bytes: &[u8]) -> Vec<ChecksumKernelOutcome> {
    let mut input = PairingInputBeingRead::from_the_start_of(bytes);
    let outcome_count = input.take_count();
    let outcomes = (0..outcome_count)
        .map(|_outcome| ChecksumKernelOutcome {
            computed_crc32c: u32::try_from(input.take_number()).expect("写下的 CRC 装得进 u32"),
            matches_the_expected_crc32c: match input.take_number() {
                0 => false,
                1 => true,
                other => panic!("对不对得上只写 0 或 1，读回 {other}"),
            },
        })
        .collect();
    input.finish();
    outcomes
}

/// 校验和对拍参照一侧的入口（登记在 `tests/pairing_units_entries.tsv` 的参照一侧那几行）：对拍输入里每个单元的 CPU 参照结局，编成字节。只录那一段调它。
#[must_use]
pub fn reference_bytes_of_the_checksum_pairing_input(input_bytes: &[u8]) -> Vec<u8> {
    bytes_of_the_outcomes(&cpu_reference_outcomes(&units_of_the_pairing_input_bytes(
        input_bytes,
    )))
}

/// 一条校验和真卡对拍用例走这一条：`units_of_the_batch` 造的这一批编成字节过判定库，只录那一段另在 CPU 上算每个单元的参照结局、一起写进库，
/// 判段的对拍结论按键入库复用（`crate::common_pairing_input_through_the_library::pairing_concluded_on_the_cards_through_the_pairing_library`，项名用用例名，
/// 参照一侧是登记表里校验和核 CPU 参照那一组）。要现判的才调 `judged_on_the_cards`（这条用例的持卡判入口，收用例名与读回的输入、参照）。
///
/// # Panics
/// 同 `pairing_concluded_on_the_cards_through_the_pairing_library`。
pub fn checksum_units_concluded_on_the_cards_through_the_pairing_library(
    test_name: &str,
    units_of_the_batch: impl FnOnce() -> Vec<(Vec<u8>, u32)>,
    judged_on_the_cards: impl FnOnce(&str, &[u8], &[u8]) -> PairingJudged,
) {
    pairing_concluded_on_the_cards_through_the_pairing_library(
        test_name,
        REFERENCE_UNITS_GROUP_OF_THE_CHECKSUM_KERNELS,
        || pairing_input_bytes_of_the_units(&units_of_the_batch()),
        reference_bytes_of_the_checksum_pairing_input,
        judged_on_the_cards,
    );
}

/// 持卡判那一段解读回的输入与参照：这一批单元（字节与期望值）与 CPU 参照的逐条结局。
///
/// # Panics
/// 字节与布局对不上（短了、取完还有剩）。
#[must_use]
pub fn checksum_units_and_reference_outcomes_read_back(
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> ChecksumUnitsAndTheirReferenceOutcomes {
    let unit_bytes_and_expected = units_of_the_pairing_input_bytes(input_bytes);
    let reference_outcomes = outcomes_of_the_bytes(reference_bytes);
    assert_the_independent_crc32c_agrees_with(
        &unit_bytes_and_expected,
        &reference_outcomes,
        "读回的 CPU 参照",
    );
    (unit_bytes_and_expected, reference_outcomes)
}

/// 三方比里独立参照那一方（`crate::common_independent_references`）：每个单元的 CRC-32C 与对不对得上期望值，另算一遍，
/// 与 `outcomes`（产品 CPU 参照或 GPU 核交回的，`whose` 说是哪一份）逐条比。持卡判那一段读回之后先拿读回的 CPU 参照比一次，
/// GPU 两份核再各与 CPU 参照比：三方两两相等。
///
/// # Panics
/// 条数不等，或有一条与独立参照不同。
pub fn assert_the_independent_crc32c_agrees_with(
    unit_bytes_and_expected: &[(Vec<u8>, u32)],
    outcomes: &[ChecksumKernelOutcome],
    whose: &str,
) {
    assert_eq!(
        outcomes.len(),
        unit_bytes_and_expected.len(),
        "{whose}与单元一个对一个"
    );
    let disagreeing: Vec<String> = unit_bytes_and_expected
        .iter()
        .zip(outcomes)
        .enumerate()
        .filter_map(|(position, ((unit_bytes, expected_crc32c), outcome))| {
            let independent_crc32c = crc32c_by_the_most_significant_bit_first_table(unit_bytes);
            let independent = ChecksumKernelOutcome {
                computed_crc32c: independent_crc32c,
                matches_the_expected_crc32c: independent_crc32c == *expected_crc32c,
            };
            (*outcome != independent).then(|| {
                format!(
                    "第 {position} 个单元（{} 字节）：{whose} {outcome:?}，独立参照 {independent:?}",
                    unit_bytes.len()
                )
            })
        })
        .collect();
    assert!(
        disagreeing.is_empty(),
        "{whose}与独立参照（不反射正规式查表的 CRC-32C）逐条相同：{disagreeing:#?}"
    );
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
    CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
        EntryCountOfAChunk(UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS)
            .input_bytes_of_the_entries(InputLengthOnTheHostInBytes(4096)),
        EntryCountOfAChunk(UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS),
    )
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
    let handed_over = handed_over_gpus_of_the_declared_count_or_report_not_run(
        test_name,
        declared_gpu_count,
        wait_limit,
    )?;
    let handed_over_gpus = handed_over.gpus;
    let background_gpu = opened_background_gpu(
        &handed_over.vulkan_instance,
        &selection_over_the_handed_over_gpus(&handed_over_gpus, memory_limit, wait_limit),
    );
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

/// 按卡的顺序填满那一条的一批：比每张都填满还多一次的单元（`declared_gpu_count` + 1 次的量，每次 [`UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS`] 个
/// 4096 字节的单元）。
#[must_use]
pub fn units_of_the_fill_over_the_declared_count_of_gpus(
    declared_gpu_count: usize,
) -> Vec<(Vec<u8>, u32)> {
    let submissions_saturating_every_card = u64::try_from(declared_gpu_count).expect("卡数") + 1;
    units_of_four_kibibytes(
        UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS * submissions_saturating_every_card,
    )
}

/// 真卡用例一张、两张、三张卡各跑一遍，按卡的顺序填满、测任务饱和，这是它们持卡判那一段（三个目标各自的持卡判入口带着自己源头声明的张数调它）：
/// 点名服务交来的头 `declared_gpu_count` 张卡、每张的上限一次只交得下 [`UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS`] 个 4096 字节的单元，交读回的那一批
/// （[`units_of_the_fill_over_the_declared_count_of_gpus`]）：前面的卡各填满一次、排在最后的那张多交一次。两份核逐条与读回的 CPU 参照相同，
/// 这一段记下分到活的正好是点名的这几张、按点名的次序，交成 `declared_gpu_count` + 1 次、全部单元都交了。
/// 服务没交这一台的卡时打 `NOT_RUN_NO_GPU_ADAPTER` 那一行（没判、不入库）；交来的比声明的少判红。
///
/// # Panics
/// 没跑成时带记号 panic（`crate::common_pairing_input_through_the_library::judgement_of_the_assertions_on_the_cards`）。
pub fn judged_on_the_declared_count_of_gpus_that_every_gpu_of_a_batch_is_given_work_and_submits(
    test_name: &str,
    declared_gpu_count: usize,
    wait_limit: Duration,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let (unit_bytes_and_expected, reference_outcomes) =
            checksum_units_and_reference_outcomes_read_back(input_bytes, reference_bytes);
        let submissions_saturating_every_card =
            u64::try_from(declared_gpu_count).expect("卡数") + 1;
        let unit_count =
            UNITS_PER_SUBMISSION_OF_THE_FILL_OVER_THE_GPUS * submissions_saturating_every_card;
        assert_eq!(
            u64::try_from(unit_bytes_and_expected.len()).expect("单元数"),
            unit_count,
            "读回的这一批是声明的张数各填满一次、最后那张多交一次的量"
        );
        let Some((mut background_gpu, named_cards)) =
            background_gpu_on_the_declared_count_of_handed_over_gpus_or_report_not_run(
                test_name,
                declared_gpu_count,
                memory_limit_of_the_fill_over_the_gpus(),
                wait_limit,
            )
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
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
            reference_outcomes
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
        ComparisonOnTheCards::EveryComparisonHeld
    })
}
