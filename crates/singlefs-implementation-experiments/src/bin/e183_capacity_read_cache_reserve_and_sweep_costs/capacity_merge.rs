use super::*;

// ============================================================================================
// H2 的合并：读回 45 片，算 E37 里 H2 的判定（R2a–R2f、PC-线性、PC-平、几何敏感性、判别力自证 ①、T4、G1、G2、G6）。
// ============================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AllocateCall {
    pub(super) window: MeasuredWindow,
    /// 落点离单元区起点的槽数；分配被拒是 `None`。
    pub(super) slots_from_unit_area_start: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PublishCall {
    pub(super) window: MeasuredWindow,
    pub(super) published: bool,
}

/// 一格一轮量到的东西（一片里读回的）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CellRoundReading {
    pub(super) heap_control_holds: bool,
    pub(super) records: u64,
    pub(super) copy_bytes: u64,
    copy_nanoseconds: u64,
    pub(super) allocate: Vec<AllocateCall>,
    flat_peak_over_baseline_bytes: u64,
    pub(super) publish: Vec<PublishCall>,
}

/// X22（登记 r2 第一节 1.1、第七节 A42）：阈值 T = 24 GiB（systemd 的 G 是 2^30）。登记写死的数，不来自 `crates/`，不回比。
pub(super) const MEMORY_THRESHOLD_BYTES: u64 = 24 * GIBIBYTE;
/// X22 的下沿比阈值低这一截（64 MiB）：回收不掉的那一份里页表与映射进来的文件页的余量（登记 X22「为什么」一格）。
const MEMORY_THRESHOLD_MARGIN_BYTES: u64 = 64 * MEBIBYTE;
/// `/proc/self/status` 的 VmHWM 以 KiB 记。
const KIBIBYTE: u64 = 1024;
/// PC-高水位（登记 r2 5.2、A44）：PC-堆 写满 64 MiB 前后，进程高水位至少要涨这么多字节（64 MiB 减 4 MiB 余量）。
const HIGH_WATER_MARK_CONTROL_MINIMUM_BYTES: u64 = 62_914_560;
/// X26：服务结局文件里唯一认的结局（撞上限、已到两台最大）。
const SERVICE_OUTCOME_AT_THE_MAXIMUM: &str = "memory-cap-at-maximum";

/// X22 的下沿：H 不超过它才算在阈值内。
fn memory_threshold_lower_edge(threshold_bytes: u64) -> u64 {
    threshold_bytes - MEMORY_THRESHOLD_MARGIN_BYTES
}

/// VmHWM（KiB）换成字节：X22 的 H。
fn high_water_mark_bytes_of(kibibytes: u64) -> u64 {
    kibibytes * KIBIBYTE
}

/// PC-高水位：写满 64 MiB 之后，高水位涨了不少于 [`HIGH_WATER_MARK_CONTROL_MINIMUM_BYTES`]。
pub(super) fn high_water_mark_control_holds(before_kibibytes: u64, after_kibibytes: u64) -> bool {
    high_water_mark_bytes_of(after_kibibytes.saturating_sub(before_kibibytes))
        >= HIGH_WATER_MARK_CONTROL_MINIMUM_BYTES
}

/// 一片的 `h2_config`、`positive_control_heap`、`h2_process` 里给 X22、X24、A43 的读数；读不到的记 `None`，不当 0（V37）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CellRoundFacts {
    pub(super) host: Option<String>,
    pub(super) image_filesystem: Option<String>,
    pub(super) service_attempt: Option<String>,
    pub(super) unit_area_slots: Option<u64>,
    pub(super) high_water_mark_before_kibibytes: Option<u64>,
    pub(super) high_water_mark_after_kibibytes: Option<u64>,
    pub(super) high_water_mark_kibibytes: Option<u64>,
}

/// X22 的 H 能不能用：读不到（V37）、PC-高水位不过（V40）、能用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HighWaterMark {
    Unreadable,
    ControlFailed,
    Usable { bytes: u64 },
}

impl CellRoundFacts {
    fn of_the_lines(lines: &[ResultLine]) -> CellRoundFacts {
        let configuration = lines_named(lines, "h2_config").into_iter().next();
        let heap = lines_named(lines, "positive_control_heap")
            .into_iter()
            .next();
        let process = lines_named(lines, "h2_process").into_iter().next();
        let configuration_text = |key: &str| {
            configuration
                .and_then(|line| line.field(key))
                .map(str::to_string)
        };
        CellRoundFacts {
            host: configuration_text("host"),
            image_filesystem: configuration_text("image_filesystem"),
            service_attempt: configuration_text("service_attempt"),
            unit_area_slots: configuration.and_then(|line| line.number("unit_area_slots")),
            high_water_mark_before_kibibytes: heap
                .and_then(|line| line.number("high_water_mark_before_kibibytes")),
            high_water_mark_after_kibibytes: heap
                .and_then(|line| line.number("high_water_mark_after_kibibytes")),
            high_water_mark_kibibytes: process
                .and_then(|line| line.number("vm_high_water_mark_kibibytes")),
        }
    }

    fn high_water_mark(&self) -> HighWaterMark {
        let Some(high_water_mark_bytes) =
            self.high_water_mark_kibibytes.map(high_water_mark_bytes_of)
        else {
            return HighWaterMark::Unreadable;
        };
        match (
            self.high_water_mark_before_kibibytes,
            self.high_water_mark_after_kibibytes,
        ) {
            (Some(before_kibibytes), Some(after_kibibytes)) => {
                if high_water_mark_control_holds(before_kibibytes, after_kibibytes) {
                    HighWaterMark::Usable {
                        bytes: high_water_mark_bytes,
                    }
                } else {
                    HighWaterMark::ControlFailed
                }
            }
            (Some(_), None) | (None, Some(_)) | (None, None) => HighWaterMark::Unreadable,
        }
    }

    /// 「贴着阈值」（X22 末句、G9）：|H − T| ≤ 装置程序文件的字节数 + 64 MiB。H 或程序文件大小读不到交回 `None`。
    fn is_near_the_threshold(&self, binary_bytes: Option<u64>) -> Option<bool> {
        let high_water_mark_bytes = self
            .high_water_mark_kibibytes
            .map(high_water_mark_bytes_of)?;
        Some(
            high_water_mark_bytes.abs_diff(MEMORY_THRESHOLD_BYTES)
                <= binary_bytes? + MEMORY_THRESHOLD_MARGIN_BYTES,
        )
    }
}

/// A43（登记 r2 第七节）：第二段每格 `h2_config` 的 `unit_area_slots`，登记按 L = 768 MiB、单元区起点 50176 现算的五个数（记录格、占用格按容量取）。
const REGISTERED_UNIT_AREA_SLOTS_BY_DEVICE_BYTES: [(u64, u64); 5] = [
    (4 * GIBIBYTE, 211_968),
    (64 * GIBIBYTE, 4_144_128),
    (TEBIBYTE, 67_058_688),
    (4 * TEBIBYTE, 268_385_280),
    (16 * TEBIBYTE - MEBIBYTE, 1_073_691_584),
];

fn registered_unit_area_slots(device_bytes: u64) -> Option<u64> {
    REGISTERED_UNIT_AREA_SLOTS_BY_DEVICE_BYTES
        .iter()
        .find(|(registered_device_bytes, _)| *registered_device_bytes == device_bytes)
        .map(|(_, slots)| *slots)
}

/// T9：这一格这一轮装置报的单元区槽数与 A43 不等（读不到也算不等）。
fn unit_area_slots_differ_from_the_registered(cell: CapacityCell, facts: &CellRoundFacts) -> bool {
    facts.unit_area_slots != registered_unit_area_slots(cell.device_bytes)
}

/// 片做完、程序文件与合并相同的一格一轮：类由阈值定（X22），阈值敏感性拿同一份 H 换阈值再判（登记 r2 第八节）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FinishedCellRound {
    pub(super) facts: CellRoundFacts,
    /// 走到了测量、量全了；没走到（哪一步没成）或缺行缺字段交回那一步的结局。
    measured: Result<CellRoundReading, String>,
}

/// 一格一轮的结局（登记 r2 X22、X26）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CellRoundStatus {
    /// 片做完：「阈值内跑完」「内存上限之外」由 H 定；H 读不到或 PC-高水位不过的这一轮作废（V37、V40）。
    Finished(FinishedCellRound),
    /// 那一台文件系统拒（X22 第四类）：镜像建不出；台在读数里。
    FilesystemRefused {
        facts: CellRoundFacts,
        outcome: String,
    },
    /// 服务结局文件（X26）：撞上限、已到两台最大，`reached` 是服务给到的最大上限。
    ServiceGaveUpAtTheMaximum {
        reached_bytes: u64,
    },
    /// 片没做完，同一格已有别的轮在阈值之外：跑法脚本不再起它（X26「够判后未跑」）。
    NotRunAfterAnotherRoundWentBeyond,
    /// `h2_config` 缺行缺字段、服务结局文件写坏了、或这一格的别的轮在阈值之外（不算结果，见 [`statuses_counted_as_results`]）。
    NotMeasured(String),
    Missing,
    BinaryOtherThanTheMerge(String),
}

/// X22 的四类，加上「没有类」（作废、没走到测量、片不在……）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemoryThresholdClass {
    WithinTheThreshold,
    BeyondTheMemoryThreshold,
    Undeterminable,
    FilesystemRefused,
    NoClass,
}

