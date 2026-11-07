//! 验证包模块：crash、crash_identity、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器里池级 checker 那一份判里程碑三收尾补的三处，与 CPU 池级 checker（`crates/singlefs-checker/src/walk.rs`）逐位相同：
//! I-1.5（系统配置记管道身份，C868；走读内核择系统配置时逐盘判）、I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数，C411；
//! 引用与快照那一截在 I-3.15 那一份版本清单上判）、I-8.10 往非末条扩的那一半（同一次发布的每条记录新根段相同，C738；走读内核逐盘按组判）。
//! 每一处一条用例、各两张镜像：干净的那一张（这一条判到、成立）与只改坏这一处的那一张（这一条判红），两张都两边各判一遍，
//! 逐条不变量的违例位与判到位整份比。I-3.18 那一张另钉 I-3.1 报部分没判（配不上的那块全落在 I-3.18 判红的单元里）。
//! I-3.1 取宽只认整份改记读法、另加逐槽两判（C937，判决 `research/prompts/m3-close-b19-reason-r1-main-verification.md` 第二节 Q3「改法」那一行）
//! 另有三条：宽、窄两张坏镜像两边都判 I-3.1 红，记录比单元那一类宽而改记之后整份成立的那一张两边都报部分没判；
//! 取宽那一支加的第五判（改记之后未释放记录不与别的记录相交，判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节 Q1「改法」）
//! 与新立的 I-3.21（走到的引用起点是最新根计入的分配记录的起点，同一判决 Q2「归属」）各一条，两边都判红。
//! 另有三条（Q2 第 1 问、Q3 第 2 问）：
//! 最新根的树表指针全零那一张两边 I-3.21 都没判到（CPU 报部分没判）；树表 0 条、写过行的那一版上漏记写行那次发布新写的那片实例表那一张
//! 两边都照根记录直接持有的那棵分配记录树判 I-3.21 红（C963 候选一）；读回全零的根环槽并进抬 F 上限那一对（C961，攻方第二样本 `first_example`）
//! 两边都判合法写者那张 I-7.9 成立、坏写者那张红。改坏的那一张照 `checker_known_bad_images.rs` 里同名那几份坏镜像的坏法造
//! （那一份钉 CPU 判器、这一份钉 GPU 判器）；共用的搭建经 `mod` 引 `tests/common*/mod.rs`，改字节的那几步就地写。GPU 少判哪一处，那一张改坏的镜像上 GPU 少一位违例、与 CPU 对不上。
//! 每张镜像两边的输入都经对拍那一类的库：GPU 那一侧把镜像当基线、一个空持久集合建判表，判表入库再读回上卡；CPU 那一侧把镜像作一张
//! 给定状态表写进库、读回，池级 checker 判读回的那一份；两边判出的东西作一份对拍结论入库（
//! `crash_library::pairing_conclusion_reused_or_judged`），写法照 `tests/crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs`。
//! GPU 判不了判红。要本机一张 NVIDIA 独显：一张适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。
//! 这几条是 GPU 判器（加速路径）与 CPU 池级 checker（参照路径）的对拍，不标 `#[ignore]`，不带 `--ignored` 的 `cargo test` 就跑到。
#![cfg(feature = "gpu")]

mod common;
mod common_pointer_head_corruption;
mod common_root_ring_holes;

use std::collections::BTreeSet;

