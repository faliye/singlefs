//! checker 档模块：crash、crash_identity、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器里池级 checker 那一份判里程碑三收尾补的三处，与 CPU 池级 checker（`crates/singlefs-checker/src/walk.rs`）逐位相同：
//! I-1.5（系统配置记管道身份，C868；走读内核择系统配置时逐盘判）、I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数，C411；
//! 引用与快照那一截在 I-3.15 那一份版本清单上判）、I-8.10 往非末条扩的那一半（同一次发布的每条记录新根段相同，C738；走读内核逐盘按组判）。
//! 每一处一条用例、各两张镜像：干净的那一张（这一条判到、成立）与只改坏这一处的那一张（这一条判红），两张都两边各判一遍，
//! 逐条不变量的违例位与判到位整份比。改坏的那一张照 harness 档 `checker_known_bad_images.rs` 里同名那几份坏镜像的坏法造
//! （那一份钉 CPU 判器、这一份钉 GPU 判器）；共用的搭建经 `#[path]` 指回 harness 档的 `tests/common*/mod.rs`，改字节的那几步就地写
//! （harness 档不依赖 checker 档）。GPU 少判哪一处，那一张改坏的镜像上 GPU 少一位违例、与 CPU 对不上。
//! 每张镜像两边的输入都经对拍那一类的库：GPU 那一侧把镜像当基线、一个空持久集合建判表，判表入库再读回上卡；CPU 那一侧把镜像作一张
//! 给定状态表写进库、读回，池级 checker 判读回的那一份；两边判出的东西作一份对拍结论入库（用户 2026-10-15 定「对拍结论入库」，
//! `crash_library::pairing_conclusion_reused_or_judged`），写法照 `tests/crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs`。
//! GPU 判不了判红。要本机一张 NVIDIA 独显：一张适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。
//! 这几条是 GPU 判器（加速路径）与 CPU 池级 checker（参照路径）的对拍，照 `.claude/rules/test-script-discipline.md`「分快慢两档」一律进快档、
//! 不看快慢判据，都不标 `#[ignore]`。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;

