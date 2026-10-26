//! checker 档模块：crash、crash_identity、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器里池级 checker 那一份照池级 checker 审计复核的两项（`records/2026-10-24-singlefs-checker-审计复核.md` 的 C01、C02）同改之后，
//! 与 CPU 池级 checker（`crates/singlefs-checker/src/walk.rs` 的 `check_pool_image`）逐条不变量的违例位与判到位相同：
//! C01：别的版本先走过的单元被这一版又引用时，按这一条引用定的那几格照判（走读内核 `w_judge_the_cells_the_reference_decides_on_a_unit_walked_earlier`）——
//! 暖机写的第一条根（实例 1、txg 1，非最新候选根）改一条指针，让它指着别的根先走过的单元：树表指针的出生 txg（I-1.2 只红它）、
//! 映射根指针改指最新根的树表而出生树加一（I-1.3）、实例表指针改指最新根先走过的数据单元（I-1.6 类那一半）；
//! C02：逐盘 journal 记录表按 (计数器, 实例代号) 收、同键按扫环次序先到先留（走读内核 `checker_insert_record`），I-8.6 按本实例找逻辑前一条——
//! 第 1 批那四种排列（同一块盘同一键两条、同一计数器两个实例各一条，各两种次序），再加一张同一计数器两个实例、其中一个实例的下一条
//! 反向链接着本实例那一条的（I-8.6 要按本实例找才判得成立；只按计数器建键或只按计数器找时 GPU 判红、CPU 不红）。
//! 每张改坏的镜像都有干净的那一张作对照（mkfs → 暖机 → 新池新建文件，零违例）。
//! 之后的欠账那一批（`records/2026-10-26-checker欠账C896-C897-C914.md`，`.claude/kb/checks-owed.md` 的 C896、C897）另加三组：
//! C896：别的版本先走过的码 2 节点被这一版的父条目再引用时，照判这一条父条目定的层级、key 区间与子树覆盖（I-1.1；走读内核
//! `w_index_node_an_earlier_walk_parsed` 与它的四处调用方）——中央映射树压小容量之后两版共享的叶、旧那一版的父节点把它的分隔 key 改大
//! （多层码 2 树的 ②），与分配记录树两版共享的叶、旧那一版的第 1 层节点把它的条目 key 改指邻格（按位置寻址的树）；
//! C897：I-8.13（自证过的 journal 记录落在它计数器该落的环槽，走读内核 `checker_scan_journal_of_device`）——上面 C02 那五张冲突布局
//! 都有放错槽的记录、判红，干净的池判到而成立；读者收录口径：同一块盘同一键先到的那一条恢复的读者读不出（序号 0）、后到的读得出，
//! 两种次序（走读内核恢复视图与重放读的那一份环只收读得出的，`journal_record_reads_as_the_recovery_reader_reads_it`）。
//! 这三组钉的是池级 checker 那一侧同一批的改法（实现员甲的补丁）：池级 checker 没跟着改时 CPU 不红、GPU 红，逐位比照样红。
//! 欠账第 2 批另加六组：inode 树根命中别的版本走过的叶容器时照判这一版父条目的分隔 key（I-9.12，走读内核
//! `w_note_the_separator_of_an_inode_child_over_the_leaves_from`）；extent 树上段与墓碑树命中别的版本走过的节点时照判这一版父条目定的位置与子树覆盖
//! （I-1.1）；点名项 flags 字节非 0 的记录恢复视图与重放读的那一份环照恢复的读者收、判不变量那一份表不收（走读内核
//! `checker_record_valid_past_the_named_entry_flags`），同盘同键两种次序；恢复视图只收读得出的那一滤（由记录施加出来的那一版的记录标志位 1 置位）；
//! 重放读的那一份环不收新根段指针头部不是第一版取值的记录（树表指针 MAC 非 0）。inode 分隔 key 与点名项 flags 那两组钉的是池级 checker
//! 同一批（实现员丙的补丁）的改法，丙的补丁没打时 CPU 那一侧判法不同、逐位比红。
//! 坏镜像照 harness 档 `crates/singlefs-harness/tests/checker_cached_reference_and_journal_conflicts.rs` 的坏法造（那一份钉 CPU 池级 checker、
//! 这一份钉 GPU 判器）；共用的搭建经 `#[path]` 指回 harness 档的 `tests/common/mod.rs`，改字节的那几步就地写（harness 档不依赖 checker 档）。
//! 每张镜像两边的输入都经对拍那一类的库：GPU 那一侧把镜像当基线、一个空持久集合建判表，判表入库再读回上卡；CPU 那一侧把镜像作一张
//! 给定状态表写进库、读回，池级 checker 判读回的那一份；两边判出的东西作一份对拍结论入库，写法照
//! `tests/crash_judge_gpu_writer_identity_allocation_record_spans_and_new_root_segments_of_every_record_like_the_cpu_checker.rs`。
//! GPU 判不了判红。要本机一张 NVIDIA 独显：一张适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。
//! 这几条是 GPU 判器（加速路径）与 CPU 池级 checker（参照路径）的对拍，照 `.claude/rules/test-script-discipline.md`「分快慢两档」一律进快档、
//! 不看快慢判据，都不标 `#[ignore]`。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_tree_split/mod.rs"]
mod common_tree_split;

use common::{
    build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records,
    crash_state_devices, geometry, memory_pool_of_sparse_devices,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    publish_overwrite_in_process, tear_the_root_slot_written_for_txg, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use common_tree_split::{
    capacities, propagate_the_new_checksum, read_unit_on_device, reseal_unit_by_its_class,
    write_unit_to_the_same_slot_on_both_devices, TreeSplitPool, ACCOUNTING_OF_THE_NODE_FORMAT,
};
use singlefs_core::allocation_record_tree::{
    allocation_record_key_bytes, AllocationRecordTreeNode, AllocationRecordTreeNodePosition,
};
use singlefs_core::bytes::ByteReader;
use singlefs_core::pointer::NodePointer;

use singlefs_checker::check_journal_record;
use singlefs_checker::image::{
    chosen_system_configurations, root_slot_positions, ImageReader, InvariantVerdict,
    EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
    IMPLEMENTED_INVARIANTS, NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
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
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InodeNumber,
    InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
    ContainerTreeNodePosition,
};
use singlefs_core::extent_tree::{extent_key_bytes, ExtentUpperNodePosition};
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JOURNAL_HEADER_CHECKSUM_OFFSET, JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET,
    JOURNAL_RECORD_FLAGS_OFFSET,
};
use singlefs_core::lineage_tree::{
    LineageContainerCapacity, LineageRecordsPerContainerCappedForTests,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::choose_system_configuration;
use singlefs_core::retirement_tree::RetirementContainerCapacity;
use singlefs_core::root_record::RootRecord;
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite_of_an_inode, publish_sequential_write_of_an_inode,
    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty,
    CodeTwoTreeNodeCapacities, PoolWriter, PublishedUnit, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{
    data_unit_payload_capacity, parse_index_node, unit_filesystem_identifier,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_format::{
    ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, ALLOCATION_RECORD_TREE_LEAF_SLOTS,
    EXTENT_TREE_UPPER_LEAF_INODES, INODE_LEAF_RECORDS, JOURNAL_HEADER_BYTES,
    JOURNAL_NAMED_ENTRY_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, NODE_BYTES,
    SLOT_BYTES,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite, WrittenContents};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::sha256_digest;
use singlefs_harness::{RetainedOperation, SharedStream};

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
            "gpu judge cached references and journal conflicts pairing inputs\nbase={}\ntables={}\n",
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
/// 每趟打一行 `GPU_CACHED_REFERENCE_AND_JOURNAL_CONFLICT_PAIRING_CONCLUSION table=<表名> source=<ReusedFromTheLibrary|JudgedNow>`。
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
        "GPU_CACHED_REFERENCE_AND_JOURNAL_CONFLICT_PAIRING_CONCLUSION table={table_name} source={source:?}"
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

/// 这一张镜像上 `invariant` 该判成什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedVerdictOfTheInvariant {
    JudgedAndHolds,
    Violated,
    /// 判到位与违例位都没置（池级 checker 报不适用）。
    NotJudged,
}

/// 两边逐位相同（违例位与判到位整份比，每一条不变量），再核 `invariant` 在 CPU 那一侧判成 `expected`（两边逐位相同，GPU 也是这样）。
/// 交回 CPU 判红的那几条。
fn assert_the_gpu_agrees_with_the_cpu_and_the_invariant_is_judged_as_expected(
    table_name: &str,
    both: &BothJudgesOnAnImage,
    invariant: &str,
    expected: ExpectedVerdictOfTheInvariant,
) -> Vec<&'static str> {
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
        (true, _) => ExpectedVerdictOfTheInvariant::Violated,
        (false, true) => ExpectedVerdictOfTheInvariant::JudgedAndHolds,
        (false, false) => ExpectedVerdictOfTheInvariant::NotJudged,
    };
    assert_eq!(
        observed,
        expected,
        "{table_name}：{invariant} 要判成 {expected:?}，CPU 判的是 {verdict_on_the_cpu:?}（两边逐位相同，GPU 也是这样）"
    );
    invariant_names(cpu_violated)
}

/// 一条用例：借卡，干净的那一张与改坏的那一张各两边判一次、逐位比，干净的那一张 `invariant` 判到而成立、改坏的那一张判红。
/// 交回改坏的那一张 CPU 判红的那几条；借不到卡、只录那一趟时交 `None`（「本次未跑」那一行已打）。
fn judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
    test_name: &str,
    invariant: &str,
    clean: &MemoryPool,
    changed: &MemoryPool,
) -> Option<Vec<&'static str>> {
    let cards = gpu_card_pool_or_report_not_run(test_name)?;
    let clean_name = format!("{test_name}/clean");
    let both_on_the_clean_image = judge_the_image_on_both_judges(cards, &clean_name, clean)?;
    let red_on_the_clean_image =
        assert_the_gpu_agrees_with_the_cpu_and_the_invariant_is_judged_as_expected(
            &clean_name,
            &both_on_the_clean_image,
            invariant,
            ExpectedVerdictOfTheInvariant::JudgedAndHolds,
        );
    assert_eq!(
        red_on_the_clean_image,
        Vec::<&str>::new(),
        "{clean_name}：干净的那一张（没改坏的同一段历史）零违例"
    );
    let changed_name = format!("{test_name}/changed");
    let both_on_the_changed_image = judge_the_image_on_both_judges(cards, &changed_name, changed)?;
    Some(
        assert_the_gpu_agrees_with_the_cpu_and_the_invariant_is_judged_as_expected(
            &changed_name,
            &both_on_the_changed_image,
            invariant,
            ExpectedVerdictOfTheInvariant::Violated,
        ),
    )
}