use common::{
    build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records,
    crash_state_devices, memory_pool_of_sparse_devices, parameters,
    rewrite_journal_record_and_rechain_the_records_after_it, PoolImageGeometry, IMAGE_BYTES,
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
    WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::allocator::ALLOCATION_RECORD_RELEASED_FLAG;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::{record_offset, JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::records::TREE_KIND_ALLOCATION;
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_core::unit::{
    index_node_header_bytes, parse_index_node, unit_filesystem_identifier, UNIT_CLASS_DATA,
    UNIT_MAGIC,
};
use singlefs_format::{
    ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_KEY_BYTES, JOURNAL_RECORD_BYTES,
    JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use singlefs_verification::crash::CrashPointReplaySegmentExpansion;
use singlefs_verification::crash_amplification::RecordedGivenStatesTable;
use singlefs_verification::crash_identity::{judging_units_key, JudgingUnits, SectorImage};
use singlefs_verification::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_verification::crash_judge_gpu::{
    cross_check_words_of_gpu_verdict, pool_checker_violated_and_judged_bits, VERDICT_WORDS,
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
    /// 两边比下来一致不一致（对拍结论照它记红绿，记着不一致的下一趟现判）：GPU 判得了，违例位与判到位两边逐位相同
    /// （与比法那一句断言同一个比法）。
    fn agreement_of_the_two_judges(&self) -> PairingAgreement {
        let gpu = cross_check_words_of_gpu_verdict(&self.gpu_verdict_words);
        if !gpu.gpu_could_not_judge
            && (gpu.pool_checker_violated_bits, gpu.pool_checker_judged_bits)
                == pool_checker_violated_and_judged_bits(&self.cpu_verdicts)
        {
            PairingAgreement::BothSidesAgree
        } else {
            PairingAgreement::SidesDisagree
        }
    }

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
) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
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
/// 各判成该判成的那样（干净的判到而成立、改坏的判红）。交回 CPU 在改坏的那一张上逐条的判定（两边逐位相同已核过），给要再钉别的不变量的用例用。
/// 借不到卡、只录那一趟时收手、交 `None`（「本次未跑」那一行已打）。
fn judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
    test_name: &str,
    invariant: &str,
    clean: &MemoryPool,
    changed: &MemoryPool,
) -> Option<Vec<(&'static str, InvariantVerdict)>> {
    let cards = gpu_card_pool_or_report_not_run(test_name)?;
    let mut cpu_verdicts_on_the_changed_image = None;
    for (label, image, expected) in [
        (
            "clean",
            clean,
            ExpectedVerdictOfTheInvariant::JudgedAndHolds,
        ),
        ("changed", changed, ExpectedVerdictOfTheInvariant::Violated),
    ] {
        let table_name = format!("{test_name}/{label}");
        let both = judge_the_image_on_both_judges(cards, &table_name, image)?;
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
        if expected == ExpectedVerdictOfTheInvariant::Violated {
            cpu_verdicts_on_the_changed_image = Some(both.cpu_verdicts);
        }
    }
    cpu_verdicts_on_the_changed_image
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

/// 最新根那棵分配记录树走得到的每片叶（第 0 层节点）的槽：每块盘各一片，一片叶只装它那块盘上的记录。
fn allocation_record_tree_leaves_of_the_newest_root(image: &MemoryPool) -> Vec<SlotNumber> {
    let allocation_tree = tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION);
    allocation_record_tree_nodes_as_far_as_readable(image, &allocation_tree.root)
        .iter()
        .map(the_slot_of)
        .filter(|slot| {
            parse_index_node(&unit_bytes_at(image, *slot, node_bytes()))
                .expect("分配记录树走得到的节点解得开")
                .level
                == 0
        })
        .collect()
}

/// 写完新池新建文件那一张上，最新根的分配记录树每片叶里罩住码 1 数据单元的未释放记录跨度从 2 改成 1（两块盘的叶各一条），
/// 每片叶重封、新的整单元校验和沿引用链补到根槽、根抄进记录的新根段。交回改了几条。
fn narrow_the_records_of_the_data_units_to_one_slot(image: &mut MemoryPool) -> usize {
    let mut narrowed = 0;
    for leaf in allocation_record_tree_leaves_of_the_newest_root(image) {
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
/// 改坏的那一张上 I-3.1 按记录自述的跨度三判不成立（数据单元第二槽遍历走到、不在记录里），配不上的那块全落在 I-3.18 判红的单元里：
/// CPU 每块盘报部分没判、说明点名 I-3.18；
/// GPU 扫描内核不按单元那一类再算一份界时那一位判红、与 CPU 逐位不同。
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
    let Some(cpu_verdicts_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME,
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            &clean,
            &changed,
        )
    else {
        return;
    };
    let allocated_statistic = cpu_verdicts_on_the_changed_image
        .iter()
        .find(|(invariant, _)| *invariant == ALLOCATED_STATISTIC_INVARIANT)
        .map(|(_, verdict)| verdict);
    match allocated_statistic {
        Some(InvariantVerdict::PartiallyJudged(unjudged)) => assert!(
            unjudged.contains("（I-3.18 判它）"),
            "改坏的那一张上 I-3.1 部分没判的说明点名 I-3.18：{unjudged}"
        ),
        other_verdict @ (None
        | Some(
            InvariantVerdict::Holds
            | InvariantVerdict::Violated(_)
            | InvariantVerdict::NotApplicable(_),
        )) => panic!(
            "改坏的那一张上配不上的全落在 I-3.18 判红的单元里：CPU 的 I-3.1 该报部分没判（两边逐位相同，GPU 也是这样），实际 {other_verdict:?}"
        ),
    }
}

/// I-3.1（已分配统计对得上）在池级 checker 逐条判定里的名字。
const ALLOCATED_STATISTIC_INVARIANT: &str = "I-3.1";

// ───────────────────────── I-3.1 取宽：只认整份改记读法、另加逐槽两判（C937） ─────────────────────────

/// 分配记录树叶（key 宽 10）码 2 头里条目数那 2 字节的偏移：82 + 2 × key 宽（D18（块里携带什么信息） 已定项 18 的偏移表）。
/// [`change_the_allocation_record_tree_leaves`] 每片叶先拿 `parse_index_node` 解出的条目数核它，再改。
const ALLOCATION_RECORD_TREE_LEAF_ENTRY_COUNT_OFFSET: usize = 102;
/// 码 2 头里声明长度那 2 字节的偏移（共同前缀里；声明长度 = 条目数 × 条目宽）。
const INDEX_NODE_DECLARED_LENGTH_OFFSET: usize = 8;
/// 插进叶里的那条泄漏记录的跨度：罩一槽。
const LEAKED_RECORD_SPAN_SLOTS: u16 = 1;
/// 宽的方向：映射树根那条一槽单元的记录改成跨 3，罩住映射树根、树表与树表后面那一槽。
const SPAN_SLOTS_WIDENED_OVER_THE_TREE_TABLE: u16 = 3;
/// 比码 1 数据单元那一类多一槽的跨度：多罩的那一槽盘上全零、遍历走不到。
const SPAN_SLOTS_ONE_PAST_THE_DATA_UNIT: u16 = 3;
/// 窄的方向里那条泄漏记录罩的槽（同 harness 那一份）：数据单元之后、单元区里下一个单元之前那一段里的一槽，盘上全零、不是单元，遍历走不到。
const ALL_ZERO_SLOT_A_LEAKED_RECORD_COVERS: SlotNumber = SlotNumber(50200);

/// 一条分配记录在叶里占的字节数（key 设备 4 + 槽号 6，value 跨度段 2 + 分配代或释放代 8）。
fn allocation_record_bytes() -> usize {
    usize::try_from(ALLOCATION_RECORD_BYTES).expect("20")
}

/// 叶里第 `index` 条记录的字节范围：条目区从码 2 头（明文头加预留位，key 宽取分配记录的 key 宽）之后起，每条定宽。
fn allocation_record_range_in_the_leaf(index: usize) -> std::ops::Range<usize> {
    let entry_start =
        index_node_header_bytes(usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"));
    let start = entry_start + allocation_record_bytes() * index;
    start..start + allocation_record_bytes()
}

fn allocation_record_count_of_the_leaf(leaf: &[u8]) -> usize {
    usize::from(u16::from_le_bytes(
        leaf[ALLOCATION_RECORD_TREE_LEAF_ENTRY_COUNT_OFFSET
            ..ALLOCATION_RECORD_TREE_LEAF_ENTRY_COUNT_OFFSET + 2]
            .try_into()
            .expect("条目数 2 字节"),
    ))
}

/// 条目数改成 `count`，声明长度跟着改成 `count` × 条目宽。
fn set_the_allocation_record_count_of_the_leaf(leaf: &mut [u8], count: usize) {
    let count_field = u16::try_from(count).expect("条目数装得进 u16");
    leaf[ALLOCATION_RECORD_TREE_LEAF_ENTRY_COUNT_OFFSET
        ..ALLOCATION_RECORD_TREE_LEAF_ENTRY_COUNT_OFFSET + 2]
        .copy_from_slice(&count_field.to_le_bytes());
    let declared_length =
        u16::try_from(count * allocation_record_bytes()).expect("声明长度装得进 u16");
    leaf[INDEX_NODE_DECLARED_LENGTH_OFFSET..INDEX_NODE_DECLARED_LENGTH_OFFSET + 2]
        .copy_from_slice(&declared_length.to_le_bytes());
}

/// 一条记录（叶里条目的原样）起点的槽号。
fn slot_of_the_allocation_record(record: &[u8]) -> SlotNumber {
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&record[ALLOCATION_RECORD_SLOT]);
    SlotNumber(u64::from_le_bytes(slot_bytes))
}

/// 叶里起点槽是 `slot` 的那条记录的下标；没有交 `None`。
fn index_of_the_allocation_record_of(leaf: &[u8], slot: SlotNumber) -> Option<usize> {
    (0..allocation_record_count_of_the_leaf(leaf)).find(|index| {
        slot_of_the_allocation_record(&leaf[allocation_record_range_in_the_leaf(*index)]) == slot
    })
}

/// 最新根分配记录树的每片叶：先核叶头（条目宽、条目数与第一条的位置同 `parse_index_node` 解出的），再把原样交给 `change`
/// （交回它在这片叶里改了几条记录）；改了的叶重封、新的整单元校验和沿引用链补到根槽、根抄进记录的新根段，没改的叶不动。
/// 交回各片合起来的条数。
fn change_the_allocation_record_tree_leaves(
    image: &mut MemoryPool,
    change: impl Fn(&mut Vec<u8>) -> usize,
) -> usize {
    let leaves = allocation_record_tree_leaves_of_the_newest_root(image);
    change_these_allocation_record_tree_leaves(image, leaves, change)
}

/// 同 [`change_the_allocation_record_tree_leaves`]，改的是 `leaves` 这几片叶（槽）。
fn change_these_allocation_record_tree_leaves(
    image: &mut MemoryPool,
    leaves: Vec<SlotNumber>,
    change: impl Fn(&mut Vec<u8>) -> usize,
) -> usize {
    let mut changed_records = 0;
    for leaf in leaves {
        let mut leaf_bytes = unit_bytes_at(image, leaf, node_bytes());
        let parsed = parse_index_node(&leaf_bytes).expect("分配记录树的叶解得开");
        assert_eq!(
            (
                parsed.entry_width,
                parsed.entries.len(),
                parsed.entries.first().map(Vec::as_slice)
            ),
            (
                allocation_record_bytes(),
                allocation_record_count_of_the_leaf(&leaf_bytes),
                Some(&leaf_bytes[allocation_record_range_in_the_leaf(0)])
            ),
            "槽 {} 那片叶：叶头的条目宽、条目数与条目区起点同 parse_index_node 解出的",
            leaf.0
        );
        let changed_in_this_leaf = change(&mut leaf_bytes);
        if changed_in_this_leaf == 0 {
            continue;
        }
        changed_records += changed_in_this_leaf;
        change_the_unit_and_propagate_its_checksum_on(
            image,
            &PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters(),
            leaf,
            node_bytes(),
            |bytes| *bytes = leaf_bytes,
        );
    }
    changed_records
}

/// 叶里起点槽是 `slot` 的那条记录跨度段改成 `span_slots`（已释放标志照留）；交回改了几条。
fn set_the_span_of_the_allocation_record_of(
    leaf: &mut [u8],
    slot: SlotNumber,
    span_slots: u16,
) -> usize {
    let Some(index) = index_of_the_allocation_record_of(leaf, slot) else {
        return 0;
    };
    let record_start = allocation_record_range_in_the_leaf(index).start;
    let span_field = record_start + ALLOCATION_RECORD_SPAN_FIELD.start
        ..record_start + ALLOCATION_RECORD_SPAN_FIELD.end;
    let span_before =
        u16::from_le_bytes(leaf[span_field.clone()].try_into().expect("跨度段 2 字节"));
    leaf[span_field].copy_from_slice(
        &((span_before & ALLOCATION_RECORD_RELEASED_FLAG) | span_slots).to_le_bytes(),
    );
    1
}

/// 叶里排在末尾、起点槽是 `slot` 的那条记录整条拿掉：末尾那一条清零、条目数与声明长度各减一条（分配记录树按位置寻址，
/// 头里按位置定的 key 区间不随条目变）。末条不是它交 0。
fn remove_the_last_allocation_record_of(leaf: &mut [u8], slot: SlotNumber) -> usize {
    let Some(last_index) = allocation_record_count_of_the_leaf(leaf).checked_sub(1) else {
        return 0;
    };
    let last = allocation_record_range_in_the_leaf(last_index);
    if slot_of_the_allocation_record(&leaf[last.clone()]) != slot {
        return 0;
    }
    leaf[last].fill(0);
    set_the_allocation_record_count_of_the_leaf(leaf, last_index);
    1
}

/// 叶里紧跟起点槽是 `after` 的那条记录插一条起在 `slot`、跨 1 槽、未释放的记录（盘与分配代抄 `after` 那一条）：后面的记录往后挪一条、
/// 条目数与声明长度各加一条。叶里没有 `after` 那条交 0。
fn insert_an_unreleased_one_slot_allocation_record_after(
    leaf: &mut [u8],
    after: SlotNumber,
    slot: SlotNumber,
) -> usize {
    let count = allocation_record_count_of_the_leaf(leaf);
    let Some(index) = index_of_the_allocation_record_of(leaf, after) else {
        return 0;
    };
    let source = allocation_record_range_in_the_leaf(index);
    let inserted = allocation_record_range_in_the_leaf(index + 1);
    assert!(
        index + 1 < count
            && after.0 < slot.0
            && slot.0 < slot_of_the_allocation_record(&leaf[inserted.clone()]).0,
        "插的那条（槽 {}）排在槽 {} 那条与下一条之间",
        slot.0,
        after.0
    );
    let end_of_the_records = allocation_record_range_in_the_leaf(count).start;
    leaf.copy_within(inserted.start..end_of_the_records, inserted.end);
    leaf.copy_within(source, inserted.start);
    leaf[inserted.start + ALLOCATION_RECORD_SLOT.start
        ..inserted.start + ALLOCATION_RECORD_SLOT.end]
        .copy_from_slice(&slot.0.to_le_bytes()[..ALLOCATION_RECORD_SLOT.len()]);
    leaf[inserted.start + ALLOCATION_RECORD_SPAN_FIELD.start
        ..inserted.start + ALLOCATION_RECORD_SPAN_FIELD.end]
        .copy_from_slice(&LEAKED_RECORD_SPAN_SLOTS.to_le_bytes());
    set_the_allocation_record_count_of_the_leaf(leaf, count + 1);
    1
}

/// 写完新池新建文件那一张上，罩住码 1 数据单元的那几条未释放、跨两槽的记录（两块盘的叶各一条）共同的起点槽。
fn slot_of_the_data_unit(image: &MemoryPool) -> SlotNumber {
    let mut slots: Vec<SlotNumber> = allocation_record_tree_leaves_of_the_newest_root(image)
        .into_iter()
        .flat_map(|leaf| {
            parse_index_node(&unit_bytes_at(image, leaf, node_bytes()))
                .expect("分配记录树的叶解得开")
                .entries
        })
        .filter(|entry| covers_an_unreleased_data_unit_over_two_slots(image, entry))
        .map(|entry| slot_of_the_allocation_record(&entry))
        .collect();
    slots.dedup();
    let [slot] = slots.as_slice() else {
        panic!("数据单元只有一个落点，记录的起点槽实际 {slots:?}");
    };
    *slot
}

/// 每块盘上 `slot` 那一槽全零（不是单元，遍历走不到）。
fn assert_every_device_holds_only_zeros_at(image: &MemoryPool, slot: SlotNumber, what: &str) {
    for (device, block_device) in &image.devices {
        assert!(
            block_device
                .read(slot.to_device_offset(), node_bytes())
                .iter()
                .all(|byte| *byte == 0),
            "{what}（槽 {}）在盘 {} 上全零",
            slot.0,
            device.0
        );
    }
}

/// I-3.1 取宽宽的方向那一形（同 harness
/// `record_widened_over_a_walked_one_slot_neighbor_without_its_own_record_reddens_the_allocated_statistic_and_the_span_invariants`）：
/// 写完新池新建文件那一张上，最新根的映射树根（码 2、一槽）与紧跟其后的树表（一槽）各有一条未释放记录（两块盘的叶各一条，树表那条排在叶末尾）；
/// 映射树根那条跨度改成 3（罩到树表后面那一槽），树表那条整条拿掉，记账不动。交回改了几条。
fn widen_the_record_of_the_mapping_root_over_the_tree_table_and_remove_the_record_of_the_tree_table(
    image: &mut MemoryPool,
) -> usize {
    let newest = newest_root(image);
    let mapping_root = the_slot_of(&newest.mapping_root);
    let tree_table = the_slot_of(&newest.tree_table);
    assert_eq!(tree_table.0, mapping_root.0 + 1, "树表紧跟在映射树根后面");
    assert_every_device_holds_only_zeros_at(image, SlotNumber(tree_table.0 + 1), "树表后面那一槽");
    change_the_allocation_record_tree_leaves(image, |leaf| {
        set_the_span_of_the_allocation_record_of(
            leaf,
            mapping_root,
            SPAN_SLOTS_WIDENED_OVER_THE_TREE_TABLE,
        ) + remove_the_last_allocation_record_of(leaf, tree_table)
    })
}

/// 一张改坏的镜像上 CPU 判红的那几条（两边逐位相同已核过，GPU 判红的也是这几条）。
fn violated_invariants_of(
    cpu_verdicts: &[(&'static str, InvariantVerdict)],
) -> BTreeSet<&'static str> {
    invariant_names(pool_checker_violated_and_judged_bits(cpu_verdicts).0)
        .into_iter()
        .collect()
}

/// I-3.1 取宽宽的方向（C937，判决 Q3 宽那一行的攻方最小镜像「两个一槽单元、一条跨 3 的记录、记账 2 槽」）：按记录自述 E 比记账多一槽，
/// (L) 不成立；改记成一槽之后遍历走到的树表不在记录里：只认改记那一份时整份不成立，两边都判 I-3.1 红，I-3.18 判跨 3 的那条也红，
/// 走到的引用起点（树表那一槽）上没有一条记录起头，I-3.21 也红（新立 I-3.21 之后多红这一条；别的会话合入后验证任务 6224 量的，
/// harness 那一份同形），别的都不红。GPU 扫描内核退回逐判混取时那一位报部分没判、与 CPU 逐位不同。
#[test]
fn the_gpu_judge_reddens_the_allocated_statistic_under_a_record_widened_over_a_walked_one_slot_neighbor_without_its_own_record_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_allocated_statistic_under_a_record_widened_over_a_walked_one_slot_neighbor_without_its_own_record_like_the_cpu_checker";
    let clean = build_pool("gpu-judge-record-widened-over-a-neighbor").memory_pool();
    let mut changed = clean.clone();
    assert_eq!(
        widen_the_record_of_the_mapping_root_over_the_tree_table_and_remove_the_record_of_the_tree_table(
            &mut changed
        ),
        4,
        "映射树根那条记录改宽、树表那条记录拿掉，两盘各一条"
    );
    let Some(cpu_verdicts_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME,
            ALLOCATED_STATISTIC_INVARIANT,
            &clean,
            &changed,
        )
    else {
        return;
    };
    assert_eq!(
        violated_invariants_of(&cpu_verdicts_on_the_changed_image),
        BTreeSet::from([
            ALLOCATED_STATISTIC_INVARIANT,
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
        ]),
        "改坏的那一张两边都只红 I-3.1、I-3.18 与 I-3.21：{cpu_verdicts_on_the_changed_image:?}"
    );
}

/// I-3.1 取宽窄的方向（C937，判决 Q3 窄那一行的攻方最小镜像）：起步是 I-3.18 那一张（数据单元那条未释放记录只记一槽，两盘各一条），
/// 另在两盘各插一条罩着全零槽的未释放泄漏记录，记账不动（窄记录漏数的那一槽与泄漏多数的那一槽在个数上抵掉）。改记成两槽之后 E 比记账多一槽、
/// 泄漏那一槽不在遍历走到的槽里也不在 defer 里：只认改记那一份时 (L) 与逐槽的上沿不成立，两边都判 I-3.1 红，I-3.18 判跨 1 的那条也红，
/// 别的都不红（harness 那一份钉 CPU 只红这两条）。GPU 扫描内核退回逐判混取时那一位报部分没判、与 CPU 逐位不同。
#[test]
fn the_gpu_judge_reddens_the_allocated_statistic_under_a_narrowed_record_with_a_leaked_record_over_an_all_zero_slot_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_allocated_statistic_under_a_narrowed_record_with_a_leaked_record_over_an_all_zero_slot_like_the_cpu_checker";
    let clean = build_pool("gpu-judge-narrowed-record-with-a-leak").memory_pool();
    let data_unit = slot_of_the_data_unit(&clean);
    let mut changed = clean.clone();
    assert_eq!(
        narrow_the_records_of_the_data_units_to_one_slot(&mut changed),
        2,
        "数据单元那一个落点，两块盘的叶各一条"
    );
    assert_eq!(
        change_the_allocation_record_tree_leaves(&mut changed, |leaf| {
            insert_an_unreleased_one_slot_allocation_record_after(
                leaf,
                data_unit,
                ALL_ZERO_SLOT_A_LEAKED_RECORD_COVERS,
            )
        }),
        2,
        "泄漏记录两盘各一条"
    );
    assert_every_device_holds_only_zeros_at(
        &changed,
        ALL_ZERO_SLOT_A_LEAKED_RECORD_COVERS,
        "泄漏记录罩的那一槽",
    );
    let Some(cpu_verdicts_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME,
            ALLOCATED_STATISTIC_INVARIANT,
            &clean,
            &changed,
        )
    else {
        return;
    };
    assert_eq!(
        violated_invariants_of(&cpu_verdicts_on_the_changed_image),
        BTreeSet::from([
            ALLOCATED_STATISTIC_INVARIANT,
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
        ]),
        "改坏的那一张两边都只红 I-3.1 与 I-3.18：{cpu_verdicts_on_the_changed_image:?}"
    );
}

/// 取宽之后整份成立那一形（池级 checker 库单测 `record_wider_than_its_two_slot_unit_is_left_to_the_span_invariant` 在盘上的样子）：
/// 写完新池新建文件那一张上，数据单元那两条未释放记录（两盘各一条）跨度改成 3，多罩的那一槽盘上全零、遍历走不到，记账照两槽、不动。
/// 按记录自述下沿比记账多一槽，三判不成立；改记成两槽之后整份三判与逐槽两判都成立：I-3.18 两边都判红，I-3.1 CPU 报部分没判、
/// 说明点名 I-3.18，GPU 那一位既不判到也不违例（两边逐位相同）。GPU 扫描内核按单元那一类数时只往宽改（记录比那一类宽的照跨度段数）
/// 时那一位判红、与 CPU 逐位不同。
#[test]
fn the_gpu_judge_leaves_the_allocated_statistic_partially_judged_under_a_record_one_slot_wider_than_its_two_slot_data_unit_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_leaves_the_allocated_statistic_partially_judged_under_a_record_one_slot_wider_than_its_two_slot_data_unit_like_the_cpu_checker";
    let clean = build_pool("gpu-judge-data-unit-record-widened").memory_pool();
    let data_unit = slot_of_the_data_unit(&clean);
    let mut changed = clean.clone();
    assert_eq!(
        change_the_allocation_record_tree_leaves(&mut changed, |leaf| {
            set_the_span_of_the_allocation_record_of(
                leaf,
                data_unit,
                SPAN_SLOTS_ONE_PAST_THE_DATA_UNIT,
            )
        }),
        2,
        "数据单元那一个落点，两块盘的叶各一条"
    );
    assert_every_device_holds_only_zeros_at(
        &changed,
        SlotNumber(data_unit.0 + u64::from(SPAN_SLOTS_ONE_PAST_THE_DATA_UNIT) - 1),
        "改宽的那条多罩的那一槽",
    );
    let Some(cpu_verdicts_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME,
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            &clean,
            &changed,
        )
    else {
        return;
    };
    let allocated_statistic = cpu_verdicts_on_the_changed_image
        .iter()
        .find(|(invariant, _)| *invariant == ALLOCATED_STATISTIC_INVARIANT)
        .map(|(_, verdict)| verdict);
    match allocated_statistic {
        Some(InvariantVerdict::PartiallyJudged(unjudged)) => assert!(
            unjudged.contains("（I-3.18 判它）"),
            "改坏的那一张上 I-3.1 部分没判的说明点名 I-3.18：{unjudged}"
        ),
        other_verdict @ (None
        | Some(
            InvariantVerdict::Holds
            | InvariantVerdict::Violated(_)
            | InvariantVerdict::NotApplicable(_),
        )) => panic!(
            "改记成两槽之后整份成立：CPU 的 I-3.1 该报部分没判（两边逐位相同，GPU 那一位也既不判到也不违例），实际 {other_verdict:?}"
        ),
    }
}

// ───────────────────────── I-3.1 取宽那一支第五判与 I-3.21（判决 reason-r2 第二节 Q1「改法」、Q2「归属」） ─────────────────────────

/// 改宽之后一槽单元那条记录的跨度：罩它自己与紧跟其后的那一槽。
const SPAN_SLOTS_OVER_THE_NEXT_SLOT: u16 = 2;

/// I-3.1 取宽那一支第五判（判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节 Q1「改法」那一行），辩方承接腿
/// 「窄记录加尾槽上另一条记录」那一形（同 `narrowed_record_with_another_record_on_its_tail_slot_reddens_the_allocated_statistic_and_the_span_invariants`）：
/// 数据单元那两条未释放记录只记一槽、尾槽上各插一条未释放、跨 1 的记录、树表那条一槽单元的记录改宽成 2（罩到盘上全零的那一槽），记账不动。
/// 改记之后三判与逐槽两判都成立，而数据单元那条改记之后与尾槽上那条相交：两边都判 I-3.1 红、I-3.18 红，别的都不红。
/// GPU 扫描内核取宽那一支不判相交时那一位报部分没判、与 CPU 逐位不同。
#[test]
fn the_gpu_judge_reddens_the_allocated_statistic_under_a_narrowed_record_with_another_record_on_its_tail_slot_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_allocated_statistic_under_a_narrowed_record_with_another_record_on_its_tail_slot_like_the_cpu_checker";
    let clean = build_pool("gpu-judge-narrowed-record-with-a-record-on-its-tail").memory_pool();
    let data_unit = slot_of_the_data_unit(&clean);
    let tree_table = the_slot_of(&newest_root(&clean).tree_table);
    let mut changed = clean.clone();
    assert_eq!(
        narrow_the_records_of_the_data_units_to_one_slot(&mut changed),
        2,
        "数据单元那一个落点，两块盘的叶各一条"
    );
    assert_every_device_holds_only_zeros_at(
        &changed,
        SlotNumber(tree_table.0 + 1),
        "树表后面那一槽",
    );
    assert_eq!(
        change_the_allocation_record_tree_leaves(&mut changed, |leaf| {
            insert_an_unreleased_one_slot_allocation_record_after(
                leaf,
                data_unit,
                SlotNumber(data_unit.0 + 1),
            ) + set_the_span_of_the_allocation_record_of(
                leaf,
                tree_table,
                SPAN_SLOTS_OVER_THE_NEXT_SLOT,
            )
        }),
        4,
        "尾槽上那条两盘各插一条、树表那条两盘各改宽一条"
    );
    let Some(cpu_verdicts_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME,
            ALLOCATED_STATISTIC_INVARIANT,
            &clean,
            &changed,
        )
    else {
        return;
    };
    assert_eq!(
        violated_invariants_of(&cpu_verdicts_on_the_changed_image),
        BTreeSet::from([
            ALLOCATED_STATISTIC_INVARIANT,
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
        ]),
        "改坏的那一张两边都只红 I-3.1 与 I-3.18：{cpu_verdicts_on_the_changed_image:?}"
    );
}

