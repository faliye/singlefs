//! harness 测试文件 `crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    with_unreadable_ranges, DeviceWithUnreadableRanges, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_root_ring_holes::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, publish_empty,
    readable_root_at_txg, recording_of, SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, raise_rollback_floor_to_the_admission_ceiling,
    MountError, RaiseToTheAdmissionCeiling, ShadowLedger,
};
use singlefs_core::recovery::allocation_records_under_root;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）、F 不高于上限（I-7.9）。
pub const INVARIANTS_THE_RAISE_KEEPS_GREEN: [&str; 6] =
    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8", "I-7.9"];

/// 回收门槛那一条把 F 抬到的值：攻方 `rfo3-17-persist.out` 那一组准入抬 F 抬到的 6（改 B2 之前那份盘面上的上限），低于 X 的 txg 21——
/// 门槛由「窗口里最旧有效根」那一半定，读不出的 X 算不算进来分得出两种取法。（抬到 B2 之后的上限 22 时 F 高于 X、门槛由 F 定，分不出。）
pub const FLOOR_RAISED_BELOW_X: CheckpointTxg = CheckpointTxg(6);

/// X 那一版之后、正常退出之前的空发布次数（攻方 `AFTER_X` 的默认值）。
pub const EMPTY_PUBLISHES_AFTER_X: u64 = 2;

pub type DevicesOfTheFourthMount = Vec<(
    DeviceIdentity,
    DeviceWithUnreadableRanges<RecordingBlockDevice<SparseBlockDevice>>,
)>;

/// 准入抬 F 之前的那一刻：第四次可写挂载的盘（X 那一槽按 `x_slot_reads` 读坏）、录下的流与它的起点盘面、分配器、现行那一版。
pub struct PoolJustBeforeTheAdmissionRaise {
    pub holes: Vec<u64>,
    pub x_txg: u64,
    pub image_before_the_fourth_mount: MemoryPool,
    pub stream: SharedStream,
    pub devices: DevicesOfTheFourthMount,
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
    pub instance: InstanceGeneration,
}

impl PoolJustBeforeTheAdmissionRaise {
    /// 盘上此刻的样子：起点盘面加上第四次挂载以来录下的全部写（读故障只在读那一侧）。
    fn image_now(&self) -> MemoryPool {
        let mut image = self.image_before_the_fourth_mount.clone();
        image.apply(&self.stream.retained_operations());
        image
    }
}

