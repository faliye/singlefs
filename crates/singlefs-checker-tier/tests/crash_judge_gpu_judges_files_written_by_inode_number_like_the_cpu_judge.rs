//! checker 档模块：crash、crash_given_states、crash_identity、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器罩按 inode 号写出的多个文件（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2、M6）：每张镜像两边各判一遍，
//! 对拍字（`crash_judge_gpu::CrossCheckWords`：两遍恢复各自落到的根、走读结局与失败点，池级 checker 逐条的违例位与判到位，I-1.13 的结局码；
//! 给定状态那一路两边都不判 oracle，那十四位去掉）逐字段比。
//!
//! M6：恢复内核按上段叶条目逐条收文件，与 CPU 恢复（`crates/singlefs-core/src/recovery.rs` 的 `extents_of_the_first_file` 与冷走读）同一套——
//! 第一个文件之外的文件至少一个单元、单元从 0 起连号、尾部没有洞，它们的单元数并进映射条目数；上段叶条目按块分批读（一批 64 条，
//! `crash_judge_common.wgsl` 的 `MAX_UPPER_ENTRIES`），七十一个文件那一形跨过一批的边界。镜像：按 inode 号写出的两文件、上段长到常规层、
//! 删到只剩别的文件、删掉别的文件降回崩溃点重放、别的文件删除中途、七十一个文件。
//! M2：池级 checker 新立的 I-9.40（extent 上段形状由有条目的 inode 号集合定、根层级最低，下标 102）GPU 扫描内核在删除态扫描里同判、置位：
//! 上面那几形都成立，三个必红世界（根层级高一层、多一个脊线节点、少一片该有的叶——号 150 的条目挪进第 0 片叶，规范形要的 (0, 1) 不在）
//! 两边都红、GPU 红在下标 102。必红世界在写路径的镜像上换节点字节造（新的整单元校验和沿引用链补上去），镜像照 harness 档
//! `unlink_of_files_other_than_the_first.rs` 那几份用例的搭法（harness 档不依赖 checker 档，搭建函数抄在这里）。
//!
//! 每张镜像两边的输入都经对拍那一类的库：GPU 那一侧把镜像当基线、一个空持久集合建判表，判表入库再读回上卡；CPU 那一侧把镜像作一张给定状态表
//! 写进库、读回，给定状态的 CPU 判法（`crash_given_states::judge_a_given_state_on_the_cpu`：两遍恢复、池级 checker、记录核对器）判读回的那一份；
//! 两边判出的东西作一份对拍结论入库（用户 2026-10-15 定「对拍结论入库」，`crash_library::pairing_conclusion_reused_or_judged`），
//! 写法照 `tests/crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs`。GPU 判不了判红。
//! 要本机一张 NVIDIA 独显：一张适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。
//! 这几条是 GPU 判器（加速路径）与 CPU 判法（参照路径）的对拍，照 `.claude/rules/test-script-discipline.md`「分快慢两档」一律进快档、
//! 不看快慢判据，都不标 `#[ignore]`。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use common_pointer_head_corruption::{
    node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of, unit_bytes_at,
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
    scan_rebuild_outcome_name, CrossCheckWords, LandedRootWords,
    KERNEL_INVARIANT_INDEX_OF_THE_UPPER_SHAPE_JUDGE, RECOVERY_OUTCOME_CODE_FILE_READ,
    RECOVERY_OUTCOME_CODE_NO_FILE, VERDICT_WORDS,
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
use singlefs_core::address::{InodeNumber, InstanceGeneration};
use singlefs_core::extent_tree::{
    build_upper_internal_node, ExtentTreeNodeIdentity, ExtentUpperNodePosition,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::transaction::{
    publish_new_inodes, publish_sequential_write_of_an_inode, publish_unlink_intent,
    publish_unlink_step, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
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
            "gpu judge files written by inode number pairing inputs\nbase={}\ntables={}\n",
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
/// 每趟打一行 `GPU_FILES_WRITTEN_BY_INODE_NUMBER_PAIRING_CONCLUSION table=<表名> source=<ReusedFromTheLibrary|JudgedNow>`。
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
        "GPU_FILES_WRITTEN_BY_INODE_NUMBER_PAIRING_CONCLUSION table={table_name} source={source:?}"
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

/// I-9.40 在违例位与判到位里的那一位（内核写死的下标，`crash_judge_gpu::KERNEL_INVARIANT_INDEX_OF_THE_UPPER_SHAPE_JUDGE`）。
fn upper_shape_bit() -> u128 {
    let [(_, index)] = KERNEL_INVARIANT_INDEX_OF_THE_UPPER_SHAPE_JUDGE;
    1u128 << index
}

/// 这一形 I-9.40 该判成什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedUpperShapeVerdict {
    Holds,
    Violated,
}

/// 两边逐字段相同（对拍字整份：恢复两遍、违例位、判到位、I-1.13 的结局码），看 journal 那一遍的走读结局是 `expected_consulted_outcome`，
/// I-9.40 两边都判成 `expected_upper_shape`（违例时 GPU 红在下标 102）。
fn assert_the_gpu_judge_agrees_with_the_cpu_judge(
    table_name: &str,
    both: &CrossCheckWordsOfBothJudges,
    expected_consulted_outcome: Option<u32>,
    expected_upper_shape: ExpectedUpperShapeVerdict,
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
    if let Some(expected) = expected_consulted_outcome {
        assert_eq!(
            (
                both.cpu.consulted_pass_outcome,
                both.gpu.consulted_pass_outcome
            ),
            (expected, expected),
            "{table_name}：看 journal 那一遍的走读结局（CPU、GPU）；CPU 失败点 {}",
            both.cpu.consulted_pass_failure_point
        );
    }
    let judged = both.gpu.pool_checker_judged_bits & upper_shape_bit() != 0;
    let violated = both.gpu.pool_checker_violated_bits & upper_shape_bit() != 0;
    match expected_upper_shape {
        ExpectedUpperShapeVerdict::Holds => assert!(
            judged && !violated,
            "{table_name}：I-9.40 两边都判、都成立（GPU 判到 {judged}、违例 {violated}）"
        ),
        ExpectedUpperShapeVerdict::Violated => assert!(
            violated,
            "{table_name}：必红那一形 GPU 红在 I-9.40 那一位（下标 102），GPU 判红的 {:?}",
            invariant_names(both.gpu.pool_checker_violated_bits)
        ),
    }
}

/// 一条用例：借卡、两边各判这张镜像、逐字段比。借不到卡、只录那一趟时收手（「本次未跑」那一行已打）。
fn judge_the_image_and_assert_both_judges_agree(
    test_name: &str,
    image: &MemoryPool,
    expected_consulted_outcome: Option<u32>,
    expected_upper_shape: ExpectedUpperShapeVerdict,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(both) = judge_the_image_on_both_judges(cards, test_name, image) else {
        return;
    };
    assert_the_gpu_judge_agrees_with_the_cpu_judge(
        test_name,
        &both,
        expected_consulted_outcome,
        expected_upper_shape,
    );
}

// ───────────────────────── 镜像：照 harness 档 `unlink_of_files_other_than_the_first.rs` 的搭法 ─────────────────────────

/// 这个进程（新池新建文件与它之后在同一个进程里的发布）的实例代号。
const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);
/// 建 inode、按 inode 号写、unlink 那几版的写入时间：比池里第一个文件晚。
const WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE: u64 = FIXED_WRITE_TIME_SECONDS + 600;
/// 一次建 inode 至多建这么多个（一片 inode 叶容器装 233 条，一次跨不过一次末尾分裂）。
const INODES_CREATED_PER_PUBLISH_AT_MOST: u64 = 200;
/// 第一个文件之外那个号落在上段第 3 片叶（产品几何：上段叶罩 143 个号），要层级 1。
const INODE_ON_THE_FOURTH_UPPER_LEAF: u64 = 500;
/// 第一个文件之外那个号落在上段第 1 片叶，写它上段长到常规层。
const INODE_ON_THE_SECOND_UPPER_LEAF: u64 = 150;
/// 删除的各步至多走这么多步：走不到最后一批说明计划出错，不无限循环。
const DELETION_STEPS_AT_MOST: usize = 16;
/// 第一个文件之外这么多个文件各一个单元：上段一片叶里 71 条叶条目，跨过恢复内核一批上段叶条目（64 条）的边界；号都落在第 0 片叶（小于 143）。
const FILES_OTHER_THAN_THE_FIRST_CROSSING_A_BATCH_OF_UPPER_ENTRIES: u64 = 70;