/// 一条用例：借卡，一张镜像两边判一次、逐位比，`expectations` 里每一条不变量判成它后面那一种。交回 CPU 判红的那几条；
/// 借不到卡、只录那一趟时交 `None`（「本次未跑」那一行已打）。
fn judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(
    test_name: &str,
    image: &MemoryPool,
    expectations: &[(&str, ExpectedVerdictOfTheInvariant)],
) -> Option<Vec<&'static str>> {
    let cards = gpu_card_pool_or_report_not_run(test_name)?;
    let both = judge_the_image_on_both_judges(cards, test_name, image)?;
    let mut red_on_the_image = Vec::new();
    for (invariant, expected) in expectations {
        red_on_the_image =
            assert_the_gpu_agrees_with_the_cpu_and_the_invariant_is_judged_as_expected(
                test_name, &both, invariant, *expected,
            );
    }
    Some(red_on_the_image)
}

// ───────────────────────── C01：非最新候选根指着别的根先走过的单元 ─────────────────────────

/// 非最新候选根里那一条：暖机写的第一条根（实例 1、txg 1）。暖机那几版没有文件，它的树表与映射根与别的候选根共享。
const OLDER_CANDIDATE_ROOT_TXG: u64 = 1;

/// 改坏之后树表指针记的出生 txg：合法历史上任何单元都不在这一代出生。
const BIRTH_TXG_NO_UNIT_CARRIES: u64 = 999;

/// 根环里读得出的每一条根：（盘，槽在盘上的字节偏移，根记录）。
struct ReadableRootsOfAnImage {
    root_slot_bytes: usize,
    roots: Vec<(u32, u64, RootRecord)>,
}

fn readable_roots_of(image: &MemoryPool) -> ReadableRootsOfAnImage {
    let geometry = chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
        .expect("系统配置择得出");
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    let roots = root_slot_positions(&geometry)
        .into_iter()
        .filter_map(|(_, _, device, offset)| {
            let bytes = ImageReader::read(image, device, offset, root_slot_bytes)?;
            RootRecord::parse_slot(&bytes, &geometry.filesystem_identifier)
                .map(|root| (device, offset, root))
        })
        .collect();
    ReadableRootsOfAnImage {
        root_slot_bytes,
        roots,
    }
}

/// 根环里实例 1、txg 1 那一条（恰一条）：（盘，偏移，根记录）。
fn the_older_candidate_root(readable: &ReadableRootsOfAnImage) -> (u32, u64, RootRecord) {
    let older_candidate_roots: Vec<&(u32, u64, RootRecord)> = readable
        .roots
        .iter()
        .filter(|(_, _, root)| {
            root.instance == InstanceGeneration(1)
                && root.checkpoint_txg == CheckpointTxg(OLDER_CANDIDATE_ROOT_TXG)
        })
        .collect();
    assert_eq!(
        older_candidate_roots.len(),
        1,
        "根环里恰有一条实例 1、txg 1 的根"
    );
    *older_candidate_roots[0]
}

/// 干净的镜像上把实例 1、txg 1 那一条根换成 `changed_root`、重封根槽，journal 里同代记录的新根段与反向链跟着重封
/// （`copy_each_root_into_the_new_root_segment_of_its_journal_records`，I-8.10 不因此红）。
fn image_with_the_older_candidate_root_replaced(
    clean: &MemoryPool,
    device: u32,
    offset: u64,
    root_slot_bytes: usize,
    changed_root: &RootRecord,
) -> MemoryPool {
    let mut image = clean.clone();
    image.apply_writes(&[RetainedWrite {
        device: DeviceIdentity(device),
        kind: StepKind::RootRecordFua,
        is_force_unit_access: true,
        offset: DeviceOffsetInBytes(offset),
        contents: WrittenContents::Bytes(changed_root.to_slot(root_slot_bytes)),
    }]);
    copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut image);
    image
}

/// C01 的 I-1.2 那一形（harness 档同名坏镜像那一形）：实例 1、txg 1 那一条根的树表指针把出生 txg 改成 999。那张树表别的候选根也指着、先被走过：
/// 这一条根走到它时命中走过的单元、不往下展开，这一条指针的出生身份照判，两边都只红 I-1.2。
/// 判别力：走读内核命中时不照判这一条引用定的那几格（改前的 `w_read_index_node`），GPU 不红 I-1.2、CPU 红，违例位不同。
#[test]
fn the_gpu_judge_reddens_only_the_birth_identity_invariant_on_an_older_candidate_root_whose_tree_table_pointer_names_another_birth_txg_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_only_the_birth_identity_invariant_on_an_older_candidate_root_whose_tree_table_pointer_names_another_birth_txg_like_the_cpu_checker";
    let built = build_pool("gpu-cached-reference-tree-table-birth-txg");
    let clean = built.memory_pool();
    let readable = readable_roots_of(&clean);
    let (device, offset, older_root) = the_older_candidate_root(&readable);
    assert!(
        readable.roots.iter().any(
            |(_, _, root)| root.checkpoint_txg != older_root.checkpoint_txg
                && root.tree_table == older_root.tree_table
        ),
        "别的根指着同一张树表（同一条树表指针）：这一条根走到它时它已被走过"
    );
    assert_ne!(
        older_root.tree_table.head.birth_txg,
        CheckpointTxg(BIRTH_TXG_NO_UNIT_CARRIES),
        "改之前树表指针的出生 txg 不是 999"
    );
    let mut changed_root = older_root;
    changed_root.tree_table.head.birth_txg = CheckpointTxg(BIRTH_TXG_NO_UNIT_CARRIES);
    let changed = image_with_the_older_candidate_root_replaced(
        &clean,
        device,
        offset,
        readable.root_slot_bytes,
        &changed_root,
    );
    let Some(red_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME, "I-1.2", &clean, &changed,
        )
    else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        vec!["I-1.2"],
        "{TEST_NAME}：两边都只红 I-1.2（树表指针的出生身份与共享树表单元头不符）"
    );
}

/// 根环里最新的那一条根（最大的 (checkpoint_txg, 实例代号)）：新池新建文件那一版，走读先走它。
fn the_newest_root(readable: &ReadableRootsOfAnImage) -> RootRecord {
    readable
        .roots
        .iter()
        .max_by_key(|(_, _, root)| (root.checkpoint_txg, root.instance))
        .expect("根环里有读得出的根")
        .2
}

/// C01 的 I-1.3 那一形：实例 1、txg 1 那一条根的映射根指针（暖机那几版是空指针）换成最新根的树表指针、出生树加一
/// （中央映射树「实际引用它的树」按映射根指针头部的出生树取：walk.rs `walk_tree_table_and_central_mapping_root`、走读内核 `w_walk_tree_table_and_mapping`）。
/// 最新根先走过那张树表：这一条根走映射根时命中，头里的树 ID 不是这条指针要的树，两边都红 I-1.3（另红 I-4.9：那张树表生在最新那一版，
/// 并进这一版时照判）。出生身份照抄（I-1.2 成立）。
/// 判别力：走读内核命中时不照判，GPU 不红 I-1.3、CPU 红。
#[test]
fn the_gpu_judge_reddens_the_tree_identifier_invariant_on_an_older_candidate_root_whose_mapping_root_pointer_names_the_newest_tree_table_under_another_birth_tree_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_tree_identifier_invariant_on_an_older_candidate_root_whose_mapping_root_pointer_names_the_newest_tree_table_under_another_birth_tree_like_the_cpu_checker";
    let built = build_pool("gpu-cached-reference-mapping-root-names-the-tree-table");
    let clean = built.memory_pool();
    let readable = readable_roots_of(&clean);
    let (device, offset, older_root) = the_older_candidate_root(&readable);
    let newest_root = the_newest_root(&readable);
    assert_ne!(
        newest_root.tree_table, older_root.tree_table,
        "最新根的树表是新池新建文件那一版写的，不是暖机那几版共享的那一张"
    );
    let mut changed_root = older_root;
    changed_root.mapping_root = newest_root.tree_table;
    changed_root.mapping_root.head.birth_tree =
        TreeIdentifier(newest_root.tree_table.head.birth_tree.0 + 1);
    let changed = image_with_the_older_candidate_root_replaced(
        &clean,
        device,
        offset,
        readable.root_slot_bytes,
        &changed_root,
    );
    let Some(red_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME, "I-1.3", &clean, &changed,
        )
    else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        vec!["I-1.3", "I-4.9"],
        "{TEST_NAME}：两边都只红 I-1.3（命中时树 ID 不是映射根指针要的树）与 I-4.9（并进这一版的树表生在后面的版本）"
    );
}

/// C01 的 I-1.6 那一形（harness 档 `checker_narrow_invariants_and_abandoned_roots.rs` 里实例表指针改指数据单元那一形的同类）：
/// 实例 1、txg 1 那一条根的实例表指针两条位置条目改指第一个文件的数据单元（槽号与整单元校验和照那个单元，读得出）。最新根先走过那个数据单元：
/// 这一条根走实例表链第一片时命中，这一条引用要码 3、头里是码 1，两边都红 I-1.6；另红两条是改实例表指针连带的：I-4.9（并进这一版的数据单元
/// 生在后面的版本）、I-3.5（这一版不再引用暖机那张实例表，后一版里它像是新出现的）。
/// 判别力：走读内核命中时不照判，GPU 不红 I-1.6、CPU 红。
#[test]
fn the_gpu_judge_reddens_the_class_label_invariant_on_an_older_candidate_root_whose_instance_table_pointer_names_a_data_unit_the_newest_root_walked_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_class_label_invariant_on_an_older_candidate_root_whose_instance_table_pointer_names_a_data_unit_the_newest_root_walked_like_the_cpu_checker";
    let built = build_pool("gpu-cached-reference-instance-table-names-a-data-unit");
    let clean = built.memory_pool();
    let readable = readable_roots_of(&clean);
    let (device, offset, older_root) = the_older_candidate_root(&readable);
    let data_unit = built
        .output
        .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    let data_unit_checksum = crc32_castagnoli(&data_unit.bytes);
    let mut changed_root = older_root;
    for location in &mut changed_root.instance_table.locations {
        location.slot = data_unit.slot;
        location.unit_checksum = data_unit_checksum;
    }
    let changed = image_with_the_older_candidate_root_replaced(
        &clean,
        device,
        offset,
        readable.root_slot_bytes,
        &changed_root,
    );
    let Some(red_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME, "I-1.6", &clean, &changed,
        )
    else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        vec!["I-1.6", "I-4.9", "I-3.5"],
        "{TEST_NAME}：两边都只红 I-1.6（命中时类标签不是实例表指针要的码 3）与改实例表指针连带的 I-4.9、I-3.5"
    );
}

// ───────────────────────── C02：同一块盘上撞键与同一计数器两个实例 ─────────────────────────

/// I-8.13（自证过的 journal 记录落在它计数器该落的环槽，C897）在池级 checker 清单里的名字。
const JOURNAL_RECORD_RING_SLOT_INVARIANT: &str = "I-8.13";

/// 种进环里的一条记录的反向链。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackChainOfAPlantedRecord {
    /// 照合法镜像里第一条记录（计数器 1）原样：0。
    AsTheFirstRecordOfThePoolCarries,
    /// 前面种进的第 `planted_index` 条记录的头算出来的链值（`singlefs_core::journal::back_chain_of`）。
    ChainValueOfAnEarlierPlantedRecord { planted_index: usize },
}