use common::{
    build_pool, parameters, rewrite_journal_record_and_rechain_the_records_after_it,
    PoolImageGeometry,
};
use common_pointer_head_corruption::{
    allocation_record_tree_nodes_as_far_as_readable, change_the_unit_and_propagate_its_checksum_on,
    image_after_a_sequential_write_of, newest_root, node_bytes, the_slot_of,
    tree_table_entry_of_kind, unit_bytes_at,
};
use singlefs_checker::image::{
    InvariantVerdict, ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
    IMPLEMENTED_INVARIANTS, LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
    SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::CrashPointReplaySegmentExpansion;
use singlefs_checker_tier::crash_amplification::RecordedGivenStatesTable;
use singlefs_checker_tier::crash_identity::{judging_units_key, JudgingUnits, SectorImage};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_judge_gpu::{
    cross_check_words_of_gpu_verdict, pool_checker_violated_and_judged_bits, VERDICT_WORDS,
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
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::allocator::ALLOCATION_RECORD_RELEASED_FLAG;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::records::TREE_KIND_ALLOCATION;
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_core::unit::{
    parse_index_node, unit_filesystem_identifier, UNIT_CLASS_DATA, UNIT_MAGIC,
};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::sha256::sha256_digest;

// ───────────────────────── 对拍：两边各判一遍、结论入库 ─────────────────────────

/// 一张镜像两边各判成什么：逐条不变量的违例位与判到位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`），加 CPU 那一侧的逐条判定（判红时写进说明）。
struct BothJudgesOnAnImage {
    cpu_verdicts: Vec<(&'static str, InvariantVerdict)>,
    cpu_bits: (u128, u128),
    gpu_bits: (u128, u128),
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
            "gpu judge writer identity allocation record spans and new root segments pairing inputs\nbase={}\ntables={}\n",
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
/// `tests/crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs` 的 `judge_the_image_on_both_judges`）。
/// 每趟打一行 `GPU_WRITER_IDENTITY_SPAN_AND_SEGMENT_PAIRING_CONCLUSION table=<表名> source=<ReusedFromTheLibrary|JudgedNow>`。
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
    println!(
        "GPU_WRITER_IDENTITY_SPAN_AND_SEGMENT_PAIRING_CONCLUSION table={table_name} source={source:?}"
    );
    let judgements = JudgementsOfBothJudgesOnAnImage::from_bytes(&conclusion)
        .expect("库里这一键下的对拍结论是这份测试写的，认得回");
    let gpu_words = cross_check_words_of_gpu_verdict(&judgements.gpu_verdict_words);
    assert!(
        !gpu_words.gpu_could_not_judge,
        "{table_name}：GPU 判器在这张镜像上判不了（结论里判不了那一位置了）"
    );
    let cpu_bits = pool_checker_violated_and_judged_bits(&judgements.cpu_verdicts);
    Some(BothJudgesOnAnImage {
        cpu_verdicts: judgements.cpu_verdicts,
        cpu_bits,
        gpu_bits: (
            gpu_words.pool_checker_violated_bits,
            gpu_words.pool_checker_judged_bits,
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

/// `invariant` 在违例位与判到位里的那一位。
fn bit_of(invariant: &str) -> u128 {
    1u128
        << IMPLEMENTED_INVARIANTS
            .iter()
            .position(|candidate| *candidate == invariant)
            .unwrap_or_else(|| panic!("{invariant} 在池级 checker 的清单里"))
}

/// 这一张镜像上 `invariant` 该判成什么：判到而成立（干净的那一张），或判红（只改坏这一处的那一张）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedVerdictOfTheInvariant {
    JudgedAndHolds,
    Violated,
}

/// 一条用例：借卡，干净的那一张与改坏的那一张各两边判一次、逐位比（违例位与判到位整份比，每一条不变量），再核 `invariant` 在两张上
/// 各判成该判成的那样（干净的判到而成立、改坏的判红）。借不到卡、只录那一趟时收手（「本次未跑」那一行已打）。
fn judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
    test_name: &str,
    invariant: &str,
    clean: &MemoryPool,
    changed: &MemoryPool,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    for (label, image, expected) in [
        (
            "clean",
            clean,
            ExpectedVerdictOfTheInvariant::JudgedAndHolds,
        ),
        ("changed", changed, ExpectedVerdictOfTheInvariant::Violated),
    ] {
        let table_name = format!("{test_name}/{label}");
        let Some(both) = judge_the_image_on_both_judges(cards, &table_name, image) else {
            return;
        };
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
        let bit = bit_of(invariant);
        let verdict_on_the_cpu = both
            .cpu_verdicts
            .iter()
            .find(|(judged, _)| *judged == invariant)
            .map(|(_, verdict)| verdict);
        let observed = match (cpu_violated & bit != 0, cpu_judged & bit != 0) {
            (true, _) => Some(ExpectedVerdictOfTheInvariant::Violated),
            (false, true) => Some(ExpectedVerdictOfTheInvariant::JudgedAndHolds),
            (false, false) => None,
        };
        assert_eq!(
            observed,
            Some(expected),
            "{table_name}：{invariant} 要判成 {expected:?}，CPU 判的是 {verdict_on_the_cpu:?}（两边逐位相同，GPU 也是这样）"
        );
    }
}

// ───────────────────────── I-1.5：系统配置记管道身份 ─────────────────────────

/// 系统配置自举头里写入者身份的实现标识那 16 字节：紧跟 fsid（偏移 102）16 字节（D22（单元原子性怎么合成） 已定项 9 的字段表）。
const SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER: std::ops::Range<usize> = 118..134;

/// 盘 1 两槽系统配置的实现标识 16 字节都清零、整槽重封（世代号与别的字段照原样）：盘 1 择到哪一槽都是全零。
fn clear_the_writer_implementation_identifier_of_every_slot_of_device_one(image: &mut MemoryPool) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device = image
        .devices
        .get_mut(&DeviceIdentity(1))
        .expect("池里有盘 1");
    for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let offset = DeviceOffsetInBytes(slot * spacing);
        let mut bytes = device.read(offset, slot_bytes);
        assert_ne!(
            bytes[SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER], [0u8; 16],
            "干净镜像上盘 1 槽 {slot} 的实现标识是 core 写的 singlefs-rs"
        );
        bytes[SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER].fill(0);
        let digest = wide_checksum_with_field_zeroed(
            &bytes,
            slot_bytes,
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
        );
        bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        device.write(offset, &bytes);
    }
}

/// I-1.5（C868）：写完新池新建文件那一张，盘 1 两槽的实现标识清零、重封。干净的那一张逐盘判到而成立，改坏的那一张 CPU 判红；
/// GPU 走读内核不判 I-1.5 时改坏的那一张上少一位 I-1.5。
#[test]
fn the_gpu_judge_reddens_a_chosen_system_configuration_whose_writer_implementation_identifier_is_all_zero_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_a_chosen_system_configuration_whose_writer_implementation_identifier_is_all_zero_like_the_cpu_checker";
    let clean = build_pool("gpu-judge-writer-identity-cleared").memory_pool();
    let mut changed = clean.clone();
    clear_the_writer_implementation_identifier_of_every_slot_of_device_one(&mut changed);
    judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
        TEST_NAME,
        SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY,
        &clean,
        &changed,
    );
}

// ───────────────────────── I-3.18：分配记录的跨度对它罩住的单元那一类 ─────────────────────────

/// 分配记录（key 设备 4 + 槽号 6，value 跨度段 2 + 分配代或释放代 8）里跨度段的偏移与宽（D3（空间分配） 已定项 7 的字段表）。
const ALLOCATION_RECORD_SPAN_FIELD: std::ops::Range<usize> = 10..12;
/// 分配记录里槽号那 6 字节。
const ALLOCATION_RECORD_SLOT: std::ops::Range<usize> = 4..10;
/// 码 1 数据单元（32 KiB）占的槽数：改坏前那几条记录的跨度。
const SLOTS_OF_A_DATA_UNIT: u16 = 2;
/// 改坏后的跨度：比码 1 那一类少一槽。
const NARROWED_SPAN_SLOTS: u16 = 1;
/// 认起点槽上是哪一类单元时读的字节数（码 1 的明文头 105 字节在里面）。
const UNIT_HEADER_BYTES_READ: usize = 4096;
/// 单元头里类标签那 1 字节（共同前缀：magic 4、格式版本 2 之后）。
const UNIT_CLASS_OFFSET: usize = 6;

/// 一条分配记录（叶里一条条目的原样）是不是未释放、跨两槽、起点槽上是码 1 数据单元。
fn covers_an_unreleased_data_unit_over_two_slots(image: &MemoryPool, entry: &[u8]) -> bool {
    let span_field = u16::from_le_bytes(
        entry[ALLOCATION_RECORD_SPAN_FIELD]
            .try_into()
            .expect("跨度段 2 字节"),
    );
    if span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0 || span_field != SLOTS_OF_A_DATA_UNIT {
        return false;
    }
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&entry[ALLOCATION_RECORD_SLOT]);
    let header = unit_bytes_at(
        image,
        SlotNumber(u64::from_le_bytes(slot_bytes)),
        UNIT_HEADER_BYTES_READ,
    );
    header[..UNIT_MAGIC.len()] == UNIT_MAGIC && header[UNIT_CLASS_OFFSET] == UNIT_CLASS_DATA
}