fn upper(level: u8, index: u64) -> ExtentUpperNodePosition {
    ExtentUpperNodePosition { level, index }
}

/// 定长的伪随机字节流（xorshift64，固定种子）：错位一个字节就对不上。
fn content_of(length: usize, seed: u64) -> Vec<u8> {
    let mut state = seed;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

/// `data_units` 个数据单元的内容（最后一个单元差几个字节不满）。
fn content_of_data_units(data_units: usize, seed: u64) -> Vec<u8> {
    content_of(data_units * data_unit_payload_capacity() - 7, seed)
}

/// 同一个进程里建 `count` 个 size 0 的 inode（号从水位起连着发），每次至多 [`INODES_CREATED_PER_PUBLISH_AT_MOST`] 个。
fn create_inodes_in_process(pool: &mut BuiltPool, count: u64) {
    let publish_parameters = parameters();
    let mut left = count;
    // 迭代上界：每一轮至少建一个，⌈count ÷ 200⌉ 轮建完。
    while left > 0 {
        let this_time = left.min(INODES_CREATED_PER_PUBLISH_AT_MOST);
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            this_time,
            WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
            INSTANCE_OF_THE_PROCESS,
        )
        .expect("建 inode");
        left -= this_time;
    }
}

/// 同一个进程里按 inode 号顺序写 `inode` 整份成 `content`。
fn write_in_process(pool: &mut BuiltPool, inode: u64, content: &[u8]) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_sequential_write_of_an_inode(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        content,
        WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
        INSTANCE_OF_THE_PROCESS,
    )
    .unwrap_or_else(|refusal| panic!("按 inode 号写 inode {inode}：{refusal:?}"));
}

