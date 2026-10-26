//! 选段打分（D26（后台整理与放置回收） 已定项 13 第 1 条点名的三段里的「丙」）：两份独立写的 WGSL 核（第 8 条，不共用核函数）对同一批聚簇段
//! 各数一次活槽数，逐段交回。这里是两份核共用的缓冲布局、打包、回读解码、CPU 参照与比对，都是纯函数；起 GPU 的在 `background_gpu`。
//!
//! 放上 GPU 的是核心层 `singlefs_core::compaction_plan::live_slots_per_cluster_segment_by_the_records` 那一步：单元区那块盘上每个整段的活槽数
//! （未释放记录罩住、落在这一段里的槽数；几条记录罩住同一个槽各算一次；落在单元区之前与末尾凑不满一段的槽不算）。
//! 判的全在核里：哪条记录在这块盘上、释放了没有、罩住这一段的哪几个槽。CPU 只按次序打包，另按（盘、槽号）排一份序号给二分查的那一份核。
//! CPU 参照（[`cpu_reference_live_slots`]）就是核心层那个函数；整段的上下界取核心层的 `CompactionRegion`（一个整段就是一个整理区域的形状）。
//!
//! 缓冲布局（两份核按同一个绑定号读写，各有各的输出缓冲；逐条走的那一份只绑 0、1、2、4，二分查的那一份五个都绑）：
//! - 绑定 0，uniform：这一段的段数、记录数、单元区那块盘的盘号、一个不用的字（缓冲占 [`LIVE_SLOT_PARAMETER_BUFFER_BYTES`]）。
//! - 绑定 1，只读存储：每个段一项，[`SEGMENT_BOUND_WORDS`] 个 `u32`：头一个槽的低 32 位与高 32 位、这一段之后第一个槽的低 32 位与高 32 位。
//! - 绑定 2，只读存储：这一批的分配记录每条一项，[`ALLOCATION_RECORD_WORDS`] 个 `u32`：盘、槽号低 32 位、高 32 位、跨度、释放了没有（1 是释放了），
//!   按交进来的次序；哪块盘的都交，核自己筛。
//! - 绑定 3，只读存储：记录在表里的序号，按（盘、槽号、序号）升序。
//! - 绑定 4，读写存储：每个段一项结果，[`LIVE_SLOT_OUTCOME_WORDS`] 个 `u32`：活槽数的低 32 位、高 32 位、判定字（1 是数完了）。
//!   开算之前整块填 [`LIVE_SLOT_OUTCOME_NOT_WRITTEN_FILL_WORD`]：核没写到的那一段读回来判定字认不得，按 GPU 出错交回。
//!
//! 记录一条都没有时绑定 2、3 照样各放一项全 0 的（wgpu 不许长度为 0 的绑定），条数照参数里的数。
//! 整份记录表每段都交一份（同区域扫描那一段的整份映射）：一批按段切，记录表不切。

use singlefs_core::address::SlotNumber;
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::ClusterSegmentEntryKey;
use singlefs_core::compaction_plan::{
    cluster_segment_entry_keys_of_the_unit_area, live_slots_per_cluster_segment_by_the_records,
    UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::CompactionRegion;

use crate::checksum_kernels::UnrecognizedVerdictWord;
use crate::memory_limit_and_chunks::ChunkFootprint;
use crate::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, ReconciledBatch, TwoKernelOutcomesOfAnEntry,
};

/// 逐条走遍记录、按区间相交算长度的那一份核。
pub const WALKING_EVERY_RECORD_KERNEL_SOURCE: &str =
    include_str!("live_slot_kernel_walking_every_record.wgsl");
pub const WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT: &str = "count_live_slots_walking_every_record";
/// 在按（盘、槽号）排好的序号上二分、逐槽数的那一份核。
pub const SEARCHING_SORTED_RECORDS_KERNEL_SOURCE: &str =
    include_str!("live_slot_kernel_searching_sorted_records.wgsl");
pub const SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT: &str =
    "count_live_slots_searching_the_sorted_records";