/// 写完新池新建文件那一张上，最新根的分配记录树每片叶里罩住码 1 数据单元的未释放记录跨度从 2 改成 1（两块盘的叶各一条），
/// 每片叶重封、新的整单元校验和沿引用链补到根槽、根抄进记录的新根段。交回改了几条。
fn narrow_the_records_of_the_data_units_to_one_slot(image: &mut MemoryPool) -> usize {
    let allocation_tree = tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION);
    let leaves: Vec<SlotNumber> =
        allocation_record_tree_nodes_as_far_as_readable(image, &allocation_tree.root)
            .iter()
            .map(the_slot_of)
            .filter(|slot| {
                parse_index_node(&unit_bytes_at(image, *slot, node_bytes()))
                    .expect("分配记录树走得到的节点解得开")
                    .level
                    == 0
            })
            .collect();
    let mut narrowed = 0;
    for leaf in leaves {
        let entries_to_narrow: Vec<Vec<u8>> =
            parse_index_node(&unit_bytes_at(image, leaf, node_bytes()))
                .expect("分配记录树的叶解得开")
                .entries
                .into_iter()
                .filter(|entry| covers_an_unreleased_data_unit_over_two_slots(image, entry))
                .collect();
        if entries_to_narrow.is_empty() {
            continue;
        }
        narrowed += entries_to_narrow.len();
        change_the_unit_and_propagate_its_checksum_on(
            image,
            &PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters(),
            leaf,
            node_bytes(),
            |bytes| {
                for entry in &entries_to_narrow {
                    let positions: Vec<usize> = bytes
                        .windows(entry.len())
                        .enumerate()
                        .filter(|(_, window)| *window == entry.as_slice())
                        .map(|(position, _)| position)
                        .collect();
                    let [position] = positions.as_slice() else {
                        panic!(
                            "叶里这条记录的原样恰好出现一次，实际 {} 次",
                            positions.len()
                        );
                    };
                    let span_field = position + ALLOCATION_RECORD_SPAN_FIELD.start
                        ..position + ALLOCATION_RECORD_SPAN_FIELD.end;
                    bytes[span_field].copy_from_slice(&NARROWED_SPAN_SLOTS.to_le_bytes());
                }
            },
        );
    }
    narrowed
}