/// 同一个进程里发 `inode` 的 unlink 意图。
fn publish_the_unlink_intent_in_process(pool: &mut BuiltPool, inode: u64) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
        INSTANCE_OF_THE_PROCESS,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
}

/// 同一个进程里发 `inode` 删除的下一步（批大小 `batch_size`）。
fn publish_the_next_unlink_step_in_process(
    pool: &mut BuiltPool,
    inode: u64,
    batch_size: UnlinkBatchSize,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_unlink_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        batch_size,
        INSTANCE_OF_THE_PROCESS,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的下一步：{refusal:?}"));
}

/// 同一个进程里删 `inode` 到底：意图那次，再按产品批大小一批批走到最后一批。
fn unlink_to_the_end_in_process(pool: &mut BuiltPool, inode: u64) {
    publish_the_unlink_intent_in_process(pool, inode);
    let mut steps = 0;
    // 迭代上界：每一批至少释放一个单元，[`DELETION_STEPS_AT_MOST`] 步之内走到最后一批（下面断言）。
    while pool.output.deleted_inodes.contains(&InodeNumber(inode)) {
        assert!(
            steps < DELETION_STEPS_AT_MOST,
            "unlink inode {inode} 走了 {steps} 步还没到最后一批"
        );
        publish_the_next_unlink_step_in_process(pool, inode, UnlinkBatchSize::OneExtentLowerLeaf);
        steps += 1;
    }
}

/// 第一个文件之外 inode 2 带三个单元（第一个文件是新池新建文件那一份）。
fn pool_with_inode_two_holding_three_data_units(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    create_inodes_in_process(&mut pool, 1);
    write_in_process(&mut pool, 2, &content_of_data_units(3, 0x2201));
    pool
}

