//! 验证包模块：crash、crash_given_states、crash_identity、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器照 CPU 池级 checker 判 I-3.5（引用区间的精确性） 缺口条款里「取号跳过」那一判的第四条件（C699（取号跳过改认出生代号），
//! 「行 + R 的实例表指针出生代号」）：
//! 走读内核每走一版记下它的实例表指针是不是这一版新建的（版本表那一行留空的最后一字），`appearance_hole_between` 照 `walk.rs` 的
//! `hole_in_the_candidate_set_between` 只在它成立时认作接得上。
//!
//! 镜像（搭法在 `tests/common_number_acquisition_skip_histories/mod.rs`，经 `#[path]` 引；harness 用例
//! `pool_checker_number_acquisition_skip_needs_the_later_version_to_create_its_instance_table.rs` 判同一张）：E198（取号跳过认出生代号的前提） nrb 那一段，
//! 实例 2 写行与暖机那三槽读坏、挂载之后第一次覆盖写那一版 R 新写的数据单元诞生代号改小到 R⁻ 的 txg。行接得上、R 的实例表指针生在写行那一版：
//! 加了第四条件认作接不上、N3 不判那个数据单元，I-3.5 判到且成立；GPU 少了第四条件就照 (a) 判它、红 I-3.5，与 CPU 不同。
//! 两边逐位比池级 checker 的违例位与判到位（两遍恢复的结局这里不比：这张镜像是读坏根槽、改小诞生代号造的，比的是 I-3.5 那一判）。
//!
//! 两边的输入都经对拍那一类的库，写法照 `tests/crash_judge_gpu_judges_the_subtree_coverage_key_range_like_the_cpu_judge.rs`：GPU 那一侧把镜像当基线、
//! 一个空持久集合建判表，判表入库再读回上卡；CPU 那一侧把镜像作一张给定状态表写进库、读回，给定状态的 CPU 判法
//! （`crash_given_states::judge_a_given_state_on_the_cpu`）判读回的那一份；两边判出的东西作一份对拍结论入库。GPU 判不了判红。
//! 要本机一张 NVIDIA 独显：一张适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。
//! 这一条是 GPU 判器（加速路径）与 CPU 判法（参照路径）的对拍，照 `.claude/rules/test-script-discipline.md`「分快慢两档」一律进快档、不标 `#[ignore]`。
#![cfg(feature = "gpu")]

mod common;
mod common_number_acquisition_skip_histories;

use common_number_acquisition_skip_histories::image_whose_first_overwrite_after_an_unreadable_row_publish_and_warm_ups_holds_a_data_unit_born_at_the_previous_version;
use singlefs_checker::image::{IMPLEMENTED_INVARIANTS, REFERENCE_INTERVALS_ARE_EXACT};
use singlefs_checker::walk::ContainerCapacityKnobs;
use singlefs_verification::crash::CrashPointReplaySegmentExpansion;
use singlefs_verification::crash_amplification::RecordedGivenStatesTable;
use singlefs_verification::crash_given_states::{
    judge_a_given_state_on_the_cpu, ClaimedVersionOfAGivenState, GivenStatesOfATable,
};
use singlefs_verification::crash_identity::{judging_units_key, JudgingUnits, SectorImage};
use singlefs_verification::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_verification::crash_judge_gpu::{
    cross_check_words_of_gpu_verdict, CrossCheckWords, LandedRootWords, VERDICT_WORDS,
};
use singlefs_verification::crash_judge_tables::{
    build_judge_tables_for_the_given_persisted_sets, JudgeFlow, JudgeTables,
};
use singlefs_verification::crash_library::{
    pairing_conclusion_reused_or_judged, toolchain_of_this_process, CrashLibrary,
    CrashLibraryClass, PairingAgreement, PairingJudged,
};
use singlefs_verification::gpu_unit_checks::gpu_test_start::{
    cpu_reference_table_read_back_from_the_pairing_library, gpu_card_pool_or_report_not_run,
    GpuCardPool,
};
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::memory_pool::MemoryPool;
use singlefs_verification::sha256::sha256_digest;

// ───────────────────────── 对拍：两边各判一遍、结论入库 ─────────────────────────

/// 存进对拍那一类库的对拍结论：CPU 给定状态判法那一个状态的对拍字，与 GPU 判器那一个状态的结论字。只存两边判出的东西、不存比的结果。
struct JudgementsOfBothJudgesOnAnImage {
    cpu_words: CrossCheckWords,
    gpu_verdict_words: [u32; VERDICT_WORDS],
}

