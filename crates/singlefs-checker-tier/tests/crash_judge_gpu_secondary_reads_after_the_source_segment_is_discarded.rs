//! checker 档模块：crash、crash_identity、crash_library、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! 池级 checker 的次级读法（只读不判的那几遍）照 D19（块指针的结构与宽度预算） 已定项 5「位置条目降为提示，中央映射是唯一入口」与
//! 已定项 8「豁免三类之外的码 1 / 2 / 3 位置条目只当提示」按这一版的中央映射认位置，GPU 判器里池级 checker 那一份（扫描内核与引用与快照那一截）
//! 与 CPU 的池级 checker（`crates/singlefs-checker/src/walk.rs` 的 `PositionOfASecondaryRead`）同判：整理搬走单元、源段整段 discard 之后，
//! 父节点里过期的位置提示读不出（内存盘与录制流施加时整段清零），两个判器都要按映射位置读到同一个单元；判定与去掉 discard 那几步的同一条流
//! （discard 交回「不支持」时盘上就是那样，源槽旧字节原样还在）逐条同种类。镜像全照真写路径走（可写挂载、整理、用户改动）；只有
//! 「提示读得出、映射位置读不出」与「一棵分配记录树读不出」两条是在真写路径的镜像上翻字节造的。
//!
//! 这几条原先另有一份在 harness 档（`crates/singlefs-harness/tests/checker_secondary_reads_after_the_source_segment_is_discarded.rs`），
//! 真写路径的镜像就地交池级 checker 判；用户 2026-10-18 定「挪进 checker 档走两段」，那一份的断言（带 discard 那一份点名的几条判成立、
//! 带与不带 discard 两份逐条同种类、分配记录树读不出时 I-3.10 报那一句不适用、读得出时判成立）逐条并进这里，判在从库里读回的 CPU 参照上。
//! 每张镜像两边的输入都经对拍那一类的库：GPU 那一侧把镜像当基线、一个空持久集合建判表，判表入库再读回上卡；CPU 那一侧把镜像作一张给定状态表
//! 写进库、读回，池级 checker 判读回的那一份。两边判出的东西（CPU 逐条的判定、GPU 那一个状态的结论字）作一份对拍结论入库（用户 2026-10-15 定
//! 「对拍结论入库」，`crash_library::pairing_conclusion_reused_or_judged`）：键是这张镜像的输入摘要与两边判法单元的键，同一张镜像再判时从库里
//! 复用、不上卡也不再判；比法与断言每趟照读回的那一份现比，不进键。逐条不变量比违例位与判到位（判到位 = 成立或违例；不适用与部分没判两边都不置位），
//! GPU 判不了判红。
//! 要本机一张 NVIDIA 独显：一张适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」。要 `gpu` 特性。
//! 这几条是 GPU 判器（加速路径）与 CPU 池级 checker（参照路径）的对拍，照 `.claude/rules/test-script-discipline.md`「分快慢两档」一律进快档、
//! 不看快慢判据，都不标 `#[ignore]`。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, parameters, BuiltPool, PoolImageGeometry, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker::walk::{
    check_pool_image, ALLOCATION_GENERATIONS_OF_AN_UNREADABLE_ALLOCATION_RECORD_TREE_ARE_NOT_JUDGED,
};
use singlefs_checker_tier::crash::CrashPointReplaySegmentExpansion;
use singlefs_checker_tier::crash_amplification::RecordedGivenStatesTable;
use singlefs_checker_tier::crash_identity::{judging_units_key, JudgingUnits, SectorImage};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_judge_gpu::{
    cross_check_words_of_gpu_verdict, pool_checker_bits_agree_in_the_cross_check,
    pool_checker_bits_compared_by_the_cross_check, pool_checker_violated_and_judged_bits,
    VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{
    build_judge_tables_for_the_given_persisted_sets, JudgeFlow, JudgeTables,
};
use singlefs_checker_tier::crash_library::{
    library_reads_are_skipped_in_this_process, pairing_conclusion_reused_or_judged,
    toolchain_of_this_process, CrashLibrary, CrashLibraryClass, PairingConclusionSource,
};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    cpu_reference_table_read_back_from_the_pairing_library, gpu_card_pool_or_report_not_run,
    GpuCardPool,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::compaction_plan::{
    CompactionBatchSize, CompactionBatchUnits, CompactionRegionExclusion, CompactionRunForcing,
    CompactionSwitches, EmptyClusterSegmentsFloor, ForcedReleaseChecksumMismatchAfterIssue,
    RelocationDestinationPolicy, RelocationPublishOrder,
};
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::{FirstFile, TransactionOutput};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::sha256::sha256_digest;
use singlefs_harness::RecordedOperationKind;

/// 聚簇段的槽数（D3（空间分配） 已定项 10 ①：段起点是 64 的整数倍）：整理按段选源、discard 按段发。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;

/// 中央映射 key 首字节的类标签（D18（块里携带什么信息） 已定项 11 的登记表）：码 1 数据单元、码 2 树节点。
const MAPPING_KEY_CLASS_DATA_UNIT: u8 = 1;
const MAPPING_KEY_CLASS_INDEX_NODE: u8 = 2;

/// 翻哪一个字节让一个单元的整单元校验和对不上：码 2 节点头里的一字节（16 KiB 之内，哪一字节都让整单元 CRC 对不上）。
const BYTE_FLIPPED_TO_BREAK_THE_UNIT_CHECKSUM: u64 = 100;

/// 整理只搬强制选中的那一段、一批八个单元（与调查复现同一组开关）；源槽在搬迁那一次发布释放（`SourceReleasedInThePublishThatMovesTheMapping`）。
fn compaction_switches_forcing_the_segment(segment_first_slot: u64) -> CompactionSwitches {
    CompactionSwitches {
        batch_size: CompactionBatchSize::ForcedForTests(
            CompactionBatchUnits::new(8).expect("8 个单元一批"),
        ),
        forced_region: Some(
            CompactionRegion::of_one_cluster_segment(
                DeviceIdentity(0),
                SlotNumber(segment_first_slot),
            )
            .expect("段起点是 64 的倍数"),
        ),
        run_forcing: CompactionRunForcing::FollowTheWatermarks,
        region_exclusion: CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
        destination_policy:
            RelocationDestinationPolicy::UserDataPlacementForDataAndOpenSegmentForTheRest,
        release_checksum_mismatch_after_issue: ForcedReleaseChecksumMismatchAfterIssue::NotForced,
        empty_cluster_segments_floor: EmptyClusterSegmentsFloor::OfTheFirstVersion,
        relocation_publish_order:
            RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
    }
}

/// 一个槽所在聚簇段的起点（单元区起始槽号是 64 的整数倍，段从它起每 64 槽一段）。
fn cluster_segment_holding_the_slot(slot: SlotNumber) -> u64 {
    let unit_area_start_slot =
        PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters()
            .unit_area_start_slot()
            .0;
    unit_area_start_slot
        + (slot.0 - unit_area_start_slot) / CLUSTER_SEGMENT_SLOTS * CLUSTER_SEGMENT_SLOTS
}

/// 整理那一趟搬走的一个单元：映射 key 的类标签、源落点与目的落点（两盘各一条）。
struct UnitRelocatedByTheRun {
    mapping_key_class: u8,
    source: Vec<(DeviceIdentity, SlotNumber)>,
    destination: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 搬迁、再推一环之后的池：搬走了哪些单元、整理强制选中的段、这一趟与之后那一环发布里 discard 交回 `Applied` 的段。
struct PoolAfterTheRelocationAndOneRevolution {
    pool: BuiltPool,
    units_relocated_by_the_run: Vec<UnitRelocatedByTheRun>,
    forced_segment_first_slot: u64,
    discarded_segments: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 推几次只建 inode 的发布源段整段可再分配、发 discard：一环 3S 次（每区根槽数 S、三个区），再多两次让最旧那条还引用源槽的根转出根环。
fn publishes_until_the_source_segment_is_discarded() -> u64 {
    3 * parameters().geometry.root_ring_slots_per_region.count() + 2
}

/// 起池、可写挂载、做 `before`（用户改动），关掉重开、强制选 `segment_of(这一版)` 整理一趟，再推一环只建 inode 的发布。
fn relocate_a_segment_then_publish_one_revolution<Before, SegmentOf>(
    tag: &str,
    before: Before,
    segment_of: SegmentOf,
) -> PoolAfterTheRelocationAndOneRevolution
where
    Before: FnOnce(&mut MountedSession, &mut Vec<(DeviceIdentity, common::Recorded)>),
    SegmentOf: FnOnce(&TransactionOutput) -> u64,
{
    let mut pool = build_pool(tag);
    let mut devices_of_the_user_change = pool.reopen_recorded();
    let mounted_for_the_user_change =
        mount_writable(&parameters(), &mut devices_of_the_user_change).expect("可写挂载");
    let (_, mut session_of_the_user_change) =
        MountedSession::of_the_mount(mounted_for_the_user_change);
    before(
        &mut session_of_the_user_change,
        &mut devices_of_the_user_change,
    );
    let forced_segment_first_slot = segment_of(
        session_of_the_user_change
            .current
            .version_of_trees()
            .expect("用户改动之后是有树的一版"),
    );
    drop(session_of_the_user_change);
    pool.devices = Some(devices_of_the_user_change);
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载一次");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .allocator
        .set_compaction_switches(compaction_switches_forcing_the_segment(
            forced_segment_first_slot,
        ));
    let run = session.run_compaction(&mut devices).expect("整理一趟");
    let units_relocated_by_the_run: Vec<UnitRelocatedByTheRun> = run
        .versions_published
        .iter()
        .flat_map(|version| version.units_relocated_by_this_publish.iter())
        .map(|relocated| UnitRelocatedByTheRun {
            mapping_key_class: relocated.mapping_key[0],
            source: relocated
                .source
                .iter()
                .map(|location| (location.device, location.slot))
                .collect(),
            destination: relocated
                .destination
                .iter()
                .map(|location| (location.device, location.slot))
                .collect(),
        })
        .collect();
    let mut discarded_segments: Vec<(DeviceIdentity, SlotNumber)> = run.discards.applied.clone();
    for _publish in 0..publishes_until_the_source_segment_is_discarded() {
        let published = session
            .publish_user_change(
                &mut devices,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
                },
            )
            .expect("建一个 inode");
        discarded_segments.extend(published.discards_after_the_change.applied.iter().copied());
    }
    drop(session);
    pool.devices = Some(devices);
    let after = PoolAfterTheRelocationAndOneRevolution {
        pool,
        units_relocated_by_the_run,
        forced_segment_first_slot,
        discarded_segments,
    };
    assert!(
        after.discarded_segments.contains(&(
            DeviceIdentity(0),
            SlotNumber(after.forced_segment_first_slot)
        )),
        "整理选中的段 {} 在那一环发布里整段 discard 了：{:?}",
        after.forced_segment_first_slot,
        after.discarded_segments
    );
    after
}

/// 同一条录制流施加出来的镜像：`DiscardsKept` 是整条流（discard 那几步整段清零），`DiscardsLeftOut` 去掉 discard 那几步
/// （discard 交回「不支持」时盘上就是这样，源槽旧字节原样还在）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiscardsOfTheStream {
    DiscardsKept,
    DiscardsLeftOut,
}

fn image_of(pool: &BuiltPool, discards: DiscardsOfTheStream) -> MemoryPool {
    let operations: Vec<_> = pool
        .retained_operations()
        .into_iter()
        .filter(|retained| match discards {
            DiscardsOfTheStream::DiscardsKept => true,
            DiscardsOfTheStream::DiscardsLeftOut => {
                retained.operation.kind != RecordedOperationKind::Discard
            }
        })
        .collect();
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    image.apply(&operations);
    image
}

/// 两个数据单元以上的文件的那几次写落在挂载开的段里；整理搬空 extent 上段根所在的那一段，上段根原样拷到目的地、里面指向下段根的
/// 那条指针仍是源槽；只建 inode 的那一环之后源段 discard。交回池与下段根的源槽。
fn two_unit_file_whose_extent_roots_are_relocated(
    tag: &str,
) -> (PoolAfterTheRelocationAndOneRevolution, SlotNumber) {
    let mut lower_root_slot = None;
    let after = relocate_a_segment_then_publish_one_revolution(
        tag,
        |session, devices| {
            let content: Vec<u8> = (0..100_000usize)
                .map(|index| u8::try_from(index % 241).expect("小于 256"))
                .collect();
            session
                .publish_user_change(
                    devices,
                    UserChange::SequentialWrite(FirstFile {
                        content: &content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                    }),
                )
                .expect("顺序写成两个以上数据单元");
        },
        |current| {
            lower_root_slot = Some(current.extent_tree.lower_nodes[0].1.locations[0].slot);
            cluster_segment_holding_the_slot(current.extent_tree.upper_nodes[0].1.locations[0].slot)
        },
    );
    let lower_root_slot = lower_root_slot.expect("上面那一版的 extent 树有下段根");
    assert!(
        after.units_relocated_by_the_run.iter().any(|relocated| {
            relocated.mapping_key_class == MAPPING_KEY_CLASS_INDEX_NODE
                && relocated
                    .source
                    .contains(&(DeviceIdentity(0), lower_root_slot))
        }),
        "整理把 extent 树的下段根（槽 {}）搬走了",
        lower_root_slot.0
    );
    (after, lower_root_slot)
}

/// 一张镜像两边各判成什么：逐条不变量的违例位与判到位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`），加 CPU 那一侧的逐条判定（断言用），
/// 与这一份对拍结论是从库里复用的还是这一趟判的。
struct BothJudgesOnAnImage {
    cpu_verdicts: Vec<(&'static str, InvariantVerdict)>,
    cpu_bits: (u128, u128),
    gpu_bits: (u128, u128),
    source: PairingConclusionSource,
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
            "gpu judge secondary reads pairing inputs\nbase={}\ntables={}\n",
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

/// 一张镜像两边各判一次（GPU 那一侧把镜像当基线、一个空持久集合建判表）。对拍结论入库（用户 2026-10-15 定「对拍结论入库」，
/// `crash_library::pairing_conclusion_reused_or_judged`）：键是这张镜像的输入摘要（[`pairing_inputs_digest`]）与两边判法单元的键
/// （GPU 判器那一组；CPU 那一侧调的池级 checker `check_pool_image` 在判镜像的定义那一组 `judgement_of_the_crash_images` 的闭包里），
/// 库里有就复用、不上卡也不再判；没有就两边都判、两边判出的东西按键存进库。两边哪一边的判法单元拒算：照判、不入库。
/// 每趟打一行 `GPU_SECONDARY_READS_PAIRING_CONCLUSION table=<表名> source=<ReusedFromTheLibrary|JudgedNow>`。
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
    println!("GPU_SECONDARY_READS_PAIRING_CONCLUSION table={table_name} source={source:?}");
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
        source,
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

/// 两边逐条同种类：违例位与判到位各经对拍那一份去掉内核不置位的那几条再比；对不上时点名差在哪几条。
fn assert_the_gpu_judge_agrees_with_the_cpu_pool_checker(
    table_name: &str,
    both: &BothJudgesOnAnImage,
) {
    let (cpu_violated, cpu_judged) = both.cpu_bits;
    let (gpu_violated, gpu_judged) = both.gpu_bits;
    let differing_violations = pool_checker_bits_compared_by_the_cross_check(cpu_violated)
        ^ pool_checker_bits_compared_by_the_cross_check(gpu_violated);
    let differing_judgements = pool_checker_bits_compared_by_the_cross_check(cpu_judged)
        ^ pool_checker_bits_compared_by_the_cross_check(gpu_judged);
    assert!(
        pool_checker_bits_agree_in_the_cross_check(gpu_violated, cpu_violated)
            && pool_checker_bits_agree_in_the_cross_check(gpu_judged, cpu_judged),
        "{table_name}：GPU 判器与 CPU 池级 checker 逐条不同种类——违例位不同的 {:?}，判到位不同的 {:?}（CPU 判到 {:?}；GPU 判到 {:?}）",
        invariant_names(differing_violations),
        invariant_names(differing_judgements),
        invariant_names(pool_checker_bits_compared_by_the_cross_check(cpu_judged) & differing_judgements),
        invariant_names(pool_checker_bits_compared_by_the_cross_check(gpu_judged) & differing_judgements),
    );
}

fn verdict_named<'verdicts>(
    verdicts: &'verdicts [(&'static str, InvariantVerdict)],
    invariant: &str,
) -> &'verdicts InvariantVerdict {
    &verdicts
        .iter()
        .find(|(name, _)| *name == invariant)
        .unwrap_or_else(|| panic!("check_pool_image 的判定里有 {invariant}"))
        .1
}