/// 两份核的工作组都是 64 个调用（两份 WGSL 里的 `@workgroup_size(64)`），每个段一个调用。
pub const LIVE_SLOT_INVOCATIONS_PER_WORKGROUP: u32 = 64;
/// 参数缓冲：四个 `u32`。
pub const LIVE_SLOT_PARAMETER_BUFFER_BYTES: u64 = 16;
pub const SEGMENT_BOUND_WORDS: usize = 4;
pub const SEGMENT_BOUND_BYTES: u64 = 16;
pub const ALLOCATION_RECORD_WORDS: usize = 5;
pub const LIVE_SLOT_OUTCOME_WORDS: usize = 3;
pub const LIVE_SLOT_OUTCOME_BYTES: u64 = 12;
/// 结果缓冲开算之前的填充字：判定字只会被核写成 [`VERDICT_WORD_COUNTED`]。
pub const LIVE_SLOT_OUTCOME_NOT_WRITTEN_FILL_WORD: u32 = 0xFFFF_FFFF;
/// 判定字：这一段数完了（两份 WGSL 里各写一份同值的常量）。
const VERDICT_WORD_COUNTED: u32 = 1;
/// 释放了没有那一个字（两份 WGSL 里各写一份同值的常量）。
const RELEASED_WORD_OF_A_RELEASED_RECORD: u32 = 1;
const RELEASED_WORD_OF_A_LIVE_RECORD: u32 = 0;
/// 两份核各一个结果缓冲与一个回读暂存缓冲。
const LIVE_SLOT_OUTCOME_AND_READBACK_BUFFERS: u64 = 4;

fn low_and_high_words_of_the_slot(slot: SlotNumber) -> [u32; 2] {
    [
        u32::try_from(slot.0 & u64::from(u32::MAX)).expect("取了低 32 位"),
        u32::try_from(slot.0 >> 32).expect("盘上槽号是 6 字节，高 32 位装得进 u32"),
    ]
}

/// 交给两份核的分配记录：记录表与按（盘、槽号）排的序号（见本模块文件头的布局）。一批建一次，同一批的各段共用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocationRecordTablesForTheKernels {
    /// 记录表的字；没有记录时是一项全 0 的。
    pub record_words: Vec<u32>,
    /// 按（盘、槽号、序号）排的序号；没有记录时是一个 0。
    pub record_positions_by_device_and_slot: Vec<u32>,
    pub record_count: u32,
}

impl AllocationRecordTablesForTheKernels {
    /// 走一遍记录建两样。
    ///
    /// # Panics
    /// 记录数装不进 `u32`：一批的分配记录远少于此（一块 64 TiB 的盘满打满算 2^32 个 16 KiB 槽）。
    #[must_use]
    pub fn of_the_allocation_records(allocation_records: &[AllocationRecord]) -> Self {
        let mut record_words =
            Vec::with_capacity(allocation_records.len() * ALLOCATION_RECORD_WORDS);
        for record in allocation_records {
            let [slot_low, slot_high] = low_and_high_words_of_the_slot(record.slot);
            let released_word = if record.is_released {
                RELEASED_WORD_OF_A_RELEASED_RECORD
            } else {
                RELEASED_WORD_OF_A_LIVE_RECORD
            };
            record_words.extend_from_slice(&[
                record.device.0,
                slot_low,
                slot_high,
                u32::from(record.span_slots),
                released_word,
            ]);
        }
        let record_count =
            u32::try_from(allocation_records.len()).expect("一批的分配记录数装得进 u32");
        let mut record_positions_by_device_and_slot: Vec<u32> = (0..record_count).collect();
        record_positions_by_device_and_slot.sort_by_key(|position| {
            let record = &allocation_records[usize::try_from(*position).expect("u32 装得进 usize")];
            (record.device.0, record.slot.0, *position)
        });
        if record_words.is_empty() {
            record_words.resize(ALLOCATION_RECORD_WORDS, 0);
            record_positions_by_device_and_slot.push(0);
        }
        Self {
            record_words,
            record_positions_by_device_and_slot,
            record_count,
        }
    }

    /// 两个缓冲在卡上合占的字节数。
    #[must_use]
    pub fn bytes_on_the_gpu(&self) -> u64 {
        let words = self.record_words.len() + self.record_positions_by_device_and_slot.len();
        4 * u64::try_from(words).expect("装得进 u64")
    }

    /// 两个缓冲里大的那一个的字节数：它也要装得进这张卡的绑定上限。
    #[must_use]
    pub fn largest_buffer_bytes(&self) -> u64 {
        let words = self
            .record_words
            .len()
            .max(self.record_positions_by_device_and_slot.len());
        4 * u64::try_from(words).expect("装得进 u64")
    }
}