/// 造盘面到准入抬 F 之前：实例 1 覆盖写到 `last`，`hole_count` 次挂载崩在写行根槽之前，第三次（洞之后那一次）挂载写下 X 再空发布两次，
/// 第四次挂载（X 那一槽按 `x_slot_reads` 读坏）之后空发布推到锚。
pub fn pool_just_before_the_admission_raise(
    last: u64,
    hole_count: usize,
    x_slot_reads: FailingReadsOfARange,
) -> PoolJustBeforeTheAdmissionRaise {
    let parameters = parameters_of_the_ring_hole_pools();
    let slots_per_region = parameters.geometry.root_ring_slots_per_region;
    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
    let mut image = pool_overwritten_through(last);
    let mut holes = Vec::new();
    // 迭代上界：`hole_count` 次挂载，每次一个洞；跨轮携带的是崩溃之后的盘面。
    for _ in 0..hole_count {
        let (crashed, row_txg) = crash_a_mount_before_its_row_root(&image);
        holes.push(row_txg);
        image = crashed;
    }
    let x_txg = {
        let mut devices = devices_of(&image);
        let mounted = mount_writable(&parameters, &mut devices).expect("洞之后那一次可写挂载");
        let x_txg = mounted.output.row_publish.root().checkpoint_txg.0;
        let instance = mounted.output.instance;
        let mut allocator = mounted.allocator.clone();
        let mut current = mounted.current.into_file_version().expect("这一版带文件");
        // 迭代上界：两次空发布。
        for _ in 0..EMPTY_PUBLISHES_AFTER_X {
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            current =
                publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
        }
        image = pool_of(&devices);
        x_txg
    };
    assert!(
        readable_root_at_txg(&image, x_txg).is_some(),
        "X 那一版的根落盘了"
    );
    let x_slot = target_for_publish(CheckpointTxg(x_txg), slots_per_region);
    let unreadable_x_slot = SharedUnreadableRanges::new(
        vec![UnreadableRange {
            device: parameters.region_devices[usize::try_from(x_slot.region).expect("区域号")],
            offset_in_bytes: slot_offset(x_slot, parameters.geometry.fixed_structure_slot_spacing)
                .0,
            length_in_bytes: u64::from(parameters.geometry.physical_block_size),
            failing_reads: x_slot_reads,
        }],
        UnreadableRangeReadBack::DeviceError,
    );
    let stream = SharedStream::retaining_contents();
    let mut devices = with_unreadable_ranges(recording_of(&image, &stream), &unreadable_x_slot);
    let mounted = mount_writable(&parameters, &mut devices).expect("第四次可写挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.into_file_version().expect("这一版带文件");
    let anchor = holes[0] + ring_length - 1;
    // 迭代上界：txg 逐次加一推到锚。
    while current.root.checkpoint_txg.0 < anchor {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
    }
    PoolJustBeforeTheAdmissionRaise {
        holes,
        x_txg,
        image_before_the_fourth_mount: image,
        stream,
        devices,
        allocator,
        current,
        instance,
    }
}

/// 场景：实例 1 覆盖写到 txg 14，两次挂载崩在写行根槽之前（洞 15、16），X = 17；第四次挂载没有读故障，推到锚 26，准入抬 F。
/// 预期：抬 F 做成（不拒 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`），抬完盘上那几条不变量不红。
/// 改之前上限读洞里窗口外 txg 3 那条旧根的树表，那个槽已被复用、读不出，准入抬 F 被拒（攻方 `rf-14-none.out`）。
pub fn admission_raising_the_floor_above_holes_at_the_oldest_end_of_the_window_takes_its_ceiling_only_from_roots_inside_the_window(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_just_before_the_admission_raise(14, 2, FailingReadsOfARange::Never);
    assert_eq!(
        (pool.holes.clone(), pool.x_txg),
        (vec![15, 16], 17),
        "两个洞紧跟着 X"
    );
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &parameters_of_the_ring_hole_pools(),
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.current,
        ShadowLedger::On,
    );
    assert!(
        matches!(
            raised,
            Ok(RaiseToTheAdmissionCeiling::Raised(_)
                | RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. })
        ),
        "上限只取窗口里的有效根，不读窗口外旧根的树表：{raised:?}"
    );
    assert!(
        !matches!(
            raised,
            Err(MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. })
        ),
        "没有读故障的合法历史上准入抬 F 不因窗口外旧根的树表被拒"
    );
    images.hand_out(
        "抬完之后",
        || pool.image_now(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_RAISE_KEEPS_GREEN),
    );
}

/// I-3.1 那一条红在记账比遍历多的一边：说明里「记账的已分配 Some(a)，遍历全部有效根得到 b」a 大于 b。
pub fn allocated_statistic_counts_more_than_the_walk(violation: &str) -> bool {
    let after = |marker: &str| {
        violation.split(marker).nth(1).and_then(|rest| {
            rest.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u64>()
                .ok()
        })
    };
    matches!(
        (after("记账的已分配 Some("), after("遍历全部有效根得到 ")),
        (Some(counted), Some(walked)) if counted > walked
    )
}