/// I-3.18（C411）：写完新池新建文件那一张，码 1 数据单元那两条未释放记录（两块盘的叶各一条）只记一槽。干净的那一张判到而成立，
/// 改坏的那一张 CPU 判红；GPU 引用与快照那一截不判 I-3.18 时改坏的那一张上少一位 I-3.18。
#[test]
fn the_gpu_judge_reddens_an_unreleased_allocation_record_spanning_one_slot_of_a_two_slot_data_unit_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_an_unreleased_allocation_record_spanning_one_slot_of_a_two_slot_data_unit_like_the_cpu_checker";
    let clean = build_pool("gpu-judge-data-unit-record-narrowed").memory_pool();
    let mut changed = clean.clone();
    assert_eq!(
        narrow_the_records_of_the_data_units_to_one_slot(&mut changed),
        2,
        "数据单元那一个落点，两块盘的叶各一条"
    );
    judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
        TEST_NAME,
        ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
        &clean,
        &changed,
    );
}

// ───────────────────────── I-8.10 往非末条扩：同一次发布每条记录的新根段相同 ─────────────────────────

/// 盘 0 默认 journal 环里计数器 `counter` 那一格解出来的记录；没写过、解不出交 `None`。
fn journal_record_on_device_zero(image: &MemoryPool, counter: u64) -> Option<JournalRecord> {
    let bytes = image
        .devices
        .get(&DeviceIdentity(0))
        .expect("池里有盘 0")
        .read(
            record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
            usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
        );
    JournalRecord::parse(
        &bytes,
        unit_filesystem_identifier(&parameters().filesystem_identifier),
    )
}