/// I-3.21（走到的引用起点是最新根计入的分配记录的起点）、C944（走到的单元起点没有自己的记录而被别的记录罩着）：同 harness
/// `record_widened_exactly_over_a_walked_one_slot_neighbor_without_its_own_record_reddens_only_the_span_and_the_walked_reference_start_invariants`——
/// 映射树根那条一槽单元的记录跨度改成 2（恰好罩住紧跟其后的树表）、树表那条记录拿掉，记账不动。干净的那一张 I-3.21 判到而成立，
/// 改坏的那一张两边都只红 I-3.18 与 I-3.21。GPU 扫描内核不判 I-3.21 时改坏的那一张上少一位违例、与 CPU 逐位不同。
#[test]
fn the_gpu_judge_reddens_a_walked_reference_start_without_its_own_allocation_record_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_a_walked_reference_start_without_its_own_allocation_record_like_the_cpu_checker";
    let clean = build_pool("gpu-judge-record-widened-exactly-over-a-neighbor").memory_pool();
    let newest = newest_root(&clean);
    let mapping_root = the_slot_of(&newest.mapping_root);
    let tree_table = the_slot_of(&newest.tree_table);
    assert_eq!(tree_table.0, mapping_root.0 + 1, "树表紧跟在映射树根后面");
    let mut changed = clean.clone();
    assert_eq!(
        change_the_allocation_record_tree_leaves(&mut changed, |leaf| {
            set_the_span_of_the_allocation_record_of(
                leaf,
                mapping_root,
                SPAN_SLOTS_OVER_THE_NEXT_SLOT,
            ) + remove_the_last_allocation_record_of(leaf, tree_table)
        }),
        4,
        "映射树根那条记录改宽、树表那条记录拿掉，两盘各一条"
    );
    let Some(cpu_verdicts_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME,
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
            &clean,
            &changed,
        )
    else {
        return;
    };
    assert_eq!(
        violated_invariants_of(&cpu_verdicts_on_the_changed_image),
        BTreeSet::from([
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
        ]),
        "改坏的那一张两边都只红 I-3.18 与 I-3.21：{cpu_verdicts_on_the_changed_image:?}"
    );
}