/// 选段打分一段在卡上占的：参数缓冲与整份记录表（两个缓冲，每段都交一份）；每个段一项上下界（当这一段的「输入」算）、
/// 两份核各一项结果与一项回读暂存。按条排的最大缓冲（输入之外）是结果那一个。
#[must_use]
pub fn live_slot_chunk_footprint(
    record_tables: &AllocationRecordTablesForTheKernels,
) -> ChunkFootprint {
    ChunkFootprint {
        bytes_per_chunk_besides_the_entries: LIVE_SLOT_PARAMETER_BUFFER_BYTES
            + record_tables.bytes_on_the_gpu(),
        bytes_per_entry_besides_its_input: LIVE_SLOT_OUTCOME_AND_READBACK_BUFFERS
            * LIVE_SLOT_OUTCOME_BYTES,
        bytes_per_entry_in_the_largest_entry_buffer: LIVE_SLOT_OUTCOME_BYTES,
    }
}

/// 每个段的上下界：头一个槽与这一段之后第一个槽，取核心层一个整段的整理区域的形状。
///
/// # Panics
/// 某个条目键的起点不是聚簇段的起点：条目键由核心层按单元区的整段给（`cluster_segment_entry_keys_of_the_unit_area`），单元区起点落在段边界上。
#[must_use]
pub fn segment_bound_words(segments: &[ClusterSegmentEntryKey]) -> Vec<u32> {
    segments
        .iter()
        .flat_map(|segment| {
            let region =
                CompactionRegion::of_one_cluster_segment(segment.device, segment.start_slot)
                    .expect("条目键由核心层按单元区的整段给：起点落在聚簇段边界上");
            let [start_low, start_high] = low_and_high_words_of_the_slot(region.start_slot());
            let [end_low, end_high] = low_and_high_words_of_the_slot(region.end_slot());
            [start_low, start_high, end_low, end_high]
        })
        .collect()
}

/// 参数缓冲的字节。
#[must_use]
pub fn live_slot_parameter_bytes(
    segment_count: u32,
    record_tables: &AllocationRecordTablesForTheKernels,
    unit_area: UnitAreaOfOneDevice,
) -> Vec<u8> {
    [
        segment_count,
        record_tables.record_count,
        unit_area.device.0,
        0,
    ]
    .iter()
    .flat_map(|word| word.to_le_bytes())
    .collect()
}

/// 结果缓冲开算之前的内容：每个字都是填充字。
#[must_use]
pub fn live_slot_outcome_buffer_before_the_kernel_writes(segment_count: usize) -> Vec<u8> {
    std::iter::repeat_n(
        LIVE_SLOT_OUTCOME_NOT_WRITTEN_FILL_WORD.to_le_bytes(),
        segment_count * LIVE_SLOT_OUTCOME_WORDS,
    )
    .flatten()
    .collect()
}

/// 把读回的结果字节解成每个段一个活槽数。
///
/// # Errors
/// 某一段的判定字不是 [`VERDICT_WORD_COUNTED`]（核没写到它，还是填充字，或写坏了）。
///
/// # Panics
/// 字节数不是每段 [`LIVE_SLOT_OUTCOME_BYTES`] 的整数倍：回读缓冲按段数建，长度对不上是外壳的 bug。
pub fn decoded_live_slot_outcomes(
    readback_bytes: &[u8],
) -> Result<Vec<u64>, UnrecognizedVerdictWord> {
    let (outcome_records, leftover_bytes) =
        readback_bytes.as_chunks::<{ 4 * LIVE_SLOT_OUTCOME_WORDS }>();
    assert!(
        leftover_bytes.is_empty(),
        "回读的字节数是每段 12 字节的整数倍"
    );
    outcome_records
        .iter()
        .enumerate()
        .map(|(entry_position, outcome_record)| {
            let (word_bytes, _no_leftover) = outcome_record.as_chunks::<4>();
            let [live_slots_low, live_slots_high, verdict_word] =
                std::array::from_fn(|word_position| u32::from_le_bytes(word_bytes[word_position]));
            if verdict_word != VERDICT_WORD_COUNTED {
                return Err(UnrecognizedVerdictWord {
                    entry_position,
                    verdict_word,
                });
            }
            Ok(u64::from(live_slots_low) | (u64::from(live_slots_high) << 32))
        })
        .collect()
}