/// 第一个文件与 inode 500 各带数据：有条目的号集合 {1, 500}，上段 (0, 0)、(0, 3)、(1, 0)，根层级 1。
fn pool_with_the_first_file_and_inode_five_hundred(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    create_inodes_in_process(&mut pool, INODE_ON_THE_FOURTH_UPPER_LEAF - 1);
    write_in_process(
        &mut pool,
        INODE_ON_THE_FOURTH_UPPER_LEAF,
        &content_of_data_units(2, 0x5001),
    );
    assert_eq!(
        pool.output
            .extent_tree
            .upper_nodes
            .iter()
            .map(|(position, _)| *position)
            .collect::<Vec<_>>(),
        vec![upper(0, 0), upper(0, 3), upper(1, 0)],
        "起点：S = {{1, 500}}"
    );
    pool
}

/// 第一个文件与 inode 150 各带一个单元：上段 (0, 0)、(0, 1)、(1, 0)，根层级 1。交回池与上段三个节点的指针（根、第 0 片叶、第 1 片叶）。
fn pool_with_the_first_file_and_inode_one_hundred_and_fifty(
    tag: &str,
) -> (BuiltPool, NodePointer, NodePointer, NodePointer) {
    let mut pool = build_pool(tag);
    create_inodes_in_process(&mut pool, INODE_ON_THE_SECOND_UPPER_LEAF - 1);
    write_in_process(
        &mut pool,
        INODE_ON_THE_SECOND_UPPER_LEAF,
        &content_of_data_units(1, 0x1503),
    );
    let pointer_of = |position: ExtentUpperNodePosition| {
        pool.output
            .extent_tree
            .upper_pointer_of(position)
            .unwrap_or_else(|| panic!("上段有 {position:?}"))
    };
    let root = pointer_of(upper(1, 0));
    let leaf_of_the_first_file = pointer_of(upper(0, 0));
    let leaf_of_inode_one_hundred_and_fifty = pointer_of(upper(0, 1));
    (
        pool,
        root,
        leaf_of_the_first_file,
        leaf_of_inode_one_hundred_and_fifty,
    )
}

/// 一个上段节点的身份字段照 `pointer` 指着的那一版取（出生 txg、实例代号；出生序号由调用方另传）：换上去的节点与指着它的指针对得上。
fn identity_of_the_node_at<'identity>(
    pool: &BuiltPool,
    pointer: &NodePointer,
    filesystem_identifier: &'identity [u8; 16],
) -> ExtentTreeNodeIdentity<'identity> {
    ExtentTreeNodeIdentity {
        tree: pool.output.tree_identifiers.extent,
        birth_txg: pointer.head.birth_txg,
        filesystem_identifier,
        instance: pointer.instance,
    }
}

// ───────────────────────── M6：恢复内核收多文件（I-9.40 在这几形上都成立） ─────────────────────────

/// 按 inode 号写出的两文件：第一个文件之外 inode 2 带三个单元（下段一片叶）。两边的恢复都收下它、读到第一个文件。
#[test]
fn the_gpu_judge_recovers_two_files_written_by_inode_number_like_the_cpu_judge() {
    let pool = pool_with_inode_two_holding_three_data_units("gpu-two-files-by-inode-number");
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_recovers_two_files_written_by_inode_number_like_the_cpu_judge",
        &pool.memory_pool(),
        Some(RECOVERY_OUTCOME_CODE_FILE_READ),
        ExpectedUpperShapeVerdict::Holds,
    );
}

/// 上段长到常规层：第一个文件与 inode 500（上段第 3 片叶）。两边的恢复都走两片叶、读到第一个文件。
#[test]
fn the_gpu_judge_recovers_files_under_an_upper_segment_grown_to_level_one_like_the_cpu_judge() {
    let pool = pool_with_the_first_file_and_inode_five_hundred("gpu-upper-segment-at-level-one");
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_recovers_files_under_an_upper_segment_grown_to_level_one_like_the_cpu_judge",
        &pool.memory_pool(),
        Some(RECOVERY_OUTCOME_CODE_FILE_READ),
        ExpectedUpperShapeVerdict::Holds,
    );
}

