//! 确认丢失那一形的观测表、末段的位、按类分区与恢复读过的范围。

use super::*;

// ───── Q5 主取样点按等价类判（登记修订 2 第 2.1 节候选二；修订 3 定它） ─────
//
// 态集合与读法 7 原句相同（末段整写子集全枚举、可撕裂的写另加撕裂态）；每态的 O1 两个崩溃子列与 RefFS 一票不逐态算，由等价类的代表推出。
// 两态等价 ⇔ 键里那几个写（第 2 次调用的写 K，加上类核对 ③ 细分移进来的第 3 次调用的写）取法相同，且留在类里的第 3 次调用的写（C）
// 「至少落了一个」这一位相同。代表是「C 一个都没落」那一态；「C 至少落一个」那一类按代表推，另把「C 全落」那一态照原路径判一遍对照（类核对 ④）。

/// 「等价类对拍」那一形截到第 3 次调用发出的前几个写之后：k = 6 = C 的前三对镜像写，72 × 2⁶ = 4 608 态（修订 5 第 5.2 节丙；
/// 修订 2 第 2.1 节候选二原来是 k = 2、72 × 4 = 288 态，只罩一个单元的两份镜像）。
pub(super) const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 6;

/// [`ObservationContext`] 借的几张表，由一条流现算（与 [`export_prepared_stream`] 里同一套算法；按等价类判与「等价类对拍」用）。
pub(super) struct ObservationTables {
    pub(super) tables: CrashStateEnumerationTables,
    pub(super) facts: StateMasks,
    pub(super) torn_image_index_by_write: BTreeMap<usize, usize>,
    pub(super) reference_filesystem: ReferenceFileSystemStream,
}

impl ObservationTables {
    pub(super) fn of(stream: &PreparedStream) -> Self {
        let tables = crash_state_enumeration_tables(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        );
        let facts = StateMasks::of(&tables, &stream.segments);
        let torn_image_index_by_write = tables
            .torn_images
            .iter()
            .map(|row| (row.write_index, row.torn_image_index))
            .collect();
        let reference_filesystem = reference_filesystem_stream_of(
            &stream.calls,
            &stream.operations,
            &stream.writes,
            &stream.stream_indexes,
        );
        Self {
            tables,
            facts,
            torn_image_index_by_write,
            reference_filesystem,
        }
    }

    pub(super) fn context<'tables>(
        &'tables self,
        stream: &'tables PreparedStream,
    ) -> ObservationContext<'tables> {
        ObservationContext {
            stream,
            tables: &self.tables,
            facts: &self.facts,
            torn_image_index_by_write: &self.torn_image_index_by_write,
            reference_filesystem: &self.reference_filesystem,
            call_of_write: call_index_of_each_write(stream),
            piece_panic_injection: None,
        }
    }
}

/// 末段的写按崩溃点重放的混合进制排开（`crash.rs` 的 `CrashStatePlan::assign_landings`）：段内序号 = Σ 数字 × 权，末段第一个写是最低位，
/// 进制是它能取几态（整写 2、可撕裂的 3），数字 0 是「没落」、最大是「持久」（可撕裂的写 1 是撕裂）。末段之后没有段，
/// 段内序号取满（每位取最大）就是最后那个全持久态，所以末段与全持久态合起来恰是 `first_ordinal..first_ordinal + 组合数`。
pub(super) struct LastSegmentDigits {
    pub(super) first_ordinal: u64,
    pub(super) write_indexes: Vec<usize>,
    radixes: Vec<u64>,
    weights: Vec<u64>,
}

impl LastSegmentDigits {
    pub(super) fn of(stream: &PreparedStream, tables: &CrashStateEnumerationTables) -> Self {
        let write_indexes = stream.segments.last().cloned().unwrap_or_default();
        let first_ordinal = tables
            .state_ranges_by_segment
            .last()
            .map_or(0, |state_range| state_range.start);
        let radixes: Vec<u64> = write_indexes
            .iter()
            .map(|write_index| {
                if tables.is_tearable_by_write[*write_index] {
                    3
                } else {
                    2
                }
            })
            .collect();
        let mut weights = Vec::with_capacity(radixes.len());
        let mut weight = 1u64;
        for radix in &radixes {
            weights.push(weight);
            weight = weight
                .checked_mul(*radix)
                .expect("末段的组合数装得进 u64（崩溃点重放的计划同样要求）");
        }
        assert_eq!(
            first_ordinal + weight,
            tables.state_count,
            "末段是最后一段：末段的组合数加上它的起点，恰是整条流的态数（最后那个全持久态是段内序号取满的那一个）"
        );
        Self {
            first_ordinal,
            write_indexes,
            radixes,
            weights,
        }
    }