/// 种进环里的一条记录 core 的记录读者（`singlefs_core::journal::JournalRecord::parse`）收不收：池级 checker 的记录读者两种都收。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadingByTheRecoveryReader {
    /// 照合法镜像里第一条记录原样：收。
    ReadsIt,
    /// 本次发布内序号改 0（D23（journal 的角色与格式） 已定项 4：序号从 1 起，读到 0 当损坏）：core 不收，池级 checker 收下、I-8.9 判红。
    RefusesItForTheOrdinalZero,
}

/// 种进盘 0 journal 环的一条记录：照合法镜像里第一条记录改实例代号、计数器、`checkpoint_txg`、反向链与 core 的记录读者收不收。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PlantedRecord {
    instance: u32,
    counter: u64,
    checkpoint_txg: u64,
    back_chain: BackChainOfAPlantedRecord,
    reading_by_the_recovery_reader: ReadingByTheRecoveryReader,
}

/// 盘 0 journal 环从头起依次种下 `planted`（第 k 条落环槽 k），别的盘与别的槽清空（冲突布局，合法写者不写它）。
/// 每条都先经池级 checker 的记录读者断言收得下，core 的记录读者收不收照那一条的 `reading_by_the_recovery_reader` 断言。
fn image_with_records_planted_on_device_zero(tag: &str, planted: &[PlantedRecord]) -> MemoryPool {
    let built = build_pool(tag);
    let mut image = built.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置择得出");
    let filesystem_identifier =
        unit_filesystem_identifier(&system_configuration.immutable.filesystem_identifier);
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let first_record_of_the_pool = JournalRecord::parse(
        &ImageReader::read(&image, 0, ring_start, record_bytes).expect("盘 0 的环头读得出"),
        filesystem_identifier,
    )
    .expect("合法镜像环头那一槽是暖机写的第一条记录");
    let mut writes: Vec<RetainedWrite> = ImageReader::devices(&image)
        .into_iter()
        .map(|device| RetainedWrite {
            device: DeviceIdentity(device),
            kind: StepKind::ZeroFill,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(ring_start),
            contents: WrittenContents::Zeros {
                length: system_configuration.immutable.sizes.journal_ring_bytes,
            },
        })
        .collect();
    let mut planted_bytes: Vec<Vec<u8>> = Vec::new();
    for (record_to_plant, ring_slot) in planted.iter().zip(0u64..) {
        let mut record = first_record_of_the_pool.clone();
        record.instance = InstanceGeneration(record_to_plant.instance);
        record.counter = record_to_plant.counter;
        record.checkpoint_txg = CheckpointTxg(record_to_plant.checkpoint_txg);
        match record_to_plant.back_chain {
            BackChainOfAPlantedRecord::AsTheFirstRecordOfThePoolCarries => {}
            BackChainOfAPlantedRecord::ChainValueOfAnEarlierPlantedRecord { planted_index } => {
                record.back_chain = back_chain_of(&planted_bytes[planted_index]);
            }
        }
        match record_to_plant.reading_by_the_recovery_reader {
            ReadingByTheRecoveryReader::ReadsIt => {}
            ReadingByTheRecoveryReader::RefusesItForTheOrdinalZero => {
                record.ordinal_within_publish = JournalRecordOrdinalWithinPublish(0);
            }
        }
        let bytes = record.to_bytes();
        assert!(
            check_journal_record(&bytes, filesystem_identifier).is_ok(),
            "池级 checker 的记录读者收得下 {record_to_plant:?}"
        );
        assert_eq!(
            JournalRecord::parse(&bytes, filesystem_identifier).is_some(),
            record_to_plant.reading_by_the_recovery_reader == ReadingByTheRecoveryReader::ReadsIt,
            "core 的记录读者收不收照 {record_to_plant:?}"
        );
        writes.push(RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::JournalRecord,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(ring_start + ring_slot * JOURNAL_RECORD_BYTES),
            contents: WrittenContents::Bytes(bytes.clone()),
        });
        planted_bytes.push(bytes);
    }
    image.apply_writes(&writes);
    image
}

/// 计数器 1 上一条记录：反向链照第一条记录原样（0），core 的记录读者收得下。
const fn record_at_counter_one(instance: u32, checkpoint_txg: u64) -> PlantedRecord {
    PlantedRecord {
        instance,
        counter: 1,
        checkpoint_txg,
        back_chain: BackChainOfAPlantedRecord::AsTheFirstRecordOfThePoolCarries,
        reading_by_the_recovery_reader: ReadingByTheRecoveryReader::ReadsIt,
    }
}

/// 第 1 批那四种排列（同一块盘两条计数器 1 的记录）：两边逐位相同。I-8.6 两条都按计数器 1 判（反向链 0），两边都判到而成立；
/// 环槽 1 那一条的计数器 1 该落环槽 0（D23（journal 的角色与格式） 已定项 18），I-8.13 两边都红（C897：审计 C02 那四种排列判红）。
fn judge_two_records_at_counter_one_on_device_zero(
    test_name: &str,
    tag: &str,
    instance_and_txg_in_ring_slot_order: [(u32, u64); 2],
) {
    let image = image_with_records_planted_on_device_zero(
        tag,
        &instance_and_txg_in_ring_slot_order
            .map(|(instance, checkpoint_txg)| record_at_counter_one(instance, checkpoint_txg)),
    );
    judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(
        test_name,
        &image,
        &[
            ("I-8.6", ExpectedVerdictOfTheInvariant::JudgedAndHolds),
            (
                JOURNAL_RECORD_RING_SLOT_INVARIANT,
                ExpectedVerdictOfTheInvariant::Violated,
            ),
        ],
    );
}

#[test]
fn the_gpu_judge_keeps_the_earlier_ring_slot_of_two_records_of_one_instance_and_counter_on_one_device_like_the_cpu_checker(
) {
    judge_two_records_at_counter_one_on_device_zero(
        "the_gpu_judge_keeps_the_earlier_ring_slot_of_two_records_of_one_instance_and_counter_on_one_device_like_the_cpu_checker",
        "gpu-journal-same-key-txg-ten-first",
        [(1, 10), (1, 20)],
    );
}

#[test]
fn the_gpu_judge_keeps_the_earlier_ring_slot_of_two_records_of_one_instance_and_counter_on_one_device_in_reverse_order_like_the_cpu_checker(
) {
    judge_two_records_at_counter_one_on_device_zero(
        "the_gpu_judge_keeps_the_earlier_ring_slot_of_two_records_of_one_instance_and_counter_on_one_device_in_reverse_order_like_the_cpu_checker",
        "gpu-journal-same-key-txg-twenty-first",
        [(1, 20), (1, 10)],
    );
}

#[test]
fn the_gpu_judge_keeps_both_records_of_two_instances_at_one_counter_on_one_device_like_the_cpu_checker(
) {
    judge_two_records_at_counter_one_on_device_zero(
        "the_gpu_judge_keeps_both_records_of_two_instances_at_one_counter_on_one_device_like_the_cpu_checker",
        "gpu-journal-two-instances-instance-one-first",
        [(1, 10), (2, 20)],
    );
}

#[test]
fn the_gpu_judge_keeps_both_records_of_two_instances_at_one_counter_on_one_device_in_reverse_order_like_the_cpu_checker(
) {
    judge_two_records_at_counter_one_on_device_zero(
        "the_gpu_judge_keeps_both_records_of_two_instances_at_one_counter_on_one_device_in_reverse_order_like_the_cpu_checker",
        "gpu-journal-two-instances-instance-two-first",
        [(2, 20), (1, 10)],
    );
}

/// 盘 0 环槽 0、1、2：实例 1 计数器 1、实例 2 计数器 1、实例 2 计数器 2（反向链是实例 2 计数器 1 那一条的链值，txg 跟着加一，
/// 各成一次发布）。CPU 按 (实例 2, 计数器 1) 找到本实例逻辑前一条、链值相等，I-8.6 判到而成立；两边逐位相同。环槽 1、2 那两条
/// 该落环槽 0、1，I-8.13 两边都红。
/// 判别力：走读内核逐盘记录表只按计数器建键（实例 2 计数器 1 那一条被先到的实例 1 那一条挤掉），或 I-8.6 只按计数器找前一条
/// （找到实例 1 那一条、当成本实例第一条要链值 0），GPU 红 I-8.6、CPU 不红。
#[test]
fn the_gpu_judge_takes_the_back_chain_of_the_predecessor_of_the_same_instance_when_two_instances_share_the_previous_counter_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_takes_the_back_chain_of_the_predecessor_of_the_same_instance_when_two_instances_share_the_previous_counter_like_the_cpu_checker";
    let image = image_with_records_planted_on_device_zero(
        "gpu-journal-back-chain-of-the-same-instance",
        &[
            record_at_counter_one(1, 10),
            record_at_counter_one(2, 20),
            PlantedRecord {
                instance: 2,
                counter: 2,
                checkpoint_txg: 21,
                back_chain: BackChainOfAPlantedRecord::ChainValueOfAnEarlierPlantedRecord {
                    planted_index: 1,
                },
                reading_by_the_recovery_reader: ReadingByTheRecoveryReader::ReadsIt,
            },
        ],
    );
    judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(
        TEST_NAME,
        &image,
        &[
            ("I-8.6", ExpectedVerdictOfTheInvariant::JudgedAndHolds),
            (
                JOURNAL_RECORD_RING_SLOT_INVARIANT,
                ExpectedVerdictOfTheInvariant::Violated,
            ),
        ],
    );
}

// ───────────────────────── C897：I-8.13 与读者收录口径 ─────────────────────────

/// 合法镜像（mkfs → 暖机 → 新池新建文件）上每块盘每条自证过的记录都落在它计数器该落的环槽：I-8.13 两边都判到而成立、零违例
/// （C897 验收「合法镜像零违例」在 GPU 判器这一侧的那一张）。
/// 判别力：走读内核算该落的槽时漏了「计数器 − 1」，计数器 1 那一条要落环槽 1，GPU 红 I-8.13、CPU 不红。
#[test]
fn the_gpu_judge_holds_the_ring_slot_of_every_journal_record_of_a_clean_pool_like_the_cpu_checker()
{
    const TEST_NAME: &str =
        "the_gpu_judge_holds_the_ring_slot_of_every_journal_record_of_a_clean_pool_like_the_cpu_checker";
    let built = build_pool("gpu-journal-ring-slots-of-a-clean-pool");
    let Some(red_on_the_image) = judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(
        TEST_NAME,
        &built.memory_pool(),
        &[(
            JOURNAL_RECORD_RING_SLOT_INVARIANT,
            ExpectedVerdictOfTheInvariant::JudgedAndHolds,
        )],
    ) else {
        return;
    };
    assert_eq!(
        red_on_the_image,
        Vec::<&str>::new(),
        "{TEST_NAME}：合法镜像零违例"
    );
}

/// 同一块盘同一键的两条记录里 core 的记录读者读不出的那一条落哪个环槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RingSlotOfTheRefusedRecord {
    /// 环槽 0：读不出的那一条先到，读得出的那一条在环槽 1。
    ArrivesFirst,
    /// 环槽 1：读得出的那一条先到。
    ArrivesSecond,
}