// ───────────────────────── 全零树表指针、根记录持有的分配记录树、全零根槽并进抬 F 上限 ─────────────────────────

/// 最新那条根所在的根槽（盘, 盘上偏移）：照写者的落点式（区域 txg mod R、槽 (txg div R) mod S），区域在哪块盘按池参数认。
fn root_slot_of_the_newest_root(image: &MemoryPool) -> (DeviceIdentity, DeviceOffsetInBytes) {
    let make_filesystem_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        newest_root(image).checkpoint_txg,
        make_filesystem_parameters
            .geometry
            .root_ring_slots_per_region,
    );
    let device = make_filesystem_parameters.region_devices
        [usize::try_from(target.region).expect("区域号装得进 usize")];
    let offset = singlefs_core::root_ring::slot_offset(
        target,
        make_filesystem_parameters
            .geometry
            .fixed_structure_slot_spacing,
    );
    (device, offset)
}

/// 写完新池新建文件那一张上，最新那条根（实例 1、txg 3）的树表指针整条写零、重封根槽，每条根照抄进它那几条记录的新根段
/// （同 `checker_known_bad_images.rs` 的 `image_with_the_tree_table_pointer_of_a_root_written_all_zero`）。
fn image_whose_newest_root_carries_an_all_zero_tree_table_pointer() -> MemoryPool {
    let mut image =
        build_pool("gpu-judge-all-zero-tree-table-pointer-in-the-newest-root").memory_pool();
    let mut newest = newest_root(&image);
    newest.tree_table = singlefs_core::pointer::NodePointer::empty_root();
    let (device, offset) = root_slot_of_the_newest_root(&image);
    let root_slot_bytes =
        PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters().root_slot_bytes();
    image
        .devices
        .get_mut(&device)
        .expect("池里有这块盘")
        .write(offset, &newest.to_slot(root_slot_bytes));
    copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut image);
    image
}

