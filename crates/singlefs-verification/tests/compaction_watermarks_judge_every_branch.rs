//! 起跑判据（`compaction_plan::compaction_starts_by_the_watermarks`，D26（后台整理与放置回收） 已定项 1）的每一条分支：读数手造、纯函数直接判；
//! 另两格走分配器：新 mkfs 各容量不越线，只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）能强制进每个量那一支。
mod common;

use std::num::NonZeroU64;

use common::{build_pool, parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::BytesOnOneDevice;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{
    compaction_starts_by_the_watermarks, compaction_watermark_verdict_of_the_allocator,
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionBatchSize, CompactionObservations, CompactionPacing,
    CompactionRunForcing, CompactionStartJudgement, CompactionSwitches, CompactionTimerFallback,
    CompactionWatermarkCrossingForcing, CompactionWatermarks, CompactionWriteBandwidthLimit,
    EmptyClusterSegmentCount, EmptyClusterSegmentsWatermark, FragmentationWatermark, FreeRunCount,
    UnreclaimableBytesWatermark, WatermarkCrossing, WatermarkQuantity, WatermarkReadingOfADevice,
    WatermarkVerdict, COMPACTION_BATCH_UNITS_BY_DEFAULT,
};
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_verification::memory_pool::SparseBlockDevice;

const FIRST_DEVICE: DeviceIdentity = DeviceIdentity(0);
const SECOND_DEVICE: DeviceIdentity = DeviceIdentity(1);

fn non_zero(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("用例给的数不是 0")
}

/// 三条都关。
const ALL_OFF: CompactionWatermarks = CompactionWatermarks {
    empty_cluster_segments: EmptyClusterSegmentsWatermark::Off,
    fragmentation: FragmentationWatermark::Off,
    unreclaimable_bytes: UnreclaimableBytesWatermark::Off,
};

/// 一块读数都在「不越线」那一边的盘：1000 段全空、1 个空闲段、一个槽都没分配、不可回收 0。
fn reading_of_an_empty_device(device: DeviceIdentity) -> WatermarkReadingOfADevice {
    WatermarkReadingOfADevice {
        device,
        empty_cluster_segments: 1000,
        cluster_segments_in_the_unit_area: 1000,
        free_runs: 1,
        allocated_slots: 0,
        unreclaimable_bytes: 0,
    }
}

fn judged(
    watermarks: &CompactionWatermarks,
    readings: &[WatermarkReadingOfADevice],
) -> WatermarkVerdict {
    compaction_starts_by_the_watermarks(
        CompactionRunForcing::FollowTheWatermarks,
        watermarks,
        readings,
    )
}

/// 钟停在 0、前台没有 fsync 在等的调用方：头一次查定时兜底就到点。
struct CallerAtTheStartOfTheClock;

impl CallerOfTheBackgroundCompaction for CallerAtTheStartOfTheClock {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(std::time::Duration::ZERO)
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
    }
}

/// 起跑判据的两处都把结论留在会话上（`.claude/rules/fs-design.md` 五条硬要求第 4 条「分支必须可观测」）：用户改动之后那一判没越线、整理一趟都没起跑也留，
/// 答的就是为什么没起跑；起跑之后才记的观测照旧是默认值。定时兜底那一判同样留。
#[test]
fn the_session_keeps_the_verdict_of_each_judgement_even_when_no_run_starts() {
    let mut pool = build_pool("watermark-verdict-kept-on-the-session");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    assert_eq!(
        session.compaction.last_watermark_verdict, None,
        "挂载之后还没判过"
    );
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 1,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
        )
        .expect("建一个 inode");
    assert_eq!(
        session.compaction.last_watermark_verdict,
        Some(WatermarkVerdict::NoLineCrossed),
        "用户改动之后那一判没越线：结论留着"
    );
    assert_eq!(
        session.compaction.start_judgement,
        CompactionStartJudgement::NotDue
    );
    assert_eq!(
        session.compaction.observations,
        CompactionObservations::default(),
        "整理一趟都没起跑"
    );
    session
        .allocator
        .compaction_mut()
        .watermark_crossing_forcing =
        CompactionWatermarkCrossingForcing::ForcedForTests(WatermarkQuantity::EmptyClusterSegments);
    let _call_of_the_timer_fallback = session
        .run_compaction_in_the_background(
            &mut devices,
            CompactionPacing {
                timer_fallback: CompactionTimerFallback::EveryMilliseconds(non_zero(1)),
                write_bandwidth_limit: CompactionWriteBandwidthLimit::Unlimited,
            },
            &CallerAtTheStartOfTheClock,
        )
        .expect("盘表是挂载的那一份");
    assert_eq!(
        session.compaction.last_watermark_verdict,
        Some(WatermarkVerdict::CrossingForcedByTheTestOnlySwitch(
            WatermarkQuantity::EmptyClusterSegments
        )),
        "定时兜底那一判的结论留着"
    );
}

