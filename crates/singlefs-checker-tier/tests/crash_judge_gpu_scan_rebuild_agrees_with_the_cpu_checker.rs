//! checker 档模块：crash、crash_identity、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器的扫描重建那一截（扫描内核源码里的第三个入口 `scan_rebuild_main`，`crates/singlefs-checker-tier/src/crash_judge_checker_scan_rebuild.wgsl`）
//! 判 I-1.13（扫描重建与遍历方向的索引相同），与 CPU 池级 checker（`crates/singlefs-checker/src/walk.rs` 的
//! `judge_scan_rebuild_agrees_with_the_walked_index` 与 `crates/singlefs-checker/src/rebuild.rs`）逐位相同（欠账 C821，用户 2026-10-18 定
//! 「补 GPU 扫描重建内核」，两路结论逐位相同）：每张镜像两边各判一遍，逐条不变量的违例位与判到位整份比（不再去掉 I-1.13 那一位），
//! I-1.13 的结局（成立、违例、报不适用的哪一种理由）另比 GPU 结论里扫描重建那一字（`crash_judge_gpu::SCAN_REBUILD_OUTCOME_WORD`）与 CPU 那一句
//! 不适用的理由换成的码，再与这一形该判成的那一种比；必红那几形 GPU 一侧红在下标 92。
//!
//! 镜像照 harness 档那几份用例的搭法造（真写路径，必红世界在写路径的镜像上改字节）：`scan_rebuild_agrees_with_the_walked_index.rs`、
//! `scan_rebuild_reads_lineage_and_retirement_records.rs`、`checker_truncation_field_must_red_worlds.rs`、
//! `checker_relocated_images_judged_by_the_mapping_position.rs`、`checker_snapshot_space_accounting_must_red_worlds.rs` 里成立、必红与报不适用的那几形；
//! 共用的搭建经 `#[path]` 指回 harness 档的 `tests/common*/mod.rs`，那几份文件里各自就地写的改字节函数抄在这里（harness 档不依赖 checker 档）。
//! 每张镜像两边的输入都经对拍那一类的库：GPU 那一侧把镜像当基线、一个空持久集合建判表，判表入库再读回上卡；CPU 那一侧把镜像作一张给定状态表
//! 写进库、读回，池级 checker 判读回的那一份；两边判出的东西作一份对拍结论入库（用户 2026-10-15 定「对拍结论入库」，
//! `crash_library::pairing_conclusion_reused_or_judged`），写法照 `tests/crash_judge_gpu_secondary_reads_after_the_source_segment_is_discarded.rs`。
//! GPU 判不了判红。要本机一张 NVIDIA 独显：一张适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。
//! 这几条是 GPU 判器（加速路径）与 CPU 池级 checker（参照路径）的对拍，照 `.claude/rules/test-script-discipline.md`「分快慢两档」一律进快档、
//! 不看快慢判据，都不标 `#[ignore]`。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_truncation/mod.rs"]
mod common_truncation;