/// 最新根的树表指针全零（core 照它读盘 0 槽 0、读不出，与树表读不出同一处置）：CPU 的 I-3.21 报部分没判、
/// 不落进「树表 0 条」那一支报不适用；GPU 扫描内核那一支记射程里判不了（`c_newest_tree_table_pointer_is_all_zero`）。两边逐位相同、I-3.21 都没判到。
#[test]
fn the_gpu_judge_leaves_the_walked_reference_starts_unjudged_under_an_all_zero_tree_table_pointer_of_the_newest_root_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_leaves_the_walked_reference_starts_unjudged_under_an_all_zero_tree_table_pointer_of_the_newest_root_like_the_cpu_checker";
    let image = image_whose_newest_root_carries_an_all_zero_tree_table_pointer();
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(both) = judge_the_image_on_both_judges(cards, TEST_NAME, &image) else {
        return;
    };
    assert!(
        both.gpu_bits == both.cpu_bits,
        "GPU 判器与 CPU 池级 checker 逐位不同——违例位不同的 {:?}，判到位不同的 {:?}",
        invariant_names(both.gpu_bits.0 ^ both.cpu_bits.0),
        invariant_names(both.gpu_bits.1 ^ both.cpu_bits.1),
    );
    let walked_reference_starts = both
        .cpu_verdicts
        .iter()
        .find(|(invariant, _)| {
            *invariant
                == WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
        })
        .map(|(_, verdict)| verdict.clone());
    assert!(
        matches!(
            &walked_reference_starts,
            Some(InvariantVerdict::PartiallyJudged(unjudged)) if unjudged.contains("树表指针全零")
        ),
        "最新根的树表指针全零：CPU 的 I-3.21 报部分没判、点名树表指针全零，实际 {walked_reference_starts:?}"
    );
}