    /// 末段第 `position` 个写在 `ordinal` 那一态上的数字。
    pub(super) fn digit_of(&self, ordinal: u64, position: usize) -> u64 {
        ((ordinal - self.first_ordinal) / self.weights[position]) % self.radixes[position]
    }

    /// 这几个位置上的写各取几态、全部组合的个数。
    pub(super) fn product_of_the_radixes(&self, positions: &[usize]) -> u64 {
        positions
            .iter()
            .try_fold(1u64, |product, position| {
                product.checked_mul(self.radixes[*position])
            })
            .expect("末段的组合数装得进 u64")
    }
}

/// 留在类里的第 3 次调用的写（C）在一态上落没落。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum InFlightWritesLanded {
    NoneLanded,
    AtLeastOneLanded,
}

impl InFlightWritesLanded {
    pub(super) fn word(self) -> &'static str {
        match self {
            Self::NoneLanded => "none",
            Self::AtLeastOneLanded => "at_least_one",
        }
    }
}

/// 一态落在哪一类：键里那几个写的取法（按键里的位置排的混合进制序号）与 C 落没落。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ConfirmedLossClassKey {
    pub(super) keyed_assignment: u64,
    pub(super) in_flight_writes_landed: InFlightWritesLanded,
}

/// 等价关系的分法：键里的写（K 加上细分移进来的 C 写）与留在类里的 C 写，都是末段里的位置（[`LastSegmentDigits`] 的下标），升序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConfirmedLossPartition {
    pub(super) keyed_positions: Vec<usize>,
    pub(super) class_positions: Vec<usize>,
}

impl ConfirmedLossPartition {
    /// 细分之前的起点分法：键 = 末段里不属于 `in_flight_call` 的写，类里 = 属于它的写（`call_of_write` 是 [`call_index_of_each_write`] 的交回）。
    pub(super) fn before_any_refinement(
        call_of_write: &[Option<usize>],
        digits: &LastSegmentDigits,
        in_flight_call: usize,
    ) -> Self {
        let (class_positions, keyed_positions): (Vec<usize>, Vec<usize>) =
            (0..digits.write_indexes.len()).partition(|position| {
                call_of_write[digits.write_indexes[*position]] == Some(in_flight_call)
            });
        Self {
            keyed_positions,
            class_positions,
        }
    }

    /// 键里那几个写有几种取法（每种取法两类：C 一个没落、C 至少落一个；C 一个都不剩时只有前一类）。
    pub(super) fn keyed_assignments(&self, digits: &LastSegmentDigits) -> u64 {
        digits.product_of_the_radixes(&self.keyed_positions)
    }

    /// 键里第 `keyed_assignment` 种取法、C 一个都没落的那一态（这一种取法的代表）。
    pub(super) fn representative_ordinal(
        &self,
        digits: &LastSegmentDigits,
        keyed_assignment: u64,
    ) -> u64 {
        let mut remaining = keyed_assignment;
        let mut ordinal = digits.first_ordinal;
        for position in &self.keyed_positions {
            ordinal += (remaining % digits.radixes[*position]) * digits.weights[*position];
            remaining /= digits.radixes[*position];
        }
        assert_eq!(
            remaining, 0,
            "取法序号 {keyed_assignment} 超出键里那几个写的组合数"
        );
        ordinal
    }

    /// 同一种取法、C 全落的那一态。
    pub(super) fn fully_landed_ordinal(
        &self,
        digits: &LastSegmentDigits,
        keyed_assignment: u64,
    ) -> u64 {
        self.representative_ordinal(digits, keyed_assignment)
            + self
                .class_positions
                .iter()
                .map(|position| (digits.radixes[*position] - 1) * digits.weights[*position])
                .sum::<u64>()
    }