/// C897 并进的读者收录口径：盘 0 环槽 0、1 两条同键（实例 1、计数器 1）的记录，core 的记录读者读不出的那一条（序号 0，txg 10）与
/// 读得出的那一条（txg 20）按 `refused_record` 排。逐盘判不变量的那一份表照旧先到先留（读不出的先到时它占着这个键）；恢复视图与重放读的那一份环
/// 只收读得出的（走读内核 `checker_scan_journal_as_the_recovery_reads`、`collect_the_records_the_replay_reads`，池级 checker 逐盘的
/// `records_the_recovery_reader_reads`）。两边逐位相同；环槽 1 那一条 I-8.13 两边都红。交回 CPU 判红的那几条。
fn judge_a_refused_and_a_readable_record_of_one_key_on_device_zero(
    test_name: &str,
    tag: &str,
    refused_record: RingSlotOfTheRefusedRecord,
    further_expectations: &[(&str, ExpectedVerdictOfTheInvariant)],
) -> Option<Vec<&'static str>> {
    let refused = PlantedRecord {
        reading_by_the_recovery_reader: ReadingByTheRecoveryReader::RefusesItForTheOrdinalZero,
        ..record_at_counter_one(1, 10)
    };
    let readable = record_at_counter_one(1, 20);
    let planted = match refused_record {
        RingSlotOfTheRefusedRecord::ArrivesFirst => [refused, readable],
        RingSlotOfTheRefusedRecord::ArrivesSecond => [readable, refused],
    };
    let image = image_with_records_planted_on_device_zero(tag, &planted);
    let mut expectations = vec![(
        JOURNAL_RECORD_RING_SLOT_INVARIANT,
        ExpectedVerdictOfTheInvariant::Violated,
    )];
    expectations.extend_from_slice(further_expectations);
    judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(test_name, &image, &expectations)
}

/// 读不出的那一条先到：逐盘判不变量的那一份表留它（序号 0，I-8.9 两边都红），恢复视图与重放留后到的那一条读得出的（与 core 扫环选的同一份）。
#[test]
fn the_gpu_judge_reads_the_later_readable_record_when_a_record_the_recovery_reader_refuses_arrives_first_on_one_device_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reads_the_later_readable_record_when_a_record_the_recovery_reader_refuses_arrives_first_on_one_device_like_the_cpu_checker";
    judge_a_refused_and_a_readable_record_of_one_key_on_device_zero(
        TEST_NAME,
        "gpu-journal-refused-record-first",
        RingSlotOfTheRefusedRecord::ArrivesFirst,
        &[("I-8.9", ExpectedVerdictOfTheInvariant::Violated)],
    );
}

/// 读得出的那一条先到：两份表都留它，读不出的那一条只在 I-8.13 那一判里出现（落在环槽 1，红）。
#[test]
fn the_gpu_judge_keeps_the_earlier_readable_record_when_a_record_the_recovery_reader_refuses_arrives_second_on_one_device_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_keeps_the_earlier_readable_record_when_a_record_the_recovery_reader_refuses_arrives_second_on_one_device_like_the_cpu_checker";
    judge_a_refused_and_a_readable_record_of_one_key_on_device_zero(
        TEST_NAME,
        "gpu-journal-refused-record-second",
        RingSlotOfTheRefusedRecord::ArrivesSecond,
        &[],
    );
}

// ───────────────────────── C896：命中别的版本走过的码 2 节点时照判父条目定的那几格 ─────────────────────────

/// 码 2 节点的字节数。
fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

/// 一版里一个码 2 节点的落点（这几份镜像上两盘同槽，取第一条位置条目的槽）。
fn slot_of(pointer: &NodePointer) -> SlotNumber {
    pointer.locations[0].slot
}

/// `node_bytes` 那个码 2 内部节点里子指针落点是 `child_slot` 的那一条条目：交回它在节点字节里的起点与头里自述的 key 宽（叶没有子指针，交 `None`）。
fn entry_naming_the_child_at(node_bytes: &[u8], child_slot: SlotNumber) -> Option<(usize, usize)> {
    let node = parse_index_node(node_bytes).expect("码 2 节点解得开");
    if node.level == 0 {
        return None;
    }
    let entry = node.entries.iter().find(|entry| {
        let child = NodePointer::read_from(&mut ByteReader::at(entry, node.key_width));
        child.locations[0].slot == child_slot
    })?;
    let start = node_bytes
        .windows(entry.len())
        .position(|window| window == entry.as_slice())
        .expect("条目在节点字节里");
    Some((start, node.key_width))
}

/// 镜像上 `version` 那一版的一个码 2 节点（落点 `node_slot`，两盘同槽）把 `entry_start` 起那一条条目的 key 换成 `new_key`：按类重封、写回两盘，
/// 新的整单元校验和沿那一版的引用链（`version.units`：父节点、中央映射条目、树表）补到根槽，再把每条根抄进它那几条 journal 记录的新根段
/// （`common_tree_split::propagate_the_new_checksum`，I-8.10 不因此红）。
fn image_with_an_entry_key_of_a_version_changed(
    pool: &TreeSplitPool,
    version: &TransactionOutput,
    node_slot: SlotNumber,
    entry_start: usize,
    new_key: &[u8],
) -> MemoryPool {
    let mut image = pool.memory_pool();
    let before = read_unit_on_device(&image, 0, node_slot.0, node_bytes());
    let mut after = before.clone();
    after[entry_start..entry_start + new_key.len()].copy_from_slice(new_key);
    reseal_unit_by_its_class(&mut after);
    write_unit_to_the_same_slot_on_both_devices(&mut image, node_slot.0, &after);
    let units: Vec<(u64, usize)> = version
        .units
        .iter()
        .map(|unit: &PublishedUnit| (unit.slot.0, unit.bytes.len()))
        .collect();
    propagate_the_new_checksum(
        &mut image,
        &units,
        (
            node_slot.0,
            crc32_castagnoli(&before),
            crc32_castagnoli(&after),
        ),
    );
    image
}

/// 中央映射树每片叶装 3 条、每个内部节点 3 个孩子（记账树照节点格式的容量）：第一个文件版本与其后一次空发布两版的映射树都多层，
/// 共享没改过的那几片叶。
const CENTRAL_MAPPING_CAPPED_TO_THREE_ENTRIES_AND_THREE_CHILDREN: (usize, usize) = (3, 3);

/// 中央映射树（多层码 2 树）的 ②：第一个文件版本（非最新候选根）自己的那个内部节点里，指着两版共享的那片叶的那一条条目，分隔 key 换成
/// 那片叶头里的最大 key（仍严格大于左邻、小于右邻，父节点自己的分隔 key 照旧递增）。最新根先走过那片叶：旧那一版走到它时命中、不往下走，
/// 这一条父条目给的分隔 key 大于孩子头里的最小 key，两边都只红 I-1.1（走读内核 `w_walk_code_two_tree` 命中时把孩子头里的区间交给父节点比 ② ③）。
/// 判别力：走读内核命中时照旧按「别的根走过」交、不比 ②（改前的样子），GPU 不红 I-1.1、CPU 红。
#[test]
fn the_gpu_judge_reddens_the_separator_key_an_older_mapping_node_gives_a_leaf_the_newest_root_walked_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_separator_key_an_older_mapping_node_gives_a_leaf_the_newest_root_walked_like_the_cpu_checker";
    let capped = capacities(
        ACCOUNTING_OF_THE_NODE_FORMAT,
        CENTRAL_MAPPING_CAPPED_TO_THREE_ENTRIES_AND_THREE_CHILDREN,
    );
    let mut pool = TreeSplitPool::with_the_first_file_version_under(capped);
    pool.empty_publish(capped);
    let clean = pool.memory_pool();
    let older_version = pool.version_before_the_current_one().clone();
    let newest_slots: Vec<SlotNumber> = pool
        .output
        .central_mapping_tree
        .pointers
        .iter()
        .map(slot_of)
        .collect();
    let shared_leaf_slot = older_version
        .central_mapping_tree
        .pointers
        .iter()
        .map(slot_of)
        .find(|slot| newest_slots.contains(slot))
        .expect("两版的中央映射树共享一个节点");
    let shared_leaf = parse_index_node(&read_unit_on_device(
        &clean,
        0,
        shared_leaf_slot.0,
        node_bytes(),
    ))
    .expect("共享的那个节点解得开");
    assert_eq!(shared_leaf.level, 0, "两版共享的是一片叶");
    assert_ne!(
        shared_leaf.smallest_key, shared_leaf.largest_key,
        "那片叶不止一条：最大 key 大于最小 key"
    );
    let (parent_slot, entry_start, key_width) = older_version
        .central_mapping_tree
        .pointers
        .iter()
        .map(slot_of)
        .filter(|slot| !newest_slots.contains(slot))
        .find_map(|slot| {
            entry_naming_the_child_at(
                &read_unit_on_device(&clean, 0, slot.0, node_bytes()),
                shared_leaf_slot,
            )
            .map(|(entry_start, key_width)| (slot, entry_start, key_width))
        })
        .expect("旧那一版自己的一个内部节点指着两版共享的那片叶");
    assert_eq!(
        key_width,
        shared_leaf.largest_key.len(),
        "分隔 key 与叶头里的 key 同宽"
    );
    let changed = image_with_an_entry_key_of_a_version_changed(
        &pool,
        &older_version,
        parent_slot,
        entry_start,
        &shared_leaf.largest_key,
    );
    let Some(red_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME, "I-1.1", &clean, &changed,
        )
    else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        vec!["I-1.1"],
        "{TEST_NAME}：两边都只红 I-1.1（旧那一版的分隔 key 大于共享叶头里的最小 key）"
    );
}

/// 找最新那两版共享一片分配记录树叶的那一对版本时至多覆盖写几次：叶 61 罩着单元区起点那几百个槽、每次覆盖写都改它（新分配与释放都落在它里面），
/// 落点越过叶 61 之后一阵两片都改，再之后落点绕回叶 61 里释放出来的槽、叶 62 不再改（探针在产品容量的池上量的：第 36 次覆盖写那两版共享盘 0、
/// 盘 1 的叶 62），留出一倍的余量。
const MOST_OVERWRITES_BEFORE_THE_NEWEST_TWO_VERSIONS_SHARE_A_LEAF: usize = 72;

/// 分配记录树 key（设备 4、槽号 6，小端）里槽号那 6 字节（`singlefs_core::allocation_record_tree::allocation_record_key_bytes`）。
const ALLOCATION_RECORD_KEY_SLOT_BYTES: std::ops::Range<usize> = 4..10;

/// 分配记录树第 `level` 层、盘 `device` 上第 `index_in_device` 格的节点。
fn allocation_record_tree_node_below_the_root(
    level: u8,
    device: DeviceIdentity,
    index_in_device: u64,
) -> AllocationRecordTreeNode {
    AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
        level,
        device,
        index_in_device,
    })
}