impl MemoryThresholdClass {
    pub(super) fn name(self) -> &'static str {
        match self {
            MemoryThresholdClass::WithinTheThreshold => "within_the_threshold",
            MemoryThresholdClass::BeyondTheMemoryThreshold => "beyond_the_memory_threshold",
            MemoryThresholdClass::Undeterminable => "undeterminable",
            MemoryThresholdClass::FilesystemRefused => "filesystem_refused",
            MemoryThresholdClass::NoClass => "no_class",
        }
    }
}

/// 一档整格跑完只认「阈值内跑完」；「内存上限之外」的档不算（X22）。
fn counts_toward_a_completed_capacity(class: MemoryThresholdClass) -> bool {
    match class {
        MemoryThresholdClass::WithinTheThreshold => true,
        MemoryThresholdClass::BeyondTheMemoryThreshold => false,
        MemoryThresholdClass::Undeterminable
        | MemoryThresholdClass::FilesystemRefused
        | MemoryThresholdClass::NoClass => false,
    }
}

impl CellRoundStatus {
    /// X22：这一格这一轮在阈值 `threshold_bytes` 下的类。
    fn memory_threshold_class(&self, threshold_bytes: u64) -> MemoryThresholdClass {
        match self {
            CellRoundStatus::Finished(finished) => match finished.facts.high_water_mark() {
                HighWaterMark::Unreadable | HighWaterMark::ControlFailed => {
                    MemoryThresholdClass::NoClass
                }
                HighWaterMark::Usable { bytes } => {
                    if bytes > memory_threshold_lower_edge(threshold_bytes) {
                        MemoryThresholdClass::BeyondTheMemoryThreshold
                    } else if finished.measured.is_ok() {
                        MemoryThresholdClass::WithinTheThreshold
                    } else {
                        MemoryThresholdClass::NoClass
                    }
                }
            },
            CellRoundStatus::ServiceGaveUpAtTheMaximum { reached_bytes } => {
                if *reached_bytes >= threshold_bytes {
                    MemoryThresholdClass::BeyondTheMemoryThreshold
                } else {
                    MemoryThresholdClass::Undeterminable
                }
            }
            CellRoundStatus::FilesystemRefused { .. } => MemoryThresholdClass::FilesystemRefused,
            CellRoundStatus::NotRunAfterAnotherRoundWentBeyond
            | CellRoundStatus::NotMeasured(_)
            | CellRoundStatus::Missing
            | CellRoundStatus::BinaryOtherThanTheMerge(_) => MemoryThresholdClass::NoClass,
        }
    }

    pub(super) fn text(&self) -> String {
        match self {
            CellRoundStatus::Finished(finished) => match finished.facts.high_water_mark() {
                HighWaterMark::Unreadable => "voided_high_water_mark_unreadable".to_string(),
                HighWaterMark::ControlFailed => "voided_high_water_mark_control_failed".to_string(),
                HighWaterMark::Usable { .. } => {
                    match (
                        self.memory_threshold_class(MEMORY_THRESHOLD_BYTES),
                        &finished.measured,
                    ) {
                        (MemoryThresholdClass::BeyondTheMemoryThreshold, _) => {
                            "beyond_the_memory_threshold".to_string()
                        }
                        (
                            MemoryThresholdClass::WithinTheThreshold
                            | MemoryThresholdClass::Undeterminable
                            | MemoryThresholdClass::FilesystemRefused
                            | MemoryThresholdClass::NoClass,
                            Err(outcome),
                        ) => format!("not_measured:{outcome}"),
                        (
                            MemoryThresholdClass::WithinTheThreshold
                            | MemoryThresholdClass::Undeterminable
                            | MemoryThresholdClass::FilesystemRefused
                            | MemoryThresholdClass::NoClass,
                            Ok(_),
                        ) => "within_the_threshold".to_string(),
                    }
                }
            },
            CellRoundStatus::FilesystemRefused { outcome, .. } => outcome.clone(),
            CellRoundStatus::ServiceGaveUpAtTheMaximum { reached_bytes } => format!(
                "{}_by_the_service:reached_bytes_{reached_bytes}",
                self.memory_threshold_class(MEMORY_THRESHOLD_BYTES).name()
            ),
            CellRoundStatus::NotRunAfterAnotherRoundWentBeyond => {
                "not_run_after_another_round_went_beyond".to_string()
            }
            CellRoundStatus::NotMeasured(outcome) => format!("not_measured:{outcome}"),
            CellRoundStatus::Missing => "missing".to_string(),
            CellRoundStatus::BinaryOtherThanTheMerge(sha256) => {
                format!("binary_other_than_the_merge:{sha256}")
            }
        }
    }

    /// 算进结果的读数：X22 在登记的阈值 T 下判成「阈值内跑完」的那一格那一轮（登记 r2 第六节「参与斜率与 R2f 的只有 X22 判成「阈值内跑完」的档」）。
    pub(super) fn reading(&self) -> Option<&CellRoundReading> {
        if counts_toward_a_completed_capacity(self.memory_threshold_class(MEMORY_THRESHOLD_BYTES)) {
            self.measured_reading()
        } else {
            None
        }
    }

    /// 量到的读数，不看 X22（小盘单测核测量那一条路本身用）。
    pub(super) fn measured_reading(&self) -> Option<&CellRoundReading> {
        match self {
            CellRoundStatus::Finished(finished) => finished.measured.as_ref().ok(),
            CellRoundStatus::FilesystemRefused { .. }
            | CellRoundStatus::ServiceGaveUpAtTheMaximum { .. }
            | CellRoundStatus::NotRunAfterAnotherRoundWentBeyond
            | CellRoundStatus::NotMeasured(_)
            | CellRoundStatus::Missing
            | CellRoundStatus::BinaryOtherThanTheMerge(_) => None,
        }
    }

    /// 片做完时读到的台、高水位与几何（X24、X22、A43）。
    pub(super) fn facts(&self) -> Option<&CellRoundFacts> {
        match self {
            CellRoundStatus::Finished(finished) => Some(&finished.facts),
            CellRoundStatus::FilesystemRefused { facts, .. } => Some(facts),
            CellRoundStatus::ServiceGaveUpAtTheMaximum { .. }
            | CellRoundStatus::NotRunAfterAnotherRoundWentBeyond
            | CellRoundStatus::NotMeasured(_)
            | CellRoundStatus::Missing
            | CellRoundStatus::BinaryOtherThanTheMerge(_) => None,
        }
    }
}

fn window_of_a_line(line: &ResultLine) -> Option<MeasuredWindow> {
    Some(MeasuredWindow {
        nanoseconds: line.number("nanoseconds")?,
        peak_over_baseline_bytes: line.number("peak_over_baseline_bytes")?,
        baseline_bytes: line.number("baseline_bytes")?,
        in_use_after_bytes: line.number("in_use_after_bytes")?,
    })
}

/// 走到了测量的一片读成读数；缺行、缺字段、X4 或 X5 不是 5 次都交回 `None`（V37：读不到 ≠ 读到 0）。
fn measured_reading_of_the_lines(lines: &[ResultLine]) -> Option<CellRoundReading> {
    let heap = lines_named(lines, "positive_control_heap")
        .into_iter()
        .next()?;
    let copy = lines_named(lines, "h2_allocator_copy").into_iter().next()?;
    let flat = lines_named(lines, "positive_control_flat")
        .into_iter()
        .next()?;
    let allocate: Option<Vec<AllocateCall>> = lines_named(lines, "h2_allocate")
        .into_iter()
        .map(|line| {
            Some(AllocateCall {
                window: window_of_a_line(line)?,
                slots_from_unit_area_start: line.number("slots_from_unit_area_start"),
            })
        })
        .collect();
    let publish: Option<Vec<PublishCall>> = lines_named(lines, "h2_publish")
        .into_iter()
        .map(|line| {
            Some(PublishCall {
                window: window_of_a_line(line)?,
                published: line.field("outcome")? == "ok",
            })
        })
        .collect();
    let reading = CellRoundReading {
        heap_control_holds: heap.truth("holds")?,
        records: copy.number("records")?,
        copy_bytes: copy.number("copy_bytes")?,
        copy_nanoseconds: copy.number("copy_nanoseconds")?,
        allocate: allocate?,
        flat_peak_over_baseline_bytes: window_of_a_line(flat)?.peak_over_baseline_bytes,
        publish: publish?,
    };
    let calls = usize::try_from(MEASURED_CALLS_PER_ROUND).expect("5");
    (reading.allocate.len() == calls && reading.publish.len() == calls).then_some(reading)
}

/// 一片的行读成一格一轮的结局：程序文件对不上的不读；镜像建不出记「那一台文件系统拒」；其余记「片做完」，类在 [`CellRoundStatus::memory_threshold_class`] 里按 H 判。
pub(super) fn status_of_a_capacity_shard(
    lines: &[ResultLine],
    binary_sha256: &str,
) -> CellRoundStatus {
    let fields_missing = || CellRoundStatus::NotMeasured("fields_missing".to_string());
    let Some(configuration) = lines_named(lines, "h2_config").into_iter().next() else {
        return fields_missing();
    };
    match configuration.field("binary_sha256") {
        Some(sha256) if sha256 == binary_sha256 => {}
        Some(sha256) => return CellRoundStatus::BinaryOtherThanTheMerge(sha256.to_string()),
        None => return fields_missing(),
    }
    let facts = CellRoundFacts::of_the_lines(lines);
    let outcome = lines_named(lines, "h2_steps")
        .into_iter()
        .next()
        .and_then(|steps| steps.field("outcome"));
    let measured = match outcome {
        Some(refused) if refused.starts_with("filesystem_refused") => {
            return CellRoundStatus::FilesystemRefused {
                facts,
                outcome: refused.to_string(),
            };
        }
        Some("ok") => {
            measured_reading_of_the_lines(lines).ok_or_else(|| "fields_missing".to_string())
        }
        Some(failed) => Err(failed.to_string()),
        None => Err("fields_missing".to_string()),
    };
    CellRoundStatus::Finished(FinishedCellRound { facts, measured })
}