/// 删到只剩别的文件：上段在常规层 时删掉第一个文件，inode 500 那片叶留在层级 1 的根下面。上段里没有第一个文件那一条，两边都答没有文件。
#[test]
fn the_gpu_judge_recovers_the_version_left_with_another_file_after_the_first_file_is_deleted_like_the_cpu_judge(
) {
    let mut pool =
        pool_with_the_first_file_and_inode_five_hundred("gpu-first-file-deleted-beside-another");
    unlink_to_the_end_in_process(&mut pool, FIRST_INODE_NUMBER);
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_recovers_the_version_left_with_another_file_after_the_first_file_is_deleted_like_the_cpu_judge",
        &pool.memory_pool(),
        Some(RECOVERY_OUTCOME_CODE_NO_FILE),
        ExpectedUpperShapeVerdict::Holds,
    );
}

/// 删掉 inode 500：上段降回层级 0 的第 0 片叶（规范形），两边都读到第一个文件。
#[test]
fn the_gpu_judge_recovers_the_version_whose_upper_segment_was_lowered_by_deleting_another_file_like_the_cpu_judge(
) {
    let mut pool =
        pool_with_the_first_file_and_inode_five_hundred("gpu-another-file-deleted-lowering");
    unlink_to_the_end_in_process(&mut pool, INODE_ON_THE_FOURTH_UPPER_LEAF);
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_recovers_the_version_whose_upper_segment_was_lowered_by_deleting_another_file_like_the_cpu_judge",
        &pool.memory_pool(),
        Some(RECOVERY_OUTCOME_CODE_FILE_READ),
        ExpectedUpperShapeVerdict::Holds,
    );
}

/// 别的文件删除中途：inode 2 三个单元、批大小 1，意图与第一批之后它剩两个单元（标签 1 的下段照收），两边都读到第一个文件。
#[test]
fn the_gpu_judge_recovers_the_version_whose_other_file_is_half_deleted_like_the_cpu_judge() {
    let mut pool = pool_with_inode_two_holding_three_data_units("gpu-another-file-half-deleted");
    publish_the_unlink_intent_in_process(&mut pool, 2);
    publish_the_next_unlink_step_in_process(
        &mut pool,
        2,
        UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=144 里"),
        ),
    );
    assert!(
        pool.output.deleted_inodes.contains(&InodeNumber(2)),
        "第一批之后 inode 2 还在删除队列里"
    );
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_recovers_the_version_whose_other_file_is_half_deleted_like_the_cpu_judge",
        &pool.memory_pool(),
        Some(RECOVERY_OUTCOME_CODE_FILE_READ),
        ExpectedUpperShapeVerdict::Holds,
    );
}

/// 第一个文件之外七十个文件各一个单元（号 2..=71，都在上段第 0 片叶）：上段 71 条叶条目，恢复内核分两批记，两批的文件都照收、单元数都并进映射条目数。
#[test]
fn the_gpu_judge_recovers_files_whose_upper_entries_cross_a_batch_like_the_cpu_judge() {
    let mut pool = build_pool("gpu-upper-entries-crossing-a-batch");
    create_inodes_in_process(
        &mut pool,
        FILES_OTHER_THAN_THE_FIRST_CROSSING_A_BATCH_OF_UPPER_ENTRIES,
    );
    for inode in 2..=FILES_OTHER_THAN_THE_FIRST_CROSSING_A_BATCH_OF_UPPER_ENTRIES + 1 {
        write_in_process(&mut pool, inode, &content_of_data_units(1, 0x7000 + inode));
    }
    assert_eq!(
        pool.output.files_other_than_the_first.len(),
        usize::try_from(FILES_OTHER_THAN_THE_FIRST_CROSSING_A_BATCH_OF_UPPER_ENTRIES)
            .expect("七十"),
        "第一个文件之外七十个文件各带数据"
    );
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_recovers_files_whose_upper_entries_cross_a_batch_like_the_cpu_judge",
        &pool.memory_pool(),
        Some(RECOVERY_OUTCOME_CODE_FILE_READ),
        ExpectedUpperShapeVerdict::Holds,
    );
}