/// 覆盖写一次的内容：半个数据单元，按第几次覆盖写换字节（每次都是一版新的内容）。
fn content_of_the_overwrite_numbered(overwrite_number: usize) -> Vec<u8> {
    (0..data_unit_payload_capacity() / 2)
        .map(|index| u8::try_from((index * 13 + overwrite_number * 5 + 3) % 251).expect("小于 256"))
        .collect()
}

/// 最新那两版共享的一片叶：在哪块盘、第几片，罩着它的第 1 层节点两版不同（旧那一版的第 1 层节点是它自己的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AllocationRecordLeafTheNewestTwoVersionsShare {
    device: DeviceIdentity,
    leaf_index_in_device: u64,
}

/// `older` 与 `newest` 两版共享、罩着它的第 1 层节点两版不同的那一片叶（按盘、按片号的次序取第一片）。
fn allocation_record_leaf_shared_under_different_level_one_nodes(
    older: &TransactionOutput,
    newest: &TransactionOutput,
) -> Option<AllocationRecordLeafTheNewestTwoVersionsShare> {
    older
        .allocation_record_tree
        .nodes
        .iter()
        .find_map(|(node, pointer)| match node {
            AllocationRecordTreeNode::BelowTheRoot(position) if position.level == 0 => {
                let level_one = allocation_record_tree_node_below_the_root(
                    1,
                    position.device,
                    position.index_in_device / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
                );
                let shared = newest.allocation_record_tree.pointer_of(*node) == Some(*pointer)
                    && older.allocation_record_tree.pointer_of(level_one).is_some()
                    && older.allocation_record_tree.pointer_of(level_one)
                        != newest.allocation_record_tree.pointer_of(level_one);
                shared.then_some(AllocationRecordLeafTheNewestTwoVersionsShare {
                    device: position.device,
                    leaf_index_in_device: position.index_in_device,
                })
            }
            AllocationRecordTreeNode::BelowTheRoot(_) | AllocationRecordTreeNode::Root => None,
        })
}

/// 产品容量的池连着覆盖写，直到最新那一版与它前一版共享一片分配记录树叶、罩着它的第 1 层节点两版不同。交回池与那一片叶。
fn pool_whose_newest_two_versions_share_an_allocation_record_leaf(
) -> (TreeSplitPool, AllocationRecordLeafTheNewestTwoVersionsShare) {
    let mut pool = TreeSplitPool::with_the_first_file_version_under(
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    // 迭代上界 MOST_OVERWRITES_BEFORE_THE_NEWEST_TWO_VERSIONS_SHARE_A_LEAF 次覆盖写；跨轮携带的只有池本身。
    for overwrite_number in 0..MOST_OVERWRITES_BEFORE_THE_NEWEST_TWO_VERSIONS_SHARE_A_LEAF {
        pool.overwrite(
            &content_of_the_overwrite_numbered(overwrite_number),
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
        );
        if let Some(shared_leaf) = allocation_record_leaf_shared_under_different_level_one_nodes(
            pool.version_before_the_current_one(),
            &pool.output,
        ) {
            return (pool, shared_leaf);
        }
    }
    panic!(
        "{MOST_OVERWRITES_BEFORE_THE_NEWEST_TWO_VERSIONS_SHARE_A_LEAF} 次覆盖写之内最新那两版没共享一片分配记录树叶"
    );
}

/// 第 1 层节点里共享叶那一条条目的 key 改指的那一格：紧挨着它、空着、仍落在这个第 1 层节点罩的那一段里、孩子仍按位置严格递增的那一格——
/// 后一格空着取后一格，否则前一格空着取前一格。`cells_named_by_the_entries` 是这个第 1 层节点每条条目 key 那一格的片号（升序）。
fn vacant_neighbouring_cell(cells_named_by_the_entries: &[u64], shared_cell: u64) -> u64 {
    let position = cells_named_by_the_entries
        .iter()
        .position(|cell| *cell == shared_cell)
        .expect("共享叶那一格在条目里");
    let next_cell = shared_cell + 1;
    let next_is_vacant = cells_named_by_the_entries
        .get(position + 1)
        .is_none_or(|following| *following > next_cell)
        && next_cell / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
            == shared_cell / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
    if next_is_vacant {
        return next_cell;
    }
    let previous_cell = shared_cell
        .checked_sub(1)
        .expect("后一格不空时共享叶不是这一段的第 0 格");
    assert!(
        position == 0 || cells_named_by_the_entries[position - 1] < previous_cell,
        "共享叶前后两格都有孩子：这一张镜像改不出只差位置的那一条"
    );
    previous_cell
}

/// 按位置寻址的树（分配记录树）：最新那两版共享一片叶，旧那一版（非最新候选根）自己的第 1 层节点里指着它的那一条条目，key 换成紧挨着的空格的起点
/// （孩子仍按位置严格递增、落在这个第 1 层节点罩的那一段里）。最新根先走过那片叶：旧那一版走到它时命中、不往下走，按这一条父条目给的位置它头里的
/// key 区间不是那一格罩的那一段，两边都只红 I-1.1（走读内核 `w_judge_the_position_of_an_allocation_node_an_earlier_version_walked`）。
/// 判别力：走读内核命中时不照判位置（改前的样子），GPU 不红 I-1.1、CPU 红。
#[test]
fn the_gpu_judge_reddens_the_position_an_older_level_one_node_gives_an_allocation_record_leaf_the_newest_root_walked_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_position_an_older_level_one_node_gives_an_allocation_record_leaf_the_newest_root_walked_like_the_cpu_checker";
    let (pool, shared_leaf) = pool_whose_newest_two_versions_share_an_allocation_record_leaf();
    let clean = pool.memory_pool();
    let older_version = pool.version_before_the_current_one().clone();
    let level_one_slot = slot_of(
        &older_version
            .allocation_record_tree
            .pointer_of(allocation_record_tree_node_below_the_root(
                1,
                shared_leaf.device,
                shared_leaf.leaf_index_in_device / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
            ))
            .expect("旧那一版有罩着共享叶的第 1 层节点"),
    );
    let level_one_bytes = read_unit_on_device(&clean, 0, level_one_slot.0, node_bytes());
    let key_of_the_cell = |cell_index: u64| {
        allocation_record_key_bytes(
            shared_leaf.device,
            SlotNumber(cell_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS),
        )
    };
    let level_one_node = parse_index_node(&level_one_bytes).expect("第 1 层节点解得开");
    let cells_named_by_the_entries: Vec<u64> = level_one_node
        .entries
        .iter()
        .map(|entry| {
            let mut slot_bytes = [0u8; 8];
            slot_bytes[..ALLOCATION_RECORD_KEY_SLOT_BYTES.len()]
                .copy_from_slice(&entry[ALLOCATION_RECORD_KEY_SLOT_BYTES]);
            u64::from_le_bytes(slot_bytes) / ALLOCATION_RECORD_TREE_LEAF_SLOTS
        })
        .collect();
    let new_cell = vacant_neighbouring_cell(
        &cells_named_by_the_entries,
        shared_leaf.leaf_index_in_device,
    );
    let shared_leaf_slot = slot_of(
        &older_version
            .allocation_record_tree
            .pointer_of(allocation_record_tree_node_below_the_root(
                0,
                shared_leaf.device,
                shared_leaf.leaf_index_in_device,
            ))
            .expect("旧那一版有共享的那片叶"),
    );
    let (entry_start, key_width) = entry_naming_the_child_at(&level_one_bytes, shared_leaf_slot)
        .expect("旧那一版的第 1 层节点里有指着共享叶的那一条条目");
    assert_eq!(
        level_one_bytes[entry_start..entry_start + key_width],
        key_of_the_cell(shared_leaf.leaf_index_in_device)[..],
        "那一条条目的 key 是共享叶那一格的起点"
    );
    let changed = image_with_an_entry_key_of_a_version_changed(
        &pool,
        &older_version,
        level_one_slot,
        entry_start,
        &key_of_the_cell(new_cell),
    );
    let Some(red_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME, "I-1.1", &clean, &changed,
        )
    else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        vec!["I-1.1"],
        "{TEST_NAME}：两边都只红 I-1.1（旧那一版给共享叶的位置不是它头里罩的那一段）"
    );
}

// ───────────────────────── 欠账第 2 批：命中别的版本走过的单元时照判这一版父条目定的格（inode 树、extent 树上段、墓碑树） ─────────────────────────

/// 镜像上一个码 2 节点（落点 `node_slot`，两盘同槽）按 `change` 改字节、按类重封、写回两盘，新的整单元校验和沿 `versions` 那几版的单元
/// （父节点、中央映射条目、树表）补到根槽，再把每条根抄进它那几条 journal 记录的新根段（`common_tree_split::propagate_the_new_checksum`，
/// I-8.10 不因此红）。`versions` 要含引用这个节点的每一版。
fn image_with_a_node_of_the_versions_changed(
    image: &MemoryPool,
    versions: &[&TransactionOutput],
    node_slot: SlotNumber,
    change: impl FnOnce(&mut [u8]),
) -> MemoryPool {
    let mut changed = image.clone();
    let before = read_unit_on_device(&changed, 0, node_slot.0, node_bytes());
    let mut after = before.clone();
    change(&mut after);
    reseal_unit_by_its_class(&mut after);
    write_unit_to_the_same_slot_on_both_devices(&mut changed, node_slot.0, &after);
    let mut units: Vec<(u64, usize)> = versions
        .iter()
        .flat_map(|version| {
            version
                .units
                .iter()
                .map(|unit: &PublishedUnit| (unit.slot.0, unit.bytes.len()))
        })
        .collect();
    units.sort_unstable();
    units.dedup();
    propagate_the_new_checksum(
        &mut changed,
        &units,
        (
            node_slot.0,
            crc32_castagnoli(&before),
            crc32_castagnoli(&after),
        ),
    );
    changed
}

/// 码 2 节点里子指针从条目内偏移 `pointer_offset_in_the_entry` 起、第一条位置条目落在 `child_slot` 的那一条条目在节点字节里的起点。
fn start_of_the_entry_whose_child_pointer_names(
    node: &[u8],
    pointer_offset_in_the_entry: usize,
    child_slot: SlotNumber,
) -> usize {
    let parsed = parse_index_node(node).expect("码 2 节点解得开");
    let entry = parsed
        .entries
        .iter()
        .find(|entry| {
            NodePointer::read_from(&mut ByteReader::at(entry, pointer_offset_in_the_entry))
                .locations[0]
                .slot
                == child_slot
        })
        .expect("节点里有一条条目指着那个孩子");
    node.windows(entry.len())
        .position(|window| window == entry.as_slice())
        .expect("条目在节点字节里")
}

/// 新池新建文件之后、实例 1 的同一个进程里建 `count` 个 size 0 的 inode（号从记账水位起），交回这一次发布、把它记成池的当前版。
fn create_inodes_and_advance_the_pool(pool: &mut BuiltPool, count: u64) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let output = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            count,
            FIXED_WRITE_TIME_SECONDS + WRITE_SECONDS_AFTER_THE_FIRST_FILE,
            InstanceGeneration(1),
        )
        .expect("建 inode")
    };
    pool.output = output.clone();
    output
}