/// X26 的服务结局文件（一行 `type=memory-cap-at-maximum reached=<字节> peak=… task=… shard=… attempt=…`）读成结局；别的写法记写坏了。
fn status_of_a_service_outcome(text: &str) -> CellRoundStatus {
    let line = ResultLine::parse(&format!("E7RESULT {}", text.trim()));
    let service_type = line.as_ref().and_then(|line| line.field("type"));
    let reached = line.as_ref().and_then(|line| line.number("reached"));
    match (service_type, reached) {
        (Some(SERVICE_OUTCOME_AT_THE_MAXIMUM), Some(reached_bytes)) => {
            CellRoundStatus::ServiceGaveUpAtTheMaximum { reached_bytes }
        }
        (Some(_) | None, Some(_) | None) => {
            CellRoundStatus::NotMeasured("service_outcome_unreadable".to_string())
        }
    }
}

/// 一格一轮报的量（登记第六节 R2a–R2e、5.3 的 PC-线性、PC-平）。堆量在 PC-堆 没过的那一片上不出（V30）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CapacityQuantity {
    AllocateNanoseconds,
    PublishNanoseconds,
    AllocatePeakOverBaselineBytes,
    PublishPeakOverBaselineBytes,
    AllocatorCopyBytes,
    AllocatorCopyNanoseconds,
    FlatControlPeakOverBaselineBytes,
}

impl CapacityQuantity {
    const EVERY: [CapacityQuantity; 7] = [
        CapacityQuantity::AllocateNanoseconds,
        CapacityQuantity::PublishNanoseconds,
        CapacityQuantity::AllocatePeakOverBaselineBytes,
        CapacityQuantity::PublishPeakOverBaselineBytes,
        CapacityQuantity::AllocatorCopyBytes,
        CapacityQuantity::AllocatorCopyNanoseconds,
        CapacityQuantity::FlatControlPeakOverBaselineBytes,
    ];
    /// R2a、R2b、R2d 两行：对容量、对记录数都求的那四样。
    const JUDGED: [CapacityQuantity; 4] = [
        CapacityQuantity::AllocateNanoseconds,
        CapacityQuantity::PublishNanoseconds,
        CapacityQuantity::AllocatePeakOverBaselineBytes,
        CapacityQuantity::PublishPeakOverBaselineBytes,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            CapacityQuantity::AllocateNanoseconds => "allocate_nanoseconds",
            CapacityQuantity::PublishNanoseconds => "publish_nanoseconds",
            CapacityQuantity::AllocatePeakOverBaselineBytes => "allocate_peak_over_baseline_bytes",
            CapacityQuantity::PublishPeakOverBaselineBytes => "publish_peak_over_baseline_bytes",
            CapacityQuantity::AllocatorCopyBytes => "allocator_copy_bytes",
            CapacityQuantity::AllocatorCopyNanoseconds => "allocator_copy_nanoseconds",
            CapacityQuantity::FlatControlPeakOverBaselineBytes => {
                "flat_control_peak_over_baseline_bytes"
            }
        }
    }

    /// 这一量在一格一轮里的每一个读数（X4、X5 各 5 次；X8、PC-平 各 1 次）。有一次分配被拒或发布没发成，那一轮这一量整个不出。
    fn samples(self, reading: &CellRoundReading) -> Option<Vec<u64>> {
        let heap_counts = reading.heap_control_holds;
        let every_allocation_placed = reading
            .allocate
            .iter()
            .all(|call| call.slots_from_unit_area_start.is_some());
        let every_publish_done = reading.publish.iter().all(|call| call.published);
        let of_allocate = |pick: fn(&MeasuredWindow) -> u64| {
            reading
                .allocate
                .iter()
                .map(|call| pick(&call.window))
                .collect::<Vec<u64>>()
        };
        let of_publish = |pick: fn(&MeasuredWindow) -> u64| {
            reading
                .publish
                .iter()
                .map(|call| pick(&call.window))
                .collect::<Vec<u64>>()
        };
        match self {
            CapacityQuantity::AllocateNanoseconds => {
                every_allocation_placed.then(|| of_allocate(|window| window.nanoseconds))
            }
            CapacityQuantity::PublishNanoseconds => {
                every_publish_done.then(|| of_publish(|window| window.nanoseconds))
            }
            CapacityQuantity::AllocatePeakOverBaselineBytes => (heap_counts
                && every_allocation_placed)
                .then(|| of_allocate(|window| window.peak_over_baseline_bytes)),
            CapacityQuantity::PublishPeakOverBaselineBytes => (heap_counts && every_publish_done)
                .then(|| of_publish(|window| window.peak_over_baseline_bytes)),
            CapacityQuantity::AllocatorCopyBytes => heap_counts.then(|| vec![reading.copy_bytes]),
            CapacityQuantity::AllocatorCopyNanoseconds => Some(vec![reading.copy_nanoseconds]),
            CapacityQuantity::FlatControlPeakOverBaselineBytes => {
                heap_counts.then(|| vec![reading.flat_peak_over_baseline_bytes])
            }
        }
    }

    /// 轮内中位数（登记 R2a「X4 的 5 次取轮内中位数」）。
    fn round_median(self, reading: &CellRoundReading) -> Option<u64> {
        self.samples(reading)
            .and_then(|samples| median_of(&samples))
    }
}

/// 求斜率的那一维：盘容量，或记录数（X9 的实际值）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlopeAxis {
    Capacity,
    Records,
}

impl SlopeAxis {
    pub(super) fn name(self) -> &'static str {
        match self {
            SlopeAxis::Capacity => "capacity",
            SlopeAxis::Records => "records",
        }
    }

    fn abscissa_of(self, cell: CapacityCell, reading: &CellRoundReading) -> u64 {
        match self {
            SlopeAxis::Capacity => cell.device_bytes,
            SlopeAxis::Records => reading.records,
        }
    }
}

/// 读回的全部片：（格，轮，结局）。
pub(super) type CapacityStatuses = Vec<(CapacityCell, u64, CellRoundStatus)>;

fn status_of(
    statuses: &CapacityStatuses,
    cell: CapacityCell,
    round: u64,
) -> Option<&CellRoundStatus> {
    statuses
        .iter()
        .find(|(status_cell, status_round, _)| *status_cell == cell && *status_round == round)
        .map(|(_, _, status)| status)
}

/// 一轮里给定几格上这一量的斜率：只取这一轮量到了、这一量也出了的格（「一轮里全部跑完的容量档」，X10）。
fn slope_in_one_round(
    statuses: &CapacityStatuses,
    cells: &[CapacityCell],
    round: u64,
    quantity: CapacityQuantity,
    axis: SlopeAxis,
) -> Option<SlopeReading> {
    let points: Vec<(u64, u64)> = cells
        .iter()
        .filter_map(|cell| {
            let reading = status_of(statuses, *cell, round)?.reading()?;
            Some((
                axis.abscissa_of(*cell, reading),
                quantity.round_median(reading)?,
            ))
        })
        .collect();
    log_log_least_squares_slope(&points)
}

fn ladder_cells() -> Vec<CapacityCell> {
    CAPACITY_LADDER_DEVICE_BYTES
        .map(|device_bytes| CapacityCell {
            device_bytes,
            sequentially_written_units: 0,
        })
        .to_vec()
}

fn record_cells() -> Vec<CapacityCell> {
    RECORD_CELL_SEQUENTIALLY_WRITTEN_UNIT_COUNTS
        .map(|units| CapacityCell {
            device_bytes: 4 * GIBIBYTE,
            sequentially_written_units: units,
        })
        .to_vec()
}

fn occupied_capacity_cells() -> Vec<CapacityCell> {
    OCCUPANCY_SENSITIVITY_DEVICE_BYTES
        .map(|device_bytes| CapacityCell {
            device_bytes,
            sequentially_written_units: OCCUPANCY_SENSITIVITY_UNITS,
        })
        .to_vec()
}

/// 一行斜率判定：哪一量、哪一维、哪几格、对应登记第六节的哪一行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SlopeRow {
    pub(super) row: &'static str,
    quantity: CapacityQuantity,
    axis: SlopeAxis,
    pub(super) group: SlopeGroup,
}

/// 求斜率用哪几格：容量阶梯（n = 0）、记录格（4 GiB 上 n 三档）、几何敏感性那三档容量（n = 10240）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SlopeGroup {
    CapacityLadder,
    RecordCells,
    OccupiedCapacityCells,
}

impl SlopeGroup {
    pub(super) fn name(self) -> &'static str {
        match self {
            SlopeGroup::CapacityLadder => "capacity_ladder",
            SlopeGroup::RecordCells => "record_cells",
            SlopeGroup::OccupiedCapacityCells => "occupied_capacity_cells",
        }
    }

    fn capacity_cells_of_the_group(self) -> Vec<CapacityCell> {
        match self {
            SlopeGroup::CapacityLadder => ladder_cells(),
            SlopeGroup::RecordCells => record_cells(),
            SlopeGroup::OccupiedCapacityCells => occupied_capacity_cells(),
        }
    }
}

