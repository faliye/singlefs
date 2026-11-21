//! 起跑判据的三条水位线（全空聚簇段、碎片、不可回收字节）：每块盘的读数、越线的结论与起跑判据（`compaction_starts_by_the_watermarks`）。

use super::*;

/// 全空聚簇段线的产品默认：整段数的 5%（D26（后台整理与放置回收） 已定项 1，数是拍的）。
pub const EMPTY_CLUSTER_SEGMENTS_LINE_PERCENT_BY_DEFAULT: u64 = 5;
/// 全空聚簇段线的下限：2 段（前台写入类发布在全空段低于 `N₀` = 1 时被合取 7 拒，线至少 `N₀ + 1`，不然前台先被拒、整理才起跑）。
pub const EMPTY_CLUSTER_SEGMENTS_LINE_LOWEST_IN_SEGMENTS_BY_DEFAULT: u64 = 2;

/// 全空聚簇段这条水位（越线：全空聚簇段数 < 线）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyClusterSegmentsWatermark {
    Off,
    /// 线 = max(⌈整段数 × 百分比 ÷ 100⌉, 下限)，再不超过整段数（只有一两个整段的小盘刚 mkfs 时不越线）。
    PercentOfTheClusterSegments {
        percent: u64,
        lowest_line_in_segments: u64,
    },
}

impl EmptyClusterSegmentsWatermark {
    /// 这块盘的线（段数）；关着是 `None`。
    #[must_use]
    pub fn line_in_segments(self, cluster_segments_in_the_unit_area: u64) -> Option<u64> {
        match self {
            EmptyClusterSegmentsWatermark::Off => None,
            EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
                percent,
                lowest_line_in_segments,
            } => {
                let percent_of_the_segments = (u128::from(cluster_segments_in_the_unit_area)
                    * u128::from(percent))
                .div_ceil(100);
                let lowest = u128::from(lowest_line_in_segments);
                let line = percent_of_the_segments
                    .max(lowest)
                    .min(u128::from(cluster_segments_in_the_unit_area));
                Some(u64::try_from(line).expect("线不超过整段数"))
            }
        }
    }
}

/// 碎片度这条水位（越线：`空闲段数 ÷ 已分配槽数 > 线`，且空闲段数不少于绝对下限；D26 已定项 1 的 `F = runs / L`，`L` 取已分配槽数，
/// 条款没定义它，归 C326（`runs` 口径换了之后的 L 没定义））。默认关。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentationWatermark {
    Off,
    /// 线是 `free_runs_above ÷ per_allocated_slots`；`at_least_free_runs` 是这一支开了必带的绝对下限：
    /// 用得很少的盘上已分配槽数很小而空闲段数至少 1，只看比值近乎空盘就越线。
    FreeRunsPerAllocatedSlotAbove {
        free_runs_above: NonZeroU64,
        per_allocated_slots: NonZeroU64,
        at_least_free_runs: NonZeroU64,
    },
}

/// 不可回收字节这条水位（越线：不可回收字节 ≥ 线）。读数第一版恒 0（[`UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED`]），开了也不越线；默认关。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnreclaimableBytesWatermark {
    Off,
    AtLeast(NonZeroU64),
}

/// 三条水位线（D26 已定项 1：三水位任一越线起跑，带设备维）。挂载参数，不存盘：盘上系统配置那 24 字节是全 0 的预留。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionWatermarks {
    pub empty_cluster_segments: EmptyClusterSegmentsWatermark,
    pub fragmentation: FragmentationWatermark,
    pub unreclaimable_bytes: UnreclaimableBytesWatermark,
}

impl CompactionWatermarks {
    /// 产品默认：全空聚簇段线 5%、下限 2；碎片度与不可回收字节关。
    pub const BY_DEFAULT: CompactionWatermarks = CompactionWatermarks {
        empty_cluster_segments: EmptyClusterSegmentsWatermark::PercentOfTheClusterSegments {
            percent: EMPTY_CLUSTER_SEGMENTS_LINE_PERCENT_BY_DEFAULT,
            lowest_line_in_segments: EMPTY_CLUSTER_SEGMENTS_LINE_LOWEST_IN_SEGMENTS_BY_DEFAULT,
        },
        fragmentation: FragmentationWatermark::Off,
        unreclaimable_bytes: UnreclaimableBytesWatermark::Off,
    };
}