/// 这一批的发布比池里第一个文件晚这么多秒写（具体哪一秒不承重）。
const WRITE_SECONDS_AFTER_THE_FIRST_FILE: u64 = 600;

/// inode 树内部条目里子指针的起点：分隔 key 8 + 出生树 8 + 类型段 2 + 容器号 8 + 容器出生代 8（D8（核心索引结构） 已定项 6 的字段表）。
const INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET: usize = 34;

/// inode 树根（层级 1）的分隔 key（I-9.12）：建 233 个 inode 那一次 inode 树分裂成两片叶容器（`many_inodes_write_and_read` 那一形），再建 1 个只重写
/// 右边那片与根，最左那片两版共享。分裂那一版（非最新候选根）自己的根里指着最左那片的那一条，分隔 key 换成那片叶装的最小记录号加一
/// （仍不大于它的最大记录号、仍小于右邻条目的分隔 key）。最新根先走过那片叶：分裂那一版走到它时命中、不往下走，这一版父条目的分隔 key
/// 大于孩子的最小 key，两边都只红 I-9.12（走读内核 `w_note_the_separator_of_an_inode_child_over_the_leaves_from`，池级 checker
/// `walk_inode_internal_node` 对命中的孩子同判——实现员丙那一份补丁；没打它时 CPU 不红、GPU 红）。
/// 判别力：走读内核命中的叶不记分隔 key 那一行（改前的样子），GPU 不红 I-9.12、CPU 红。
#[test]
fn the_gpu_judge_reddens_the_separator_an_older_inode_root_gives_a_leaf_container_the_newest_root_walked_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_separator_an_older_inode_root_gives_a_leaf_container_the_newest_root_walked_like_the_cpu_checker";
    let mut pool = build_pool("gpu-cached-reference-inode-root-separator");
    let split = create_inodes_and_advance_the_pool(&mut pool, INODE_LEAF_RECORDS);
    let newest = create_inodes_and_advance_the_pool(&mut pool, 1);
    let clean = pool.memory_pool();
    let leftmost_leaf_of = |version: &TransactionOutput| {
        version
            .unit(TransactionUnit::InodeLeafContainer(
                InodeLeafContainerIndexInTree::LEFTMOST,
            ))
            .slot
    };
    assert_eq!(
        leftmost_leaf_of(&split),
        leftmost_leaf_of(&newest),
        "最左那片叶容器两版共享"
    );
    let split_root_slot = split.unit(TransactionUnit::InodeRoot).slot;
    assert_ne!(
        split_root_slot,
        newest.unit(TransactionUnit::InodeRoot).slot,
        "inode 树根两版不同：分裂那一版的根是它自己的"
    );
    let inodes_of_the_leftmost_leaf: Vec<u64> = split.inode_leaf_containers[0]
        .contents
        .records
        .iter()
        .map(|record| record.inode)
        .collect();
    let smallest_inode = *inodes_of_the_leftmost_leaf
        .iter()
        .min()
        .expect("最左那片叶有记录");
    let largest_inode = *inodes_of_the_leftmost_leaf
        .iter()
        .max()
        .expect("最左那片叶有记录");
    let separator_past_the_smallest_inode = smallest_inode + 1;
    assert!(
        separator_past_the_smallest_inode <= largest_inode,
        "改过的分隔 key 仍落在那片叶的区间里：只有「大于最小 key」那一判红"
    );
    let split_root_bytes = read_unit_on_device(&clean, 0, split_root_slot.0, node_bytes());
    let entry_start = start_of_the_entry_whose_child_pointer_names(
        &split_root_bytes,
        INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET,
        leftmost_leaf_of(&split),
    );
    let changed =
        image_with_a_node_of_the_versions_changed(&clean, &[&split], split_root_slot, |bytes| {
            bytes[entry_start..entry_start + 8]
                .copy_from_slice(&separator_past_the_smallest_inode.to_le_bytes());
        });
    let Some(red_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME, "I-9.12", &clean, &changed,
        )
    else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        vec!["I-9.12"],
        "{TEST_NAME}：两边都只红 I-9.12（分裂那一版根里的分隔 key 大于共享叶容器的最小 key）"
    );
}

/// 按 inode 号写一版（`whole_write` 是整份顺序写、否则是一个数据单元以内的覆盖写），交回这一次发布、把它记成池的当前版。
fn write_an_inode_and_advance_the_pool(
    pool: &mut BuiltPool,
    inode: u64,
    content: &[u8],
    whole_write: InodeWrite,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let output = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        match whole_write {
            InodeWrite::SequentialWrite => publish_sequential_write_of_an_inode(
                &mut writer,
                &mut pool.allocator,
                &previous,
                InodeNumber(inode),
                content,
                FIXED_WRITE_TIME_SECONDS + WRITE_SECONDS_AFTER_THE_FIRST_FILE,
                InstanceGeneration(1),
            ),
            InodeWrite::OverwriteWithinOneDataUnit => publish_overwrite_of_an_inode(
                &mut writer,
                &mut pool.allocator,
                &previous,
                InodeNumber(inode),
                content,
                FIXED_WRITE_TIME_SECONDS + WRITE_SECONDS_AFTER_THE_FIRST_FILE,
                InstanceGeneration(1),
            ),
        }
        .expect("按 inode 号写")
    };
    pool.output = output.clone();
    output
}

/// 按 inode 号写的两种写法。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InodeWrite {
    SequentialWrite,
    OverwriteWithinOneDataUnit,
}

/// inode 号落在上段第一片叶之外（上段一片叶罩 143 个号，`writes_by_inode_number_of_several_files` 那一形用的号 150）。
const INODE_PAST_THE_FIRST_EXTENT_UPPER_LEAF: u64 = 150;

/// extent 树上段（按 inode 号按位置寻址）：建到号 150、写 150 那一版上段长成两层（根在第 1 层、叶 0 与叶 1），再覆盖写 inode 2 只重写叶 0 与根，
/// 叶 1 两版共享。长高那一版（非最新候选根）自己的根里指着叶 1 的那一条，key 换成第 2 格的起点（286，仍按位置严格递增、仍落在根罩的那一段里）。
/// 最新根先走过叶 1：长高那一版走到它时命中、不往下走，按这一条父条目给的位置它头里的 key 区间不是第 2 格罩的那一段，两边都红 I-1.1
/// （走读内核 `w_judge_the_position_of_an_extent_upper_node_an_earlier_version_walked`）；另红 I-9.40（那一版上段的节点按父条目认成叶 0 与叶 2，
/// 不是有条目的号定的规范形）。
/// 判别力：走读内核命中时不照判位置（改前的样子），GPU 不红 I-1.1、CPU 红。
#[test]
fn the_gpu_judge_reddens_the_position_an_older_extent_upper_root_gives_a_leaf_the_newest_root_walked_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_position_an_older_extent_upper_root_gives_a_leaf_the_newest_root_walked_like_the_cpu_checker";
    let mut pool = build_pool("gpu-cached-reference-extent-upper-position");
    create_inodes_and_advance_the_pool(&mut pool, INODE_PAST_THE_FIRST_EXTENT_UPPER_LEAF);
    let grown = write_an_inode_and_advance_the_pool(
        &mut pool,
        INODE_PAST_THE_FIRST_EXTENT_UPPER_LEAF,
        &[7u8; 4000],
        InodeWrite::SequentialWrite,
    );
    let newest = write_an_inode_and_advance_the_pool(
        &mut pool,
        2,
        &[9u8; 3000],
        InodeWrite::OverwriteWithinOneDataUnit,
    );
    let clean = pool.memory_pool();
    let upper_node_slot_of = |version: &TransactionOutput, level: u8, index: u64| {
        version
            .extent_tree
            .upper_nodes
            .iter()
            .find(|(position, _)| *position == ExtentUpperNodePosition { level, index })
            .map(|(_, pointer)| slot_of(pointer))
    };
    let grown_root_slot = upper_node_slot_of(&grown, 1, 0).expect("写号 150 之后上段根在第 1 层");
    let shared_leaf_slot = upper_node_slot_of(&grown, 0, 1).expect("上段叶 1");
    assert_eq!(
        upper_node_slot_of(&newest, 0, 1),
        Some(shared_leaf_slot),
        "上段叶 1 两版共享"
    );
    assert_ne!(
        upper_node_slot_of(&newest, 1, 0),
        Some(grown_root_slot),
        "上段根两版不同：长高那一版的根是它自己的"
    );
    let grown_root_bytes = read_unit_on_device(&clean, 0, grown_root_slot.0, node_bytes());
    let extent_key_bytes_wide = extent_key_bytes(0, 0).len();
    let entry_start = start_of_the_entry_whose_child_pointer_names(
        &grown_root_bytes,
        extent_key_bytes_wide,
        shared_leaf_slot,
    );
    let key_of_the_vacant_cell = extent_key_bytes(2 * EXTENT_TREE_UPPER_LEAF_INODES, 0);
    let changed =
        image_with_a_node_of_the_versions_changed(&clean, &[&grown], grown_root_slot, |bytes| {
            bytes[entry_start..entry_start + extent_key_bytes_wide]
                .copy_from_slice(&key_of_the_vacant_cell);
        });
    let Some(red_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME, "I-1.1", &clean, &changed,
        )
    else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        vec![
            "I-1.1",
            EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY
        ],
        "{TEST_NAME}：两边都红 I-1.1（长高那一版给共享叶的位置不是它头里罩的那一段）与改父条目连带的 I-9.40"
    );
}

/// 墓碑容器压到一片 1 条、容器树节点压到 2 条（`container_trees_grow_a_level_when_the_root_is_full` 那一组）：第 3 片容器时墓碑树根满长一层。
fn container_tree_capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(1).expect("1..=1359"),
        ),
        retirement_container: RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
        ),
    }
}

/// unlink 一个空 inode（写入口装 [`container_tree_capacities_of_one_record_and_two_entries`]）：意图那次与走空那几次，交回意图那一版与走空之后那一版。
fn unlink_an_empty_inode_and_advance_the_pool(
    pool: &mut BuiltPool,
    inode: u64,
) -> [TransactionOutput; 2] {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(container_tree_capacities_of_one_record_and_two_entries());
    let intent = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        FIXED_WRITE_TIME_SECONDS + WRITE_SECONDS_AFTER_THE_FIRST_FILE,
        InstanceGeneration(1),
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    let mut emptied = intent.clone();
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut emptied,
        UnlinkBatchSize::OneExtentLowerLeaf,
        InstanceGeneration(1),
    )
    .expect("空 inode 没有数据单元，一批走空");
    pool.output = emptied.clone();
    [intent, emptied]
}