/// CPU 那一份实现（连续不确定的段交它判；对拍也拿它当参照）：核心层的 `live_slots_per_cluster_segment_by_the_records`，
/// 按段起点从小到大一个整段一个。
#[must_use]
pub fn cpu_reference_live_slots(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
) -> Vec<u64> {
    live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area)
}

/// 选段打分一批的比对（第 8 条）：两份核数出的活槽数交进来，逐段比、更新计数；连续不确定到 N 轮的段交 CPU 参照现判
/// （整块单元区的 CPU 参照只在头一次要它时数一遍）。这一批出错就原样交回出错、计数不动、CPU 参照不调。
///
/// # Errors
/// `both_kernel_live_slots` 是 `Err` 时交回它。
///
/// # Panics
/// `segments` 不是 `unit_area` 的整段按起点升序一个不漏（核心层 `BackgroundGpuCompute::count_live_slots_per_cluster_segment` 的约定）；
/// 两份核交回的段数与 `segments` 不等。
pub fn reconciled_live_slots_of_the_segments<BatchFailure>(
    allocation_records: &[AllocationRecord],
    unit_area: UnitAreaOfOneDevice,
    segments: &[ClusterSegmentEntryKey],
    both_kernel_live_slots: Result<Vec<(u64, u64)>, BatchFailure>,
    uncertainty: &mut ConsecutiveUncertaintyCounts<ClusterSegmentEntryKey>,
) -> Result<ReconciledBatch<u64>, BatchFailure> {
    assert_eq!(
        segments,
        cluster_segment_entry_keys_of_the_unit_area(unit_area).as_slice(),
        "丙段交进来的是单元区的整段、按起点升序、一段一个"
    );
    let both_kernel_outcomes = both_kernel_live_slots.map(|live_slots| {
        assert_eq!(live_slots.len(), segments.len(), "两份核每段各交一个数");
        segments
            .iter()
            .zip(live_slots)
            .map(
                |(segment, (walking, searching))| TwoKernelOutcomesOfAnEntry {
                    entry_key: *segment,
                    outcome_by_one_kernel: walking,
                    outcome_by_the_independently_written_kernel: searching,
                },
            )
            .collect()
    });
    let mut cpu_reference: Option<Vec<u64>> = None;
    uncertainty.reconcile_unless_the_batch_failed(both_kernel_outcomes, |entry_position| {
        cpu_reference.get_or_insert_with(|| cpu_reference_live_slots(allocation_records, unit_area))
            [entry_position]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
    use singlefs_core::allocator::{AllocationRecordQuarantineMark, UnitAreaStart};

    const UNIT_AREA_DEVICE: DeviceIdentity = DeviceIdentity(0);
    const OTHER_DEVICE: DeviceIdentity = DeviceIdentity(1);

    fn record(
        device: DeviceIdentity,
        slot: u64,
        span_slots: u16,
        is_released: bool,
    ) -> AllocationRecord {
        AllocationRecord {
            device,
            slot: SlotNumber(slot),
            span_slots,
            generation: CheckpointTxg(3),
            is_released,
            quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
        }
    }

    /// 默认环长下的单元区，三个整段加末尾 10 个槽。
    fn unit_area_of_three_segments() -> UnitAreaOfOneDevice {
        let segment_slots = CompactionRegion::of_one_cluster_segment(
            UNIT_AREA_DEVICE,
            UnitAreaStart::of_the_default_journal_ring().slot(),
        )
        .expect("单元区起点落在段边界上")
        .slot_count();
        UnitAreaOfOneDevice {
            device: UNIT_AREA_DEVICE,
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 3 * segment_slots + 10,
        }
    }

    #[test]
    fn allocation_records_are_listed_in_order_with_their_released_word_and_sorted_by_device_and_slot(
    ) {
        let wide_slot = (5_u64 << 32) | 9;
        let tables = AllocationRecordTablesForTheKernels::of_the_allocation_records(&[
            record(UNIT_AREA_DEVICE, 900, 2, false),
            record(OTHER_DEVICE, 100, 1, false),
            record(UNIT_AREA_DEVICE, wide_slot, 3, true),
            record(UNIT_AREA_DEVICE, 100, 1, false),
        ]);
        assert_eq!(
            tables.record_words,
            vec![
                0, 900, 0, 2, 0, //
                1, 100, 0, 1, 0, //
                0, 9, 5, 3, 1, //
                0, 100, 0, 1, 0,
            ],
            "盘、槽号低 32 位、高 32 位、跨度、释放了没有，按交进来的次序"
        );
        assert_eq!(
            tables.record_positions_by_device_and_slot,
            vec![3, 0, 2, 1],
            "按（盘、槽号）升序：盘 0 的槽 100、900、2^34 多，再盘 1"
        );
        assert_eq!(tables.record_count, 4);
    }

    #[test]
    fn records_at_the_same_device_and_slot_are_sorted_by_their_position() {
        let tables = AllocationRecordTablesForTheKernels::of_the_allocation_records(&[
            record(UNIT_AREA_DEVICE, 70, 1, false),
            record(UNIT_AREA_DEVICE, 70, 2, true),
            record(UNIT_AREA_DEVICE, 60, 1, false),
        ]);
        assert_eq!(
            tables.record_positions_by_device_and_slot,
            vec![2, 0, 1],
            "同盘同槽的按在表里的序号"
        );
    }

    #[test]
    fn an_empty_record_table_still_has_one_item_in_both_buffers() {
        let tables = AllocationRecordTablesForTheKernels::of_the_allocation_records(&[]);
        assert_eq!(tables.record_words, vec![0; ALLOCATION_RECORD_WORDS]);
        assert_eq!(tables.record_positions_by_device_and_slot, vec![0]);
        assert_eq!(tables.record_count, 0);
        assert_eq!(
            tables.bytes_on_the_gpu(),
            4 * u64::try_from(ALLOCATION_RECORD_WORDS + 1).expect("小数")
        );
    }

    #[test]
    fn segment_bounds_are_the_first_slot_and_the_slot_past_each_cluster_segment() {
        let high_start = SlotNumber((1_u64 << 32) + 64);
        let words = segment_bound_words(&[
            ClusterSegmentEntryKey {
                device: UNIT_AREA_DEVICE,
                start_slot: SlotNumber(128),
            },
            ClusterSegmentEntryKey {
                device: UNIT_AREA_DEVICE,
                start_slot: high_start,
            },
        ]);
        assert_eq!(
            words,
            vec![128, 0, 192, 0, 64, 1, 128, 1],
            "头一个槽与之后第一个槽各按低 32 位、高 32 位"
        );
    }

    #[test]
    fn parameters_carry_the_segment_count_the_record_count_and_the_device_of_the_unit_area() {
        let tables = AllocationRecordTablesForTheKernels::of_the_allocation_records(&[record(
            OTHER_DEVICE,
            1,
            1,
            false,
        )]);
        let unit_area = UnitAreaOfOneDevice {
            device: DeviceIdentity(7),
            start: unit_area_of_three_segments().start,
            slots: unit_area_of_three_segments().slots,
        };
        assert_eq!(
            live_slot_parameter_bytes(3, &tables, unit_area),
            [3_u32, 1, 7, 0]
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<u8>>()
        );
    }

    fn outcome_bytes(outcome_words: [u32; LIVE_SLOT_OUTCOME_WORDS]) -> Vec<u8> {
        outcome_words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }

    #[test]
    fn readback_records_decode_into_sixty_four_bit_live_slot_counts() {
        let mut readback = outcome_bytes([64, 0, VERDICT_WORD_COUNTED]);
        readback.extend(outcome_bytes([5, 2, VERDICT_WORD_COUNTED]));
        assert_eq!(
            decoded_live_slot_outcomes(&readback),
            Ok(vec![64, (2_u64 << 32) | 5]),
            "低 32 位、高 32 位拼成一个数"
        );
    }

    #[test]
    fn an_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word() {
        let mut readback = outcome_bytes([1, 0, VERDICT_WORD_COUNTED]);
        readback.extend(live_slot_outcome_buffer_before_the_kernel_writes(1));
        assert_eq!(
            decoded_live_slot_outcomes(&readback),
            Err(UnrecognizedVerdictWord {
                entry_position: 1,
                verdict_word: LIVE_SLOT_OUTCOME_NOT_WRITTEN_FILL_WORD
            }),
            "位置 1 的那一段还是填充字：核没写到它"
        );
    }

    /// 第 8 条：两份核对中间那一段数得不一样，这一轮不确定、不问 CPU；下一轮又不一样，交 CPU 参照现判，按核心层数的那个数。
    #[test]
    fn a_segment_counted_differently_by_the_two_kernels_is_uncertain_once_and_judged_by_the_cpu_reference_the_next_round(
    ) {
        let unit_area = unit_area_of_three_segments();
        let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
        let middle_segment_start = segments[1].start_slot.0;
        let records = vec![
            record(UNIT_AREA_DEVICE, middle_segment_start, 2, false),
            record(UNIT_AREA_DEVICE, middle_segment_start + 5, 1, false),
        ];
        let reference = cpu_reference_live_slots(&records, unit_area);
        assert_eq!(reference, vec![0, 3, 0], "核心层数的：中间那一段 3 个活槽");
        let mut uncertainty = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
        let disagreeing_on_the_middle_segment = || Ok::<_, ()>(vec![(0, 0), (3, 4), (0, 0)]);
        let first_round = reconciled_live_slots_of_the_segments(
            &records,
            unit_area,
            &segments,
            disagreeing_on_the_middle_segment(),
            &mut uncertainty,
        )
        .expect("这一批没出错");
        assert_eq!(
            (
                first_round.confirmed_entries,
                first_round.uncertain_entries,
                first_round.entries_judged_by_the_cpu_reference
            ),
            (2, 1, 0),
            "头一轮：不确定一段"
        );
        let second_round = reconciled_live_slots_of_the_segments(
            &records,
            unit_area,
            &segments,
            disagreeing_on_the_middle_segment(),
            &mut uncertainty,
        )
        .expect("这一批没出错");
        assert_eq!(
            second_round.decisions[1],
            crate::two_kernel_reconciliation::EntryDecision::JudgedByTheCpuReference(3),
            "连续第 2 轮不确定：按核心层数的 3 处理"
        );
    }

    #[test]
    #[should_panic(expected = "丙段交进来的是单元区的整段")]
    fn segments_other_than_every_full_segment_of_the_unit_area_panic() {
        let unit_area = unit_area_of_three_segments();
        let mut segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
        segments.pop();
        let _reconciled = reconciled_live_slots_of_the_segments(
            &[],
            unit_area,
            &segments,
            Ok::<_, ()>(vec![(0, 0), (0, 0)]),
            &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain(),
        );
    }

    /// 两份核过得了 wgpu 自带的着色器校验（naga 的 WGSL 前端与校验器，起管线时 wgpu 走的同一步），入口点在、是计算着色器。不要 GPU。
    #[test]
    fn both_live_slot_kernels_pass_the_shader_validator() {
        for (kernel_source, entry_point) in [
            (
                WALKING_EVERY_RECORD_KERNEL_SOURCE,
                WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT,
            ),
            (
                SEARCHING_SORTED_RECORDS_KERNEL_SOURCE,
                SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT,
            ),
        ] {
            let module =
                wgpu::naga::front::wgsl::parse_str(kernel_source).unwrap_or_else(|error| {
                    panic!(
                        "{entry_point} 那一份解析不成：{}",
                        error.emit_to_string(kernel_source)
                    )
                });
            wgpu::naga::valid::Validator::new(
                wgpu::naga::valid::ValidationFlags::all(),
                wgpu::naga::valid::Capabilities::empty(),
            )
            .validate(&module)
            .unwrap_or_else(|error| {
                panic!(
                    "{entry_point} 那一份过不了校验：{}",
                    error.emit_to_string(kernel_source)
                )
            });
            assert!(
                module.entry_points.iter().any(|module_entry_point| {
                    module_entry_point.name == entry_point
                        && module_entry_point.stage == wgpu::naga::ShaderStage::Compute
                }),
                "{entry_point} 是这一份里的计算入口"
            );
        }
    }

    #[test]
    fn the_two_kernels_are_separate_sources_with_their_own_entry_points() {
        assert!(WALKING_EVERY_RECORD_KERNEL_SOURCE
            .contains(&format!("fn {WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT}(")));
        assert!(SEARCHING_SORTED_RECORDS_KERNEL_SOURCE.contains(&format!(
            "fn {SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT}("
        )));
        assert!(
            !SEARCHING_SORTED_RECORDS_KERNEL_SOURCE
                .contains(WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT)
                && !WALKING_EVERY_RECORD_KERNEL_SOURCE
                    .contains(SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT),
            "两份核不互相引用"
        );
    }
}