// ───────────────────────── M2：I-9.40 的必红世界 ─────────────────────────

/// 根层级高一层（harness 档 `a_root_left_one_level_above_the_only_leaf_with_an_entry_reddens_the_upper_shape_invariant` 那一形）：{1, 150} 那一版
/// 的层级 1 根换成只指第 0 片叶的那一份。读到的有条目的号集合是 {1}，规范形是层级 0 的第 0 片叶一个，读到的根却在层级 1。
#[test]
fn the_gpu_judge_reddens_the_upper_shape_whose_root_is_one_level_too_high_like_the_cpu_judge() {
    let (pool, root, leaf_of_the_first_file, _) =
        pool_with_the_first_file_and_inode_one_hundred_and_fifty("gpu-upper-shape-root-too-high");
    let filesystem_identifier = parameters().filesystem_identifier;
    let root_with_only_the_first_leaf = build_upper_internal_node(
        &identity_of_the_node_at(&pool, &root, &filesystem_identifier),
        upper(1, 0),
        &[(upper(0, 0), leaf_of_the_first_file)],
        root.birth_sequence,
    );
    let mut image = pool.memory_pool();
    replace_the_unit_and_propagate_its_checksum(
        &mut image,
        the_slot_of(&root),
        &root_with_only_the_first_leaf,
    );
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_reddens_the_upper_shape_whose_root_is_one_level_too_high_like_the_cpu_judge",
        &image,
        None,
        ExpectedUpperShapeVerdict::Violated,
    );
}

/// 多一个脊线节点（只升不降那种形状）：{1, 150} 那一版的根换成层级 2 的 (2, 0)、只指 (1, 0)；(1, 0) 写在原来 150 那片叶的槽上、只指第 0 片叶。
/// 读到的号集合是 {1}，规范形是层级 0 一片叶，读到的根在层级 2，多出 (2, 0)、(1, 0) 两个脊线节点。
#[test]
fn the_gpu_judge_reddens_the_upper_shape_with_an_extra_spine_node_like_the_cpu_judge() {
    let (pool, root, leaf_of_the_first_file, leaf_of_inode_one_hundred_and_fifty) =
        pool_with_the_first_file_and_inode_one_hundred_and_fifty("gpu-upper-shape-extra-spine");
    let filesystem_identifier = parameters().filesystem_identifier;
    let root_at_level_two = build_upper_internal_node(
        &identity_of_the_node_at(&pool, &root, &filesystem_identifier),
        upper(2, 0),
        &[(upper(1, 0), leaf_of_inode_one_hundred_and_fifty)],
        root.birth_sequence,
    );
    let spine_node_at_level_one = build_upper_internal_node(
        &identity_of_the_node_at(
            &pool,
            &leaf_of_inode_one_hundred_and_fifty,
            &filesystem_identifier,
        ),
        upper(1, 0),
        &[(upper(0, 0), leaf_of_the_first_file)],
        leaf_of_inode_one_hundred_and_fifty.birth_sequence,
    );
    let mut image = pool.memory_pool();
    // 先换根（它指着 150 那片叶的槽，指针里的校验和还是那片叶的），再换那个槽：新的整单元校验和沿引用链补进新根与根槽
    replace_the_unit_and_propagate_its_checksum(&mut image, the_slot_of(&root), &root_at_level_two);
    replace_the_unit_and_propagate_its_checksum(
        &mut image,
        the_slot_of(&leaf_of_inode_one_hundred_and_fifty),
        &spine_node_at_level_one,
    );
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_reddens_the_upper_shape_with_an_extra_spine_node_like_the_cpu_judge",
        &image,
        None,
        ExpectedUpperShapeVerdict::Violated,
    );
}