fn capacity_slope_rows() -> Vec<SlopeRow> {
    let on_the_ladder = |row, quantity| SlopeRow {
        row,
        quantity,
        axis: SlopeAxis::Capacity,
        group: SlopeGroup::CapacityLadder,
    };
    let on_the_records = |row, quantity| SlopeRow {
        row,
        quantity,
        axis: SlopeAxis::Records,
        group: SlopeGroup::RecordCells,
    };
    let on_the_occupied = |row, quantity| SlopeRow {
        row,
        quantity,
        axis: SlopeAxis::Capacity,
        group: SlopeGroup::OccupiedCapacityCells,
    };
    vec![
        on_the_ladder("r2a", CapacityQuantity::AllocateNanoseconds),
        on_the_ladder("r2b", CapacityQuantity::PublishNanoseconds),
        on_the_ladder(
            "r2d_allocate",
            CapacityQuantity::AllocatePeakOverBaselineBytes,
        ),
        on_the_ladder(
            "r2d_publish",
            CapacityQuantity::PublishPeakOverBaselineBytes,
        ),
        on_the_ladder("r2c", CapacityQuantity::AllocatorCopyBytes),
        on_the_ladder(
            "positive_control_linear_copy_time",
            CapacityQuantity::AllocatorCopyNanoseconds,
        ),
        on_the_ladder(
            "positive_control_flat",
            CapacityQuantity::FlatControlPeakOverBaselineBytes,
        ),
        on_the_records("r2e_allocate", CapacityQuantity::AllocateNanoseconds),
        on_the_records("r2e_publish", CapacityQuantity::PublishNanoseconds),
        on_the_records(
            "r2e_allocate_peak",
            CapacityQuantity::AllocatePeakOverBaselineBytes,
        ),
        on_the_records(
            "r2e_publish_peak",
            CapacityQuantity::PublishPeakOverBaselineBytes,
        ),
        on_the_occupied("r2a_at_occupancy", CapacityQuantity::AllocateNanoseconds),
        on_the_occupied("r2b_at_occupancy", CapacityQuantity::PublishNanoseconds),
    ]
}

/// 一行的 5 轮斜率与类。
#[derive(Clone, Debug, PartialEq)]
struct SlopeRowJudgement {
    pub(super) row: SlopeRow,
    pub(super) slopes: Vec<Option<SlopeReading>>,
    pub(super) class: ClassAcrossRounds,
}

impl CapacityQuantity {
    /// 这一量来自 X5 的发布。
    fn comes_from_the_publish(self) -> bool {
        match self {
            CapacityQuantity::PublishNanoseconds
            | CapacityQuantity::PublishPeakOverBaselineBytes => true,
            CapacityQuantity::AllocateNanoseconds
            | CapacityQuantity::AllocatePeakOverBaselineBytes
            | CapacityQuantity::AllocatorCopyBytes
            | CapacityQuantity::AllocatorCopyNanoseconds
            | CapacityQuantity::FlatControlPeakOverBaselineBytes => false,
        }
    }
}

impl SlopeGroup {
    /// 这一组格上的发布是不是登记的 X5：容量阶梯的首个文件一个数据单元，覆盖写它就是 X5；记录格与占用格的文件有 n 个单元，
    /// `UserChange` 里没有只改第 0 个单元以内 4096 字节的入口（登记第十二节修订附 grep 与零命中），整份 `Overwrite` 把文件缩成一个单元，不是 X5。
    fn publish_is_the_registered_step(self) -> bool {
        match self {
            SlopeGroup::CapacityLadder => true,
            SlopeGroup::RecordCells | SlopeGroup::OccupiedCapacityCells => false,
        }
    }
}

impl SlopeRow {
    /// G10（登记 r2 E43）：R2a、R2b（容量阶梯上的分配与发布耗时）每一轮的 b 只在那一轮的点出自同一台时出。
    fn needs_one_host_per_round(self) -> bool {
        let timing = match self.quantity {
            CapacityQuantity::AllocateNanoseconds | CapacityQuantity::PublishNanoseconds => true,
            CapacityQuantity::AllocatePeakOverBaselineBytes
            | CapacityQuantity::PublishPeakOverBaselineBytes
            | CapacityQuantity::AllocatorCopyBytes
            | CapacityQuantity::AllocatorCopyNanoseconds
            | CapacityQuantity::FlatControlPeakOverBaselineBytes => false,
        };
        let on_the_ladder = match self.group {
            SlopeGroup::CapacityLadder => true,
            SlopeGroup::RecordCells | SlopeGroup::OccupiedCapacityCells => false,
        };
        timing && on_the_ladder
    }
}

/// 这一轮里算进结果的那几格（斜率用到的点）记的台不止一个（X24 的 `host=`）。
fn round_spans_more_than_one_host(
    statuses: &CapacityStatuses,
    cells: &[CapacityCell],
    round: u64,
) -> bool {
    hosts_of_the_round(statuses, cells, round).len() > 1
}

fn hosts_of_the_round<'statuses>(
    statuses: &'statuses CapacityStatuses,
    cells: &[CapacityCell],
    round: u64,
) -> BTreeSet<&'statuses str> {
    cells
        .iter()
        .filter_map(|cell| {
            let status = status_of(statuses, *cell, round)?;
            status.reading()?;
            status.facts()?.host.as_deref()
        })
        .collect()
}

fn judge_slope_row(statuses: &CapacityStatuses, row: SlopeRow) -> SlopeRowJudgement {
    if row.quantity.comes_from_the_publish() && !row.group.publish_is_the_registered_step() {
        return SlopeRowJudgement {
            row,
            slopes: vec![None; usize::try_from(CAPACITY_CELL_ROUNDS).expect("5")],
            class: ClassAcrossRounds::RegisteredPublishCannotBeMade,
        };
    }
    let cells = row.group.capacity_cells_of_the_group();
    let slopes: Vec<Option<SlopeReading>> = (1..=CAPACITY_CELL_ROUNDS)
        .map(|round| {
            if row.needs_one_host_per_round()
                && round_spans_more_than_one_host(statuses, &cells, round)
            {
                None
            } else {
                slope_in_one_round(statuses, &cells, round, row.quantity, row.axis)
            }
        })
        .collect();
    let class = class_across_rounds(
        &slopes
            .iter()
            .map(|slope| slope.map(|reading| reading.slope))
            .collect::<Vec<_>>(),
        usize::try_from(CAPACITY_CELL_ROUNDS).expect("5"),
    );
    SlopeRowJudgement { row, slopes, class }
}

/// PC-线性 的拷贝字节与拷贝耗时、PC-平：5 轮每一轮的斜率都要在门槛那一侧（缺一轮也不算过）。
fn every_round_slope_satisfies(judgement: &SlopeRowJudgement, holds: fn(f64) -> bool) -> bool {
    judgement
        .slopes
        .iter()
        .all(|slope| slope.is_some_and(|reading| holds(reading.slope)))
}

/// A37：分配器拷贝字节 ÷ (2 × 单元区槽数) ∈ [3.5, 4.0]，按百万分之一比。
const COPY_BYTES_PER_SLOT_LOWER_MILLIONTHS: u128 = 3_500_000;
const COPY_BYTES_PER_SLOT_UPPER_MILLIONTHS: u128 = 4_000_000;

fn bytes_per_slot_per_device_millionths(copy_bytes: u64, device_bytes: u64) -> u128 {
    u128::from(copy_bytes) * 1_000_000
        / u128::from(POOL_DEVICE_COUNT * unit_area_slots(device_bytes, JOURNAL_RING_DEFAULT_BYTES))
}

fn copy_bytes_per_slot_ratio_holds(millionths: u128) -> bool {
    (COPY_BYTES_PER_SLOT_LOWER_MILLIONTHS..=COPY_BYTES_PER_SLOT_UPPER_MILLIONTHS)
        .contains(&millionths)
}

/// 一格跨轮比的确定性字段（T4；登记 R2d「确定性的量，5 轮应逐字段相同」）：哪几样不同。
fn fields_differing_across_rounds(readings: &[&CellRoundReading]) -> Vec<&'static str> {
    let Some(first) = readings.first() else {
        return Vec::new();
    };
    let allocate_peaks = |reading: &CellRoundReading| {
        reading
            .allocate
            .iter()
            .map(|call| call.window.peak_over_baseline_bytes)
            .collect::<Vec<_>>()
    };
    let allocate_slots = |reading: &CellRoundReading| {
        reading
            .allocate
            .iter()
            .map(|call| call.slots_from_unit_area_start)
            .collect::<Vec<_>>()
    };
    let publish_peaks = |reading: &CellRoundReading| {
        reading
            .publish
            .iter()
            .map(|call| call.window.peak_over_baseline_bytes)
            .collect::<Vec<_>>()
    };
    let mut differing = Vec::new();
    let mut note = |name: &'static str, equal: bool| {
        if !equal {
            differing.push(name);
        }
    };
    note(
        "records",
        readings
            .iter()
            .all(|reading| reading.records == first.records),
    );
    note(
        "copy_bytes",
        readings
            .iter()
            .all(|reading| reading.copy_bytes == first.copy_bytes),
    );
    note(
        "allocate_peaks",
        readings
            .iter()
            .all(|reading| allocate_peaks(reading) == allocate_peaks(first)),
    );
    note(
        "allocate_slots",
        readings
            .iter()
            .all(|reading| allocate_slots(reading) == allocate_slots(first)),
    );
    note(
        "publish_peaks",
        readings
            .iter()
            .all(|reading| publish_peaks(reading) == publish_peaks(first)),
    );
    note(
        "flat_peak",
        readings.iter().all(|reading| {
            reading.flat_peak_over_baseline_bytes == first.flat_peak_over_baseline_bytes
        }),
    );
    differing
}