/// 前提（CPU 一侧）：`invariants_that_must_hold` 每条判成立——这张镜像上只有按映射位置读到搬走的单元才判得到它们，GPU 照提示读就判不到、
/// 判到位对不上。
fn assert_the_cpu_pool_checker_holds(
    table_name: &str,
    both: &BothJudgesOnAnImage,
    invariants_that_must_hold: &[&str],
) {
    for invariant in invariants_that_must_hold {
        assert_eq!(
            verdict_named(&both.cpu_verdicts, invariant),
            &InvariantVerdict::Holds,
            "{table_name}：前提——CPU 池级 checker 按这一版的中央映射读到搬走的单元，{invariant} 判成立"
        );
    }
}

fn verdict_kind(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "成立",
        InvariantVerdict::Violated(_) => "违例",
        InvariantVerdict::NotApplicable(_) => "不适用",
        InvariantVerdict::PartiallyJudged(_) => "部分没判",
    }
}

/// CPU 池级 checker 在带 discard 与去掉 discard 两张镜像上（各从库里读回的那一份）逐条同种类：discard 不改任何一条的判定。
fn assert_the_discard_changes_no_verdict_of_the_cpu_pool_checker(
    test_name: &str,
    with_the_discards: &BothJudgesOnAnImage,
    without_the_discards: &BothJudgesOnAnImage,
) {
    assert_eq!(
        with_the_discards.cpu_verdicts.len(),
        without_the_discards.cpu_verdicts.len(),
        "{test_name}：两张镜像上池级 checker 判的条数相同"
    );
    for ((invariant, with_the_discard), (invariant_without, without_the_discard)) in
        with_the_discards
            .cpu_verdicts
            .iter()
            .zip(&without_the_discards.cpu_verdicts)
    {
        assert_eq!(
            invariant, invariant_without,
            "{test_name}：两张镜像上逐条按同一个次序判"
        );
        assert_eq!(
            verdict_kind(with_the_discard),
            verdict_kind(without_the_discard),
            "{test_name}：{invariant}：带 discard 的判成 {with_the_discard:?}，去掉 discard 的判成 {without_the_discard:?}"
        );
    }
}