    /// 一态落在哪一类。
    pub(super) fn class_of_ordinal(
        &self,
        digits: &LastSegmentDigits,
        ordinal: u64,
    ) -> ConfirmedLossClassKey {
        let mut keyed_assignment = 0u64;
        let mut place = 1u64;
        for position in &self.keyed_positions {
            keyed_assignment += digits.digit_of(ordinal, *position) * place;
            place *= digits.radixes[*position];
        }
        let in_flight_writes_landed = if self
            .class_positions
            .iter()
            .any(|position| digits.digit_of(ordinal, *position) != 0)
        {
            InFlightWritesLanded::AtLeastOneLanded
        } else {
            InFlightWritesLanded::NoneLanded
        };
        ConfirmedLossClassKey {
            keyed_assignment,
            in_flight_writes_landed,
        }
    }

    /// 「C 至少落一个」那一类的态数：C 每个写的取法数之积，减去一个都没落的那一态（C 都只取两态时是 2 的 C 写数次方减一）。
    pub(super) fn states_with_the_in_flight_writes_landed(
        &self,
        digits: &LastSegmentDigits,
    ) -> u64 {
        digits.product_of_the_radixes(&self.class_positions) - 1
    }

    /// 「C 至少落一个」那一类按落了几个 C 写排开的态数（下标 j 是落了几个，第 0 格是 0）：Π (1 + (进制 − 1)·x) 的各项系数去掉常数项，
    /// C 都只取两态时第 j 格是 C(C 写数, j)。撕裂算落了（与 `q5_trajectory` 按落地串数「不是 0」同一口径）。
    pub(super) fn states_by_in_flight_writes_landed(&self, digits: &LastSegmentDigits) -> Vec<u64> {
        let mut coefficients = vec![1u64];
        for position in &self.class_positions {
            let landed_choices = digits.radixes[*position] - 1;
            let mut next = vec![0u64; coefficients.len() + 1];
            for (landed, states) in coefficients.iter().enumerate() {
                next[landed] += states;
                next[landed + 1] += states * landed_choices;
            }
            coefficients = next;
        }
        coefficients[0] = 0;
        coefficients
    }

    /// 要照原路径判的态数：每种取法的代表，加上 C 还剩写时同一取法的「C 全落」那一态。
    pub(super) fn judged_state_count(&self, digits: &LastSegmentDigits) -> u64 {
        let per_assignment = if self.class_positions.is_empty() {
            1
        } else {
            2
        };
        self.keyed_assignments(digits)
            .checked_mul(per_assignment)
            .expect("要判的态数装得进 u64")
    }
}

/// 恢复读过的一段字节（盘、起点、终点，都按字节；终点不含），与这一段是从哪来的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RecordedReadRange {
    pub(super) device: DeviceIdentity,
    start_in_bytes: u64,
    end_in_bytes: u64,
    pub(super) source: RecordedReadSource,
}

/// 一段记读从哪来：一次 `read`，或 journal 位置提示记的整个环（修订 5 第 5.4 节 U3：类核对 ③ 照旧用全部段，用例分得出提示那一段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecordedReadSource {
    Read,
    JournalRecordOffsetsHint,
}

/// 类核对 ③ 的读者：包在崩溃镜像外面、只转发、只记（做法照 `e161_crash_state_dedup_and_time_split.rs` 的 `RecordingReader`，
/// 本装置另写一份、不引它）。答案一律原样交回。每一次读记它的字节段；journal 位置提示的答由环里落了的写定，记成读了整个环
/// （宽了只会多细分，不会漏）；盘身份与盘的字节数只看基镜像，与哪几个写落了无关，不记。
struct ReadRecordingPoolReader<'reader, 'image> {
    pub(super) image: &'reader CrashImage<'image>,
    pub(super) read_ranges: RefCell<Vec<RecordedReadRange>>,
}

impl PoolReader for ReadRecordingPoolReader<'_, '_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.image)
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.image, device)
    }
    fn physical_block_size_probed_on_the_device(
        &self,
        device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        PoolReader::physical_block_size_probed_on_the_device(self.image, device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.read_ranges.borrow_mut().push(RecordedReadRange {
            device,
            start_in_bytes: offset.0,
            end_in_bytes: offset.0 + u64::try_from(length).expect("读长装得进 u64"),
            source: RecordedReadSource::Read,
        });
        PoolReader::read(self.image, device, offset, length)
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.read_ranges.borrow_mut().push(RecordedReadRange {
            device,
            start_in_bytes: ring_start.0,
            end_in_bytes: ring_start.0 + ring_bytes,
            source: RecordedReadSource::JournalRecordOffsetsHint,
        });
        PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes)
    }
}

