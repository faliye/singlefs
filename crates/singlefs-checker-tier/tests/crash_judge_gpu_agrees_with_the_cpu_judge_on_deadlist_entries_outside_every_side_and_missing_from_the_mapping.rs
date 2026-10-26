//! checker 档模块：crash、crash_given_states、crash_identity、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器按 CPU 判法同一个读法判 I-3.6（deadlist 紧界） 新并进的两格：前驱链首尾相接而 deadlist 条目落不进任何一侧判红
//! （`.claude/kb/checks-owed.md` C145 第一条，用户 2026-10-26 定并进 I-3.6），deadlist 条目的映射 key 在被判那一版的中央映射里查不到判红
//! （C300 的 checker 一半，用户 2026-10-26 定第一版判损坏停住）；扫描内核 `s_judge_the_deadlist_against_the_head_and_the_live_snapshots`
//! 与 `walk.rs` 的 `judge_the_deadlist_against_the_head_and_the_live_snapshots` 同改。每张镜像两边各判一遍，对拍字逐字段比，I-3.6 那一位另按预期核。
//!
//! 镜像（搭法在 harness 档 `tests/common_deadlist_pending_release/mod.rs`，经 `#[path]` 引；harness 档池级 checker 那一遍在
//! `crates/singlefs-harness/tests/deadlist_pending_release_bound_and_mapping_lookup_must_red_worlds.rs`）：快照下覆盖写之后，一条条目纪元改小、
//! 快照那条码 14 的前驱改大（链照样首尾相接），条目落不进任何一侧；快照下覆盖写之后，一条被 deadlist 记着的留存映射条目从中央映射叶里删掉。两张都两边红 I-3.6。
//!
//! 对拍的写法与库照 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_the_pending_release_bound.rs`。要本机一张 NVIDIA 独显：一张适配器都没有时
//! 打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。GPU 判器与 CPU 判法的对拍一律进快档，都不标 `#[ignore]`。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_deadlist_pending_release/mod.rs"]
mod common_deadlist_pending_release;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;

use common_deadlist_pending_release::{
    image_whose_deadlist_epoch_falls_below_every_side_of_a_contiguous_chain,
    image_whose_kept_mapping_entry_of_a_deadlist_key_is_deleted,
};

use singlefs_checker::image::IMPLEMENTED_INVARIANTS;
use singlefs_checker::walk::ContainerCapacityKnobs;
use singlefs_checker_tier::crash::CrashPointReplaySegmentExpansion;
use singlefs_checker_tier::crash_amplification::RecordedGivenStatesTable;
use singlefs_checker_tier::crash_given_states::{
    judge_a_given_state_on_the_cpu, ClaimedVersionOfAGivenState, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_identity::{judging_units_key, JudgingUnits, SectorImage};
use singlefs_checker_tier::crash_judge_conclusions::cross_check_words_without_the_oracle;
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_judge_gpu::{
    cross_check_words_agree_in_the_cross_check, cross_check_words_of_gpu_verdict,
    scan_rebuild_outcome_name, CrossCheckWords, LandedRootWords, VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{
    build_judge_tables_for_the_given_persisted_sets, JudgeFlow, JudgeTables,
};
use singlefs_checker_tier::crash_library::{
    pairing_conclusion_reused_or_judged, toolchain_of_this_process, CrashLibrary, CrashLibraryClass,
};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    cpu_reference_table_read_back_from_the_pairing_library, gpu_card_pool_or_report_not_run,
    GpuCardPool,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::sha256::sha256_digest;

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
    /// CPU 那一份对拍字逐字段（小端：红位、违例位、两遍的根各四字（`LandedRootWords::words`：标记、实例代号、checkpoint_txg 低 32 位与高 32 位）、两遍的结局、两遍的失败点、判到位、I-1.13 的结局码、判不了一字节），
    /// 之后接 GPU 结论字（小端）。
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

/// 一张镜像这一次对拍的输入摘要：镜像按扇区终值与判表的摘要，只按内容取，表名、用例名不进。前面那一行是区分这种对拍输入的固定标签，不带版本号。
fn pairing_inputs_digest(
    image: &MemoryPool,
    tables: &JudgeTables,
) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
    sha256_digest(
        format!(
            "gpu judge deadlist side coverage and mapping lookup pairing inputs\nbase={}\ntables={}\n",
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

/// 一张镜像两边判出的对拍字：CPU 那一份，与 GPU 结论字换出来、去掉两遍 oracle 那十四位的那一份。
struct CrossCheckWordsOfBothJudges {
    cpu: CrossCheckWords,
    gpu: CrossCheckWords,
}

/// 一张镜像两边各判一次（GPU 那一侧把镜像当基线、一个空持久集合建判表），对拍结论按键入库、键相同的复用。
/// 每趟打一行 `GPU_DEADLIST_SIDE_COVERAGE_AND_MAPPING_LOOKUP_PAIRING_CONCLUSION table=<表名> source=<ReusedFromTheLibrary|JudgedNow>`。
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
                judgements_of_both_judges_on_the_image(cards, table_name, image, &tables)
                    .unwrap_or_else(|| std::panic::panic_any(PairingNotRun))
                    .to_bytes()
            },
        )
    }));
    let (conclusion, source) = match paired {
        Ok(Ok(paired)) => paired,
        Ok(Err(reason)) => panic!("{table_name}：对拍结论读写库失败：{reason}"),
        Err(payload) if payload.is::<PairingNotRun>() => return None,
        Err(payload) => std::panic::resume_unwind(payload),
    };
    println!(
        "GPU_DEADLIST_SIDE_COVERAGE_AND_MAPPING_LOOKUP_PAIRING_CONCLUSION table={table_name} source={source:?}"
    );
    let judgements = JudgementsOfBothJudgesOnAnImage::from_bytes(&conclusion)
        .expect("库里这一键下的对拍结论是这份测试写的，认得回");
    let gpu = cross_check_words_without_the_oracle(cross_check_words_of_gpu_verdict(
        &judgements.gpu_verdict_words,
    ));
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

