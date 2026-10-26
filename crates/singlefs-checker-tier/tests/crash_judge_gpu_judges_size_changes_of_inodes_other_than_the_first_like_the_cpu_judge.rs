//! checker 档模块：crash、crash_given_states、crash_identity、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器罩第一个文件之外的 inode 缩短与变长（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S2「GPU 检查内核与恢复内核
//! 对『有记录、没有条目、size > 0』判同一个结论」，第六节交主 agent 的「GPU 两个内核与 CPU 对拍」）：harness 档场景
//! `tests/common_images_for_the_checker_tier/size_changes_of_inodes_other_than_the_first.rs` 交出的每张镜像两边各判一遍，对拍字
//! （`crash_judge_gpu::CrossCheckWords`：两遍恢复各自落到的根、走读结局与失败点，池级 checker 逐条的违例位与判到位，I-1.13 的结局码；
//! 给定状态那一路两边都不判 oracle，那十四位去掉）逐字段比，场景写了「判红的恰好是这几条」的那几张两边都恰好红那几条。
//! 罩的形：别的文件粘住的标签 0（变长、再变长、缩到 0）、内联单元补零重写、跨单元变长的标签 4、截断意图与各批（截断字段、size 之外挂着的单元）、
//! 带洞时被 unlink 的标签换回，与「有记录、没有条目、size > 0」那一张（两边都只红 I-4.6、两遍恢复都拒）。
//! 对拍过库的写法照 `tests/crash_judge_gpu_judges_files_written_by_inode_number_like_the_cpu_judge.rs`（那一份的对拍结论读写抄在这里，
//! 键的标签行换成这一份自己的）。要本机一张 NVIDIA 独显：一张适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。
//! 这几条是 GPU 判器（加速路径）与 CPU 判法（参照路径）的对拍，照 `.claude/rules/test-script-discipline.md`「分快慢两档」一律进快档、不标 `#[ignore]`。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/mod.rs"]
mod common_images_for_the_checker_tier;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_truncation/mod.rs"]
mod common_truncation;
#[path = "../../singlefs-harness/tests/common_images_for_the_checker_tier/size_changes_of_inodes_other_than_the_first.rs"]
mod scenarios_of_size_changes_of_inodes_other_than_the_first;

use common_images_for_the_checker_tier::PoolCheckerImages;
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
            "gpu judge size changes of inodes other than the first pairing inputs\nbase={}\ntables={}\n",
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
/// 每趟打一行 `GPU_SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST_PAIRING_CONCLUSION table=<表名> source=<ReusedFromTheLibrary|JudgedNow>`。
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
        "GPU_SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST_PAIRING_CONCLUSION table={table_name} source={source:?}"
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

/// 一个场景交出的每张镜像两边各判一遍、逐字段比；场景写了「判红的恰好是这几条」的，两边判红的都恰好是那几条。借不到卡、只录那一趟时收手
/// （「本次未跑」那一行已打）。场景一张镜像都没交判红。
fn judge_every_image_of_the_scenario_on_both_judges(
    test_name: &str,
    scenario: impl FnOnce(&mut PoolCheckerImages),
) {
    let mut images = PoolCheckerImages::collected_for_the_checker_tier();
    scenario(&mut images);
    let collected = images.into_collected();
    assert!(!collected.is_empty(), "{test_name}：场景一张镜像都没交");
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    for handed_out in collected {
        let table_name = format!("{test_name}-{}", handed_out.label);
        let Some(both) = judge_the_image_on_both_judges(cards, &table_name, &handed_out.image)
        else {
            return;
        };
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
        if let Some(exactly_violated) = &handed_out.expected.exactly_violated {
            let mut expected: Vec<&str> = exactly_violated.clone();
            expected.sort_unstable();
            for (judge, words) in [("CPU", &both.cpu), ("GPU", &both.gpu)] {
                let mut violated = invariant_names(words.pool_checker_violated_bits);
                violated.sort_unstable();
                assert_eq!(
                    violated, expected,
                    "{table_name}：{judge} 判红的恰好是场景写的那几条"
                );
            }
        }
    }
}

/// 与 harness 档同名场景同一组镜像（见 `scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader`），两边逐字段相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader(
) {
    judge_every_image_of_the_scenario_on_both_judges(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader",
        scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_without_an_entry_writes_a_sticky_tag_zero_entry_read_as_zeros_by_every_reader,
    );
}

/// 与 harness 档同名场景同一组镜像（见 `scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros`），两边逐字段相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros(
) {
    judge_every_image_of_the_scenario_on_both_judges(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros",
        scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_inside_its_one_data_unit_rewrites_that_unit_and_every_reader_reads_the_zeros,
    );
}

/// 与 harness 档同名场景同一组镜像（见 `scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size`），两边逐字段相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size(
) {
    judge_every_image_of_the_scenario_on_both_judges(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size",
        scenarios_of_size_changes_of_inodes_other_than_the_first::extending_an_inode_across_data_units_declares_the_hole_and_every_reader_reads_zeros_up_to_the_size,
    );
}

/// 与 harness 档同名场景同一组镜像（见 `scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch`），两边逐字段相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch(
) {
    judge_every_image_of_the_scenario_on_both_judges(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch",
        scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_an_inode_inside_its_one_data_unit_rewrites_it_through_an_intent_and_a_final_batch,
    );
}

/// 与 harness 档同名场景同一组镜像（见 `scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond`），两边逐字段相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond(
) {
    judge_every_image_of_the_scenario_on_both_judges(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond",
        scenarios_of_size_changes_of_inodes_other_than_the_first::truncating_a_three_unit_inode_into_unit_zero_writes_its_truncation_field_and_releases_the_units_beyond,
    );
}

/// 与 harness 档同名场景同一组镜像（见 `scenarios_of_size_changes_of_inodes_other_than_the_first::unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it`），两边逐字段相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it(
) {
    judge_every_image_of_the_scenario_on_both_judges(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it",
        scenarios_of_size_changes_of_inodes_other_than_the_first::unlinking_another_inode_with_a_hole_switches_its_tag_back_and_deletes_it,
    );
}

/// 与 harness 档同名场景同一组镜像（见 `scenarios_of_size_changes_of_inodes_other_than_the_first::a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count`），两边逐字段相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count(
) {
    judge_every_image_of_the_scenario_on_both_judges(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count",
        scenarios_of_size_changes_of_inodes_other_than_the_first::a_record_with_a_size_but_no_upper_leaf_entry_is_refused_by_the_readers_and_reddens_only_the_unit_count,
    );
}