use common::{
    build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, publish_overwrite_in_process,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use common_deletion_in_progress::{
    append_a_copy_of_the_first_tombstone_of_the_newest_root, change_the_inode_record,
    change_the_unit_and_propagate_its_checksum, content_of_three_data_units,
    first_tombstone_container_slot_of_the_newest_root, image_after_the_intent_of_a_three_unit_file,
    object_birth_of, pool_with_a_three_unit_file, pool_with_new_inodes,
    publish_new_inodes_and_advance_the_pool, publish_the_intent,
    publish_the_next_step_in_batches_of_one, roll_back_to, set_tombstone_field, unlink_to_the_end,
    INSTANCE_OF_THE_PUBLISHES, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
    WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use common_pointer_head_corruption::{
    allocation_record_tree_nodes_as_far_as_readable, data_unit_bytes, newest_root, node_bytes,
    replace_the_unit_and_propagate_its_checksum, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at,
};
use common_truncation::{
    batches_of_one, change_the_size_of_the_first_file_and_advance, payload_capacity,
    publish_the_next_truncation_step_and_advance, truncation_entry_of,
};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::CrashPointReplaySegmentExpansion;
use singlefs_checker_tier::crash_amplification::RecordedGivenStatesTable;
use singlefs_checker_tier::crash_identity::{judging_units_key, JudgingUnits, SectorImage};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_judge_gpu::{
    cross_check_words_of_gpu_verdict, pool_checker_violated_and_judged_bits,
    scan_rebuild_outcome_name, scan_rebuild_outcome_of_the_gpu_verdict,
    scan_rebuild_outcome_of_the_pool_checker_verdicts,
    KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE, SCAN_REBUILD_OUTCOMES, VERDICT_WORDS,
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
use singlefs_core::address::{
    DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::ALLOCATION_RECORD_RELEASED_FLAG;
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::records::{
    build_mapping_entry, mapping_key_for_data, mapping_key_for_node, mapping_key_sort_key,
    TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_EXTENT,
};
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_core::transaction::{
    publish_create_snapshot, publish_new_inodes, publish_sequential_write_of_an_inode, PoolWriter,
    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{
    build_index_node, parse_index_node, seal_header_checksum, UNIT_CLASS_PACKED,
};
use singlefs_format::{
    CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES, CLUSTER_SEGMENT_SLOTS, MAPPING_ENTRY_BYTES,
    MAPPING_KEY_BYTES, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, TOMBSTONE_RECORD_BYTES,
    UNIT_AREA_START_SLOT,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::sha256::sha256_digest;

// ───────────────────────── 对拍：两边各判一遍、结论入库 ─────────────────────────

/// 一张镜像两边各判成什么：逐条不变量的违例位与判到位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）、I-1.13 的结局码（CPU 那一侧从理由换成码），
/// 加 CPU 那一侧的逐条判定（判红时写进说明）。
struct BothJudgesOnAnImage {
    cpu_verdicts: Vec<(&'static str, InvariantVerdict)>,
    cpu_bits: (u128, u128),
    gpu_bits: (u128, u128),
    cpu_scan_rebuild_outcome: u32,
    gpu_scan_rebuild_outcome: u32,
}

/// 存进对拍那一类库的对拍结论：CPU 池级 checker 在读回的那一份上逐条的判定，与 GPU 判器那一个状态的结论字。只存两边判出的东西、
/// 不存比的结果：比法与断言每趟照读回的这一份现比，所以比法不进键。
struct JudgementsOfBothJudgesOnAnImage {
    cpu_verdicts: Vec<(&'static str, InvariantVerdict)>,
    gpu_verdict_words: [u32; VERDICT_WORDS],
}

/// 一条判定在对拍结论字节里的种类码与它带的那句话（成立不带话）。
fn kind_code_and_text_of(verdict: &InvariantVerdict) -> (u8, &str) {
    match verdict {
        InvariantVerdict::Holds => (0, ""),
        InvariantVerdict::Violated(detail) => (1, detail),
        InvariantVerdict::NotApplicable(reason) => (2, reason),
        InvariantVerdict::PartiallyJudged(detail) => (3, detail),
    }
}

fn put_length_prefixed(bytes: &mut Vec<u8>, field: &[u8]) {
    bytes.extend_from_slice(
        &u64::try_from(field.len())
            .expect("长度装得进 u64")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(field);
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

    fn take_u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn take_length_prefixed(&mut self) -> Option<&'bytes [u8]> {
        let length_in_bytes = usize::try_from(self.take_u64()?).ok()?;
        self.take(length_in_bytes)
    }
}

impl JudgementsOfBothJudgesOnAnImage {
    /// 逐条判定（不变量名、种类码、那句话，名与话各带长度）之前写条数，之后接 GPU 结论字（小端）。
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(
            &u64::try_from(self.cpu_verdicts.len())
                .expect("条数装得进 u64")
                .to_le_bytes(),
        );
        for (invariant, verdict) in &self.cpu_verdicts {
            put_length_prefixed(&mut bytes, invariant.as_bytes());
            let (kind_code, text) = kind_code_and_text_of(verdict);
            bytes.push(kind_code);
            put_length_prefixed(&mut bytes, text.as_bytes());
        }
        for word in &self.gpu_verdict_words {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    /// [`Self::to_bytes`] 的逆：不变量名对回 `IMPLEMENTED_INVARIANTS` 里那一项；字节不全、多出、名字不在清单里、种类码认不得都交 `None`。
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = ConclusionBytesReader { remaining: bytes };
        let verdict_count = usize::try_from(reader.take_u64()?).ok()?;
        let mut cpu_verdicts = Vec::with_capacity(verdict_count);
        for _verdict in 0..verdict_count {
            let invariant_name = reader.take_length_prefixed()?;
            let invariant: &'static str = IMPLEMENTED_INVARIANTS
                .iter()
                .copied()
                .find(|candidate| candidate.as_bytes() == invariant_name)?;
            let kind_code = reader.take(1)?[0];
            let text = String::from_utf8(reader.take_length_prefixed()?.to_vec()).ok()?;
            let verdict = match kind_code {
                0 => InvariantVerdict::Holds,
                1 => InvariantVerdict::Violated(text),
                // 不适用的那句话是 `&'static str`：读回的这一份活到测试进程结束，泄漏这几句话换得与池级 checker 交的同一种判定
                2 => InvariantVerdict::NotApplicable(text.leak()),
                3 => InvariantVerdict::PartiallyJudged(text),
                _unrecognized_kind_code => return None,
            };
            cpu_verdicts.push((invariant, verdict));
        }
        let mut gpu_verdict_words = [0u32; VERDICT_WORDS];
        for word in &mut gpu_verdict_words {
            *word = u32::from_le_bytes(reader.take(4)?.try_into().ok()?);
        }
        reader.remaining.is_empty().then_some(Self {
            cpu_verdicts,
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
            "gpu judge scan rebuild pairing inputs\nbase={}\ntables={}\n",
            hexadecimal_text(&SectorImage::of_base(image).digest()),
            hexadecimal_text(&tables.digest())
        )
        .as_bytes(),
    )
}

/// CPU 那一侧：镜像作一张给定状态表写进对拍那一类的库、读回，池级 checker 判读回的那一份。只录那一趟交 `None`。
// cpu-reference-for-gpu-comparison
fn cpu_verdicts_of_the_image_read_back_from_the_pairing_library(
    table_name: &str,
    image: &MemoryPool,
) -> Option<Vec<(&'static str, InvariantVerdict)>> {
    let read_back =
        cpu_reference_table_read_back_from_the_pairing_library(&RecordedGivenStatesTable {
            table_name: table_name.to_string(),
            base: image.clone(),
            writes: Vec::new(),
            segments: Vec::new(),
            seams: Vec::new(),
            persisted_sets: vec![Vec::new()],
        })?;
    Some(check_pool_image(&read_back.base))
}

/// 对拍本身：CPU 那一侧从库里读回的那张表上判、GPU 那一侧从库里读回的判表上判（判表入库再读回上卡），交回两边判出的东西。
/// 只录那一趟、卡借不到交 `None`。
fn judgements_of_both_judges_on_the_image(
    cards: &'static GpuCardPool,
    table_name: &str,
    image: &MemoryPool,
    tables: &JudgeTables,
) -> Option<JudgementsOfBothJudgesOnAnImage> {
    let cpu_verdicts =
        cpu_verdicts_of_the_image_read_back_from_the_pairing_library(table_name, image)?;
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, table_name, tables)?;
    let [gpu_verdict_words] = gpu.as_slice() else {
        panic!("{table_name}：一个状态一份结论，交回 {} 份", gpu.len());
    };
    Some(JudgementsOfBothJudgesOnAnImage {
        cpu_verdicts,
        gpu_verdict_words: *gpu_verdict_words,
    })
}

/// 一张镜像两边各判一次（GPU 那一侧把镜像当基线、一个空持久集合建判表），对拍结论按键入库、键相同的复用（同
/// `tests/crash_judge_gpu_secondary_reads_after_the_source_segment_is_discarded.rs` 的 `judge_the_image_on_both_judges`）。
/// 每趟打一行 `GPU_SCAN_REBUILD_PAIRING_CONCLUSION table=<表名> source=<ReusedFromTheLibrary|JudgedNow>`。
/// 只录那一趟、卡借不到交 `None`（不存进库）；GPU 判不了（建不出判表，或结论里判不了那一位）判红。
fn judge_the_image_on_both_judges(
    cards: &'static GpuCardPool,
    table_name: &str,
    image: &MemoryPool,
) -> Option<BothJudgesOnAnImage> {
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
            &judging_units(JudgingUnits::JudgementOfTheCrashImages),
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
    println!("GPU_SCAN_REBUILD_PAIRING_CONCLUSION table={table_name} source={source:?}");
    let judgements = JudgementsOfBothJudgesOnAnImage::from_bytes(&conclusion)
        .expect("库里这一键下的对拍结论是这份测试写的，认得回");
    let gpu_words = cross_check_words_of_gpu_verdict(&judgements.gpu_verdict_words);
    assert!(
        !gpu_words.gpu_could_not_judge,
        "{table_name}：GPU 判器在这张镜像上判不了（结论里判不了那一位置了）"
    );
    let cpu_bits = pool_checker_violated_and_judged_bits(&judgements.cpu_verdicts);
    let cpu_scan_rebuild_outcome =
        scan_rebuild_outcome_of_the_pool_checker_verdicts(&judgements.cpu_verdicts)
            .unwrap_or_else(|reason| panic!("{table_name}：{reason}"));
    Some(BothJudgesOnAnImage {
        cpu_verdicts: judgements.cpu_verdicts,
        cpu_bits,
        gpu_bits: (
            gpu_words.pool_checker_violated_bits,
            gpu_words.pool_checker_judged_bits,
        ),
        cpu_scan_rebuild_outcome,
        gpu_scan_rebuild_outcome: scan_rebuild_outcome_of_the_gpu_verdict(
            &judgements.gpu_verdict_words,
        ),
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

/// 结局表里名字是 `constant_name` 的那一种的码。
fn scan_rebuild_outcome_code_named(constant_name: &str) -> u32 {
    SCAN_REBUILD_OUTCOMES
        .iter()
        .find(|outcome| outcome.constant_name == constant_name)
        .unwrap_or_else(|| panic!("结局表里有 {constant_name}"))
        .code
}

/// I-1.13 在违例位与判到位里的那一位（内核写死的下标，`crash_judge_gpu::KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE`）。
fn scan_rebuild_bit() -> u128 {
    let [(_, index)] = KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE;
    1u128 << index
}

/// 两边逐位相同：违例位与判到位整份比（每一条不变量，连同 I-1.13）、I-1.13 的结局码两边相同，并且就是这一形该判成的 `expected_outcome`；
/// 该判违例的，GPU 那一侧红在 I-1.13 那一位。
fn assert_the_gpu_scan_rebuild_agrees_with_the_cpu_checker(
    table_name: &str,
    both: &BothJudgesOnAnImage,
    expected_outcome: &str,
) {
    let (cpu_violated, cpu_judged) = both.cpu_bits;
    let (gpu_violated, gpu_judged) = both.gpu_bits;
    assert!(
        gpu_violated == cpu_violated && gpu_judged == cpu_judged,
        "{table_name}：GPU 判器与 CPU 池级 checker 逐位不同——违例位不同的 {:?}，判到位不同的 {:?}（CPU 判红的 {:?}；GPU 判红的 {:?}）",
        invariant_names(gpu_violated ^ cpu_violated),
        invariant_names(gpu_judged ^ cpu_judged),
        invariant_names(cpu_violated),
        invariant_names(gpu_violated),
    );
    assert_eq!(
        (
            scan_rebuild_outcome_name(both.cpu_scan_rebuild_outcome),
            scan_rebuild_outcome_name(both.gpu_scan_rebuild_outcome)
        ),
        (expected_outcome.to_string(), expected_outcome.to_string()),
        "{table_name}：I-1.13 的结局（CPU、GPU）都要是 {expected_outcome}；CPU 那一侧 I-1.13 判成 {:?}",
        both.cpu_verdicts
            .iter()
            .find(|(invariant, _)| *invariant
                == singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX)
            .map(|(_, verdict)| verdict)
    );
    if both.gpu_scan_rebuild_outcome
        == scan_rebuild_outcome_code_named("SCAN_REBUILD_OUTCOME_VIOLATED")
    {
        assert!(
            gpu_violated & scan_rebuild_bit() != 0,
            "{table_name}：必红那一形 GPU 一侧红在 I-1.13 那一位，GPU 判红的 {:?}",
            invariant_names(gpu_violated)
        );
    }
}

/// 一条用例：借卡、两边各判这张镜像、逐位比，再比 I-1.13 的结局是 `expected_outcome`。借不到卡、只录那一趟时收手（「本次未跑」那一行已打）。
fn judge_the_image_and_assert_the_scan_rebuild_outcome(
    test_name: &str,
    image: &MemoryPool,
    expected_outcome: &str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(both) = judge_the_image_on_both_judges(cards, test_name, image) else {
        return;
    };
    assert_the_gpu_scan_rebuild_agrees_with_the_cpu_checker(test_name, &both, expected_outcome);
}

// ───────────────────────── 镜像：照 harness 档那几份用例的搭法 ─────────────────────────

/// 中央映射树节点（码 2，key 宽 27）的叶条目 55 = key 27 + 位置条目 14 × 2；第一条位置条目的槽号在 key 之后 4 字节起（`scan_rebuild_agrees_with_the_walked_index.rs`）。
const MAPPING_KEY_WIDTH: usize = 27;
const FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY: usize = MAPPING_KEY_WIDTH + 4;
/// 码 1 单元头里对象 key 那一段（五元组：类标签 42、树 43、对象 51、对象出生代 59、锚点偏移 67）。
const DATA_UNIT_OBJECT_KEY: std::ops::Range<usize> = 42..75;
/// 单元头所在的第一扇区（内存盘按 512 字节的扇区写）。
const HEADER_SECTOR_BYTES: usize = 512;
/// 码 1 数据单元在中央映射 key 里的类标签。
const MAPPING_KEY_CLASS_OF_A_DATA_UNIT: u8 = 1;
/// 换上去的那个槽离原槽多远：单元区里新池新建文件之后没写过的一格。
const SLOTS_THE_MAPPING_LOCATION_IS_MOVED_BY: u64 = 2;

/// 码 3 单元头里声明长度、载荷 CRC、记录数、记录宽的位置，明文头到 107、记录从 136 起（`scan_rebuild_reads_lineage_and_retirement_records.rs`）。
const PACKED_DECLARED_LENGTH_OFFSET: usize = 8;
const PACKED_PAYLOAD_CHECKSUM_OFFSET: usize = 89;
const PACKED_RECORD_COUNT_OFFSET: usize = 69;
const PACKED_RECORD_WIDTH_OFFSET: usize = 71;
const PACKED_PLAIN_HEADER_END: usize = 107;
const PACKED_RECORDS_OFFSET: usize = 136;
/// 第一个文件之外再建的空 inode 个数：234 条记录装成两片 inode 叶容器。
const NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER: u64 = 233;
/// 右边那片叶里唯一的那条记录：unlink 它，那片叶删空摘掉、退役。
const THE_ONLY_INODE_OF_THE_RIGHT_LEAF: u64 = 234;
/// 左边那片叶里的一条：unlink 它，左片吸收右片、右片退役。
const AN_INODE_WHOSE_UNLINK_ABSORBS_THE_RIGHT_LEAF: u64 = 2;

/// inode 记录 140 里截断字段的两段：T 在偏移 120、N_w 在偏移 128（`checker_truncation_field_must_red_worlds.rs`）。
const INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET: usize = 120;
const INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET: usize = 128;
/// 系统配置槽里 incompat 位图第一个字节的偏移与 incompat 位 2（inode 记录带截断字段）。
const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
const INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT: u8 = 0x04;
/// 码 1 单元头里诞生代号那 8 字节的偏移。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
/// 码 2 / 码 3 映射 key 里出生序号那 4 字节的起点：类标签 1、出生树 8、出生 txg 8、实例代号 4 之后（`singlefs_core::records::mapping_key_for_node`）。
const PACKED_MAPPING_KEY_BIRTH_SEQUENCE_OFFSET: usize = 21;

/// 一个和第一个文件一样长、内容不同的版本（第 `seed` 版）。
fn content_of_the_same_length(seed: usize) -> Vec<u8> {
    file_content()
        .iter()
        .map(|byte| byte.wrapping_add(u8::try_from(seed).expect("版本序号小于 256")))
        .collect()
}

/// 码 2 节点改过之后重封：先算载荷 CRC（罩 [86 + 2k, 末尾)、住 76 + 2k），再算头校验和（罩 [0, 86 + 2k)）。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 必红世界 ①：最新根的中央映射树（新池新建文件之后是根兼叶）里第一个文件那个数据单元的条目，第一条位置条目的槽号挪到别的槽；
/// 映射叶重封、新的整单元校验和沿引用链补到根槽。数据单元与 extent 树都不动。
fn move_the_first_location_of_the_data_unit_mapping_entry(image: &mut MemoryPool) {
    let mapping_root = newest_root(image).mapping_root;
    let leaf_slot = the_slot_of(&mapping_root);
    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
    assert_eq!(
        usize::from(leaf[51]),
        MAPPING_KEY_WIDTH,
        "中央映射树的根兼叶 key 宽 27"
    );
    assert_eq!(leaf[50], 0, "新池新建文件之后中央映射树只有一层：根就是叶");
    let entries_start = 115 + 2 * MAPPING_KEY_WIDTH;
    let entry_count = usize::from(u16::from_le_bytes(
        leaf[82 + 2 * MAPPING_KEY_WIDTH..84 + 2 * MAPPING_KEY_WIDTH]
            .try_into()
            .expect("2 字节"),
    ));
    let data_unit_entry_start = (0..entry_count)
        .map(|index| entries_start + index * mapping_entry_bytes())
        .find(|start| leaf[*start] == MAPPING_KEY_CLASS_OF_A_DATA_UNIT)
        .expect("中央映射里有第一个文件那个数据单元的条目");
    let slot_field_start = data_unit_entry_start + FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY;
    let original_slot = read_six_bytes_at(&leaf, slot_field_start);
    let moved_slot = original_slot + SLOTS_THE_MAPPING_LOCATION_IS_MOVED_BY;
    leaf[slot_field_start..slot_field_start + 6].copy_from_slice(&moved_slot.to_le_bytes()[..6]);
    reseal_index_node(&mut leaf);
    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
}

/// 一份码 1 副本头里的对象 key 清零、不重封：内存盘按扇区写，读回头所在那一扇区、改、整扇区写回。
fn zero_the_object_key_of_the_copy_at(image: &mut MemoryPool, location: LocationEntry) {
    let device = image
        .devices
        .get_mut(&location.device)
        .expect("位置条目指的盘在池里");
    let sector_start = location.slot.to_device_offset();
    let mut first_sector = device.read(sector_start, HEADER_SECTOR_BYTES);
    first_sector[DATA_UNIT_OBJECT_KEY].fill(0);
    device.write(sector_start, &first_sector);
}

/// 第一个文件删一半的那张镜像上，映射条目的落点挪多远：单元区里那几次发布都没写到的一格（harness 档同名常量，挪两槽会落在 inode 2 的下一个单元上）。
const SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY: u64 = 4096;

/// 最新根的中央映射树（根兼叶）里 key 是 `mapping_key` 的那条条目，第一条位置条目的槽号挪
/// [`SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY`] 槽；映射叶重封、新的整单元校验和沿引用链补到根槽（harness 档
/// `scan_rebuild_agrees_with_the_walked_index.rs` 的同名函数，harness 档不依赖 checker 档，抄在这里）。
fn move_the_first_location_of_the_mapping_entry_keyed(image: &mut MemoryPool, mapping_key: &[u8]) {
    let mapping_root = newest_root(image).mapping_root;
    let leaf_slot = the_slot_of(&mapping_root);
    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
    assert_eq!(leaf[50], 0, "这张镜像上中央映射树只有一层：根就是叶");
    let entries_start = 115 + 2 * MAPPING_KEY_WIDTH;
    let entry_count = usize::from(u16::from_le_bytes(
        leaf[82 + 2 * MAPPING_KEY_WIDTH..84 + 2 * MAPPING_KEY_WIDTH]
            .try_into()
            .expect("2 字节"),
    ));
    let entry_start = (0..entry_count)
        .map(|index| entries_start + index * mapping_entry_bytes())
        .find(|start| leaf[*start..*start + MAPPING_KEY_WIDTH] == *mapping_key)
        .expect("中央映射里有这把 key 的条目");
    let slot_field_start = entry_start + FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY;
    let original_slot = read_six_bytes_at(&leaf, slot_field_start);
    let moved_slot = original_slot + SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY;
    leaf[slot_field_start..slot_field_start + 6].copy_from_slice(&moved_slot.to_le_bytes()[..6]);
    reseal_index_node(&mut leaf);
    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
}

/// 攻方 F 段那一形（harness 档 `scan_rebuild_agrees_with_the_walked_index.rs` 的同名函数）：第一个文件顺序写成三个单元，建 inode 2、按 inode 号写它
/// 两个单元，再删第一个文件：发完意图与第一批（批大小 1）。这一版里 inode 1 删除中（墓碑压着、记录还在、剩两个单元），inode 2 活着。
fn pool_with_the_first_file_half_deleted_beside_inode_two(tag: &str) -> BuiltPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let content_of_inode_two: Vec<u8> = (0..2 * payload_capacity() - 5)
        .map(|index| u8::try_from((index * 7 + 3) % 251).expect("小于 256"))
        .collect();
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let written = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write_of_an_inode(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(2),
            &content_of_inode_two,
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("按 inode 号顺序写 inode 2 两个单元")
    };
    pool.output = written;
    publish_the_intent(&mut pool);
    let after_the_first_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    assert_eq!(
        (
            after_the_first_batch.data_pointers.len(),
            after_the_first_batch.deleted_inodes.clone()
        ),
        (2, vec![InodeNumber(FIRST_INODE_NUMBER)]),
        "第一批之后第一个文件剩两个单元、还在删除队列里"
    );
    pool
}

/// 第一个文件那一版之后覆盖写一版（同长、每个字节加一），再挂着回退到第一个文件那一版：谱系树里一条记录。
fn pool_rolled_back_past_an_overwrite_of_the_same_length(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    let content_differing_in_every_byte: Vec<u8> = file_content()
        .iter()
        .map(|byte| byte.wrapping_add(1))
        .collect();
    overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_differing_in_every_byte,
    );
    roll_back_to(&mut pool, &first_file_version);
    assert_eq!(
        pool.output
            .lineage_containers
            .iter()
            .map(|container| container.contents.records.len())
            .sum::<usize>(),
        1,
        "回退那次发布写了一条谱系记录"
    );
    pool
}

/// 再建 233 个空 inode 之后 unlink `inode` 到底：右片叶退役，退役树里一条记录。
fn pool_whose_right_leaf_retired_after_the_unlink_of(tag: &str, inode: u64) -> BuiltPool {
    let mut pool = pool_with_new_inodes(tag, NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER);
    assert_eq!(
        pool.output.inode_leaf_containers.len(),
        2,
        "234 条记录装成两片叶"
    );
    unlink_to_the_end(&mut pool, inode);
    assert_eq!(
        (
            pool.output.inode_leaf_containers.len(),
            pool.output
                .retirement_containers
                .iter()
                .map(|container| container.contents.records.len())
                .sum::<usize>(),
        ),
        (1, 1),
        "unlink inode {inode} 到底之后只剩一片叶，右片退役、退役树里一条记录"
    );
    pool
}

/// 一片码 3 容器的记录全部抹掉：记录数、声明长度清零，记录那一段清零，载荷 CRC 与头校验和重封，新的整单元校验和沿引用链补到根槽。
fn erase_every_record_of_the_container(image: &mut MemoryPool, pointer: &NodePointer) {
    let slot = the_slot_of(pointer);
    let mut unit = unit_bytes_at(image, slot, data_unit_bytes());
    let record_count = usize::from(read_u16_at(&unit, PACKED_RECORD_COUNT_OFFSET));
    let record_width = usize::from(read_u16_at(&unit, PACKED_RECORD_WIDTH_OFFSET));
    assert!(record_count > 0, "要抹的那片容器里有记录");
    unit[PACKED_RECORDS_OFFSET..PACKED_RECORDS_OFFSET + record_count * record_width].fill(0);
    unit[PACKED_RECORD_COUNT_OFFSET..PACKED_RECORD_COUNT_OFFSET + 2].fill(0);
    unit[PACKED_DECLARED_LENGTH_OFFSET..PACKED_DECLARED_LENGTH_OFFSET + 2].fill(0);
    let payload_checksum = crc32_castagnoli(&unit[PACKED_PLAIN_HEADER_END..]);
    unit[PACKED_PAYLOAD_CHECKSUM_OFFSET..PACKED_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(&mut unit, PACKED_PLAIN_HEADER_END);
    replace_the_unit_and_propagate_its_checksum(image, slot, &unit);
}

/// 中央映射树节点（码 2，key 宽 27）头里随条目变的几处：声明长度 8、层级 50、最小 key 52、最大 key 79、条目数 136
/// （`scan_rebuild_reads_lineage_and_retirement_records.rs`）。
const MAPPING_NODE_DECLARED_LENGTH_OFFSET: usize = 8;
const MAPPING_NODE_LEVEL_OFFSET: usize = 50;
const MAPPING_NODE_SMALLEST_KEY_OFFSET: usize = 52;
const MAPPING_NODE_LARGEST_KEY_OFFSET: usize = 79;
const MAPPING_NODE_ENTRY_COUNT_OFFSET: usize = 136;

/// 最新根的中央映射树（这张镜像上是根兼叶）里按 key 序插进 `entry` 那一条：条目数、声明长度、最小与最大 key 跟着改，重封，
/// 新的整单元校验和沿引用链补到根槽（harness 档 `scan_rebuild_reads_lineage_and_retirement_records.rs` 的同名函数，harness 档不依赖 checker 档，抄在这里）。
fn insert_the_entry_into_the_newest_mapping_leaf(image: &mut MemoryPool, entry: &[u8]) {
    let key_width = mapping_key_bytes();
    let entry_width = mapping_entry_bytes();
    let entries_start = usize::try_from(CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES).expect("169");
    assert_eq!(entry.len(), entry_width, "要插的是一条整的映射叶条目");
    let leaf_slot = the_slot_of(&newest_root(image).mapping_root);
    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
    assert_eq!(
        leaf[MAPPING_NODE_LEVEL_OFFSET], 0,
        "这张镜像上中央映射树只有一层：根就是叶"
    );
    let entry_count = usize::from(read_u16_at(&leaf, MAPPING_NODE_ENTRY_COUNT_OFFSET));
    let mut entries: Vec<Vec<u8>> = (0..entry_count)
        .map(|index| {
            let start = entries_start + index * entry_width;
            leaf[start..start + entry_width].to_vec()
        })
        .collect();
    let sort_key_of_the_entry = mapping_key_sort_key(&entry[..key_width]);
    let position = entries.partition_point(|existing| {
        mapping_key_sort_key(&existing[..key_width]) < sort_key_of_the_entry
    });
    assert!(
        entries
            .get(position)
            .is_none_or(|next| next[..key_width] != entry[..key_width]),
        "插之前映射叶里没有这把 key"
    );
    entries.insert(position, entry.to_vec());
    assert!(
        entries_start + entries.len() * entry_width <= leaf.len(),
        "映射叶装得下多一条"
    );
    for (index, each_entry) in entries.iter().enumerate() {
        let start = entries_start + index * entry_width;
        leaf[start..start + entry_width].copy_from_slice(each_entry);
    }
    leaf[MAPPING_NODE_ENTRY_COUNT_OFFSET..MAPPING_NODE_ENTRY_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(entries.len())
            .expect("条目数 2 字节")
            .to_le_bytes(),
    );
    leaf[MAPPING_NODE_DECLARED_LENGTH_OFFSET..MAPPING_NODE_DECLARED_LENGTH_OFFSET + 2]
        .copy_from_slice(
            &u16::try_from(entries.len() * entry_width)
                .expect("声明长度 2 字节")
                .to_le_bytes(),
        );
    leaf[MAPPING_NODE_SMALLEST_KEY_OFFSET..MAPPING_NODE_SMALLEST_KEY_OFFSET + key_width]
        .copy_from_slice(&entries.first().expect("刚插进一条")[..key_width]);
    leaf[MAPPING_NODE_LARGEST_KEY_OFFSET..MAPPING_NODE_LARGEST_KEY_OFFSET + key_width]
        .copy_from_slice(&entries.last().expect("刚插进一条")[..key_width]);
    reseal_index_node(&mut leaf);
    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
}

/// 墓碑记录里对象 ID 的偏移（D18（块里携带什么信息） 已定项 24 第 2 条：对象 ID 0、对象出生代 8、区间起 16、区间止 24、死亡 txg 32）。
const TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: usize = 0;

/// 判决 `research/prompts/m4-c860-r1-main-verification.md` 第二节 K3（二）的坏法（harness 档 `scan_rebuild_reads_lineage_and_retirement_records.rs`
/// 的同名函数）：最新根那一版墓碑树第一片容器里只有删除中那个文件的一条墓碑，原样追加一条，再把追加的那条改成压着 (`inode`, `object_birth`)：
/// 区间（整对象）与死亡 txg（在谱系上）照抄，重封、补校验和。
fn append_a_tombstone_pressing(image: &mut MemoryPool, inode: u64, object_birth: u64) {
    append_a_copy_of_the_first_tombstone_of_the_newest_root(image);
    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
    let tombstone_record_bytes = usize::try_from(TOMBSTONE_RECORD_BYTES).expect("40");
    let second_tombstone = PACKED_RECORDS_OFFSET + tombstone_record_bytes;
    change_the_unit_and_propagate_its_checksum(
        image,
        container_slot,
        data_unit_bytes(),
        |container| {
            let appended =
                &mut container[second_tombstone..second_tombstone + tombstone_record_bytes];
            set_tombstone_field(appended, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET, inode);
            set_tombstone_field(appended, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET, object_birth);
        },
    );
}

/// 第一个文件删一半那张镜像上多一条墓碑压着带两个数据单元的活文件 inode 2（判决 m4-c860-r1 第二节 K3（二）的镜像）。交回搭出那张镜像的池
/// （取 inode 2 的数据指针用）与改过的镜像。
fn image_with_an_extra_tombstone_pressing_the_live_inode_two(tag: &str) -> (BuiltPool, MemoryPool) {
    let pool = pool_with_the_first_file_half_deleted_beside_inode_two(tag);
    let object_birth_of_inode_two = object_birth_of(&pool.output, 2)
        .expect("inode 2 在最新那一版 inode 树里有记录")
        .0;
    let mut image = pool.memory_pool();
    append_a_tombstone_pressing(&mut image, 2, object_birth_of_inode_two);
    (pool, image)
}

/// 最新的根那一版第一个文件的记录里截断字段写成 (T, N_w)，重封叶、补校验和。
fn write_the_truncation_field_of_the_first_file(
    image: &mut MemoryPool,
    truncation_field_txg: u64,
    truncation_field_size_in_bytes: u64,
) {
    change_the_inode_record(image, FIRST_INODE_NUMBER, |record| {
        record[INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET
            ..INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET + 8]
            .copy_from_slice(&truncation_field_txg.to_le_bytes());
        record[INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET
            ..INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET + 8]
            .copy_from_slice(&truncation_field_size_in_bytes.to_le_bytes());
    });
}

/// 每块盘两个系统配置槽 incompat 第一个字节置上位 2（inode 记录带截断字段），整槽校验和重封。
fn set_incompat_bit_two(image: &mut MemoryPool) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let slot_spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for device in image.devices.values_mut() {
        for slot_offset in [DeviceOffsetInBytes(0), DeviceOffsetInBytes(slot_spacing)] {
            let mut slot = device.read(slot_offset, slot_bytes);
            slot[FIRST_INCOMPAT_BYTE_OFFSET] |= INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT;
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device.write(slot_offset, &slot);
        }
    }
}

/// 这一版第 `index` 个数据单元单元头里的诞生代号（盘 0 那一份）。
fn birth_txg_of_the_data_unit(
    image: &MemoryPool,
    version: &TransactionOutput,
    index: usize,
) -> u64 {
    let pointer = &version.data_pointers[index];
    read_u64_at(
        &unit_bytes_at(image, pointer.locations[0].slot, data_unit_bytes()),
        DATA_UNIT_BIRTH_TXG_OFFSET,
    )
}

/// 三个单元（2.5 个净荷）在同一次顺序写里写下，交回池、写下它们的那一版与它们共同的诞生代号。
fn three_unit_file_born_together(tag: &str) -> (BuiltPool, TransactionOutput, u64) {
    let pool = pool_with_a_three_unit_file(tag);
    let written = pool.output.clone();
    let image = pool.memory_pool();
    let birth_txg = birth_txg_of_the_data_unit(&image, &written, 0);
    (pool, written, birth_txg)
}

/// 三个单元截在恰好一个净荷（单元 0 不重写）、按批放完，再变长到三个净荷（之后到 size 是洞）：交回池与截断意图那一版的 txg。
fn pool_extended_after_a_truncation_at_a_data_unit_boundary(tag: &str) -> (BuiltPool, u64) {
    let (mut pool, written, _) = three_unit_file_born_together(tag);
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, payload_capacity());
    assert!(
        truncation_entry_of(&intent).is_some()
            && intent.data_pointers.len() == 3
            && intent.data_pointers[0] == written.data_pointers[0],
        "意图那一版：条目挂着、三个单元都在、单元 0 没重写"
    );
    let mut steps = 0;
    while truncation_entry_of(&pool.output).is_some() {
        assert!(steps < 4, "批大小 1、放两个单元：几步之内放完");
        publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
        steps += 1;
    }
    change_the_size_of_the_first_file_and_advance(&mut pool, 3 * payload_capacity());
    (pool, intent.root.checkpoint_txg.0)
}

/// 建一个快照之后覆盖写一次（`checker_snapshot_space_accounting_must_red_worlds.rs`）：活头换下的单元进 deadlist、映射条目留存。
fn pool_with_a_snapshot_and_an_overwrite_under_it(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    assert!(
        pool.output
            .units
            .iter()
            .any(|unit| matches!(unit.identity, TransactionUnit::Data(_))),
        "第一个文件有一个数据单元"
    );
    let publish_parameters = parameters();
    let first_file_version = pool.output.clone();
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        pool.output = publish_create_snapshot(
            &mut writer,
            &mut pool.allocator,
            &first_file_version,
            InstanceGeneration(1),
        )
        .expect("建快照");
    }
    let content: Vec<u8> = (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + 7) % 239).expect("小于 256"))
        .collect();
    let snapshot_version = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &snapshot_version,
        &content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    pool
}

// ───────────────────────── 合法搬迁过的镜像（照 `checker_relocated_images_judged_by_the_mapping_position.rs` 抄）─────────────────────────

const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
/// 中央映射叶条目 55 字节：key 27 之后两条位置条目各 14（设备身份 4、槽号 6、整单元校验和 4，D19 已定项 10）。
const MAPPING_LOCATION_ENTRY_BYTES: usize = 14;
/// 映射 key 里出生 txg 那 8 字节的起点：类标签 1 + 出生树 8（D19 已定项 6）。
const MAPPING_KEY_BIRTH_TXG_OFFSET: usize = 1 + 8;
/// 分配记录 20 字节：设备身份 4、起点槽 6、跨度 2（最高位是已释放标志）、分配代或释放代 8（D3（空间分配） 已定项 7）。
const ALLOCATION_RECORD_BYTES: usize = 20;
const ALLOCATION_RECORD_SPAN_OFFSET: usize = 10;
const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;
/// 记账条目 34 字节：key 22（统计量 2 + 树 8 + 设备 4 + 代 8）之后是值 8（D8（核心索引结构） 已定项 11）。
const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 2 + 8;
const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
/// 记账统计量的行号（D5（快照 / 空间记账机制） 已定项 4）：已分配 1、空闲 2、defer 待释放 5，单位字节；runs 10（单元区空闲槽的连续段数）、
/// 全空聚簇段数 11，单位个；都带设备维。
const STATISTIC_ALLOCATED_BYTES: u16 = 1;
const STATISTIC_FREE_BYTES: u16 = 2;
const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
const STATISTIC_FRAGMENTATION_RUNS: u16 = 10;
const STATISTIC_EMPTY_CLUSTER_SEGMENTS: u16 = 11;
/// 目的槽离这一版分配记录里最后一个在册单元的末槽隔开这么多槽：落在没人用过的地方（不碰被更早的根引用过、已回收的槽）。
const DESTINATION_GAP_IN_SLOTS: u64 = 64;

fn mapping_entry_bytes() -> usize {
    usize::try_from(MAPPING_ENTRY_BYTES).expect("55")
}

fn mapping_key_bytes() -> usize {
    usize::try_from(MAPPING_KEY_BYTES).expect("27")
}

fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}

fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn read_six_bytes_at(bytes: &[u8], offset: usize) -> u64 {
    let mut wide = [0u8; 8];
    wide[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(wide)
}

/// 新池新建文件（txg 3）之后建一个空 inode（txg 4）：第一个文件的数据单元与 extent 树留在 txg 3 写的槽里，被 txg 3 与 txg 4 两条根共同引用。
fn pool_with_an_empty_inode_after_the_first_file(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        1,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("建一个空 inode");
    pool
}

/// 搬哪一个单元：它在两块盘上共同的源槽、占几槽（码 1 两槽、码 2 一槽）。
#[derive(Clone, Copy, Debug)]
struct UnitToRelocate {
    source_slot: SlotNumber,
    span_in_slots: u64,
}

/// 搬完之后它在哪：目的槽与它的诞生代号（映射 key 里的出生 txg，目的记录的分配代取它）。
#[derive(Clone, Copy, Debug)]
struct RelocatedUnit {
    unit: UnitToRelocate,
    destination_slot: SlotNumber,
    birth_txg: u64,
}

/// 分配记录与记账跟不跟着搬迁改。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocationRecordsOfTheRelocation {
    /// 合法搬迁：源记录记释放、目的记一条已分配，记账的已分配、空闲、defer 都跟着改。
    SourceReleasedAndDestinationAllocated,
}

/// 最新那一版里第一个文件唯一的数据单元的槽与 extent 树的根节点的槽（两盘同槽）。
fn data_unit_and_extent_root_of_the_newest_version(image: &MemoryPool) -> [UnitToRelocate; 2] {
    let extent_root = tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root;
    let mapping = parse_index_node(&unit_bytes_at(
        image,
        the_slot_of(&newest_root(image).mapping_root),
        node_bytes(),
    ))
    .expect("中央映射根解得开");
    let data_units: Vec<SlotNumber> = mapping
        .entries
        .iter()
        .filter(|entry| entry[0] == 1)
        .map(|entry| SlotNumber(read_six_bytes_at(entry, mapping_key_bytes() + 4)))
        .collect();
    assert_eq!(
        data_units.len(),
        1,
        "第一个文件只有一个数据单元：{data_units:?}"
    );
    [
        UnitToRelocate {
            source_slot: data_units[0],
            span_in_slots: 2,
        },
        UnitToRelocate {
            source_slot: the_slot_of(&extent_root),
            span_in_slots: 1,
        },
    ]
}

/// 最新那一版分配记录树里装着盘 `device` 的记录、按位置罩住 `slot` 的那片叶（槽号、叶的字节）。
fn allocation_leaf_holding(
    image: &MemoryPool,
    device: DeviceIdentity,
    slot: SlotNumber,
) -> (SlotNumber, Vec<u8>) {
    let root = tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION).root;
    allocation_record_tree_nodes_as_far_as_readable(image, &root)
        .into_iter()
        .map(|pointer| {
            let node_slot = the_slot_of(&pointer);
            (node_slot, unit_bytes_at(image, node_slot, node_bytes()))
        })
        .find(|(_, node)| {
            let header = parse_index_node(node).expect("分配记录树节点解得开");
            header.level == 0
                && header.entries.iter().any(|record| {
                    read_u32_at(record, 0) == device.0 && read_six_bytes_at(record, 4) == slot.0
                })
        })
        .expect("最新那一版的分配记录里有这个单元的记录")
}

/// 目的槽的起点：两块盘上装着源记录的那几片叶里在册单元的末槽取最大、隔开 [`DESTINATION_GAP_IN_SLOTS`] 再对齐到 2 槽，
/// 交回之前核它与搬的几个单元依次排开之后仍落在每片叶按位置罩的那一段里。
fn first_destination_slot(image: &MemoryPool, units: &[UnitToRelocate]) -> SlotNumber {
    let mut end_of_the_last_unit = 0u64;
    let mut end_of_the_leaf_ranges = u64::MAX;
    for device in DEVICES {
        for unit in units {
            let (_, leaf) = allocation_leaf_holding(image, device, unit.source_slot);
            let header = parse_index_node(&leaf).expect("分配记录叶解得开");
            for record in &header.entries {
                if read_u32_at(record, 0) == device.0 {
                    let span = u64::from(
                        read_u16_at(record, ALLOCATION_RECORD_SPAN_OFFSET)
                            & !ALLOCATION_RECORD_RELEASED_FLAG,
                    );
                    end_of_the_last_unit =
                        end_of_the_last_unit.max(read_six_bytes_at(record, 4) + span);
                }
            }
            end_of_the_leaf_ranges =
                end_of_the_leaf_ranges.min(read_six_bytes_at(&header.largest_key, 4));
        }
    }
    let first = (end_of_the_last_unit + DESTINATION_GAP_IN_SLOTS).next_multiple_of(2);
    let span_of_every_unit: u64 = units.iter().map(|unit| unit.span_in_slots).sum();
    assert!(
        first + span_of_every_unit <= end_of_the_leaf_ranges,
        "目的槽 {first} 起 {span_of_every_unit} 槽落在叶罩的那一段（末槽 {end_of_the_leaf_ranges}）里"
    );
    SlotNumber(first)
}

/// 映射叶里位置条目落在 `slot`（两条都在这一槽）的那一条的下标。
fn mapping_entry_of_the_unit_at(mapping: &[Vec<u8>], slot: SlotNumber) -> usize {
    let at_the_slot: Vec<usize> = mapping
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            (0..2).all(|location| {
                read_six_bytes_at(
                    entry,
                    mapping_key_bytes() + location * MAPPING_LOCATION_ENTRY_BYTES + 4,
                ) == slot.0
            })
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(at_the_slot.len(), 1, "映射里恰好一条指着槽 {slot:?}");
    at_the_slot[0]
}

/// 一个码 2 节点换成同一个头、`entries` 这些条目（key 区间沿用原头的：分配记录叶按位置罩那一段，与条目无关），重封。
fn rebuilt_with_entries(node: &[u8], entries: &[Vec<u8>]) -> Vec<u8> {
    let header = parse_index_node(node).expect("节点解得开");
    let mut rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽装得进 u16"),
        entries,
    );
    reseal_index_node(&mut rebuilt);
    rebuilt
}

/// 在最新那一版里搬 `units`（见文件头）。交回每个单元搬到了哪。
fn relocate_in_the_newest_version(
    image: &mut MemoryPool,
    units: &[UnitToRelocate],
    allocation_records: AllocationRecordsOfTheRelocation,
) -> Vec<RelocatedUnit> {
    let newest_txg = newest_root(image).checkpoint_txg.0;
    let mut destination = first_destination_slot(image, units);
    let mapping_entries = parse_index_node(&unit_bytes_at(
        image,
        the_slot_of(&newest_root(image).mapping_root),
        node_bytes(),
    ))
    .expect("中央映射根解得开")
    .entries;
    let mut relocated = Vec::new();
    for unit in units {
        let entry =
            &mapping_entries[mapping_entry_of_the_unit_at(&mapping_entries, unit.source_slot)];
        relocated.push(RelocatedUnit {
            unit: *unit,
            destination_slot: destination,
            birth_txg: read_u64_at(entry, MAPPING_KEY_BIRTH_TXG_OFFSET),
        });
        destination = SlotNumber(destination.0 + unit.span_in_slots);
    }
    // 单元原样拷过去：两盘同槽，逐字节不变（头不盖戳、birth 不变，判决 G3）。
    for relocation in &relocated {
        let length = usize::try_from(relocation.unit.span_in_slots * SLOT_BYTES).expect("单元宽");
        for device in DEVICES {
            let sparse = image.devices.get_mut(&device).expect("池里有这块盘");
            let bytes = sparse.read(relocation.unit.source_slot.to_device_offset(), length);
            sparse.write(relocation.destination_slot.to_device_offset(), &bytes);
        }
    }
    // 分配记录：每块盘上装着源记录的那片叶各改一次（搬的几个单元落在同一片叶里时一起改）。
    for device in DEVICES {
        let mut leaves_rewritten: Vec<SlotNumber> = Vec::new();
        for relocation in &relocated {
            let (leaf_slot, leaf) =
                allocation_leaf_holding(image, device, relocation.unit.source_slot);
            if leaves_rewritten.contains(&leaf_slot) {
                continue;
            }
            leaves_rewritten.push(leaf_slot);
            let mut records = parse_index_node(&leaf).expect("分配记录叶解得开").entries;
            let relocated_in_this_leaf: Vec<RelocatedUnit> = relocated
                .iter()
                .filter(|candidate| {
                    records.iter().any(|record| {
                        read_u32_at(record, 0) == device.0
                            && read_six_bytes_at(record, 4) == candidate.unit.source_slot.0
                    })
                })
                .copied()
                .collect();
            for in_this_leaf in &relocated_in_this_leaf {
                let source = records
                    .iter_mut()
                    .find(|record| {
                        read_u32_at(record, 0) == device.0
                            && read_six_bytes_at(record, 4) == in_this_leaf.unit.source_slot.0
                    })
                    .expect("源记录在这片叶里");
                let span = read_u16_at(source, ALLOCATION_RECORD_SPAN_OFFSET);
                source[ALLOCATION_RECORD_SPAN_OFFSET..ALLOCATION_RECORD_SPAN_OFFSET + 2]
                    .copy_from_slice(&(span | ALLOCATION_RECORD_RELEASED_FLAG).to_le_bytes());
                source
                    [ALLOCATION_RECORD_GENERATION_OFFSET..ALLOCATION_RECORD_GENERATION_OFFSET + 8]
                    .copy_from_slice(&newest_txg.to_le_bytes());
                match allocation_records {
                    AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated => {
                        let mut destination_record = vec![0u8; ALLOCATION_RECORD_BYTES];
                        destination_record[..4].copy_from_slice(&device.0.to_le_bytes());
                        destination_record[4..10]
                            .copy_from_slice(&in_this_leaf.destination_slot.0.to_le_bytes()[..6]);
                        destination_record
                            [ALLOCATION_RECORD_SPAN_OFFSET..ALLOCATION_RECORD_SPAN_OFFSET + 2]
                            .copy_from_slice(&span.to_le_bytes());
                        destination_record[ALLOCATION_RECORD_GENERATION_OFFSET
                            ..ALLOCATION_RECORD_GENERATION_OFFSET + 8]
                            .copy_from_slice(&in_this_leaf.birth_txg.to_le_bytes());
                        records.push(destination_record);
                    }
                }
            }
            records.sort_by_key(|record| (read_u32_at(record, 0), read_six_bytes_at(record, 4)));
            let rebuilt = rebuilt_with_entries(&leaf, &records);
            replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &rebuilt);
        }
    }
    // 记账：每块盘上已分配加、空闲减目的那几槽，defer 加源那几槽。目的槽离最后一个在册单元隔开至少 64 个空槽、依次排开，
    // 后面没有在册的单元（[`first_destination_slot`]）：落在一段空闲槽中间、成一个新的占用块，runs 加一；它罩到的聚簇段原来一槽都不占
    // （段起点不早于最后一个在册单元的末槽），全空聚簇段数减它罩到的段数（源槽还在 defer、源记录所在的段里还有别的未释放记录，两种口径同减）。
    let segments_the_destinations_reach: std::collections::BTreeSet<u64> = relocated
        .iter()
        .flat_map(|relocation| {
            relocation.destination_slot.0
                ..relocation.destination_slot.0 + relocation.unit.span_in_slots
        })
        .map(|slot| (slot - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS)
        .collect();
    let segments_newly_occupied =
        u64::try_from(segments_the_destinations_reach.len()).expect("段数");
    let accounting_slot = the_slot_of(&tree_table_entry_of_kind(image, TREE_KIND_ACCOUNTING).root);
    let accounting = unit_bytes_at(image, accounting_slot, node_bytes());
    let mut rows = parse_index_node(&accounting).expect("记账根解得开").entries;
    let relocated_bytes: u64 = relocated
        .iter()
        .map(|relocation| relocation.unit.span_in_slots * SLOT_BYTES)
        .sum();
    for row in &mut rows {
        let statistic = read_u16_at(row, 0);
        if !DEVICES
            .iter()
            .any(|device| device.0 == read_u32_at(row, ACCOUNTING_ROW_DEVICE_OFFSET))
        {
            continue;
        }
        let value = read_u64_at(row, ACCOUNTING_ROW_VALUE_OFFSET);
        let changed = match (statistic, allocation_records) {
            (
                STATISTIC_ALLOCATED_BYTES,
                AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
            ) => value + relocated_bytes,
            (
                STATISTIC_FREE_BYTES,
                AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
            ) => value - relocated_bytes,
            (STATISTIC_DEFER_QUEUE_BYTES, _) => value + relocated_bytes,
            (
                STATISTIC_FRAGMENTATION_RUNS,
                AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
            ) => value + 1,
            (
                STATISTIC_EMPTY_CLUSTER_SEGMENTS,
                AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
            ) => value - segments_newly_occupied,
            (_, _) => value,
        };
        row[ACCOUNTING_ROW_VALUE_OFFSET..ACCOUNTING_ROW_VALUE_OFFSET + 8]
            .copy_from_slice(&changed.to_le_bytes());
    }
    let rebuilt_accounting = rebuilt_with_entries(&accounting, &rows);
    replace_the_unit_and_propagate_its_checksum(image, accounting_slot, &rebuilt_accounting);
    // 映射条目换位置：上面几步补校验和时映射叶已经改过，这里重读。
    let mapping_slot = the_slot_of(&newest_root(image).mapping_root);
    let mapping = unit_bytes_at(image, mapping_slot, node_bytes());
    let mut entries = parse_index_node(&mapping)
        .expect("中央映射根解得开")
        .entries;
    for relocation in &relocated {
        let index = mapping_entry_of_the_unit_at(&entries, relocation.unit.source_slot);
        for location in 0..2 {
            let slot_at = mapping_key_bytes() + location * MAPPING_LOCATION_ENTRY_BYTES + 4;
            entries[index][slot_at..slot_at + 6]
                .copy_from_slice(&relocation.destination_slot.0.to_le_bytes()[..6]);
        }
        assert_eq!(
            entries[index].len(),
            usize::try_from(MAPPING_ENTRY_BYTES).expect("55")
        );
    }
    let rebuilt_mapping = rebuilt_with_entries(&mapping, &entries);
    replace_the_unit_and_propagate_its_checksum(image, mapping_slot, &rebuilt_mapping);
    relocated
}

// ───────────────────────── 用例：每一形一条 ─────────────────────────

#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_the_new_pool_file_creation() {
    let pool = build_pool("gpu-scan-rebuild-new-pool");
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_the_new_pool_file_creation",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 第一个文件之后一次建 233 个空 inode：234 条记录装成两片 inode 叶容器，在同一次发布里诞生（同一棵树、同一个 txg、同一个实例），
/// 出生序号一片 0、一片 1，两片的中央映射 key 只差尾段的出生序号；扫描重建按单元头现算的 key 与遍历方向逐项相同、成立
/// （harness 档 `checker_narrow_invariants_and_abandoned_roots.rs` 的 `two_leaf_containers_in_key_order_hold_every_invariant` 那一类，这里走真写路径）。
/// 码 3 映射 key 尾段的取样点：新池新建文件那一版每棵树的码 3 容器只有一片、出生序号都是 0，key 丢了出生序号照样成立；这一形丢了它，
/// 两片叶的 key 撞成一把、遍历方向那一把没比到。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_two_inode_leaf_containers_born_in_one_publish(
) {
    let pool = pool_with_new_inodes(
        "gpu-scan-rebuild-two-leaves-born-together",
        NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER,
    );
    assert_eq!(
        pool.output.inode_leaf_containers.len(),
        2,
        "234 条记录装成两片叶"
    );
    let packed_mapping_keys: Vec<&Vec<u8>> = pool
        .output
        .mapping_keys
        .iter()
        .filter(|key| key[0] == UNIT_CLASS_PACKED)
        .collect();
    assert!(
        packed_mapping_keys.iter().any(|key| {
            key[PACKED_MAPPING_KEY_BIRTH_SEQUENCE_OFFSET
                ..PACKED_MAPPING_KEY_BIRTH_SEQUENCE_OFFSET + 4]
                != [0u8; 4]
        }),
        "有一片码 3 容器的出生序号不是 0（两片叶同一次发布诞生）：{packed_mapping_keys:?}"
    );
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_two_inode_leaf_containers_born_in_one_publish",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 同一个文件覆盖写三版（长度不变）：盘上四版数据单元与四版 inode 叶容器，按写序与全序键择最后一版。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_three_overwrites_of_the_same_length() {
    let mut pool = build_pool("gpu-scan-rebuild-overwrites");
    for seed in 1..=3 {
        overwrite_in_process_as_instance_one_and_advance_the_pool(
            &mut pool,
            &content_of_the_same_length(seed),
        );
    }
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_three_overwrites_of_the_same_length",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 再建两个空 inode 之后把第一个文件 unlink 到底：它的数据单元由墓碑判死。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_an_unlink_to_the_end() {
    let mut pool = pool_with_new_inodes("gpu-scan-rebuild-unlink", 2);
    unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_an_unlink_to_the_end",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 必红世界 ①：映射叶里数据单元那条条目的落点挪了槽，中央映射那张表对不上。
#[test]
fn the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_a_mapping_entry_whose_location_names_another_slot(
) {
    let pool = build_pool("gpu-scan-rebuild-moved-mapping-location");
    let mut image = pool.memory_pool();
    move_the_first_location_of_the_data_unit_mapping_entry(&mut image);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_a_mapping_entry_whose_location_names_another_slot",
        &image,
        "SCAN_REBUILD_OUTCOME_VIOLATED",
    );
}

/// 必红世界 ②：第一个文件那个数据单元两份副本的对象 key 清零、不重封：扫描重建不认领它，每个 inode 的数据单元集合对不上。
#[test]
fn the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_a_data_unit_whose_object_key_is_zeroed_on_both_copies(
) {
    let pool = build_pool("gpu-scan-rebuild-zeroed-object-key");
    let mut image = pool.memory_pool();
    for location in pool.output.data_pointers[0].locations {
        zero_the_object_key_of_the_copy_at(&mut image, location);
    }
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_a_data_unit_whose_object_key_is_zeroed_on_both_copies",
        &image,
        "SCAN_REBUILD_OUTCOME_VIOLATED",
    );
}