/// 一块盘此刻的三个量（分配器增量维护的计数，读一次 O(1)，与记账树写出去的是同一份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WatermarkReadingOfADevice {
    pub device: DeviceIdentity,
    pub empty_cluster_segments: u64,
    pub cluster_segments_in_the_unit_area: u64,
    pub free_runs: u64,
    pub allocated_slots: u64,
    pub unreclaimable_bytes: u64,
}

impl WatermarkReadingOfADevice {
    /// 从分配器上这块盘的空闲图读。全空聚簇段数是记录口径：一段里只剩 defer 槽也算全空
    /// （`allocator::DeviceFreeMap::empty_segments`，开段看的是另一个量，`allocator::DeviceFreeMap::lowest_empty_segment`）。
    #[must_use]
    pub fn of_the_device_map(device_map: &DeviceFreeMap) -> Self {
        WatermarkReadingOfADevice {
            device: device_map.device,
            empty_cluster_segments: device_map.empty_segments(),
            cluster_segments_in_the_unit_area: device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS,
            free_runs: device_map.free_runs(),
            allocated_slots: device_map.allocated_slots(),
            unreclaimable_bytes: UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED.0,
        }
    }
}

/// 三个量。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatermarkQuantity {
    EmptyClusterSegments,
    Fragmentation,
    UnreclaimableBytes,
}

/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：不看读数，判起跑判据时当这个量越了线。产品路径恒 [`Self::NotForced`]。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionWatermarkCrossingForcing {
    NotForced,
    ForcedForTests(WatermarkQuantity),
}

/// 一块盘上全空聚簇段的段数：全空聚簇段那条水位的读数与线。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmptyClusterSegmentCount(pub u64);

/// 一块盘上空闲段（连续空闲槽的一段）的段数：碎片度那条水位的读数，线换算到这块盘此刻的已分配槽数上也是它。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FreeRunCount(pub u64);

/// 头一个越线的量，连同它的读数与线。三个量的单位各不相同，各带各的类型。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatermarkCrossing {
    EmptyClusterSegments {
        reading: EmptyClusterSegmentCount,
        line: EmptyClusterSegmentCount,
    },
    /// `line` 是把线换算到这块盘的已分配槽数上的空闲段数。
    Fragmentation {
        reading: FreeRunCount,
        line: FreeRunCount,
    },
    UnreclaimableBytes {
        reading: BytesOnOneDevice,
        line: BytesOnOneDevice,
    },
}

/// 起跑判据的结论，连同走了哪一支（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatermarkVerdict {
    /// 只供测试的开关强制起跑（[`CompactionRunForcing::ForcedToStartForTests`]）。
    ForcedToStartByTheTestOnlySwitch,
    /// 只供测试的开关强制不起跑（[`CompactionRunForcing::ForcedToStopForTests`]）。
    ForcedToStopByTheTestOnlySwitch,
    /// 只供测试的开关当这个量越了线（[`CompactionWatermarkCrossingForcing::ForcedForTests`]）。
    CrossingForcedByTheTestOnlySwitch(WatermarkQuantity),
    /// 每块盘的三个量都没越线。
    NoLineCrossed,
    /// 按盘的次序、每块盘按全空聚簇段、碎片度、不可回收字节的次序头一个越线的。
    Crossed {
        device: DeviceIdentity,
        crossing: WatermarkCrossing,
    },
}

impl WatermarkVerdict {
    /// 这一结论起不起跑。
    #[must_use]
    pub const fn starts(self) -> bool {
        match self {
            WatermarkVerdict::ForcedToStartByTheTestOnlySwitch
            | WatermarkVerdict::CrossingForcedByTheTestOnlySwitch(_)
            | WatermarkVerdict::Crossed { .. } => true,
            WatermarkVerdict::ForcedToStopByTheTestOnlySwitch | WatermarkVerdict::NoLineCrossed => {
                false
            }
        }
    }
}