/// 墓碑树（容器树）的子树覆盖：unlink 2、3、4 号之后墓碑树根满长成两层（根在第 2 层，(1, 0) 是旧根原地留下的那一个），再 unlink 5 号只重写 (1, 1)
/// 与根，(1, 0) 两版共享。长高那几版（非最新候选根）的根头里的最小 key 改小 1（条目的分隔 key 照旧落在区间里）。最新根先走过 (1, 0)：
/// 长高那几版走到它时命中、不往下走，这一版父节点的最小 key 不是第一个孩子头里的最小 key，两边都红 I-1.1（走读内核命中那一支
/// `w_judge_a_container_tree_child_node_under_its_parent_entry`）。镜像是压小容量写的、交给池级 checker 的旧入口（当没装旋钮）：
/// 干净的那一张两边都只红 I-9.21（不是最右的墓碑容器只装 1 条），改坏的那一张多红 I-1.1。
/// 判别力：走读内核命中时不照判子树覆盖与层级（改前的样子），GPU 不红 I-1.1、CPU 红。
#[test]
fn the_gpu_judge_reddens_the_coverage_an_older_tombstone_root_gives_a_node_the_newest_root_walked_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_the_coverage_an_older_tombstone_root_gives_a_node_the_newest_root_walked_like_the_cpu_checker";
    let mut pool = build_pool("gpu-cached-reference-tombstone-coverage");
    create_inodes_and_advance_the_pool(&mut pool, 5);
    let mut versions: Vec<TransactionOutput> = Vec::new();
    for inode in 2..=4 {
        versions.extend(unlink_an_empty_inode_and_advance_the_pool(&mut pool, inode));
    }
    let grown = pool.output.clone();
    versions.extend(unlink_an_empty_inode_and_advance_the_pool(&mut pool, 5));
    let newest = pool.output.clone();
    let clean = pool.memory_pool();
    let node_below_the_root_slot_of =
        |version: &TransactionOutput, level: u8, index_in_level: u32| {
            version
                .tombstone_tree_index_nodes
                .pointers_below_the_root
                .iter()
                .find(|(position, _)| {
                    *position
                        == ContainerTreeNodePosition {
                            level,
                            index_in_level,
                        }
                })
                .map(|(_, pointer)| slot_of(pointer))
        };
    assert_eq!(
        grown.tombstone_tree_index_nodes.shape.root_level(),
        2,
        "第 3 片容器时墓碑树长成两层"
    );
    let shared_node_slot = node_below_the_root_slot_of(&grown, 1, 0).expect("(1, 0)");
    assert_eq!(
        node_below_the_root_slot_of(&newest, 1, 0),
        Some(shared_node_slot),
        "(1, 0) 两版共享"
    );
    let grown_root_slot = grown.unit(TransactionUnit::TombstoneRoot).slot;
    assert_ne!(
        newest.unit(TransactionUnit::TombstoneRoot).slot,
        grown_root_slot,
        "墓碑树根两版不同"
    );
    let grown_root_bytes = read_unit_on_device(&clean, 0, grown_root_slot.0, node_bytes());
    let grown_root = parse_index_node(&grown_root_bytes).expect("长高那一版的根解得开");
    let smallest_key = u64::from_le_bytes(
        grown_root.smallest_key[..]
            .try_into()
            .expect("墓碑树的 key 是 8 字节的容器号"),
    );
    let smallest_key_moved_down = smallest_key
        .checked_sub(1)
        .expect("第一片容器号不是 0：改小 1 之后仍是合法的 key");
    // 头里 key 区间那两段紧挨着（最小 key 接最大 key），按两段连起来的字节在头里定位
    let key_range_bytes = [
        grown_root.smallest_key.clone(),
        grown_root.largest_key.clone(),
    ]
    .concat();
    let smallest_key_start = grown_root_bytes
        .windows(key_range_bytes.len())
        .position(|window| window == key_range_bytes.as_slice())
        .expect("头里的 key 区间");
    let version_references: Vec<&TransactionOutput> = versions.iter().collect();
    let changed = image_with_a_node_of_the_versions_changed(
        &clean,
        &version_references,
        grown_root_slot,
        |bytes| {
            bytes[smallest_key_start..smallest_key_start + 8]
                .copy_from_slice(&smallest_key_moved_down.to_le_bytes());
        },
    );
    let Some(red_on_the_clean_image) = judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(
        &format!("{TEST_NAME}/clean"),
        &clean,
        &[("I-1.1", ExpectedVerdictOfTheInvariant::JudgedAndHolds)],
    ) else {
        return;
    };
    assert_eq!(
        red_on_the_clean_image,
        vec![NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "{TEST_NAME}：干净的那一张两边都只红 I-9.21（压小容量写的镜像交给旧入口）"
    );
    let Some(red_on_the_changed_image) = judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(
        &format!("{TEST_NAME}/changed"),
        &changed,
        &[("I-1.1", ExpectedVerdictOfTheInvariant::Violated)],
    ) else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        vec![
            "I-1.1",
            NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE
        ],
        "{TEST_NAME}：两边都红 I-1.1（长高那一版根头里的最小 key 不是共享节点头里的最小 key）与干净那一张就有的 I-9.21"
    );
}

// ───────────────────────── 欠账第 2 批：恢复视图与重放读的那一份环收哪些记录 ─────────────────────────

/// journal 记录里载荷校验和的偏移：本次发布内序号 4、反向链 4 之后（`singlefs_core::journal::JournalRecord::to_bytes` 的字段次序）。
const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET + 4 + 4;

/// 新根段紧跟载荷校验和，段首是树表指针（86 字节，头部 MAC 16 字节在最前）。
const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = JOURNAL_PAYLOAD_CHECKSUM_OFFSET + 4;

/// 点名项 56 字节里 flags 那 1 字节的偏移（位置条目 14 × 2 + 类标签 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 之后）。
const JOURNAL_NAMED_ENTRY_FLAGS_OFFSET: usize = 55;

/// 改过字节的一条 journal 记录照写者的算法重封：载荷校验和（点名项那一段的 CRC-32C）与整条 4096 字节的头校验和（D23（journal 的角色与格式） 已定项 13）。
fn reseal_journal_record(bytes: &mut [u8]) {
    let named_count = usize::try_from(u32::from_le_bytes(
        bytes[12..16].try_into().expect("点名项数 4 字节"),
    ))
    .expect("点名项数装得进 usize");
    let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("496");
    let payload_end =
        header_bytes + named_count * usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56");
    let payload_checksum = crc32_castagnoli(&bytes[header_bytes..payload_end]);
    bytes[JOURNAL_PAYLOAD_CHECKSUM_OFFSET..JOURNAL_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let digest =
        wide_checksum_with_field_zeroed(bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
    bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + digest.len()]
        .copy_from_slice(&digest);
}

/// 环里按计数器 `counter` 落的那一槽（`(计数器 − 1) mod 槽数 × 4096`）在盘 `device` 上的偏移。
fn journal_ring_offset_of_the_counter(counter: u64) -> DeviceOffsetInBytes {
    record_offset(counter, parameters().geometry.journal_ring_bytes)
}

fn journal_record_bytes_on(image: &MemoryPool, device: u32, counter: u64) -> Vec<u8> {
    ImageReader::read(
        image,
        device,
        journal_ring_offset_of_the_counter(counter).0,
        usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
    )
    .expect("环里那一槽读得到")
}

/// `bytes` 两盘各写进计数器 `counter_of_the_ring_slot` 落的那一槽。
fn write_journal_record_on_both_devices(
    image: &mut MemoryPool,
    counter_of_the_ring_slot: u64,
    bytes: &[u8],
) {
    image.apply_writes(&[0u32, 1].map(|device| RetainedWrite {
        device: DeviceIdentity(device),
        kind: StepKind::JournalRecord,
        is_force_unit_access: false,
        offset: journal_ring_offset_of_the_counter(counter_of_the_ring_slot),
        contents: WrittenContents::Bytes(bytes.to_vec()),
    }));
}

/// 计数器 `counter` 那条记录（两盘同一份）按 `change` 改字节、重封。
fn journal_record_changed(
    image: &MemoryPool,
    counter: u64,
    change: impl FnOnce(&mut [u8]),
) -> Vec<u8> {
    let mut bytes = journal_record_bytes_on(image, 0, counter);
    assert_eq!(
        bytes,
        journal_record_bytes_on(image, 1, counter),
        "两盘上计数器 {counter} 那条记录同一份"
    );
    change(&mut bytes);
    reseal_journal_record(&mut bytes);
    bytes
}

/// 新池新建文件之后再覆盖写一版 B、把 B 的根槽改坏（只留单元与记录，`common::tear_the_root_slot_written_for_txg`）：
/// 下一次挂载在 A 的根上施加 B 的记录（池级 checker 的 I-4.1 走施加出来的那一版、判到而成立）。交回镜像与 B 那条记录的计数器。
fn image_whose_replay_applies_the_record_of_an_overwrite(tag: &str) -> (MemoryPool, u64) {
    let mut pool = build_pool(tag);
    let overwrite =
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &[5u8; 3000]);
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, overwrite.root.checkpoint_txg.0);
    (image, overwrite.record.counter)
}

/// core 的记录读者收不收：`singlefs_core::journal::JournalRecord::parse`。
fn the_recovery_reader_reads(bytes: &[u8]) -> bool {
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    JournalRecord::parse(bytes, filesystem_identifier).is_some()
}

/// 池级 checker 的记录读者收不收：`singlefs_checker::check_journal_record`。
fn the_pool_checker_reader_reads(bytes: &[u8]) -> bool {
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    check_journal_record(bytes, filesystem_identifier).is_ok()
}

/// 同一块盘同一键的两条记录里点名项 flags 字节非 0 的那一条落哪个环槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RingSlotOfTheRecordWithNamedEntryFlags {
    /// B 那条计数器落的那一槽：它先到，读得出的那一条在下一槽。
    ArrivesFirst,
    /// 下一槽：读得出的那一条先到。
    ArrivesSecond,
}