pub(super) fn joined_or_none<Item: ToString>(items: &[Item]) -> String {
    if items.is_empty() {
        return "none".to_string();
    }
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// 每一量在这几份读数上的中位数、最小、最大（R2f 的绝对值）。
fn quantity_value_fields(readings: &[&CellRoundReading]) -> Vec<String> {
    CapacityQuantity::EVERY
        .into_iter()
        .map(|quantity| {
            let samples: Vec<u64> = readings
                .iter()
                .filter_map(|reading| quantity.samples(reading))
                .flatten()
                .collect();
            format!(
                "{name}_median={} {name}_minimum={} {name}_maximum={}",
                optional_number_text(median_of(&samples)),
                optional_number_text(samples.iter().min().copied()),
                optional_number_text(samples.iter().max().copied()),
                name = quantity.name(),
            )
        })
        .collect()
}

/// R2f 按台（登记 r2 第六节 R2f、X25）：每格每台一行，各报中位数、最小、最大与轮数；合起来那一行另标 `across_hosts`。
fn emit_capacity_cell_values_by_host(
    emitter: &mut Emitter,
    statuses: &CapacityStatuses,
    cell: CapacityCell,
) -> Vec<String> {
    let mut by_host: BTreeMap<String, Vec<&CellRoundReading>> = BTreeMap::new();
    for round in 1..=CAPACITY_CELL_ROUNDS {
        let Some(status) = status_of(statuses, cell, round) else {
            continue;
        };
        let (Some(reading), Some(facts)) = (status.reading(), status.facts()) else {
            continue;
        };
        let host = facts.host.clone().unwrap_or_else(|| "none".to_string());
        by_host.entry(host).or_default().push(reading);
    }
    for (host, readings) in &by_host {
        let mut fields = vec![
            format!("name=h2_cell_values_by_host cell={}", cell.name()),
            format!("host={host}"),
            format!("rounds_measured={}", readings.len()),
        ];
        fields.extend(quantity_value_fields(readings));
        emitter.emit(&fields.join(" "));
    }
    by_host.into_keys().collect()
}

/// R2f 的绝对值表与每格的 T4、A37（登记第六节 R2f、第七节 A37）；交回 T4 不过的格数。
fn emit_capacity_cell_values(emitter: &mut Emitter, statuses: &CapacityStatuses) -> usize {
    let mut hidden_state_cells = 0;
    for cell in capacity_cells() {
        let readings: Vec<&CellRoundReading> = (1..=CAPACITY_CELL_ROUNDS)
            .filter_map(|round| status_of(statuses, cell, round)?.reading())
            .collect();
        let hosts = emit_capacity_cell_values_by_host(emitter, statuses, cell);
        let mut fields = vec![
            format!("name=h2_cell_values cell={}", cell.name()),
            format!("device_bytes={}", cell.device_bytes),
            format!(
                "sequentially_written_units={}",
                cell.sequentially_written_units
            ),
            format!(
                "publish_is_the_registered_single_unit_overwrite={}",
                cell.sequentially_written_units == 0
            ),
            format!("rounds_measured={}", readings.len()),
            format!(
                "records={}",
                joined_or_none(
                    &readings
                        .iter()
                        .map(|reading| reading.records)
                        .collect::<Vec<_>>()
                )
            ),
            format!("hosts={}", joined_or_none(&hosts)),
            format!("across_hosts={}", hosts.len() > 1),
        ];
        fields.extend(quantity_value_fields(&readings));
        let slots: BTreeSet<u64> = readings
            .iter()
            .flat_map(|reading| {
                reading
                    .allocate
                    .iter()
                    .filter_map(|call| call.slots_from_unit_area_start)
            })
            .collect();
        fields.push(format!(
            "slots_from_unit_area_start={}",
            joined_or_none(&slots.into_iter().collect::<Vec<_>>())
        ));
        let per_slot: Vec<u128> = readings
            .iter()
            .filter(|reading| reading.heap_control_holds)
            .map(|reading| {
                bytes_per_slot_per_device_millionths(reading.copy_bytes, cell.device_bytes)
            })
            .collect();
        fields.push(format!(
            "bytes_per_slot_per_device_millionths={} bytes_per_slot_per_device_within_the_inferred_interval={}",
            joined_or_none(&per_slot),
            !per_slot.is_empty() && per_slot.iter().all(|millionths| copy_bytes_per_slot_ratio_holds(*millionths)),
        ));
        emitter.emit(&fields.join(" "));
        let differing = fields_differing_across_rounds(&readings);
        hidden_state_cells += usize::from(!differing.is_empty());
        emitter.emit(&format!(
            "name=h2_cell_determinism cell={} rounds_compared={} equal_across_rounds={} differing_fields={}",
            cell.name(),
            readings.len(),
            differing.is_empty(),
            joined_or_none(&differing),
        ));
    }
    hidden_state_cells
}

fn class_of_the_row<'judgements>(
    judgements: &'judgements [SlopeRowJudgement],
    row: &str,
) -> &'judgements ClassAcrossRounds {
    &judgements
        .iter()
        .find(|judgement| judgement.row.row == row)
        .expect("行名取自 capacity_slope_rows")
        .class
}

/// H2 合并的结局：T4 不过 ⇒ 停机（产物不算，两轮的片原样交回）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MergeEnd {
    Complete,
    Stopped { reason: String },
}

/// 有一轮在阈值 `threshold_bytes` 之外的格。
pub(super) fn cells_beyond_the_threshold(
    statuses: &CapacityStatuses,
    threshold_bytes: u64,
) -> BTreeSet<CapacityCell> {
    statuses
        .iter()
        .filter(|(_, _, status)| {
            status.memory_threshold_class(threshold_bytes)
                == MemoryThresholdClass::BeyondTheMemoryThreshold
        })
        .map(|(cell, _, _)| *cell)
        .collect()
}

/// 算结果用的结局：有一轮在阈值 T 之外的格，它的每一轮都不算结果、不进斜率与 R2f（X22「它 5 轮的读数照报、不算结果、不进斜率」；读数照报在片的原行里）。
fn statuses_counted_as_results(statuses: &CapacityStatuses) -> CapacityStatuses {
    let beyond = cells_beyond_the_threshold(statuses, MEMORY_THRESHOLD_BYTES);
    statuses
        .iter()
        .map(|(cell, round, status)| {
            if beyond.contains(cell) {
                (
                    *cell,
                    *round,
                    CellRoundStatus::NotMeasured(
                        "another_round_of_the_cell_went_beyond_the_memory_threshold".to_string(),
                    ),
                )
            } else {
                (*cell, *round, status.clone())
            }
        })
        .collect()
}

/// 阈值 `threshold_bytes` 下整格跑完（5 轮都是「阈值内跑完」）的容量阶梯档，盘字节从小到大。
fn completed_ladder_capacities(statuses: &CapacityStatuses, threshold_bytes: u64) -> Vec<u64> {
    ladder_cells()
        .into_iter()
        .filter(|cell| {
            (1..=CAPACITY_CELL_ROUNDS).all(|round| {
                status_of(statuses, *cell, round).is_some_and(|status| {
                    counts_toward_a_completed_capacity(
                        status.memory_threshold_class(threshold_bytes),
                    )
                })
            })
        })
        .map(|cell| cell.device_bytes)
        .collect()
}

/// 「本机能造的最大稀疏盘」（X22）：阈值内整格跑完的最大一档。
fn largest_completed_capacity(statuses: &CapacityStatuses, threshold_bytes: u64) -> Option<u64> {
    completed_ladder_capacities(statuses, threshold_bytes)
        .into_iter()
        .max()
}

/// 容量阶梯上有一轮落在这一类（阈值 T 下）的档，MiB。
fn ladder_mebibytes_with_a_round_of(
    statuses: &CapacityStatuses,
    class: MemoryThresholdClass,
) -> Vec<u64> {
    ladder_cells()
        .into_iter()
        .filter(|cell| {
            (1..=CAPACITY_CELL_ROUNDS).any(|round| {
                status_of(statuses, *cell, round).is_some_and(|status| {
                    status.memory_threshold_class(MEMORY_THRESHOLD_BYTES) == class
                })
            })
        })
        .map(|cell| cell.device_bytes / MEBIBYTE)
        .collect()
}