/// 批大小的产品默认是 16（D26（后台整理与放置回收） 已定项 1，挂载参数）。
#[test]
fn the_default_batch_size_is_sixteen() {
    assert_eq!(COMPACTION_BATCH_UNITS_BY_DEFAULT.units(), 16);
    assert_eq!(
        CompactionSwitches::PRODUCT.batch_size,
        CompactionBatchSize::OfTheMount(COMPACTION_BATCH_UNITS_BY_DEFAULT)
    );
}

#[test]
fn with_every_line_off_no_reading_crosses() {
    let reading = WatermarkReadingOfADevice {
        device: FIRST_DEVICE,
        empty_cluster_segments: 0,
        cluster_segments_in_the_unit_area: 1000,
        free_runs: 1 << 40,
        allocated_slots: 1,
        unreclaimable_bytes: u64::MAX,
    };
    assert_eq!(
        judged(&ALL_OFF, &[reading]),
        WatermarkVerdict::NoLineCrossed
    );
}

/// 线 = max(⌈整段数 × 5%⌉, 2)，再不超过整段数：只有一两个整段的小盘刚 mkfs 时不越线。
#[test]
fn the_default_line_of_the_empty_cluster_segments_is_five_percent_with_a_floor_of_two_within_the_device(
) {
    let watermark = CompactionWatermarks::BY_DEFAULT.empty_cluster_segments;
    for (cluster_segments, expected_line) in [
        (0_u64, 0_u64),
        (1, 1),
        (2, 2),
        (6, 2),
        (40, 2),
        (41, 3),
        (100, 5),
        (101, 6),
        (1_048_576, 52_429),
    ] {
        assert_eq!(
            watermark.line_in_segments(cluster_segments),
            Some(expected_line),
            "{cluster_segments} 段"
        );
    }
    assert_eq!(
        EmptyClusterSegmentsWatermark::Off.line_in_segments(100),
        None
    );
}

#[test]
fn the_empty_cluster_segments_cross_strictly_below_the_line_and_the_verdict_names_them() {
    let at_the_line = WatermarkReadingOfADevice {
        empty_cluster_segments: 50,
        ..reading_of_an_empty_device(FIRST_DEVICE)
    };
    assert_eq!(
        judged(&CompactionWatermarks::BY_DEFAULT, &[at_the_line]),
        WatermarkVerdict::NoLineCrossed,
        "等于线不越线"
    );
    let below_the_line = WatermarkReadingOfADevice {
        empty_cluster_segments: 49,
        ..reading_of_an_empty_device(FIRST_DEVICE)
    };
    assert_eq!(
        judged(&CompactionWatermarks::BY_DEFAULT, &[below_the_line]),
        WatermarkVerdict::Crossed {
            device: FIRST_DEVICE,
            crossing: WatermarkCrossing::EmptyClusterSegments {
                reading: EmptyClusterSegmentCount(49),
                line: EmptyClusterSegmentCount(50),
            },
        }
    );
}

/// 任一块盘越线就起跑：第一块不越线、第二块越线，报第二块。
#[test]
fn a_line_crossed_on_any_device_starts_the_compaction_and_names_the_device() {
    let readings = [
        reading_of_an_empty_device(FIRST_DEVICE),
        WatermarkReadingOfADevice {
            empty_cluster_segments: 1,
            ..reading_of_an_empty_device(SECOND_DEVICE)
        },
    ];
    let verdict = judged(&CompactionWatermarks::BY_DEFAULT, &readings);
    assert!(verdict.starts());
    assert!(
        matches!(verdict, WatermarkVerdict::Crossed { device, .. } if device == SECOND_DEVICE),
        "{verdict:?}"
    );
}

/// 碎片度：`空闲段数 ÷ 已分配槽数 > 线`（严格大于）且不少于绝对下限；一个槽都没分配不越线。
#[test]
fn the_fragmentation_crosses_above_its_line_only_with_at_least_the_absolute_floor_of_runs() {
    let watermarks = CompactionWatermarks {
        empty_cluster_segments: EmptyClusterSegmentsWatermark::Off,
        fragmentation: FragmentationWatermark::FreeRunsPerAllocatedSlotAbove {
            free_runs_above: non_zero(1),
            per_allocated_slots: non_zero(4),
            at_least_free_runs: non_zero(10),
        },
        unreclaimable_bytes: UnreclaimableBytesWatermark::Off,
    };
    let reading_with = |free_runs: u64, allocated_slots: u64| WatermarkReadingOfADevice {
        free_runs,
        allocated_slots,
        ..reading_of_an_empty_device(FIRST_DEVICE)
    };
    assert_eq!(
        judged(&watermarks, &[reading_with(10, 40)]),
        WatermarkVerdict::NoLineCrossed,
        "正好等于线 1/4 不越线"
    );
    assert_eq!(
        judged(&watermarks, &[reading_with(11, 40)]),
        WatermarkVerdict::Crossed {
            device: FIRST_DEVICE,
            crossing: WatermarkCrossing::Fragmentation {
                reading: FreeRunCount(11),
                line: FreeRunCount(10),
            },
        },
        "高过线"
    );
    assert_eq!(
        judged(&watermarks, &[reading_with(9, 4)]),
        WatermarkVerdict::NoLineCrossed,
        "比值高过线，空闲段数不到绝对下限"
    );
    assert_eq!(
        judged(&watermarks, &[reading_with(100, 0)]),
        WatermarkVerdict::NoLineCrossed,
        "一个槽都没分配"
    );
}