/// 少一片该有的叶：{1, 150} 那一版的根换成只有 (0, 0) 一条条目，指向 150 那片叶的槽；那个槽上换成位置 (0, 0)、装着 1 与 150 两条叶条目的叶
/// （150 落在第 0 片叶的区间之外），身份照 150 那片叶取。读到的号集合是 {1, 150}、根层级 1 是最低的那一层、每个读到的节点都罩着有条目的号，
/// 只有规范形要的 (0, 1) 没读到（I-9.40 两条比法里「号找它那片叶」那一条判出）。
/// 坏叶不写在第一个文件那片叶的槽上：那片叶自 txg 3 起没重写过，txg 3、4 两条根的上段就是它（层级 0），改它的字节会让那两版在「根层级」那一条上
/// 先红，「号找它那片叶」那一条改掉也照样红。150 那片叶是 txg 5 那一版才写的，只有最新那一版引用它。
#[test]
fn the_gpu_judge_reddens_the_upper_shape_missing_the_leaf_of_an_entry_like_the_cpu_judge() {
    let (pool, root, leaf_of_the_first_file, leaf_of_inode_one_hundred_and_fifty) =
        pool_with_the_first_file_and_inode_one_hundred_and_fifty("gpu-upper-shape-missing-leaf");
    let filesystem_identifier = parameters().filesystem_identifier;
    let root_with_one_entry_on_the_slot_of_the_other_leaf = build_upper_internal_node(
        &identity_of_the_node_at(&pool, &root, &filesystem_identifier),
        upper(1, 0),
        &[(upper(0, 0), leaf_of_inode_one_hundred_and_fifty)],
        root.birth_sequence,
    );
    let mut image = pool.memory_pool();
    let first_leaf = parse_index_node(&unit_bytes_at(
        &image,
        the_slot_of(&leaf_of_the_first_file),
        node_bytes(),
    ))
    .expect("第 0 片叶解得开");
    let leaf_of_one_hundred_and_fifty = parse_index_node(&unit_bytes_at(
        &image,
        the_slot_of(&leaf_of_inode_one_hundred_and_fifty),
        node_bytes(),
    ))
    .expect("150 那片叶解得开");
    assert!(
        leaf_of_the_first_file.head.birth_txg < leaf_of_inode_one_hundred_and_fifty.head.birth_txg,
        "第一个文件那片叶比 150 那片叶早诞生（早先那几版的上段就是它），坏叶写在 150 那片叶的槽上只动最新那一版"
    );
    let entries: Vec<Vec<u8>> = first_leaf
        .entries
        .iter()
        .chain(&leaf_of_one_hundred_and_fifty.entries)
        .cloned()
        .collect();
    assert_eq!(
        entries.len(),
        2,
        "第 0 片叶一条（号 1）、第 1 片叶一条（号 150）"
    );
    let (smallest_key, largest_key) = upper(0, 0).key_range();
    let leaf_at_the_first_position_holding_both_entries = build_index_node(
        leaf_of_one_hundred_and_fifty.tree,
        0,
        leaf_of_one_hundred_and_fifty.key_width,
        &smallest_key,
        &largest_key,
        leaf_of_one_hundred_and_fifty.birth_txg,
        &filesystem_identifier,
        leaf_of_one_hundred_and_fifty.instance,
        leaf_of_one_hundred_and_fifty.birth_sequence,
        u16::try_from(leaf_of_one_hundred_and_fifty.entry_width).expect("上段叶条目宽 113"),
        &entries,
    );
    // 先换根（它的唯一一条条目指着 150 那片叶的槽，指针里的校验和还是那片叶的），再换那个槽：新的整单元校验和沿引用链补进新根与根槽
    replace_the_unit_and_propagate_its_checksum(
        &mut image,
        the_slot_of(&root),
        &root_with_one_entry_on_the_slot_of_the_other_leaf,
    );
    replace_the_unit_and_propagate_its_checksum(
        &mut image,
        the_slot_of(&leaf_of_inode_one_hundred_and_fifty),
        &leaf_at_the_first_position_holding_both_entries,
    );
    judge_the_image_and_assert_both_judges_agree(
        "the_gpu_judge_reddens_the_upper_shape_missing_the_leaf_of_an_entry_like_the_cpu_judge",
        &image,
        None,
        ExpectedUpperShapeVerdict::Violated,
    );
}