/// 从对拍结论字节的开头往后读：每读一段就把读过的切掉；字节不够交 `None`。
struct ConclusionBytesReader<'bytes> {
    remaining: &'bytes [u8],
}

impl<'bytes> ConclusionBytesReader<'bytes> {
    fn take(&mut self, length_in_bytes: usize) -> Option<&'bytes [u8]> {
        if self.remaining.len() < length_in_bytes {
            return None;
        }
        let (taken, rest) = self.remaining.split_at(length_in_bytes);
        self.remaining = rest;
        Some(taken)
    }

    fn take_u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn take_u128(&mut self) -> Option<u128> {
        Some(u128::from_le_bytes(self.take(16)?.try_into().ok()?))
    }
}

impl JudgementsOfBothJudgesOnAnImage {
    /// 两边比下来一致不一致（对拍结论照它记红绿，记着不一致的下一趟现判）：GPU 判得了，池级 checker 的违例位与判到位两边逐位相同
    /// （与比法那一句断言同一个比法）。
    fn agreement_of_the_two_judges(&self) -> PairingAgreement {
        let gpu = cross_check_words_of_gpu_verdict(&self.gpu_verdict_words);
        if !gpu.gpu_could_not_judge
            && (gpu.pool_checker_violated_bits, gpu.pool_checker_judged_bits)
                == (
                    self.cpu_words.pool_checker_violated_bits,
                    self.cpu_words.pool_checker_judged_bits,
                )
        {
            PairingAgreement::BothSidesAgree
        } else {
            PairingAgreement::SidesDisagree
        }
    }