/// 新池新建文件（A）之后顺序写一版两个数据单元的文件（B，一次发布两条记录）那一张上，B 的第一条（非末条）记录的新根段九段换成
/// 它前一条（A 那条）记录的（写者把非末条的新根段写成上一版，`research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b
/// 攻方 s1 那一形），两盘各一份、重封，它之后的记录反向链跟着重算。末条照原样带 B 的那一份，与 B 的根相同。
fn put_the_new_root_segment_of_the_publish_before_into_the_first_record_of_the_newest_publish(
    image: &mut MemoryPool,
) {
    let newest_checkpoint_txg = newest_root(image).checkpoint_txg;
    // 计数器从 1 起连着写，读到第一格解不出就停：迭代次数的上界是环里写过的记录条数。
    let records_of_the_newest_publish: Vec<JournalRecord> = (1u64..)
        .map_while(|counter| journal_record_on_device_zero(image, counter))
        .filter(|record| record.checkpoint_txg == newest_checkpoint_txg)
        .collect();
    let [first_record, last_record] = records_of_the_newest_publish.as_slice() else {
        panic!(
            "最新那次发布（txg {}）在盘 0 上落了两条记录，实际 {} 条",
            newest_checkpoint_txg.0,
            records_of_the_newest_publish.len()
        );
    };
    assert_eq!(
        (first_record.place_in_publish, last_record.place_in_publish),
        (
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
            JournalRecordPlaceInPublish::LastRecordOfThePublish
        ),
        "两条记录里只有第二条带末条标志"
    );
    let record_of_the_publish_before =
        journal_record_on_device_zero(image, first_record.counter - 1)
            .expect("最新那次发布的第一条记录之前那一格是上一次发布的记录");
    assert_eq!(
        record_of_the_publish_before.checkpoint_txg,
        CheckpointTxg(newest_checkpoint_txg.0 - 1),
        "前一条记录是上一次发布（A）的"
    );
    assert_ne!(
        record_of_the_publish_before.new_tree_table, first_record.new_tree_table,
        "上一次发布的新根段与最新那次的不同：换过去才是写旧了"
    );
    rewrite_journal_record_and_rechain_the_records_after_it(
        image,
        first_record.counter,
        |record| {
            record.new_tree_table = record_of_the_publish_before.new_tree_table;
            record.new_mapping_root = record_of_the_publish_before.new_mapping_root;
            record.new_tree_identifier_watermark =
                record_of_the_publish_before.new_tree_identifier_watermark;
            record.new_rollback_floor = record_of_the_publish_before.new_rollback_floor;
            record.new_instance_table = record_of_the_publish_before.new_instance_table;
            record.new_allocation_record_tree_root =
                record_of_the_publish_before.new_allocation_record_tree_root;
            record.new_unmount_marker = record_of_the_publish_before.new_unmount_marker;
            record.new_reclaim_threshold = record_of_the_publish_before.new_reclaim_threshold;
            record.new_pinned_ring_slot = record_of_the_publish_before.new_pinned_ring_slot;
        },
    );
}

/// I-8.10 往非末条扩的那一半（C738）：一次两条记录的发布，非末条带上一版的新根段。干净的那一张判到而成立（同一次发布两条记录新根段相同、
/// 末条与根相同），改坏的那一张 CPU 判红；GPU 走读内核只比末条与根时改坏的那一张上少一位 I-8.10。
#[test]
fn the_gpu_judge_reddens_a_non_last_record_carrying_the_new_root_segment_of_the_version_before_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_a_non_last_record_carrying_the_new_root_segment_of_the_version_before_like_the_cpu_checker";
    let clean =
        image_after_a_sequential_write_of("gpu-judge-new-root-segment-of-a-non-last-record", 2);
    let mut changed = clean.clone();
    put_the_new_root_segment_of_the_publish_before_into_the_first_record_of_the_newest_publish(
        &mut changed,
    );
    judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
        TEST_NAME,
        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
        &clean,
        &changed,
    );
}