/// 只抹一份副本的对象 key：另一份照样扫得到，两边各只留读得出的落点再比，降级不判红。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_when_the_object_key_of_one_copy_is_zeroed() {
    let pool = build_pool("gpu-scan-rebuild-one-copy-zeroed");
    let mut image = pool.memory_pool();
    zero_the_object_key_of_the_copy_at(&mut image, pool.output.data_pointers[0].locations[0]);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_when_the_object_key_of_one_copy_is_zeroed",
        &image,
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 删除中（意图之后墓碑压着、记录还在）：只对那一个对象两侧都不比（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md`
/// 第三节 M5，此前两边都报不适用），这张镜像上只有它一个文件，两侧去掉它之后三张表相同、两边都成立。
#[test]
fn the_gpu_scan_rebuild_leaves_only_the_object_being_deleted_out_and_holds_like_the_cpu_checker() {
    let image =
        image_after_the_intent_of_a_three_unit_file("gpu-scan-rebuild-deletion-in-progress");
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_leaves_only_the_object_being_deleted_out_and_holds_like_the_cpu_checker",
        &image,
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 第一个文件删一半（攻方 F 段，`research/prompts/m3-multi-file-unlink-r1-opus-output.md` U4-甲）、inode 2 带两个单元：扫描重建只对删除中的
/// inode 1 不比，inode 2 的记录、两个数据单元与它们的映射照比、两侧相同，两边都成立（harness 档
/// `scan_rebuild_agrees_with_the_walked_index.rs` 的 `while_the_first_file_is_half_deleted_the_scan_rebuild_still_judges_another_file_and_holds`）。
#[test]
fn the_gpu_scan_rebuild_judges_another_file_while_the_first_file_is_half_deleted_and_holds_like_the_cpu_checker(
) {
    let pool = pool_with_the_first_file_half_deleted_beside_inode_two(
        "gpu-scan-rebuild-half-deleted-beside-inode-two",
    );
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_judges_another_file_while_the_first_file_is_half_deleted_and_holds_like_the_cpu_checker",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// M5 必红世界：同一张镜像（第一个文件删一半）上把 inode 2 第一个数据单元那条映射条目的落点挪开：inode 2 照比，两边都判 I-1.13 违例、GPU 红在
/// 下标 92（harness 档同一份文件的 `moving_a_mapping_location_of_another_file_while_the_first_file_is_half_deleted_reddens_the_scan_rebuild`；
/// 此前这一版整张报不适用，这一处改坏看不出来）。
#[test]
fn the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_a_mapping_location_of_another_file_moved_while_the_first_file_is_half_deleted(
) {
    let pool = pool_with_the_first_file_half_deleted_beside_inode_two(
        "gpu-scan-rebuild-half-deleted-moved-mapping-of-inode-two",
    );
    let first_data_pointer_of_inode_two = pool
        .output
        .file_other_than_the_first(InodeNumber(2))
        .expect("inode 2 带数据单元")
        .data_pointers[0];
    let mut image = pool.memory_pool();
    move_the_first_location_of_the_mapping_entry_keyed(
        &mut image,
        &mapping_key_for_data(
            first_data_pointer_of_inode_two.head,
            first_data_pointer_of_inode_two.write_order,
        ),
    );
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_a_mapping_location_of_another_file_moved_while_the_first_file_is_half_deleted",
        &image,
        "SCAN_REBUILD_OUTCOME_VIOLATED",
    );
}

/// 三个单元的文件覆盖写成一个单元：后两个单元在 size 之外，读法 Z 不认领、遍历方向也不收。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_a_shorter_overwrite() {
    let mut pool = build_pool("gpu-scan-rebuild-shorter-overwrite");
    sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_three_data_units(),
    );
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &file_content());
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_a_shorter_overwrite",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 覆盖写变短再只改 size 变长：洞里放掉的旧单元出生 txg 比重写过的单元 0 小，读法 Z 加写序判死。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_a_shorter_overwrite_and_an_extension() {
    let mut pool = build_pool("gpu-scan-rebuild-shorter-overwrite-then-extension");
    sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_three_data_units(),
    );
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &file_content());
    change_the_size_of_the_first_file_and_advance(&mut pool, 3 * payload_capacity() - 1);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_a_shorter_overwrite_and_an_extension",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 三个单元截到 100、批大小 1 的意图那一版：size 之外还挂着两个单元，两侧都不收。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_on_the_intent_version_of_a_truncation() {
    let mut pool = pool_with_a_three_unit_file("gpu-scan-rebuild-truncation-intent");
    change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_on_the_intent_version_of_a_truncation",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 三个单元（2.5 个净荷）截到恰好一个净荷的意图那一版，再把第一个文件记录里的截断字段清零：单元 0 不重写、三个单元出生 txg 相同，
/// 写序那一条判不死 size 之外的单元 1、2；截断字段清零，读法 Z 第四条也不判它们；只剩 size 那一条不认领，遍历方向也不收它们，两边都成立。
/// 读法 Z 的 size 那一条的取样点：写路径写出的意图那一版带着截断字段 (T, N_w)，第四条先判死单元 1、2；覆盖写变短重写了单元 0，写序那一条先判死它们，
/// 那几形上改掉 size 那一条照样成立。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_on_a_truncation_intent_at_a_data_unit_boundary_whose_truncation_field_is_cleared(
) {
    let (mut pool, written, _) =
        three_unit_file_born_together("gpu-scan-rebuild-boundary-intent-without-its-field");
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, payload_capacity());
    assert!(
        truncation_entry_of(&intent).is_some()
            && intent.data_pointers.len() == 3
            && intent.data_pointers[0] == written.data_pointers[0],
        "意图那一版：条目挂着、三个单元都在、单元 0 没重写"
    );
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, 0, 0);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_on_a_truncation_intent_at_a_data_unit_boundary_whose_truncation_field_is_cleared",
        &image,
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 挂着回退过一次覆盖写：覆盖写那一版诞生在回退抛弃的那一段里，按谱系记录不进任何一张表。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_a_rollback_while_mounted() {
    let pool =
        pool_rolled_back_past_an_overwrite_of_the_same_length("gpu-scan-rebuild-rolled-back");
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_a_rollback_while_mounted",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 必红世界：抹掉回退留下的那条谱系记录，扫描重建择回退抛弃的那一版。
#[test]
fn the_gpu_scan_rebuild_reddens_like_the_cpu_checker_when_the_lineage_record_is_erased() {
    let pool =
        pool_rolled_back_past_an_overwrite_of_the_same_length("gpu-scan-rebuild-lineage-erased");
    let mut image = pool.memory_pool();
    erase_every_record_of_the_container(&mut image, &pool.output.lineage_containers[0].pointer);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_reddens_like_the_cpu_checker_when_the_lineage_record_is_erased",
        &image,
        "SCAN_REBUILD_OUTCOME_VIOLATED",
    );
}

/// 右片叶删空退役：它最后那一版还扫得到，按退役树不认领。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_the_unlink_that_retires_the_right_leaf() {
    let pool = pool_whose_right_leaf_retired_after_the_unlink_of(
        "gpu-scan-rebuild-emptied-leaf-retired",
        THE_ONLY_INODE_OF_THE_RIGHT_LEAF,
    );
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_the_unlink_that_retires_the_right_leaf",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 左片吸收右片、右片退役：右片最后那一版里与左片同号的记录不认领。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_the_unlink_whose_left_leaf_absorbs_the_right_one(
) {
    let pool = pool_whose_right_leaf_retired_after_the_unlink_of(
        "gpu-scan-rebuild-absorbed-leaf-retired",
        AN_INODE_WHOSE_UNLINK_ABSORBS_THE_RIGHT_LEAF,
    );
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_the_unlink_whose_left_leaf_absorbs_the_right_one",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 必红世界：抹掉删空那片叶留下的退役记录，扫描重建认领退役那片叶最后那一版。
#[test]
fn the_gpu_scan_rebuild_reddens_like_the_cpu_checker_when_the_retirement_record_is_erased() {
    let pool = pool_whose_right_leaf_retired_after_the_unlink_of(
        "gpu-scan-rebuild-retirement-erased",
        THE_ONLY_INODE_OF_THE_RIGHT_LEAF,
    );
    let mut image = pool.memory_pool();
    erase_every_record_of_the_container(&mut image, &pool.output.retirement_containers[0].pointer);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_reddens_like_the_cpu_checker_when_the_retirement_record_is_erased",
        &image,
        "SCAN_REBUILD_OUTCOME_VIOLATED",
    );
}

/// 判决 `research/prompts/m4-c860-r1-main-verification.md` 第二节 K3（一）（C860 镜像）：抹掉退役记录、再把退役那片叶最后一版的映射条目补回遍历方向，
/// 两侧唯一的差是早已删完的 inode 234 那条记录（扫描一侧墓碑压着、遍历方向没有）。删除中由两侧一起认（第三节 C2，用户 2026-10-20 选丙）：
/// 遍历方向没有这条记录，不算删除中、照比，两边都判违例、GPU 红在下标 92（harness 档同名镜像那一条）。
#[test]
fn the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_a_retired_leaf_left_in_the_walked_mapping_without_its_retirement_record(
) {
    let mut pool = pool_with_new_inodes(
        "gpu-scan-rebuild-retired-leaf-left-in-the-walked-mapping",
        NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER,
    );
    let intent = unlink_to_the_end(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
    let right_leaf_at_the_intent = &intent.inode_leaf_containers[1];
    let mut image = pool.memory_pool();
    erase_every_record_of_the_container(&mut image, &pool.output.retirement_containers[0].pointer);
    insert_the_entry_into_the_newest_mapping_leaf(
        &mut image,
        &build_mapping_entry(
            &mapping_key_for_node(UNIT_CLASS_PACKED, right_leaf_at_the_intent.pointer),
            right_leaf_at_the_intent.pointer.locations,
        ),
    );
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_a_retired_leaf_left_in_the_walked_mapping_without_its_retirement_record",
        &image,
        "SCAN_REBUILD_OUTCOME_VIOLATED",
    );
}

/// 判决 m4-c860-r1 第二节 K3（二）：第一个文件删一半时多一条墓碑压着活着的 inode 2（nlink 1、两个数据单元）。丙下两侧那条记录 nlink 都是 1、
/// 不算删除中，两侧照比：扫描重建那一侧按墓碑不认领 inode 2 的数据单元，两边都判违例（harness 档同一形那一条）。
#[test]
fn the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_an_extra_tombstone_pressing_a_live_file_while_the_first_file_is_half_deleted(
) {
    let (_pool, image) = image_with_an_extra_tombstone_pressing_the_live_inode_two(
        "gpu-scan-rebuild-extra-tombstone-on-live-inode-two",
    );
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_an_extra_tombstone_pressing_a_live_file_while_the_first_file_is_half_deleted",
        &image,
        "SCAN_REBUILD_OUTCOME_VIOLATED",
    );
}

/// 同上一条的镜像再叠上 M5 必红世界那一招（挪 inode 2 第一个数据单元那条映射条目的落点）：甲、乙下这一处改坏被那条坏墓碑遮掉，
/// 丙下 inode 2 照比，两边都判违例。
#[test]
fn the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_an_extra_tombstone_and_a_moved_mapping_location_of_the_live_file(
) {
    let (pool, mut image) = image_with_an_extra_tombstone_pressing_the_live_inode_two(
        "gpu-scan-rebuild-extra-tombstone-and-moved-mapping-of-inode-two",
    );
    let first_data_pointer_of_inode_two = pool
        .output
        .file_other_than_the_first(InodeNumber(2))
        .expect("inode 2 带数据单元")
        .data_pointers[0];
    move_the_first_location_of_the_mapping_entry_keyed(
        &mut image,
        &mapping_key_for_data(
            first_data_pointer_of_inode_two.head,
            first_data_pointer_of_inode_two.write_order,
        ),
    );
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_reddens_like_the_cpu_checker_on_an_extra_tombstone_and_a_moved_mapping_location_of_the_live_file",
        &image,
        "SCAN_REBUILD_OUTCOME_VIOLATED",
    );
}

/// 截断字段之外还有出生早于 T 的活单元：I-9.39 红，两边的 I-1.13 都报不适用、理由点名 I-9.39。
#[test]
fn the_gpu_scan_rebuild_is_not_applicable_like_the_cpu_checker_when_a_live_data_unit_is_born_before_the_truncation_field(
) {
    let (mut pool, _, birth_txg) =
        three_unit_file_born_together("gpu-scan-rebuild-truncation-field-after-the-live-units");
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, birth_txg + 1, 0);
    set_incompat_bit_two(&mut image);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_is_not_applicable_like_the_cpu_checker_when_a_live_data_unit_is_born_before_the_truncation_field",
        &image,
        "SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_9_39",
    );
}

/// 截在单元边界再变长、带截断字段：读法 Z 第四条判死放掉的单元 1、2。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_extending_past_a_truncation_with_its_field(
) {
    let (pool, intent_txg) = pool_extended_after_a_truncation_at_a_data_unit_boundary(
        "gpu-scan-rebuild-truncation-field-at-a-boundary",
    );
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, intent_txg, payload_capacity());
    set_incompat_bit_two(&mut image);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_extending_past_a_truncation_with_its_field",
        &image,
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 必红世界：同上一形而截断字段清零，扫描重建认回放掉的单元 1、2。
#[test]
fn the_gpu_scan_rebuild_reddens_like_the_cpu_checker_after_extending_past_a_truncation_without_its_field(
) {
    let (pool, _) = pool_extended_after_a_truncation_at_a_data_unit_boundary(
        "gpu-scan-rebuild-truncation-field-cleared-at-a-boundary",
    );
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, 0, 0);
    set_incompat_bit_two(&mut image);
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_reddens_like_the_cpu_checker_after_extending_past_a_truncation_without_its_field",
        &image,
        "SCAN_REBUILD_OUTCOME_VIOLATED",
    );
}

/// 合法搬迁过的镜像：遍历方向的落点是扫描重建扫到的那几份的子集（源槽留着同一版逐字节相同的副本）。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_on_an_image_relocated_by_the_central_mapping() {
    let pool = pool_with_an_empty_inode_after_the_first_file("gpu-scan-rebuild-relocated");
    let mut image = pool.memory_pool();
    let units = data_unit_and_extent_root_of_the_newest_version(&image);
    let relocated = relocate_in_the_newest_version(
        &mut image,
        &units,
        AllocationRecordsOfTheRelocation::SourceReleasedAndDestinationAllocated,
    );
    assert!(
        relocated
            .iter()
            .all(|relocation| relocation.destination_slot != relocation.unit.source_slot),
        "都搬到了别的槽：{relocated:?}"
    );
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_on_an_image_relocated_by_the_central_mapping",
        &image,
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}

/// 快照下覆盖写：活头换下的单元进 deadlist、映射条目留存，遍历方向那一张中央映射不收 deadlist 点名的那几条。
#[test]
fn the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_an_overwrite_under_a_snapshot() {
    let pool =
        pool_with_a_snapshot_and_an_overwrite_under_it("gpu-scan-rebuild-snapshot-overwrite");
    judge_the_image_and_assert_the_scan_rebuild_outcome(
        "the_gpu_scan_rebuild_holds_like_the_cpu_checker_after_an_overwrite_under_a_snapshot",
        &pool.memory_pool(),
        "SCAN_REBUILD_OUTCOME_HOLDS",
    );
}