    /// CPU 那一份对拍字逐字段（小端：红位、违例位、两遍的根各四字（`LandedRootWords::words`）、两遍的结局、两遍的失败点、判到位、
    /// I-1.13 的结局码、判不了一字节），之后接 GPU 结论字（小端）。与子树覆盖区间那一份对拍用例同一个排法。
    fn to_bytes(&self) -> Vec<u8> {
        let words = &self.cpu_words;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&words.red_bits.to_le_bytes());
        bytes.extend_from_slice(&words.pool_checker_violated_bits.to_le_bytes());
        for root in [
            words.consulted_pass_landed_root,
            words.ignored_pass_landed_root,
        ] {
            for word in root.words() {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        for word in [
            words.consulted_pass_outcome,
            words.ignored_pass_outcome,
            words.consulted_pass_failure_point,
            words.ignored_pass_failure_point,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(&words.pool_checker_judged_bits.to_le_bytes());
        bytes.extend_from_slice(&words.scan_rebuild_outcome.to_le_bytes());
        bytes.push(u8::from(words.gpu_could_not_judge));
        for word in &self.gpu_verdict_words {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    /// [`Self::to_bytes`] 的逆：字节不全、多出、判不了那一字节不是 0 或 1 交 `None`。
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = ConclusionBytesReader { remaining: bytes };
        let red_bits = reader.take_u32()?;
        let pool_checker_violated_bits = reader.take_u128()?;
        let consulted_pass_landed_root = LandedRootWords::of_the_words([
            reader.take_u32()?,
            reader.take_u32()?,
            reader.take_u32()?,
            reader.take_u32()?,
        ]);
        let ignored_pass_landed_root = LandedRootWords::of_the_words([
            reader.take_u32()?,
            reader.take_u32()?,
            reader.take_u32()?,
            reader.take_u32()?,
        ]);
        let consulted_pass_outcome = reader.take_u32()?;
        let ignored_pass_outcome = reader.take_u32()?;
        let consulted_pass_failure_point = reader.take_u32()?;
        let ignored_pass_failure_point = reader.take_u32()?;
        let pool_checker_judged_bits = reader.take_u128()?;
        let scan_rebuild_outcome = reader.take_u32()?;
        let gpu_could_not_judge = match reader.take(1)?[0] {
            0 => false,
            1 => true,
            _neither_zero_nor_one => return None,
        };
        let mut gpu_verdict_words = [0u32; VERDICT_WORDS];
        for word in &mut gpu_verdict_words {
            *word = reader.take_u32()?;
        }
        reader.remaining.is_empty().then_some(Self {
            cpu_words: CrossCheckWords {
                red_bits,
                pool_checker_violated_bits,
                consulted_pass_landed_root,
                ignored_pass_landed_root,
                consulted_pass_outcome,
                ignored_pass_outcome,
                consulted_pass_failure_point,
                ignored_pass_failure_point,
                pool_checker_judged_bits,
                scan_rebuild_outcome,
                gpu_could_not_judge,
            },
            gpu_verdict_words,
        })
    }
}

/// 判的那一侧报了「本次未跑」（卡池借不到卡、只录那一趟）：对拍结论不存进库，当场收手、由调用方交 `None`。
struct PairingNotRun;

/// 这张镜像这一次对拍的输入摘要：镜像按扇区终值与判表的摘要，只按内容取，表名、用例名不进。前面那一行是区分这种对拍输入的固定标签，不带版本号。
fn pairing_inputs_digest(
    image: &MemoryPool,
    tables: &JudgeTables,
) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
    sha256_digest(
        format!(
            "gpu judge number acquisition skip pairing inputs\nbase={}\ntables={}\n",
            hexadecimal_text(&SectorImage::of_base(image).digest()),
            hexadecimal_text(&tables.digest())
        )
        .as_bytes(),
    )
}

/// CPU 那一侧：镜像作一张给定状态表写进对拍那一类的库、读回，给定状态的 CPU 判法判读回的那一份（两遍恢复、池级 checker、记录核对器；
/// 产品那一组容器旋钮、恢复自称的那一版取看 journal 那一遍落到的）。只录那一趟交 `None`。
// cpu-reference-for-gpu-comparison
fn cpu_words_of_the_image_read_back_from_the_pairing_library(
    table_name: &str,
    image: &MemoryPool,
) -> Option<CrossCheckWords> {
    let read_back =
        cpu_reference_table_read_back_from_the_pairing_library(&RecordedGivenStatesTable {
            table_name: table_name.to_string(),
            base: image.clone(),
            writes: Vec::new(),
            segments: Vec::new(),
            seams: Vec::new(),
            persisted_sets: vec![Vec::new()],
        })?;
    let table = GivenStatesOfATable {
        table_name: read_back.table_name.clone(),
        base: &read_back.base,
        writes: &read_back.writes,
        segments: &read_back.segments,
        seams: &read_back.seams,
        persisted_sets: read_back.persisted_sets.clone(),
    };
    Some(
        judge_a_given_state_on_the_cpu(
            &table,
            ContainerCapacityKnobs::PRODUCT,
            ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery,
            &[],
        )
        .words,
    )
}

/// 对拍本身：CPU 那一侧从库里读回的那张表上判、GPU 那一侧从库里读回的判表上判，交回两边判出的东西。只录那一趟、卡借不到交 `None`。
fn judgements_of_both_judges_on_the_image(
    cards: &'static GpuCardPool,
    table_name: &str,
    image: &MemoryPool,
    tables: &JudgeTables,
) -> Option<JudgementsOfBothJudgesOnAnImage> {
    let cpu_words = cpu_words_of_the_image_read_back_from_the_pairing_library(table_name, image)?;
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, table_name, tables)?;
    let [gpu_verdict_words] = gpu.as_slice() else {
        panic!("{table_name}：一个状态一份结论，交回 {} 份", gpu.len());
    };
    Some(JudgementsOfBothJudgesOnAnImage {
        cpu_words,
        gpu_verdict_words: *gpu_verdict_words,
    })
}

/// 一张镜像两边判出的对拍字：CPU 那一份，与 GPU 结论字换出来的那一份。
struct CrossCheckWordsOfBothJudges {
    cpu: CrossCheckWords,
    gpu: CrossCheckWords,
}

/// 一张镜像两边各判一次（GPU 那一侧把镜像当基线、一个空持久集合建判表），对拍结论按键入库、键相同的复用。
/// 每趟打一行 `GPU_NUMBER_ACQUISITION_SKIP_PAIRING_CONCLUSION table=<表名> source=<ReusedFromTheLibrary|JudgedNow>`。
/// 只录那一趟、卡借不到交 `None`（不存进库）；GPU 判不了（建不出判表，或结论里判不了那一位）判红。
fn judge_the_image_on_both_judges(
    cards: &'static GpuCardPool,
    table_name: &str,
    image: &MemoryPool,
) -> Option<CrossCheckWordsOfBothJudges> {
    let not_expanded =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let tables = build_judge_tables_for_the_given_persisted_sets(
        &JudgeFlow {
            base: image,
            writes: &[],
            segments: &[],
            judged_root_index: 0,
            versions: &[],
            expansion: &not_expanded,
        },
        &[],
        vec![Vec::new()],
    )
    .unwrap_or_else(|refusal| panic!("{table_name}：建不出判表（GPU 判不了）：{refusal:?}"));
    let judging_units = |units: JudgingUnits| {
        judging_units_key(&units, toolchain_of_this_process()).map_err(|refused| refused.sentence())
    };
    let library =
        CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("对拍那一类的库开得了");
    let paired = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pairing_conclusion_reused_or_judged(
            &library,
            &pairing_inputs_digest(image, &tables),
            &judging_units(JudgingUnits::GpuJudge),
            &judging_units(JudgingUnits::JudgementOfTheGivenStates),
            || {
                let judgements =
                    judgements_of_both_judges_on_the_image(cards, table_name, image, &tables)
                        .unwrap_or_else(|| std::panic::panic_any(PairingNotRun));
                PairingJudged {
                    agreement: judgements.agreement_of_the_two_judges(),
                    conclusion: judgements.to_bytes(),
                }
            },
        )
    }));
    let (conclusion, source) = match paired {
        Ok(Ok(paired)) => paired,
        Ok(Err(reason)) => panic!("{table_name}：对拍结论读写库失败：{reason}"),
        Err(payload) if payload.is::<PairingNotRun>() => return None,
        Err(payload) => std::panic::resume_unwind(payload),
    };
    println!("GPU_NUMBER_ACQUISITION_SKIP_PAIRING_CONCLUSION table={table_name} source={source:?}");
    let judgements = JudgementsOfBothJudgesOnAnImage::from_bytes(&conclusion)
        .expect("库里这一键下的对拍结论是这份测试写的，认得回");
    let gpu = cross_check_words_of_gpu_verdict(&judgements.gpu_verdict_words);
    assert!(
        !gpu.gpu_could_not_judge,
        "{table_name}：GPU 判器在这张镜像上判不了（结论里判不了那一位置了）"
    );
    Some(CrossCheckWordsOfBothJudges {
        cpu: judgements.cpu_words,
        gpu,
    })
}