/// 场景：实例 1 覆盖写到 txg 17，三次挂载崩在写行根槽之前（洞 18、19、20），X = 21；第四次挂载起 X 那一槽一直读不出，推到锚 29，
/// 准入抬 F 到 [`FLOOR_RAISED_BELOW_X`]，再覆盖写三次（txg 32、33、34；33 写掉 X 那一槽）。
/// 预期：抬 F 做成；X 那一槽钉在它可能住过的最旧 txg 9（用户 2026-10-04 定），抬 F 那一遍与之后的进程内回收都不越过它：
/// X 那一版引用的单元在这个进程写掉那一槽之前一个都不空闲，被引用单元那几条一直不红。
/// 定案的代价：那一槽钉着的 9 比盘上环里最旧的有效根旧，池级 checker 读盘上完好的 X 按它算门槛，那几版 I-3.1 只许红在记账多的一边（安全方向）；
/// 33 写掉那一槽之后不再钉，那几条全绿。改 RR4 之前抬完 I-3.1 红在记账少的一边（攻方 `rfo3-17-persist.out`）。
pub fn admission_raising_the_floor_keeps_the_root_ring_slot_unreadable_since_the_mount_pinned_at_the_oldest_txg_it_can_have_held(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_just_before_the_admission_raise(17, 3, FailingReadsOfARange::Every);
    assert_eq!(
        (pool.holes.clone(), pool.x_txg),
        (vec![18, 19, 20], 21),
        "三个洞紧跟着 X"
    );
    let parameters = parameters_of_the_ring_hole_pools();
    let x_slot = target_for_publish(
        CheckpointTxg(pool.x_txg),
        parameters.geometry.root_ring_slots_per_region,
    );
    let txg_that_overwrites_the_x_slot = pool.x_txg + 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
    let x_root = readable_root_at_txg(&pool.image_now(), pool.x_txg).expect("盘上 X 那一槽完好");
    let slots_x_references: Vec<(DeviceIdentity, SlotNumber)> =
        allocation_records_under_root(&pool.image_now(), &x_root)
            .expect("X 那一版的账读得出")
            .iter()
            .filter(|record| !record.is_released)
            .flat_map(|record| {
                (record.slot.0..record.slot.0 + u64::from(record.span_slots))
                    .map(move |slot| (record.device, SlotNumber(slot)))
            })
            .collect();
    let slots_of_x_left_free = |allocator: &PoolAllocator| -> Vec<(DeviceIdentity, SlotNumber)> {
        slots_x_references
            .iter()
            .copied()
            .filter(|(device, slot)| {
                allocator
                    .devices
                    .iter()
                    .find(|device_map| device_map.device == *device)
                    .expect("账里的盘在分配器里")
                    .is_free(*slot)
            })
            .collect()
    };
    let mut judge = |pool_being_judged: &PoolJustBeforeTheAdmissionRaise, when: &str| {
        let txg = pool_being_judged.current.root.checkpoint_txg.0;
        if txg < txg_that_overwrites_the_x_slot {
            assert_eq!(
                slots_of_x_left_free(&pool_being_judged.allocator),
                Vec::new(),
                "{when}（txg {txg}）：X 那一槽钉在 9，X 引用的单元一个都不空闲"
            );
            // 只许 I-3.1 红在记账多的一边
            images.hand_out(
                &format!("{when}（txg {txg}，X 那一槽还钉着）"),
                || pool_being_judged.image_now(),
                PoolCheckerExpectation::only_the_named_invariants()
                    .and_none_of_these_violated(&INVARIANTS_THE_RAISE_KEEPS_GREEN[1..])
                    .and_if_violated_the_detail_satisfies(
                        INVARIANTS_THE_RAISE_KEEPS_GREEN[0],
                        "记账的已分配多于遍历全部有效根得到的",
                        allocated_statistic_counts_more_than_the_walk,
                    ),
            );
        } else {
            // X 那一槽写掉了，不再钉
            images.hand_out(
                &format!("{when}（txg {txg}，X 那一槽写掉了）"),
                || pool_being_judged.image_now(),
                PoolCheckerExpectation::only_the_named_invariants()
                    .and_none_of_these_violated(&INVARIANTS_THE_RAISE_KEEPS_GREEN),
            );
        }
    };
    assert_eq!(
        pool.allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表")
            .ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held()
            .get(&x_slot),
        Some(&CheckpointTxg(9)),
        "前提：X 那一槽钉在它可能住过的最旧 txg 9"
    );
    let raised = raise_rollback_floor(
        &parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.current,
        FLOOR_RAISED_BELOW_X,
        ShadowLedger::On,
    );
    assert!(raised.is_ok(), "准入抬 F 做成：{raised:?}");
    judge(&pool, "抬完之后");
    // 迭代上界：三次覆盖写；跨轮携带的是现行那一版。
    for step in 0..3u64 {
        let salt = 5000 + step;
        pool.current = {
            let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut pool.allocator,
                &pool.current,
                FirstFile {
                    content: &content_of_the_version(salt),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
                },
                pool.instance,
            )
            .expect("覆盖写")
        };
        judge(&pool, &format!("抬完之后第 {step} 次覆盖写"));
    }
    assert!(
        pool.current.root.checkpoint_txg.0 >= txg_that_overwrites_the_x_slot,
        "前提：覆盖写走到了写掉 X 那一槽的那一版"
    );
}