/// 每格每轮一行（R2g）：结局、X22 的类、H、PC-高水位两读数、贴着阈值、台、镜像目录的文件系统、服务的哪一次尝试、A43。
fn emit_cell_round_statuses(
    emitter: &mut Emitter,
    statuses: &CapacityStatuses,
    binary_bytes: Option<u64>,
) {
    for (cell, round, status) in statuses {
        let facts = status.facts();
        let text_of = |pick: fn(&CellRoundFacts) -> Option<String>| {
            facts.and_then(pick).unwrap_or_else(|| "none".to_string())
        };
        let number_of =
            |pick: fn(&CellRoundFacts) -> Option<u64>| optional_number_text(facts.and_then(pick));
        emitter.emit(&format!(
            "name=h2_shard_status cell={} round={round} status={} class={} high_water_mark_bytes={} high_water_mark_before_kibibytes={} high_water_mark_after_kibibytes={} near_the_threshold={} host={} image_filesystem={} service_attempt={} unit_area_slots={} registered_unit_area_slots={}",
            cell.name(),
            status.text(),
            status.memory_threshold_class(MEMORY_THRESHOLD_BYTES).name(),
            number_of(|round_facts| round_facts.high_water_mark_kibibytes.map(high_water_mark_bytes_of)),
            number_of(|round_facts| round_facts.high_water_mark_before_kibibytes),
            number_of(|round_facts| round_facts.high_water_mark_after_kibibytes),
            facts
                .and_then(|round_facts| round_facts.is_near_the_threshold(binary_bytes))
                .map_or_else(|| "none".to_string(), |near| near.to_string()),
            text_of(|round_facts| round_facts.host.clone()),
            text_of(|round_facts| round_facts.image_filesystem.clone()),
            text_of(|round_facts| round_facts.service_attempt.clone()),
            number_of(|round_facts| round_facts.unit_area_slots),
            optional_number_text(registered_unit_area_slots(cell.device_bytes)),
        ));
    }
}

/// 一次尝试里一格做完的两样：跑它的主机（片文件没记主机的是 none），和它的进程高水位字节数（读不到的是 none）。
type HostAndHighWaterMark = (Option<String>, Option<u64>);

/// X27 的装置一侧（R2h 的读数）：服务每一次尝试里做完的格的 H 最大值；执行员拿服务记的那一次尝试的峰值 P(a) 比
/// `P(a) ≥ service_peak_needed_at_least_bytes`（= H 最大(a) − 程序文件字节数 − 64 MiB）。有一格 H 读不到，那一次尝试的最大值记 none。
fn emit_service_attempts(
    emitter: &mut Emitter,
    statuses: &CapacityStatuses,
    binary_bytes: Option<u64>,
) {
    let mut by_attempt: BTreeMap<String, Vec<HostAndHighWaterMark>> = BTreeMap::new();
    for (_, _, status) in statuses {
        let Some(facts) = status.facts() else {
            continue;
        };
        let Some(attempt) = facts.service_attempt.clone() else {
            continue;
        };
        by_attempt.entry(attempt).or_default().push((
            facts.host.clone(),
            facts
                .high_water_mark_kibibytes
                .map(high_water_mark_bytes_of),
        ));
    }
    for (attempt, cells) in &by_attempt {
        let hosts: BTreeSet<String> = cells.iter().filter_map(|(host, _)| host.clone()).collect();
        let maximum = cells
            .iter()
            .map(|(_, high_water_mark)| *high_water_mark)
            .collect::<Option<Vec<u64>>>()
            .and_then(|values| values.into_iter().max());
        let needed = maximum.zip(binary_bytes).map(|(maximum, binary)| {
            maximum.saturating_sub(binary + MEMORY_THRESHOLD_MARGIN_BYTES)
        });
        emitter.emit(&format!(
            "name=h2_service_attempt service_attempt={attempt} hosts={} cells_finished={} high_water_mark_maximum_bytes={} binary_bytes={} service_peak_needed_at_least_bytes={}",
            joined_or_none(&hosts.into_iter().collect::<Vec<_>>()),
            cells.len(),
            optional_number_text(maximum),
            optional_number_text(binary_bytes),
            optional_number_text(needed),
        ));
    }
}

/// 最大一档那一行（X22、G6、G9、G12）交回给判决行的几样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LargestDisk {
    completed_count: usize,
    fewer_than_four: bool,
}

fn emit_largest_disk(
    emitter: &mut Emitter,
    statuses: &CapacityStatuses,
    binary_bytes: Option<u64>,
) -> LargestDisk {
    let completed: Vec<u64> = completed_ladder_capacities(statuses, MEMORY_THRESHOLD_BYTES)
        .into_iter()
        .map(|device_bytes| device_bytes / MEBIBYTE)
        .collect();
    let largest = completed.iter().max().copied();
    let ladder = ladder_cells();
    let ladder_endpoint = ladder[ladder.len() - 1].device_bytes / MEBIBYTE;
    // G9：最大一档与比它大一档的那几格（一档都没跑完时看最小的那一档）里贴着阈值的格。
    let next_index = ladder
        .iter()
        .position(|cell| Some(cell.device_bytes / MEBIBYTE) == largest)
        .map_or(0, |index| index + 1);
    let watched: Vec<CapacityCell> = ladder
        .iter()
        .enumerate()
        .filter(|(index, _)| *index + 1 == next_index || *index == next_index)
        .map(|(_, cell)| *cell)
        .collect();
    let near: Vec<String> = watched
        .iter()
        .flat_map(|cell| (1..=CAPACITY_CELL_ROUNDS).map(move |round| (*cell, round)))
        .filter(|(cell, round)| {
            status_of(statuses, *cell, *round)
                .and_then(CellRoundStatus::facts)
                .and_then(|facts| facts.is_near_the_threshold(binary_bytes))
                == Some(true)
        })
        .map(|(cell, round)| format!("{}-round-{round}", cell.name()))
        .collect();
    let fewer_than_four = completed.len() < 4;
    emitter.emit(&format!(
        "name=h2_largest_disk threshold_bytes={MEMORY_THRESHOLD_BYTES} lower_edge_bytes={} completed_capacities_mebibytes={} completed_count={} largest_completed_mebibytes={} largest_is_the_ladder_endpoint={} beyond_the_memory_threshold_mebibytes={} undeterminable_mebibytes={} filesystem_refused_mebibytes={} near_the_threshold_at_or_above_the_largest={} fewer_than_four_capacities={fewer_than_four}",
        memory_threshold_lower_edge(MEMORY_THRESHOLD_BYTES),
        joined_or_none(&completed),
        completed.len(),
        optional_number_text(largest),
        largest == Some(ladder_endpoint),
        joined_or_none(&ladder_mebibytes_with_a_round_of(statuses, MemoryThresholdClass::BeyondTheMemoryThreshold)),
        joined_or_none(&ladder_mebibytes_with_a_round_of(statuses, MemoryThresholdClass::Undeterminable)),
        joined_or_none(&ladder_mebibytes_with_a_round_of(statuses, MemoryThresholdClass::FilesystemRefused)),
        joined_or_none(&near),
    ));
    LargestDisk {
        completed_count: completed.len(),
        fewer_than_four,
    }
}

/// 几何敏感性（登记 r2 第八节）：同一份 H 在 T/2、T、2T 下各判一次最大一档；交回最大一档随阈值换没换。
fn emit_threshold_sensitivity(emitter: &mut Emitter, statuses: &CapacityStatuses) -> bool {
    let thresholds = [
        ("half", MEMORY_THRESHOLD_BYTES / 2),
        ("registered", MEMORY_THRESHOLD_BYTES),
        ("double", 2 * MEMORY_THRESHOLD_BYTES),
    ];
    let largest: Vec<Option<u64>> = thresholds
        .iter()
        .map(|(_, threshold_bytes)| largest_completed_capacity(statuses, *threshold_bytes))
        .collect();
    for ((label, threshold_bytes), largest_device_bytes) in thresholds.iter().zip(&largest) {
        emitter.emit(&format!(
            "name=h2_threshold_sensitivity threshold={label} threshold_bytes={threshold_bytes} lower_edge_bytes={} largest_completed_mebibytes={} completed_count={}",
            memory_threshold_lower_edge(*threshold_bytes),
            optional_number_text(largest_device_bytes.map(|device_bytes| device_bytes / MEBIBYTE)),
            completed_ladder_capacities(statuses, *threshold_bytes).len(),
        ));
    }
    let switches: Vec<String> = thresholds
        .windows(2)
        .zip(largest.windows(2))
        .filter(|(_, pair)| pair[0] != pair[1])
        .map(|(labels, _)| format!("{}_and_{}", labels[0].0, labels[1].0))
        .collect();
    let mebibytes_at = |index: usize| {
        optional_number_text(largest[index].map(|device_bytes| device_bytes / MEBIBYTE))
    };
    emitter.emit(&format!(
        "name=h2_threshold_dependence largest_at_half_mebibytes={} largest_at_registered_mebibytes={} largest_at_double_mebibytes={} depends_on_the_threshold={} switches_between={}",
        mebibytes_at(0),
        mebibytes_at(1),
        mebibytes_at(2),
        !switches.is_empty(),
        joined_or_none(&switches),
    ));
    !switches.is_empty()
}

/// 判别力自证 ④（登记 r2 第八节）：阈值挪到刚好罩住某一档 H 最大值（下沿 = 它）与刚好罩不住（下沿 = 它 − 1）两处，最大一档必须不同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MovedMemoryThreshold {
    capacity_device_bytes: u64,
    just_within_threshold_bytes: u64,
    largest_just_within: Option<u64>,
    largest_just_beyond: Option<u64>,
}

fn memory_threshold_moved_across_a_real_high_water_mark(
    statuses: &CapacityStatuses,
) -> Option<MovedMemoryThreshold> {
    ladder_cells().into_iter().find_map(|cell| {
        let peak_bytes = (1..=CAPACITY_CELL_ROUNDS)
            .filter_map(|round| {
                status_of(statuses, cell, round)?
                    .facts()?
                    .high_water_mark_kibibytes
            })
            .max()
            .map(high_water_mark_bytes_of)?;
        let just_within_threshold_bytes = peak_bytes + MEMORY_THRESHOLD_MARGIN_BYTES;
        let largest_just_within = largest_completed_capacity(statuses, just_within_threshold_bytes);
        let largest_just_beyond =
            largest_completed_capacity(statuses, just_within_threshold_bytes - 1);
        (largest_just_within != largest_just_beyond).then_some(MovedMemoryThreshold {
            capacity_device_bytes: cell.device_bytes,
            just_within_threshold_bytes,
            largest_just_within,
            largest_just_beyond,
        })
    })
}