/// 点名项 flags 字节非 0 的记录（C02 验收「池级 checker 认的恢复视图与 core 逐条相同」反方向的那一处，实现员丙那一份补丁）：
/// 施加 B 的那张镜像上，B 那条记录（计数器 n）两份——第一个点名项 flags 写 1 的那一份（core 的读者跳过这一字节照收，池级 checker 的记录读者判
/// NonZeroFlags 拒），与序号改 2 的那一份（两个读者都收，从序号 2 起接不上 A，重放不施加它）——按 `named_entry_flags_record` 排在计数器 n 那一槽
/// 与下一槽，两盘同样。池级 checker 判不变量那一份表只收序号 2 那一条（I-8.9 两边都红：一次发布从 2 起），I-8.13 两条都判（落在下一槽的那一条红）；
/// 恢复视图与重放读的那一份环照 core 收 flags 非 0 那一条、同键先到先留：它先到时重放施加它（I-4.1 判到而成立），后到时留序号 2 那一条（不施加，
/// I-4.1 不适用）。两边逐位相同。
fn judge_a_record_with_named_entry_flags_and_a_readable_record_of_one_key(
    test_name: &str,
    tag: &str,
    named_entry_flags_record: RingSlotOfTheRecordWithNamedEntryFlags,
    expected_replay: ExpectedVerdictOfTheInvariant,
) {
    let (mut image, counter) = image_whose_replay_applies_the_record_of_an_overwrite(tag);
    let record_with_named_entry_flags = journal_record_changed(&image, counter, |bytes| {
        bytes[usize::try_from(JOURNAL_HEADER_BYTES).expect("496")
            + JOURNAL_NAMED_ENTRY_FLAGS_OFFSET] = 1;
    });
    assert!(
        the_recovery_reader_reads(&record_with_named_entry_flags)
            && !the_pool_checker_reader_reads(&record_with_named_entry_flags),
        "点名项 flags 非 0：core 的读者收、池级 checker 的记录读者拒"
    );
    let record_from_the_second_ordinal = journal_record_changed(&image, counter, |bytes| {
        bytes[JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET..JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET + 4]
            .copy_from_slice(&2u32.to_le_bytes());
    });
    assert!(
        the_recovery_reader_reads(&record_from_the_second_ordinal)
            && the_pool_checker_reader_reads(&record_from_the_second_ordinal),
        "序号 2 的那一条两个读者都收"
    );
    let (in_the_slot_of_the_counter, in_the_next_slot) = match named_entry_flags_record {
        RingSlotOfTheRecordWithNamedEntryFlags::ArrivesFirst => (
            &record_with_named_entry_flags,
            &record_from_the_second_ordinal,
        ),
        RingSlotOfTheRecordWithNamedEntryFlags::ArrivesSecond => (
            &record_from_the_second_ordinal,
            &record_with_named_entry_flags,
        ),
    };
    write_journal_record_on_both_devices(&mut image, counter, in_the_slot_of_the_counter);
    write_journal_record_on_both_devices(&mut image, counter + 1, in_the_next_slot);
    let Some(red_on_the_image) = judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(
        test_name,
        &image,
        &[
            ("I-4.1", expected_replay),
            ("I-8.9", ExpectedVerdictOfTheInvariant::Violated),
            (
                JOURNAL_RECORD_RING_SLOT_INVARIANT,
                ExpectedVerdictOfTheInvariant::Violated,
            ),
        ],
    ) else {
        return;
    };
    assert_eq!(
        red_on_the_image,
        vec!["I-8.9", JOURNAL_RECORD_RING_SLOT_INVARIANT],
        "{test_name}：两边都只红 I-8.9（判不变量那一份表里那次发布从序号 2 起）与 I-8.13（落在下一槽的那一条）"
    );
}

/// flags 非 0 的那一条先到：恢复视图与重放读的那一份环留它（与 core 扫环选的同一份），重放施加它。
/// 判别力：走读内核重放读的那一份环照池级 checker 的记录读者拒 flags 非 0 的（改前的样子），留序号 2 那一条、不施加，GPU 的 I-4.1 不判、CPU 判到。
#[test]
fn the_gpu_judge_replays_a_record_whose_named_entry_flags_are_not_zero_when_it_arrives_before_a_readable_record_of_the_same_key_like_the_cpu_checker(
) {
    judge_a_record_with_named_entry_flags_and_a_readable_record_of_one_key(
        "the_gpu_judge_replays_a_record_whose_named_entry_flags_are_not_zero_when_it_arrives_before_a_readable_record_of_the_same_key_like_the_cpu_checker",
        "gpu-journal-named-entry-flags-first",
        RingSlotOfTheRecordWithNamedEntryFlags::ArrivesFirst,
        ExpectedVerdictOfTheInvariant::JudgedAndHolds,
    );
}

/// 读得出的那一条先到：两份表都留它，重放从序号 2 起接不上、不施加；flags 非 0 的那一条只在 I-8.13 那一判里出现（落在下一槽，红）。
/// 判别力：走读内核 I-8.13 照池级 checker 的记录读者拒 flags 非 0 的、不判它的落点（改前的样子），GPU 的 I-8.13 成立、CPU 红。
#[test]
fn the_gpu_judge_keeps_the_readable_record_when_a_record_whose_named_entry_flags_are_not_zero_arrives_second_on_one_device_like_the_cpu_checker(
) {
    judge_a_record_with_named_entry_flags_and_a_readable_record_of_one_key(
        "the_gpu_judge_keeps_the_readable_record_when_a_record_whose_named_entry_flags_are_not_zero_arrives_second_on_one_device_like_the_cpu_checker",
        "gpu-journal-named-entry-flags-second",
        RingSlotOfTheRecordWithNamedEntryFlags::ArrivesSecond,
        ExpectedVerdictOfTheInvariant::NotJudged,
    );
}

/// 重放读的那一份环只收新根段四条指针头部是第一版取值的记录（C897 那一行后半，走读内核 `journal_record_reads_as_the_recovery_reader_reads_it`
/// 的后四格）：施加 B 的那张镜像（I-4.1 判到而成立、零违例）上，B 那条记录新根段树表指针的 MAC 第一个字节写 1、重封（core 的读者拒，
/// 池级 checker 的记录读者照收、判不变量那一份表照判）。重放不施加它：两边 I-4.1 都不判，零违例。
/// 判别力：走读内核那一判漏了树表指针头部那一格，重放施加 B 那条记录，GPU 的 I-4.1 判到、CPU 不判。
#[test]
fn the_gpu_judge_does_not_replay_a_record_whose_new_root_segment_carries_a_pointer_head_outside_the_first_version_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_does_not_replay_a_record_whose_new_root_segment_carries_a_pointer_head_outside_the_first_version_like_the_cpu_checker";
    let (image, counter) = image_whose_replay_applies_the_record_of_an_overwrite(
        "gpu-journal-new-root-segment-pointer-head",
    );
    let Some(red_on_the_clean_image) = judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(
        &format!("{TEST_NAME}/clean"),
        &image,
        &[("I-4.1", ExpectedVerdictOfTheInvariant::JudgedAndHolds)],
    ) else {
        return;
    };
    assert_eq!(
        red_on_the_clean_image,
        Vec::<&str>::new(),
        "{TEST_NAME}：施加 B 的那张两边都零违例"
    );
    let record_with_a_pointer_head_outside_the_first_version =
        journal_record_changed(&image, counter, |bytes| {
            bytes[JOURNAL_NEW_ROOT_SEGMENT_OFFSET] = 1;
        });
    assert!(
        !the_recovery_reader_reads(&record_with_a_pointer_head_outside_the_first_version)
            && the_pool_checker_reader_reads(&record_with_a_pointer_head_outside_the_first_version),
        "新根段树表指针 MAC 非 0：core 的读者拒、池级 checker 的记录读者收"
    );
    let mut changed = image.clone();
    write_journal_record_on_both_devices(
        &mut changed,
        counter,
        &record_with_a_pointer_head_outside_the_first_version,
    );
    let Some(red_on_the_changed_image) = judge_the_image_and_assert_the_gpu_agrees_with_the_cpu(
        &format!("{TEST_NAME}/changed"),
        &changed,
        &[("I-4.1", ExpectedVerdictOfTheInvariant::NotJudged)],
    ) else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        Vec::<&str>::new(),
        "{TEST_NAME}：重放不施加那条记录，两边都零违例"
    );
}

/// 一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后的镜像（harness 档 `checker_known_bad_images.rs`
/// `image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record` 那段历史）：mkfs → 取号 → 暖机 → A → B，实例 1 再发一版
/// （txg 5）只落单元与记录；可写挂载择 B 的根、施加那条记录、给实例 1 写行。交回挂载之后的镜像与那一版记录的计数器。
fn image_after_a_writable_mount_over_a_version_applied_only_by_its_record(
    tag: &str,
) -> (MemoryPool, u64) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &[3u8; 4100],
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 B");
    let operations_through_the_second_publish = pool.retained_operations().len();
    let applied_only_by_its_record = publish_overwrite_in_process(
        &mut pool,
        &second,
        &[4u8; 3700],
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("实例 1 再发一版");
    let operations = pool.retained_operations();
    let classifier = geometry();
    let units_and_record_of_the_version: Vec<RetainedOperation> = operations
        [operations_through_the_second_publish..]
        .iter()
        .filter(|retained| match classifier.classify(&retained.operation) {
            StepKind::UnitWrite | StepKind::JournalRecord => true,
            StepKind::ZeroFill
            | StepKind::Discard
            | StepKind::RootRecordFua
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => false,
        })
        .cloned()
        .collect();
    let mut image_before_the_mount = pool.memory_pool_after_mkfs();
    image_before_the_mount
        .apply(&operations[pool.mkfs_operation_count..operations_through_the_second_publish]);
    image_before_the_mount.apply(&units_and_record_of_the_version);
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image_before_the_mount, &[], &[], &stream);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        mounted.output.journal.prefix_applied, 1,
        "可写挂载择 B 的根、施加那一版的记录"
    );
    (
        memory_pool_of_sparse_devices(&devices),
        applied_only_by_its_record.record.counter,
    )
}

/// 恢复视图只收恢复的读者读得出的（C897 那一行后半，走读内核 `checker_scan_journal_as_the_recovery_reads` 那一滤）：可写挂载接在由记录施加出来的
/// 那一版之后的镜像（两边零违例、I-3.1 判到而成立：那一版并进遍历，它被换下的固定点单元在 defer 里照算已分配）上，那一版的记录（计数器 5）
/// 记录标志位 1 置位、重封（core 的读者拒，池级 checker 的记录读者收、I-8.9 判红）。恢复视图不收它：那一版不再由记录施加出来、不并进遍历，
/// 两边都红 I-3.1（记账的已分配多出那几个单元）与 I-8.9。
/// 判别力：走读内核恢复视图不滤读不出的记录，那一版照旧并进遍历，GPU 的 I-3.1 成立、CPU 红。
#[test]
fn the_gpu_judge_does_not_take_a_version_applied_only_by_a_record_the_recovery_reader_refuses_like_the_cpu_checker(
) {
    const TEST_NAME: &str = "the_gpu_judge_does_not_take_a_version_applied_only_by_a_record_the_recovery_reader_refuses_like_the_cpu_checker";
    let (clean, counter) = image_after_a_writable_mount_over_a_version_applied_only_by_its_record(
        "gpu-journal-recovery-view-refuses-a-record",
    );
    let record_with_a_flag_bit_past_the_last_record_bit =
        journal_record_changed(&clean, counter, |bytes| {
            bytes[JOURNAL_RECORD_FLAGS_OFFSET] |= 0b10;
        });
    assert!(
        !the_recovery_reader_reads(&record_with_a_flag_bit_past_the_last_record_bit)
            && the_pool_checker_reader_reads(&record_with_a_flag_bit_past_the_last_record_bit),
        "记录标志位 1 置位：core 的读者拒、池级 checker 的记录读者收"
    );
    let mut changed = clean.clone();
    write_journal_record_on_both_devices(
        &mut changed,
        counter,
        &record_with_a_flag_bit_past_the_last_record_bit,
    );
    let Some(red_on_the_changed_image) =
        judge_the_clean_and_the_changed_image_and_assert_the_gpu_agrees_with_the_cpu(
            TEST_NAME, "I-3.1", &clean, &changed,
        )
    else {
        return;
    };
    assert_eq!(
        red_on_the_changed_image,
        vec!["I-3.1", "I-8.9"],
        "{TEST_NAME}：两边都红 I-3.1（那一版不并进遍历）与 I-8.9（记录标志位 0 之外有位）"
    );
}