/// 一条不变量在违例位与判到位里的那一位（`IMPLEMENTED_INVARIANTS` 里的下标）。
fn invariant_bit(invariant: &str) -> u128 {
    let index = IMPLEMENTED_INVARIANTS
        .iter()
        .position(|name| *name == invariant)
        .unwrap_or_else(|| panic!("{invariant} 在已实现的不变量清单里"));
    1u128 << index
}

/// 这一张镜像上 I-3.6 该判成什么。
struct ExpectedVerdictsOfTheImage {
    tight_bound_is_violated: bool,
}

/// 两边逐字段相同（对拍字整份），I-3.6 两边都判到、违例与否照 `expected`。
fn assert_the_gpu_judge_agrees_with_the_cpu_judge(
    table_name: &str,
    both: &CrossCheckWordsOfBothJudges,
    expected: &ExpectedVerdictsOfTheImage,
) {
    assert!(
        cross_check_words_agree_in_the_cross_check(&both.gpu, &both.cpu),
        "{table_name}：GPU 判器与 CPU 判法的对拍字不同——违例位不同的 {:?}，判到位不同的 {:?}，I-1.13 的结局（CPU、GPU）{:?}；\nGPU {:?}\nCPU {:?}",
        invariant_names(both.gpu.pool_checker_violated_bits ^ both.cpu.pool_checker_violated_bits),
        invariant_names(both.gpu.pool_checker_judged_bits ^ both.cpu.pool_checker_judged_bits),
        (
            scan_rebuild_outcome_name(both.cpu.scan_rebuild_outcome),
            scan_rebuild_outcome_name(both.gpu.scan_rebuild_outcome)
        ),
        both.gpu,
        both.cpu,
    );
    let tight_bound = invariant_bit("I-3.6");
    for (side, words) in [("CPU", &both.cpu), ("GPU", &both.gpu)] {
        assert_eq!(
            (
                words.pool_checker_judged_bits & tight_bound != 0,
                words.pool_checker_violated_bits & tight_bound != 0,
            ),
            (true, expected.tight_bound_is_violated),
            "{table_name}：{side} 那一边 I-3.6（判到、违例）；判红的 {:?}",
            invariant_names(words.pool_checker_violated_bits)
        );
    }
}

/// 一条用例：借卡、两边各判这张镜像、逐字段比。借不到卡、只录那一趟时收手（「本次未跑」那一行已打）。
fn judge_the_image_and_assert_both_judges_agree(
    test_name: &str,
    image: &MemoryPool,
    expected: &ExpectedVerdictsOfTheImage,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(both) = judge_the_image_on_both_judges(cards, test_name, image) else {
        return;
    };
    assert_the_gpu_judge_agrees_with_the_cpu_judge(test_name, &both, expected);
}

// ───────────────────────── 用例 ─────────────────────────

/// 快照下覆盖写之后，钉数据单元的那一条条目纪元由 4 改成 1、快照那条码 14 的前驱由 0 改成 2：前驱链首尾相接，纪元 1 落不进任何一侧，两边 I-3.6 都红。
/// 判别力：GPU 扫描内核落不进任何一侧时照旧报判不了（`crates/mutations.tsv`「C145 第一条 GPU 扫描内核落不进任何一侧照旧报判不了」），GPU 不红、CPU 红。
#[test]
fn the_gpu_judge_reddens_a_deadlist_entry_outside_every_side_of_a_contiguous_chain_like_the_cpu_judge(
) {
    let image = image_whose_deadlist_epoch_falls_below_every_side_of_a_contiguous_chain(
        "gpu-deadlist-epoch-below-every-side",
    );
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_reddens_a_deadlist_entry_outside_every_side_of_a_contiguous_chain_like_the_cpu_judge",
        &image,
        &ExpectedVerdictsOfTheImage {
            tight_bound_is_violated: true,
        },
    );
}

/// 快照下覆盖写之后，钉数据单元的那一条 deadlist 条目的留存映射条目从中央映射叶里删掉、重封到根：两边 I-3.6 都红。
/// 判别力：GPU 扫描内核不查 deadlist key（`crates/mutations.tsv`「C300 GPU 扫描内核不查 deadlist key」），GPU 不红、CPU 红。
#[test]
fn the_gpu_judge_reddens_a_deadlist_key_missing_from_the_central_mapping_like_the_cpu_judge() {
    let image = image_whose_kept_mapping_entry_of_a_deadlist_key_is_deleted(
        "gpu-deadlist-kept-mapping-entry-deleted",
    );
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_reddens_a_deadlist_key_missing_from_the_central_mapping_like_the_cpu_judge",
        &image,
        &ExpectedVerdictsOfTheImage {
            tight_bound_is_violated: true,
        },
    );
}