/// 判别力自证 ④ 的代用（真产物上各档的 H 都在 T 的同一侧时）：合成两档 H = T − 1 GiB 与 T + 1 GiB，必须一档在阈值内、一档在阈值之外。
fn synthetic_high_water_marks_fall_on_two_sides_of_the_threshold() -> bool {
    let finished_with = |high_water_mark_bytes: u64| {
        CellRoundStatus::Finished(FinishedCellRound {
            facts: CellRoundFacts {
                host: None,
                image_filesystem: None,
                service_attempt: None,
                unit_area_slots: None,
                high_water_mark_before_kibibytes: Some(0),
                high_water_mark_after_kibibytes: Some(
                    HIGH_WATER_MARK_CONTROL_MINIMUM_BYTES / KIBIBYTE,
                ),
                high_water_mark_kibibytes: Some(high_water_mark_bytes / KIBIBYTE),
            },
            measured: Ok(CellRoundReading {
                heap_control_holds: true,
                records: 0,
                copy_bytes: 0,
                copy_nanoseconds: 0,
                allocate: Vec::new(),
                flat_peak_over_baseline_bytes: 0,
                publish: Vec::new(),
            }),
        })
    };
    finished_with(MEMORY_THRESHOLD_BYTES - GIBIBYTE).memory_threshold_class(MEMORY_THRESHOLD_BYTES)
        == MemoryThresholdClass::WithinTheThreshold
        && finished_with(MEMORY_THRESHOLD_BYTES + GIBIBYTE)
            .memory_threshold_class(MEMORY_THRESHOLD_BYTES)
            == MemoryThresholdClass::BeyondTheMemoryThreshold
}