/// 看 journal 那一遍恢复在一态上的结局：交回了报告，或 panic 了（panic 的消息）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ConsultedRecovery {
    Reported(Box<RecoveryReport>),
    Panicked(String),
}

impl ConsultedRecovery {
    /// 一个判过的态上记下的那一遍（只在留了整份报告的那一趟有）：恢复 panic 的态是 `Panicked`；没留报告的交 `None`。
    pub(super) fn of_the_observation(observation: &StateObservation) -> Option<Self> {
        match (&observation.consulted_report, &observation.recovery_panic) {
            (Some(report), None) => Some(Self::Reported(Box::new(report.clone()))),
            (None, Some(message)) => Some(Self::Panicked(message.clone())),
            (None, None) | (Some(_), Some(_)) => None,
        }
    }

    pub(super) fn effective_root(&self) -> Option<(InstanceGeneration, CheckpointTxg)> {
        match self {
            Self::Reported(report) => report.effective_root,
            Self::Panicked(_) => None,
        }
    }
}

/// 在一态的崩溃镜像上经 [`ReadRecordingPoolReader`] 跑看 journal 的那一遍恢复：交回恢复的结局与它读过的字节段
/// （panic 时读段记到 panic 那一刻为止：同样的读答同样的字节，恢复在同一处 panic）。
pub(super) fn consulted_recovery_recording_the_reads(
    image: &CrashImage<'_>,
) -> (ConsultedRecovery, Vec<RecordedReadRange>) {
    let reader = ReadRecordingPoolReader {
        image,
        read_ranges: RefCell::new(Vec::new()),
    };
    let recovered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        singlefs_core::recovery::recover(&reader, singlefs_core::recovery::JournalPolicy::Consult)
    }));
    let recovery = match recovered {
        Ok(report) => ConsultedRecovery::Reported(Box::new(report)),
        Err(payload) => ConsultedRecovery::Panicked(panic_text(&*payload)),
    };
    (recovery, reader.read_ranges.into_inner())
}

/// 读过的字节段与一个写的字节段（同一块盘上）相交。
pub(super) fn recorded_reads_overlap_the_write(
    read_ranges: &[RecordedReadRange],
    write: &RetainedWrite,
) -> bool {
    let write_start = write.offset.0;
    let write_end = write_start + write.length_in_bytes();
    read_ranges.iter().any(|read| {
        read.device == write.device
            && read.start_in_bytes < write_end
            && write_start < read.end_in_bytes
    })
}

/// 类核对 ③ 细分到不动点之后的分法，与最后一遍在每个代表上的恢复（键是代表的序号）。
pub(super) struct ConfirmedLossClassification {
    pub(super) partition: ConfirmedLossPartition,
    /// 原来的 C 写数（末段里属于第 3 次调用的写）。
    pub(super) in_flight_writes: usize,
    pub(super) refinements: usize,
    /// 细分移进键的 C 写，末段里的位置。
    pub(super) moved_positions: Vec<usize>,
    pub(super) representative_recoveries: BTreeMap<u64, ConsultedRecovery>,
    /// 停在上限时最后一遍还与读段相交的 C 写个数（到了不动点是 0）。
    pub(super) overlapping_positions_left: usize,
    /// 细分之后要判的态数超过上限（停机 S13）。
    pub(super) stopped_at_the_state_limit: bool,
    /// 最后那个分法要照原路径判的态数（停在上限时是超过上限的那一个分法的）。
    pub(super) judged_states_planned: u64,
}

impl ConfirmedLossClassification {
    /// 要照原路径判的态：每种取法的代表，加上 C 还剩写时同一取法的「C 全落」那一态。
    pub(super) fn judged_ordinals(&self, digits: &LastSegmentDigits) -> BTreeSet<u64> {
        let mut judged = BTreeSet::new();
        for keyed_assignment in 0..self.partition.keyed_assignments(digits) {
            judged.insert(
                self.partition
                    .representative_ordinal(digits, keyed_assignment),
            );
            if !self.partition.class_positions.is_empty() {
                judged.insert(
                    self.partition
                        .fully_landed_ordinal(digits, keyed_assignment),
                );
            }
        }
        judged
    }
}

#[cfg(test)]
mod tests;