fn invariant_names(bits: u128) -> Vec<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(index, _)| bits & (1u128 << index) != 0)
        .map(|(_, name)| *name)
        .collect()
}

/// I-3.5 在违例位与判到位里的那一位（`IMPLEMENTED_INVARIANTS` 里的下标）。
fn reference_interval_invariant_bit() -> u128 {
    let index = IMPLEMENTED_INVARIANTS
        .iter()
        .position(|name| *name == REFERENCE_INTERVALS_ARE_EXACT)
        .expect("I-3.5 在已实现的不变量清单里");
    1u128 << index
}

// ───────────────────────── 用例 ─────────────────────────

/// 洞对 (1@10, 2@14)、行 (1, 10) 接得上、R 的实例表指针生在写行那一版 11：CPU 加了第四条件认作接不上，R 新写、诞生代号改小到 10 的数据单元
/// （用户可见树里）不判，I-3.5 判到且成立；GPU 两边的违例位与判到位逐位相同。
/// 判别力：GPU 走读内核的 `appearance_hole_between` 少了第四条件（变异「C699 甲 GPU：取号跳过去掉第四条件」），或走一版时把实例表指针记成这一版新建的
/// （变异「C699 甲 GPU：走一版时实例表指针恒记成这一版新建的」），GPU 照 (a) 判那个数据单元、红 I-3.5、CPU 不红，违例位不同。
#[test]
fn the_gpu_judge_does_not_take_a_hole_before_a_version_that_did_not_create_its_instance_table_for_a_number_acquisition_skip(
) {
    const TEST_NAME: &str = "the_gpu_judge_does_not_take_a_hole_before_a_version_that_did_not_create_its_instance_table_for_a_number_acquisition_skip";
    let image = image_whose_first_overwrite_after_an_unreadable_row_publish_and_warm_ups_holds_a_data_unit_born_at_the_previous_version();
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(both) = judge_the_image_on_both_judges(cards, TEST_NAME, &image) else {
        return;
    };
    let reference_interval = reference_interval_invariant_bit();
    assert!(
        both.cpu.pool_checker_judged_bits & reference_interval != 0
            && both.cpu.pool_checker_violated_bits & reference_interval == 0,
        "CPU 认作接不上、不判那个数据单元：I-3.5 判到且成立（CPU 判红的 {:?}）",
        invariant_names(both.cpu.pool_checker_violated_bits)
    );
    assert_eq!(
        (
            both.gpu.pool_checker_violated_bits,
            both.gpu.pool_checker_judged_bits
        ),
        (
            both.cpu.pool_checker_violated_bits,
            both.cpu.pool_checker_judged_bits
        ),
        "GPU 与 CPU 的池级 checker 违例位与判到位逐位相同——违例位不同的 {:?}，判到位不同的 {:?}",
        invariant_names(both.gpu.pool_checker_violated_bits ^ both.cpu.pool_checker_violated_bits),
        invariant_names(both.gpu.pool_checker_judged_bits ^ both.cpu.pool_checker_judged_bits)
    );
}