/// E37 里 H2 的判定，接在每片的结局行之后（产物的后半）。
pub(super) fn emit_capacity_summary(
    emitter: &mut Emitter,
    statuses: &CapacityStatuses,
    binary_bytes: Option<u64>,
) -> MergeEnd {
    emit_cell_round_statuses(emitter, statuses, binary_bytes);
    let counted = statuses_counted_as_results(statuses);
    let hidden_state_cells = emit_capacity_cell_values(emitter, &counted);
    let largest_disk = emit_largest_disk(emitter, statuses, binary_bytes);
    let largest_depends_on_the_threshold = emit_threshold_sensitivity(emitter, statuses);
    emit_service_attempts(emitter, statuses, binary_bytes);
    let ladder = ladder_cells();
    let mut ladder_rounds_across_hosts = 0;
    for round in 1..=CAPACITY_CELL_ROUNDS {
        let hosts: Vec<&str> = hosts_of_the_round(&counted, &ladder, round)
            .into_iter()
            .collect();
        ladder_rounds_across_hosts += usize::from(hosts.len() > 1);
        emitter.emit(&format!(
            "name=h2_round_hosts round={round} ladder_hosts={} one_host={}",
            joined_or_none(&hosts),
            hosts.len() <= 1
        ));
    }
    let count_of = |class: MemoryThresholdClass| {
        statuses
            .iter()
            .filter(|(_, _, status)| status.memory_threshold_class(MEMORY_THRESHOLD_BYTES) == class)
            .count()
    };
    let voided_high_water_mark = statuses
        .iter()
        .filter(|(_, _, status)| {
            status
                .facts()
                .is_some_and(|facts| match facts.high_water_mark() {
                    HighWaterMark::Unreadable | HighWaterMark::ControlFailed => true,
                    HighWaterMark::Usable { .. } => false,
                })
                && matches!(status, CellRoundStatus::Finished(_))
        })
        .count();
    let near_the_threshold = statuses
        .iter()
        .filter(|(_, _, status)| {
            status
                .facts()
                .and_then(|facts| facts.is_near_the_threshold(binary_bytes))
                == Some(true)
        })
        .count();
    let unit_area_slots_differing = statuses
        .iter()
        .filter(|(cell, _, status)| {
            status
                .facts()
                .is_some_and(|facts| unit_area_slots_differ_from_the_registered(*cell, facts))
        })
        .count();
    let moved_threshold = memory_threshold_moved_across_a_real_high_water_mark(statuses);
    let synthetic_high_water_marks =
        synthetic_high_water_marks_fall_on_two_sides_of_the_threshold();
    let threshold_self_proof_holds = moved_threshold.is_some() || synthetic_high_water_marks;
    let capacity_mebibytes_text = |device_bytes: Option<u64>| {
        optional_number_text(device_bytes.map(|bytes| bytes / MEBIBYTE))
    };
    emitter.emit(&format!(
        "name=discrimination_self_proof judgement=h2_largest_disk capacity_mebibytes={} just_within_threshold_bytes={} largest_just_within_mebibytes={} largest_just_beyond_mebibytes={} synthetic_high_water_marks_fall_on_two_sides={synthetic_high_water_marks} used={} holds={threshold_self_proof_holds}",
        capacity_mebibytes_text(moved_threshold.map(|moved| moved.capacity_device_bytes)),
        optional_number_text(moved_threshold.map(|moved| moved.just_within_threshold_bytes)),
        capacity_mebibytes_text(moved_threshold.and_then(|moved| moved.largest_just_within)),
        capacity_mebibytes_text(moved_threshold.and_then(|moved| moved.largest_just_beyond)),
        if moved_threshold.is_some() { "real_high_water_marks" } else { "synthetic_high_water_marks" },
    ));
    let memory_threshold_counts = MemoryThresholdCounts {
        within_the_threshold: count_of(MemoryThresholdClass::WithinTheThreshold),
        beyond_the_memory_threshold: count_of(MemoryThresholdClass::BeyondTheMemoryThreshold),
        undeterminable: count_of(MemoryThresholdClass::Undeterminable),
        filesystem_refused: count_of(MemoryThresholdClass::FilesystemRefused),
        voided_high_water_mark,
        near_the_threshold,
        ladder_rounds_across_hosts,
        unit_area_slots_differing,
        largest_depends_on_the_threshold,
        threshold_self_proof_holds,
    };
    let judgements: Vec<SlopeRowJudgement> = capacity_slope_rows()
        .into_iter()
        .map(|row| judge_slope_row(&counted, row))
        .collect();
    for judgement in &judgements {
        for (round_index, slope) in judgement.slopes.iter().enumerate() {
            emitter.emit(&format!(
                "name=h2_slope row={} quantity={} axis={} group={} round={} slope={} has_zero_point={} points={}",
                judgement.row.row,
                judgement.row.quantity.name(),
                judgement.row.axis.name(),
                judgement.row.group.name(),
                round_index + 1,
                slope_text(slope.map(|reading| reading.slope)),
                slope.is_some_and(|reading| reading.has_zero_point),
                slope.map_or(0, |reading| reading.points),
            ));
        }
        emitter.emit(&format!(
            "name=h2_class row={} quantity={} axis={} group={} class={}",
            judgement.row.row,
            judgement.row.quantity.name(),
            judgement.row.axis.name(),
            judgement.row.group.name(),
            judgement.class.text(),
        ));
    }
    let row_judgement = |row: &str| {
        judgements
            .iter()
            .find(|judgement| judgement.row.row == row)
            .expect("行名取自 capacity_slope_rows")
    };
    let copy_bytes_holds =
        every_round_slope_satisfies(row_judgement("r2c"), |slope| slope >= LINEAR_SLOPE_BOUNDARY);
    let copy_time_holds = every_round_slope_satisfies(
        row_judgement("positive_control_linear_copy_time"),
        |slope| slope >= LINEAR_SLOPE_BOUNDARY,
    );
    let flat_holds = every_round_slope_satisfies(row_judgement("positive_control_flat"), |slope| {
        slope < UNRELATED_SLOPE_BOUNDARY
    });
    emitter.emit(&format!(
        "name=positive_control_linear copy_bytes_slope_at_least_0_8_every_round={copy_bytes_holds} copy_time_slope_at_least_0_8_every_round={copy_time_holds}"
    ));
    emitter.emit(&format!(
        "name=positive_control_flat_slope slope_below_0_2_every_round={flat_holds}"
    ));
    for (baseline, occupied) in [("r2a", "r2a_at_occupancy"), ("r2b", "r2b_at_occupancy")] {
        let baseline_class = class_of_the_row(&judgements, baseline);
        let occupied_class = class_of_the_row(&judgements, occupied);
        let agreement = if *occupied_class == ClassAcrossRounds::RegisteredPublishCannotBeMade {
            "not_judged_x5_as_registered_cannot_be_made"
        } else if baseline_class == occupied_class {
            "two_points_agree"
        } else {
            "unstable_depends_on_occupancy"
        };
        emitter.emit(&format!(
            "name=h2_occupancy_sensitivity row={baseline} near_empty_class={} occupied_class={} agreement={agreement}",
            baseline_class.text(),
            occupied_class.text(),
        ));
    }
    let judged_slopes: Vec<f64> = judgements
        .iter()
        .filter(|judgement| CapacityQuantity::JUDGED.contains(&judgement.row.quantity))
        .flat_map(|judgement| {
            judgement
                .slopes
                .iter()
                .flatten()
                .map(|reading| reading.slope)
        })
        .collect();
    let moved = linear_boundary_moved_between_real_slopes(&judged_slopes);
    let synthetic = synthetic_slopes_fall_in_two_classes();
    let self_proof_holds = moved.is_some() || synthetic;
    emitter.emit(&format!(
        "name=discrimination_self_proof judgement=h2_slope_class real_slopes={} moved_boundary={} flipped={} synthetic_slopes_fall_in_two_classes={synthetic} used={} holds={self_proof_holds}",
        judged_slopes.len(),
        slope_text(moved.map(|boundary| boundary.boundary)),
        moved.map_or(0, |boundary| boundary.flipped),
        if moved.is_some() { "real_slopes" } else { "synthetic_slopes" },
    ));
    emit_capacity_verdict(
        emitter,
        &counted,
        &judgements,
        CapacityVerdictInputs {
            completed_count: largest_disk.completed_count,
            fewer_than_four: largest_disk.fewer_than_four,
            copy_bytes_holds,
            copy_time_holds,
            flat_holds,
            self_proof_holds,
            hidden_state_cells,
            memory_threshold_counts,
        },
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CapacityVerdictInputs {
    completed_count: usize,
    fewer_than_four: bool,
    copy_bytes_holds: bool,
    copy_time_holds: bool,
    flat_holds: bool,
    pub(super) self_proof_holds: bool,
    hidden_state_cells: usize,
    memory_threshold_counts: MemoryThresholdCounts,
}

/// 判决行里 X22 那几样（登记 r2 E39、E43、E44、E45）：各类的格轮数、作废、贴着阈值、跨台的轮、A43 不等的格轮、阈值敏感性与判别力自证 ④。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MemoryThresholdCounts {
    within_the_threshold: usize,
    beyond_the_memory_threshold: usize,
    undeterminable: usize,
    filesystem_refused: usize,
    voided_high_water_mark: usize,
    near_the_threshold: usize,
    ladder_rounds_across_hosts: usize,
    unit_area_slots_differing: usize,
    largest_depends_on_the_threshold: bool,
    threshold_self_proof_holds: bool,
}

/// H2 的判决行：每个判定各占一个字段，不合取（登记第六节开头）；G1、G2（第四节 Q8 推的与量出的不同）、G6、V30、V32、T4 各报各的。
fn emit_capacity_verdict(
    emitter: &mut Emitter,
    statuses: &CapacityStatuses,
    judgements: &[SlopeRowJudgement],
    inputs: CapacityVerdictInputs,
) -> MergeEnd {
    let class = |row: &str| class_of_the_row(judgements, row);
    let unstable_rows: Vec<&str> = ["r2a", "r2b", "r2e_allocate", "r2e_publish"]
        .into_iter()
        .filter(|row| *class(row) == ClassAcrossRounds::Unstable)
        .collect();
    let ladder_readings: Vec<(CapacityCell, &CellRoundReading)> = statuses
        .iter()
        .filter(|(cell, _, _)| cell.sequentially_written_units == 0)
        .filter_map(|(cell, _, status)| Some((*cell, status.reading()?)))
        .collect();
    let copy_bytes_per_slot_ratio_outside = ladder_readings.iter().any(|(cell, reading)| {
        reading.heap_control_holds
            && !copy_bytes_per_slot_ratio_holds(bytes_per_slot_per_device_millionths(
                reading.copy_bytes,
                cell.device_bytes,
            ))
    });
    let mut contradicted: Vec<&str> = Vec::new();
    if *class("r2c") != ClassAcrossRounds::Stable(SlopeClass::Linear)
        || copy_bytes_per_slot_ratio_outside
    {
        contradicted.push("r2c_or_a37");
    }
    if *class("r2a") != ClassAcrossRounds::Stable(SlopeClass::BasicallyUnrelated) {
        contradicted.push("r2a_not_basically_unrelated");
    }
    if *class("r2e_allocate") != ClassAcrossRounds::Stable(SlopeClass::Linear) {
        contradicted.push("r2e_allocate_not_linear");
    }
    let heap_control_failed_shards = statuses
        .iter()
        .filter(|(_, _, status)| {
            status
                .reading()
                .is_some_and(|reading| !reading.heap_control_holds)
        })
        .count();
    emitter.emit(&format!(
        "name=verdict item=h2_summary r2a_class={} r2b_class={} r2d_allocate_class={} r2d_publish_class={} r2e_allocate_class={} r2e_publish_class={} r2e_allocate_peak_class={} r2e_publish_peak_class={} r2c_class={} completed_capacities={} fewer_than_four_capacities={} positive_control_linear_copy_bytes_holds={} positive_control_linear_copy_time_holds={} positive_control_flat_holds={} heap_control_failed_shards={heap_control_failed_shards} unstable_slope_rows={} predictions_contradicted={} discrimination_self_proof_holds={} hidden_state_cells={} cell_rounds_within_the_threshold={} cell_rounds_beyond_the_memory_threshold={} cell_rounds_undeterminable={} cell_rounds_filesystem_refused={} cell_rounds_voided_high_water_mark={} cell_rounds_near_the_threshold={} ladder_rounds_across_hosts={} unit_area_slots_differing_cell_rounds={} largest_depends_on_the_threshold={} threshold_discrimination_self_proof_holds={}",
        class("r2a").text(),
        class("r2b").text(),
        class("r2d_allocate").text(),
        class("r2d_publish").text(),
        class("r2e_allocate").text(),
        class("r2e_publish").text(),
        class("r2e_allocate_peak").text(),
        class("r2e_publish_peak").text(),
        class("r2c").text(),
        inputs.completed_count,
        inputs.fewer_than_four,
        inputs.copy_bytes_holds,
        inputs.copy_time_holds,
        inputs.flat_holds,
        joined_or_none(&unstable_rows),
        joined_or_none(&contradicted),
        inputs.self_proof_holds,
        inputs.hidden_state_cells,
        inputs.memory_threshold_counts.within_the_threshold,
        inputs.memory_threshold_counts.beyond_the_memory_threshold,
        inputs.memory_threshold_counts.undeterminable,
        inputs.memory_threshold_counts.filesystem_refused,
        inputs.memory_threshold_counts.voided_high_water_mark,
        inputs.memory_threshold_counts.near_the_threshold,
        inputs.memory_threshold_counts.ladder_rounds_across_hosts,
        inputs.memory_threshold_counts.unit_area_slots_differing,
        inputs.memory_threshold_counts.largest_depends_on_the_threshold,
        inputs.memory_threshold_counts.threshold_self_proof_holds,
    ));
    if inputs.memory_threshold_counts.unit_area_slots_differing > 0 {
        return MergeEnd::Stopped {
            reason: "t9_unit_area_slots_differ_from_the_registered_values".to_string(),
        };
    }
    if inputs.hidden_state_cells > 0 {
        return MergeEnd::Stopped {
            reason: "t4_deterministic_fields_differ_across_rounds".to_string(),
        };
    }
    MergeEnd::Complete
}

/// 没做完的一片的结局：运行目录里有执行员照 X26 写的 `<片名>.service-outcome` 就读它，没有就是片不在。
fn status_of_an_unfinished_shard(directory: &Path, shard: &str) -> CellRoundStatus {
    match std::fs::read_to_string(directory.join(format!("{shard}.service-outcome"))) {
        Ok(text) => status_of_a_service_outcome(&text),
        Err(_) => CellRoundStatus::Missing,
    }
}

/// 读回运行目录里的 45 片：做完的读行；没做完的看服务结局文件（X26）。读完之后，片不在而同一格已有别的轮在阈值 T 之外的，
/// 记「够判后未跑」（跑法脚本照 `h2 cells-beyond-the-threshold` 不再起它，X26）。
pub(super) fn read_capacity_statuses(directory: &Path, binary_sha256: &str) -> CapacityStatuses {
    let statuses: CapacityStatuses = capacity_shards()
        .into_iter()
        .map(|(shard, cell, round)| {
            let text = std::fs::read_to_string(directory.join(&shard)).ok();
            let finished = text
                .as_deref()
                .and_then(|text| lines_of_a_finished_shard(text, &shard));
            let status = match finished {
                Some(lines) => {
                    let parsed: Vec<ResultLine> = lines
                        .iter()
                        .filter_map(|line| ResultLine::parse(line))
                        .collect();
                    status_of_a_capacity_shard(&parsed, binary_sha256)
                }
                None => status_of_an_unfinished_shard(directory, &shard),
            };
            (cell, round, status)
        })
        .collect();
    let beyond = cells_beyond_the_threshold(&statuses, MEMORY_THRESHOLD_BYTES);
    statuses
        .into_iter()
        .map(|(cell, round, status)| match status {
            CellRoundStatus::Missing if beyond.contains(&cell) => (
                cell,
                round,
                CellRoundStatus::NotRunAfterAnotherRoundWentBeyond,
            ),
            kept @ (CellRoundStatus::Missing
            | CellRoundStatus::Finished(_)
            | CellRoundStatus::FilesystemRefused { .. }
            | CellRoundStatus::ServiceGaveUpAtTheMaximum { .. }
            | CellRoundStatus::NotRunAfterAnotherRoundWentBeyond
            | CellRoundStatus::NotMeasured(_)
            | CellRoundStatus::BinaryOtherThanTheMerge(_)) => (cell, round, kept),
        })
        .collect()
}

#[cfg(test)]
mod tests;