/// 不可回收字节：不小于线就越线；第一版读数恒 0，开了也不越线。
#[test]
fn the_unreclaimable_bytes_cross_at_or_above_their_line() {
    let watermarks = CompactionWatermarks {
        empty_cluster_segments: EmptyClusterSegmentsWatermark::Off,
        fragmentation: FragmentationWatermark::Off,
        unreclaimable_bytes: UnreclaimableBytesWatermark::AtLeast(non_zero(4096)),
    };
    let reading_with = |unreclaimable_bytes: u64| WatermarkReadingOfADevice {
        unreclaimable_bytes,
        ..reading_of_an_empty_device(FIRST_DEVICE)
    };
    assert_eq!(
        judged(&watermarks, &[reading_with(0)]),
        WatermarkVerdict::NoLineCrossed
    );
    assert_eq!(
        judged(&watermarks, &[reading_with(4095)]),
        WatermarkVerdict::NoLineCrossed
    );
    assert_eq!(
        judged(&watermarks, &[reading_with(4096)]),
        WatermarkVerdict::Crossed {
            device: FIRST_DEVICE,
            crossing: WatermarkCrossing::UnreclaimableBytes {
                reading: BytesOnOneDevice(4096),
                line: BytesOnOneDevice(4096),
            },
        }
    );
}

/// 只供测试的开关压过读数：强制起跑、强制不起跑。
#[test]
fn the_test_only_forcing_overrides_the_readings() {
    let below_the_line = WatermarkReadingOfADevice {
        empty_cluster_segments: 0,
        ..reading_of_an_empty_device(FIRST_DEVICE)
    };
    assert_eq!(
        compaction_starts_by_the_watermarks(
            CompactionRunForcing::ForcedToStopForTests,
            &CompactionWatermarks::BY_DEFAULT,
            &[below_the_line],
        ),
        WatermarkVerdict::ForcedToStopByTheTestOnlySwitch
    );
    assert_eq!(
        compaction_starts_by_the_watermarks(
            CompactionRunForcing::ForcedToStartForTests,
            &ALL_OFF,
            &[reading_of_an_empty_device(FIRST_DEVICE)],
        ),
        WatermarkVerdict::ForcedToStartByTheTestOnlySwitch
    );
    assert!(WatermarkVerdict::ForcedToStartByTheTestOnlySwitch.starts());
    assert!(!WatermarkVerdict::ForcedToStopByTheTestOnlySwitch.starts());
    assert!(!WatermarkVerdict::NoLineCrossed.starts());
}

/// 新 mkfs 各容量判不越线（默认线）；分配器上的开关强制进每个量那一支，叫停的开关压过它。
#[test]
fn a_freshly_made_pool_of_every_capacity_is_under_the_default_line_and_the_switch_forces_each_quantity(
) {
    for device_bytes in [4_u64 << 30, 64 << 30, 1 << 40] {
        let pool_parameters = parameters();
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [FIRST_DEVICE, SECOND_DEVICE]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
        let genesis = make_filesystem(&pool_parameters, &mut devices).expect("mkfs");
        let mut allocator = allocator_after_make_filesystem(&pool_parameters, &devices, &genesis);
        assert_eq!(
            compaction_watermark_verdict_of_the_allocator(&allocator),
            WatermarkVerdict::NoLineCrossed,
            "{device_bytes} 字节的盘刚 mkfs"
        );
        for quantity in [
            WatermarkQuantity::EmptyClusterSegments,
            WatermarkQuantity::Fragmentation,
            WatermarkQuantity::UnreclaimableBytes,
        ] {
            allocator.compaction_mut().watermark_crossing_forcing =
                CompactionWatermarkCrossingForcing::ForcedForTests(quantity);
            assert_eq!(
                compaction_watermark_verdict_of_the_allocator(&allocator),
                WatermarkVerdict::CrossingForcedByTheTestOnlySwitch(quantity)
            );
        }
    }
}