/// 一块盘头一个越线的量（全空聚簇段、碎片度、不可回收字节的次序），连同读数与线。
fn watermark_crossed_on_the_device(
    watermarks: &CompactionWatermarks,
    reading: &WatermarkReadingOfADevice,
) -> Option<WatermarkCrossing> {
    if let Some(line) = watermarks
        .empty_cluster_segments
        .line_in_segments(reading.cluster_segments_in_the_unit_area)
    {
        if reading.empty_cluster_segments < line {
            return Some(WatermarkCrossing::EmptyClusterSegments {
                reading: EmptyClusterSegmentCount(reading.empty_cluster_segments),
                line: EmptyClusterSegmentCount(line),
            });
        }
    }
    match watermarks.fragmentation {
        FragmentationWatermark::Off => {}
        FragmentationWatermark::FreeRunsPerAllocatedSlotAbove {
            free_runs_above,
            per_allocated_slots,
            at_least_free_runs,
        } => {
            let line_on_this_device = u128::from(free_runs_above.get())
                * u128::from(reading.allocated_slots)
                / u128::from(per_allocated_slots.get());
            let is_above_the_line = u128::from(reading.free_runs)
                * u128::from(per_allocated_slots.get())
                > u128::from(free_runs_above.get()) * u128::from(reading.allocated_slots);
            if reading.allocated_slots > 0
                && is_above_the_line
                && reading.free_runs >= at_least_free_runs.get()
            {
                return Some(WatermarkCrossing::Fragmentation {
                    reading: FreeRunCount(reading.free_runs),
                    line: FreeRunCount(
                        u64::try_from(line_on_this_device).expect("低于空闲段数读数，装得进 u64"),
                    ),
                });
            }
        }
    }
    match watermarks.unreclaimable_bytes {
        UnreclaimableBytesWatermark::Off => None,
        UnreclaimableBytesWatermark::AtLeast(line) => (reading.unreclaimable_bytes >= line.get())
            .then_some(WatermarkCrossing::UnreclaimableBytes {
                reading: BytesOnOneDevice(reading.unreclaimable_bytes),
                line: BytesOnOneDevice(line.get()),
            }),
    }
}

/// 起跑判据（水位越线再跑；D26（后台整理与放置回收） 已定项 1、已定项 13 第 5 条：定时兜底每隔一段时间查一次，每次用户改动发成之后也查一次，
/// 两处查的都是它）：任一块盘的任一个量越线就起跑。只读交进来的读数，代价随盘数、不随盘容量（`.claude/rules/fs-design.md`「记账是事务的副产品」第一格）。
/// 水位只在起跑时查，停机不靠水位（D26 已定项 1 第 2 条），所以没有回滞。
#[must_use]
pub fn compaction_starts_by_the_watermarks(
    forcing: CompactionRunForcing,
    watermarks: &CompactionWatermarks,
    readings: &[WatermarkReadingOfADevice],
) -> WatermarkVerdict {
    match forcing {
        CompactionRunForcing::ForcedToStartForTests => {
            return WatermarkVerdict::ForcedToStartByTheTestOnlySwitch
        }
        CompactionRunForcing::ForcedToStopForTests => {
            return WatermarkVerdict::ForcedToStopByTheTestOnlySwitch
        }
        CompactionRunForcing::FollowTheWatermarks => {}
    }
    for reading in readings {
        if let Some(crossing) = watermark_crossed_on_the_device(watermarks, reading) {
            return WatermarkVerdict::Crossed {
                device: reading.device,
                crossing,
            };
        }
    }
    WatermarkVerdict::NoLineCrossed
}

/// 这个分配器此刻的起跑判据结论：开关、线与每块盘的读数都从它取（[`compaction_starts_by_the_watermarks`]）。
#[must_use]
pub fn compaction_watermark_verdict_of_the_allocator(
    allocator: &PoolAllocator,
) -> WatermarkVerdict {
    let compaction = allocator.compaction();
    match (
        compaction.switches.run_forcing,
        compaction.watermark_crossing_forcing,
    ) {
        (
            CompactionRunForcing::FollowTheWatermarks,
            CompactionWatermarkCrossingForcing::ForcedForTests(quantity),
        ) => WatermarkVerdict::CrossingForcedByTheTestOnlySwitch(quantity),
        (
            CompactionRunForcing::FollowTheWatermarks
            | CompactionRunForcing::ForcedToStartForTests
            | CompactionRunForcing::ForcedToStopForTests,
            CompactionWatermarkCrossingForcing::NotForced
            | CompactionWatermarkCrossingForcing::ForcedForTests(_),
        ) => {
            let readings: Vec<WatermarkReadingOfADevice> = allocator
                .devices
                .iter()
                .map(WatermarkReadingOfADevice::of_the_device_map)
                .collect();
            compaction_starts_by_the_watermarks(
                compaction.switches.run_forcing,
                &compaction.watermarks,
                &readings,
            )
        }
    }
}