fn table_name_of_the_image(test_name: &str, discards: DiscardsOfTheStream) -> String {
    format!(
        "{test_name}/{}",
        match discards {
            DiscardsOfTheStream::DiscardsKept => "discards-kept",
            DiscardsOfTheStream::DiscardsLeftOut => "discards-left-out",
        }
    )
}

/// 带 discard 那张镜像判完之后要不要再判一次、看对拍结论从库里复用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SecondPairingOfTheImageWithTheDiscards {
    NotJudgedAgain,
    JudgedAgainToSeeTheConclusionReused,
}

/// 一个搬迁场景的两张镜像（带 discard、去掉 discard）两边各判：带 discard 那一张上 CPU 判 `invariants_that_must_hold` 成立；两张上 CPU 逐条同种类；
/// 每一张 GPU 与 CPU 逐条同种类。`second_pairing` 要再判的，带 discard 那一张同一份输入再对拍一次：键相同，结论从库里复用、与头一趟逐条相同
/// （只写不读那一趟照判，不核从哪来）。
fn judge_both_images_of_the_relocation(
    test_name: &str,
    after: &PoolAfterTheRelocationAndOneRevolution,
    invariants_that_must_hold: &[&str],
    second_pairing: SecondPairingOfTheImageWithTheDiscards,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let table_name_with_the_discards =
        table_name_of_the_image(test_name, DiscardsOfTheStream::DiscardsKept);
    let table_name_without_the_discards =
        table_name_of_the_image(test_name, DiscardsOfTheStream::DiscardsLeftOut);
    let image_with_the_discards = image_of(&after.pool, DiscardsOfTheStream::DiscardsKept);
    let Some(with_the_discards) = judge_the_image_on_both_judges(
        cards,
        &table_name_with_the_discards,
        &image_with_the_discards,
    ) else {
        return;
    };
    let Some(without_the_discards) = judge_the_image_on_both_judges(
        cards,
        &table_name_without_the_discards,
        &image_of(&after.pool, DiscardsOfTheStream::DiscardsLeftOut),
    ) else {
        return;
    };
    assert_the_cpu_pool_checker_holds(
        &table_name_with_the_discards,
        &with_the_discards,
        invariants_that_must_hold,
    );
    assert_the_discard_changes_no_verdict_of_the_cpu_pool_checker(
        test_name,
        &with_the_discards,
        &without_the_discards,
    );
    for (table_name, both) in [
        (&table_name_with_the_discards, &with_the_discards),
        (&table_name_without_the_discards, &without_the_discards),
    ] {
        assert_the_gpu_judge_agrees_with_the_cpu_pool_checker(table_name, both);
    }
    match second_pairing {
        SecondPairingOfTheImageWithTheDiscards::NotJudgedAgain => {}
        SecondPairingOfTheImageWithTheDiscards::JudgedAgainToSeeTheConclusionReused => {
            let Some(again) = judge_the_image_on_both_judges(
                cards,
                &table_name_with_the_discards,
                &image_with_the_discards,
            ) else {
                return;
            };
            assert_eq!(
                again.cpu_verdicts, with_the_discards.cpu_verdicts,
                "{table_name_with_the_discards}：复用的 CPU 判定与头一趟逐条相同"
            );
            assert_eq!(
                again.gpu_bits, with_the_discards.gpu_bits,
                "{table_name_with_the_discards}：复用的 GPU 违例位与判到位与头一趟相同"
            );
            if !library_reads_are_skipped_in_this_process() {
                assert_eq!(
                    again.source,
                    PairingConclusionSource::ReusedFromTheLibrary,
                    "{table_name_with_the_discards}：同一份输入第二趟的对拍结论从库里复用、不现判"
                );
            }
        }
    }
}