/// 两块内存盘上 mkfs → 可写挂载（取号 1、零单元发布 txg 1、暖机 txg 2）→ 进程退出 → 再可写挂载（取号 2、写行：写行那次发布 txg 3 重写实例表、
/// 写一棵分配记录树、根指针住根记录；暖机 txg 4）。同 `checker_narrow_invariants_and_abandoned_roots.rs` 的
/// `image_after_the_row_publish_of_a_version_without_file`。
fn image_after_the_row_publish_of_a_version_without_file() -> MemoryPool {
    let stream = singlefs_verification::SharedStream::new();
    let mut devices: Vec<(
        DeviceIdentity,
        singlefs_verification::RecordingBlockDevice<
            singlefs_verification::memory_pool::SparseBlockDevice,
        >,
    )> = [0u32, 1]
        .into_iter()
        .map(|device| {
            (
                DeviceIdentity(device),
                singlefs_verification::RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device),
                    singlefs_verification::memory_pool::SparseBlockDevice::new(
                        IMAGE_BYTES,
                        singlefs_core::block_device::PhysicalBlockSizeInBytes(512),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect();
    singlefs_core::make_filesystem::make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let first_mount =
        singlefs_core::mount::mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
    drop(first_mount);
    let image_after_the_first_mount = memory_pool_of_sparse_devices(&devices);
    let mut reopened = crash_state_devices(&image_after_the_first_mount, &[], &[], &stream);
    let second_mount =
        singlefs_core::mount::mount_writable(&parameters(), &mut reopened).expect("第二次可写挂载");
    assert_eq!(
        second_mount.output.rows_written.len(),
        1,
        "上一个实例 1 那一行"
    );
    drop(second_mount);
    memory_pool_of_sparse_devices(&reopened)
}

/// 最新根的根记录直接持有的那棵分配记录树（根记录偏移 342；树表 0 条、写过行的那一版才非零）走得到的每片叶的槽。
fn leaves_of_the_allocation_record_tree_the_newest_root_record_holds(
    image: &MemoryPool,
) -> Vec<SlotNumber> {
    allocation_record_tree_nodes_as_far_as_readable(
        image,
        &newest_root(image).allocation_record_tree_root,
    )
    .iter()
    .map(the_slot_of)
    .filter(|slot| {
        parse_index_node(&unit_bytes_at(image, *slot, node_bytes()))
            .expect("分配记录树走得到的节点解得开")
            .level
            == 0
    })
    .collect()
}

/// 叶里满足 `removed` 的记录整条拿掉：留下的往前挪、末尾空出的清零、条目数与声明长度跟着改。交回拿掉了几条。
fn remove_the_allocation_records_that(leaf: &mut [u8], removed: impl Fn(&[u8]) -> bool) -> usize {
    let count = allocation_record_count_of_the_leaf(leaf);
    let kept: Vec<Vec<u8>> = (0..count)
        .map(|index| leaf[allocation_record_range_in_the_leaf(index)].to_vec())
        .filter(|record| !removed(record))
        .collect();
    let removed_count = count - kept.len();
    if removed_count == 0 {
        return 0;
    }
    for (index, record) in kept.iter().enumerate() {
        leaf[allocation_record_range_in_the_leaf(index)].copy_from_slice(record);
    }
    let end_of_the_records = allocation_record_range_in_the_leaf(count).start;
    let end_of_the_kept_records = allocation_record_range_in_the_leaf(kept.len()).start;
    leaf[end_of_the_kept_records..end_of_the_records].fill(0);
    set_the_allocation_record_count_of_the_leaf(leaf, kept.len());
    removed_count
}

/// 分配记录跨度段的跨度值那几位（去掉已释放标志与隔离记号）。
const ALLOCATION_RECORD_SPAN_VALUE_BITS: u16 = 0x3FFF;

/// C963（树表 0 条写过行的版上写行发布漏记新单元）候选一：写行之后那一张（树表 0 条、写过行的那一版）
/// 上，根记录直接持有的那棵分配记录树里写行那次发布新写的那片实例表的记录（未释放、跨 2 槽：这一版上只有它是这一形）两块盘各拿掉一条。
/// 两边都照根记录持有的那一棵判 I-3.21：干净的那一张判到而成立，改坏的那一张判红、只红它（harness
/// `checker_narrow_invariants_and_abandoned_roots.rs` 那一份钉 CPU 只红这一条）。GPU 只读树表里种类 3 的那几棵时这一版报不适用、与 CPU 对不上。
#[test]
fn the_gpu_judge_reddens_a_walked_reference_start_missing_from_the_allocation_record_tree_the_root_record_holds_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_a_walked_reference_start_missing_from_the_allocation_record_tree_the_root_record_holds_like_the_cpu_checker";
    let clean = image_after_the_row_publish_of_a_version_without_file();
    let mut changed = clean.clone();
    let leaves = leaves_of_the_allocation_record_tree_the_newest_root_record_holds(&changed);
    assert_eq!(
        change_these_allocation_record_tree_leaves(&mut changed, leaves, |leaf| {
            remove_the_allocation_records_that(leaf, |record| {
                let span_field = u16::from_le_bytes([
                    record[ALLOCATION_RECORD_SPAN_FIELD.start],
                    record[ALLOCATION_RECORD_SPAN_FIELD.start + 1],
                ]);
                span_field & ALLOCATION_RECORD_RELEASED_FLAG == 0
                    && span_field & ALLOCATION_RECORD_SPAN_VALUE_BITS == SLOTS_OF_A_DATA_UNIT
            })
        }),
        2,
        "新写的那片实例表，两块盘的叶各一条"
    );
    let Some(cpu_verdicts_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME,
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
            &clean,
            &changed,
        )
    else {
        return;
    };
    assert_eq!(
        violated_invariants_of(&cpu_verdicts_on_the_changed_image),
        BTreeSet::from([
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
        ]),
        "改坏的那一张两边都只红 I-3.21：{cpu_verdicts_on_the_changed_image:?}"
    );
}

/// C961（全零根槽没并进 checker 的抬 F 上限）：攻方第二样本模型 `first_example` 那一段历史（S = 4，txg 3 那一槽读回全零，
/// `common_root_ring_holes::pool_raising_the_floor_over_a_ring_slot_reading_back_all_zero`）上两张：照 D16（发布语义） 已定项 1 字面并进全零槽、
/// 抬到 3 的合法写者那一张两边都判 I-7.9 成立；不并全零槽、抬到 4 的坏写者那一张两边都判 I-7.9 红、只红它
/// （`checker_known_bad_images.rs` 那两份钉 CPU）。GPU 扫描内核不并读回全零的槽时坏写者那一张判成立、与 CPU 对不上。
#[test]
fn the_gpu_judge_reddens_a_floor_raised_past_a_ring_slot_reading_back_all_zero_like_the_cpu_checker(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_reddens_a_floor_raised_past_a_ring_slot_reading_back_all_zero_like_the_cpu_checker";
    let legal = common_root_ring_holes::pool_raising_the_floor_over_a_ring_slot_reading_back_all_zero(
        common_root_ring_holes::FloorRaiseOverTheRingSlotReadingBackAllZero::ToTheCeilingWithTheAllZeroSlotMerged,
    );
    let past_the_all_zero_slot =
        common_root_ring_holes::pool_raising_the_floor_over_a_ring_slot_reading_back_all_zero(
            common_root_ring_holes::FloorRaiseOverTheRingSlotReadingBackAllZero::PastTheAllZeroSlotToTheCeilingWithoutIt,
        );
    let Some(cpu_verdicts_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME,
            ROLLBACK_FLOOR_NOT_ABOVE_THE_RAISE_CEILING,
            &legal,
            &past_the_all_zero_slot,
        )
    else {
        return;
    };
    assert_eq!(
        violated_invariants_of(&cpu_verdicts_on_the_changed_image),
        BTreeSet::from([ROLLBACK_FLOOR_NOT_ABOVE_THE_RAISE_CEILING]),
        "坏写者那一张两边都只红 I-7.9：{cpu_verdicts_on_the_changed_image:?}"
    );
}

/// 回退下界 F 不高于抬 F 的上限。
const ROLLBACK_FLOOR_NOT_ABOVE_THE_RAISE_CEILING: &str = "I-7.9";

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