/// extent 下段根被搬、源段 discard 之后，走到 extent 树叶的那几遍（I-3.9 的引用集合、删除态扫描的 I-4.6 / I-4.10 / I-9.38、
/// I-1.13 遍历方向那三张表）照映射位置读到下段根：CPU 判成立、与去掉 discard 那一份逐条同种类，GPU 与 CPU 同判；GPU 照过期提示读的话读不出、
/// 判不到，判到位对不上。带 discard 那张镜像再对拍一次，对拍结论从库里复用、不现判。
#[test]
fn the_gpu_judge_reads_an_extent_lower_root_relocated_before_its_source_segment_is_discarded_like_the_cpu_checker(
) {
    let (after, _) = two_unit_file_whose_extent_roots_are_relocated(
        "gpu-judge-secondary-reads-extent-lower-root-after-discard",
    );
    judge_both_images_of_the_relocation(
        "the_gpu_judge_reads_an_extent_lower_root_relocated_before_its_source_segment_is_discarded_like_the_cpu_checker",
        &after,
        &["I-3.9", "I-4.6", "I-1.13", "I-4.10", "I-9.38"],
        SecondPairingOfTheImageWithTheDiscards::JudgedAgainToSeeTheConclusionReused,
    );
}

/// 同一形里映射位置读不出、过期提示读得出（去掉 discard 那一份、再翻掉搬迁目的地两份各一个字节）：次级读法先按提示读
/// （CRC 对得上、头里出生身份与指针相同），两个判器那几条都判成立；GPU 只按映射位置读的话读不出、判不到。
#[test]
fn the_gpu_judge_takes_a_readable_location_hint_before_a_mapping_position_that_does_not_read_back_like_the_cpu_checker(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_takes_a_readable_location_hint_before_a_mapping_position_that_does_not_read_back_like_the_cpu_checker";
    let (after, lower_root_slot) = two_unit_file_whose_extent_roots_are_relocated(
        "gpu-judge-secondary-reads-hint-before-the-mapping-position",
    );
    let lower_root_relocated = after
        .units_relocated_by_the_run
        .iter()
        .find(|relocated| {
            relocated.mapping_key_class == MAPPING_KEY_CLASS_INDEX_NODE
                && relocated
                    .source
                    .contains(&(DeviceIdentity(0), lower_root_slot))
        })
        .expect("整理把 extent 树的下段根搬走了（造池那一判）");
    let mut image = image_of(&after.pool, DiscardsOfTheStream::DiscardsLeftOut);
    for (device, slot) in &lower_root_relocated.destination {
        image.flip_byte(
            *device,
            DeviceOffsetInBytes(slot.0 * SLOT_BYTES),
            BYTE_FLIPPED_TO_BREAK_THE_UNIT_CHECKSUM,
        );
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(both) = judge_the_image_on_both_judges(cards, TEST_NAME, &image) else {
        return;
    };
    assert_the_cpu_pool_checker_holds(TEST_NAME, &both, &["I-3.9", "I-4.6", "I-4.10", "I-9.38"]);
    assert_the_gpu_judge_agrees_with_the_cpu_pool_checker(TEST_NAME, &both);
}

/// 快照与活头共用 extent 树：两单元以上的文件、建快照，再整理那次顺序写的下段根所在的段。快照树冻住不再重写，过期提示一直留着；
/// 除了上一形那几条，只读快照那一遍（I-3.5、I-3.6）也照映射位置读快照与活头的树：CPU 判成立、与去掉 discard 那一份逐条同种类，两个判器同判。
#[test]
fn the_gpu_judge_reads_an_extent_tree_shared_with_a_snapshot_relocated_before_its_source_segment_is_discarded_like_the_cpu_checker(
) {
    let after = relocate_a_segment_then_publish_one_revolution(
        "gpu-judge-secondary-reads-snapshot-shared-extent-after-discard",
        |session, devices| {
            let content: Vec<u8> = (0..100_000usize)
                .map(|index| u8::try_from(index % 241).expect("小于 256"))
                .collect();
            session
                .publish_user_change(
                    devices,
                    UserChange::SequentialWrite(FirstFile {
                        content: &content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                    }),
                )
                .expect("顺序写四个数据单元");
            session
                .publish_user_change(devices, UserChange::CreateSnapshot)
                .expect("建快照");
        },
        |current| {
            cluster_segment_holding_the_slot(current.extent_tree.lower_nodes[0].1.locations[0].slot)
        },
    );
    assert!(
        after
            .units_relocated_by_the_run
            .iter()
            .any(|relocated| relocated.mapping_key_class == MAPPING_KEY_CLASS_INDEX_NODE),
        "整理搬走了码 2 节点"
    );
    judge_both_images_of_the_relocation(
        "the_gpu_judge_reads_an_extent_tree_shared_with_a_snapshot_relocated_before_its_source_segment_is_discarded_like_the_cpu_checker",
        &after,
        &[
            "I-3.9", "I-3.5", "I-3.6", "I-4.6", "I-1.13", "I-4.10", "I-9.38",
        ],
        SecondPairingOfTheImageWithTheDiscards::NotJudgedAgain,
    );
}

/// 数据单元被搬：文件写到四十个数据单元（用户数据跨出第一段），整理第一个用户数据段。I-9.38 按 extent 叶里的数据指针读数据单元头的声明长度，
/// 两个判器都照映射位置读到搬走的数据单元：CPU 判成立、与去掉 discard 那一份逐条同种类，两个判器同判。
/// GPU 与 CPU 的对拍一律进快档、不标 `#[ignore]`（文件头）。
#[test]
fn the_gpu_judge_reads_relocated_data_units_whose_source_segment_is_discarded_like_the_cpu_checker()
{
    let after = relocate_a_segment_then_publish_one_revolution(
        "gpu-judge-secondary-reads-data-units-after-discard",
        |session, devices| {
            let content: Vec<u8> = (0..(40 * 32_000usize))
                .map(|index| u8::try_from(index % 239).expect("小于 256"))
                .collect();
            session
                .publish_user_change(
                    devices,
                    UserChange::SequentialWrite(FirstFile {
                        content: &content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                    }),
                )
                .expect("顺序写四十个数据单元");
        },
        |current| cluster_segment_holding_the_slot(current.data_pointers[0].locations[0].slot),
    );
    assert!(
        after
            .units_relocated_by_the_run
            .iter()
            .any(|relocated| relocated.mapping_key_class == MAPPING_KEY_CLASS_DATA_UNIT),
        "整理搬走了数据单元"
    );
    judge_both_images_of_the_relocation(
        "the_gpu_judge_reads_relocated_data_units_whose_source_segment_is_discarded_like_the_cpu_checker",
        &after,
        &["I-9.38"],
        SecondPairingOfTheImageWithTheDiscards::NotJudgedAgain,
    );
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）读不出一棵分配记录树时记射程里判不了：覆盖写一次让分配记录树换一棵，
/// 再翻掉上一版那棵的根（两份各一个字节）。上一版的根还在回退候选集里，它那棵树里的记录一条都判不了：CPU 报那一句不适用（判到位不置），
/// GPU 也要记射程里判不了（判到位不置）；GPU 跳过那棵、只判到最新那棵的话照报成立，判到位对不上。翻字节之前那一张两棵树都读得出，
/// CPU 判成立、GPU 同判。
#[test]
fn the_gpu_judge_leaves_the_allocation_generations_of_an_unreadable_older_allocation_record_tree_not_judged_like_the_cpu_checker(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_leaves_the_allocation_generations_of_an_unreadable_older_allocation_record_tree_not_judged_like_the_cpu_checker";
    let mut pool = build_pool("gpu-judge-allocation-generations-under-an-unreadable-tree");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let root_locations_of_the_older_tree: Vec<(DeviceIdentity, SlotNumber)> = session
        .current
        .version_of_trees()
        .expect("新池新建文件之后是有树的一版")
        .allocation_record_tree
        .root_pointer()
        .locations
        .iter()
        .map(|location| (location.device, location.slot))
        .collect();
    let content: Vec<u8> = (0..2_000usize)
        .map(|index| u8::try_from(index % 199).expect("小于 256"))
        .collect();
    session
        .publish_user_change(
            &mut devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            }),
        )
        .expect("覆盖写一次");
    let root_locations_of_the_newer_tree: Vec<(DeviceIdentity, SlotNumber)> = session
        .current
        .version_of_trees()
        .expect("覆盖写之后是有树的一版")
        .allocation_record_tree
        .root_pointer()
        .locations
        .iter()
        .map(|location| (location.device, location.slot))
        .collect();
    assert_ne!(
        root_locations_of_the_older_tree, root_locations_of_the_newer_tree,
        "覆盖写换了一棵分配记录树（根落在别处）"
    );
    drop(session);
    pool.devices = Some(devices);
    let clean_image = pool.memory_pool();
    let mut image = clean_image.clone();
    for (device, slot) in &root_locations_of_the_older_tree {
        image.flip_byte(
            *device,
            DeviceOffsetInBytes(slot.0 * SLOT_BYTES),
            BYTE_FLIPPED_TO_BREAK_THE_UNIT_CHECKSUM,
        );
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let clean_table_name = format!("{TEST_NAME}/both-allocation-record-trees-readable");
    let Some(both_on_the_clean_image) =
        judge_the_image_on_both_judges(cards, &clean_table_name, &clean_image)
    else {
        return;
    };
    let Some(both) = judge_the_image_on_both_judges(cards, TEST_NAME, &image) else {
        return;
    };
    assert_eq!(
        verdict_named(&both_on_the_clean_image.cpu_verdicts, "I-3.10"),
        &InvariantVerdict::Holds,
        "{clean_table_name}：两棵分配记录树都读得出时 CPU 池级 checker 的 I-3.10 判成立"
    );
    assert_eq!(
        verdict_named(&both.cpu_verdicts, "I-3.10"),
        &InvariantVerdict::NotApplicable(
            ALLOCATION_GENERATIONS_OF_AN_UNREADABLE_ALLOCATION_RECORD_TREE_ARE_NOT_JUDGED
        ),
        "上一版那棵分配记录树读不出：CPU 池级 checker 的 I-3.10 射程里有一部分没判，报不适用、不报成立"
    );
    for (table_name, judged) in [
        (clean_table_name.as_str(), &both_on_the_clean_image),
        (TEST_NAME, &both),
    ] {
        assert_the_gpu_judge_agrees_with_the_cpu_pool_checker(table_name, judged);
    }
}
