//! 准入抬 F 那一遍在一圈窗口最旧端有洞的根环上（D16（发布语义） 已定项 1；`research/prompts/m3-root-ring-code-r1-main-verification.md`
//! 第二节 K2-b、K2-b2 两格，攻方 `raise_floor_hidden_slot.rs`）。
//!
//! 盘面（S = 4 的 12 槽环）：实例 1 覆盖写到 txg `last`；之后几次可写挂载都崩在写行那次根槽写之前（写行那次的单元与记录已持久、根槽没持久：
//! 那一槽里留着上一圈的根，出了窗口），每次一个洞；再一次可写挂载写行那一版的根 X 落盘、接着两次空发布，正常退出。
//! 第四次可写挂载之后推空发布到锚 = 第一个洞 + 3S − 1（洞成了窗口最旧的几格），准入抬 F。
//!
//! - 上限只取窗口里的有效根（B2，用户 2026-10-03 定）：洞那几槽里上一圈的旧根在窗口外，窗口已许可它们引用的单元被复用；
//!   改之前上限要读它们的树表、读到被复用的槽，准入抬 F 被拒 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（攻方 `rf-14-none.out`）。
//! - 回收门槛把挂载时读不出的那一槽并进窗口里最旧有效根（RR4，攻方 B1）：X 那一槽第四次挂载起一直读不出，根环表按它能住的最大 txg 记着；
//!   改之前抬 F 那一遍只在读得出的根上取门槛，门槛越过 X、回收了 X 引用的槽，记账比遍历少（攻方 `rfo3-17-persist.out`：抬完 I-3.1 红）。
mod common;
mod common_root_ring_holes;

use common::{
    with_unreadable_ranges, DeviceWithUnreadableRanges, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use common_root_ring_holes::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, publish_empty,
    readable_root_at_txg, recording_of, SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, raise_rollback_floor_to_the_admission_ceiling,
    MountError, RaiseToTheAdmissionCeiling, ShadowLedger,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）、F 不高于上限（I-7.9）。
const INVARIANTS_THE_RAISE_KEEPS_GREEN: [&str; 6] =
    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8", "I-7.9"];
/// 回收门槛那一条把 F 抬到的值：攻方 `rfo3-17-persist.out` 那一组准入抬 F 抬到的 6（改 B2 之前那份盘面上的上限），低于 X 的 txg 21——
/// 门槛由「窗口里最旧有效根」那一半定，读不出的 X 算不算进来分得出两种取法。（抬到 B2 之后的上限 22 时 F 高于 X、门槛由 F 定，分不出。）
const FLOOR_RAISED_BELOW_X: CheckpointTxg = CheckpointTxg(6);
/// X 那一版之后、正常退出之前的空发布次数（攻方 `AFTER_X` 的默认值）。
const EMPTY_PUBLISHES_AFTER_X: u64 = 2;

type DevicesOfTheFourthMount = Vec<(
    DeviceIdentity,
    DeviceWithUnreadableRanges<RecordingBlockDevice<SparseBlockDevice>>,
)>;

/// 准入抬 F 之前的那一刻：第四次可写挂载的盘（X 那一槽按 `x_slot_reads` 读坏）、录下的流与它的起点盘面、分配器、现行那一版。
struct PoolJustBeforeTheAdmissionRaise {
    holes: Vec<u64>,
    x_txg: u64,
    image_before_the_fourth_mount: MemoryPool,
    stream: SharedStream,
    devices: DevicesOfTheFourthMount,
    allocator: PoolAllocator,
    current: TransactionOutput,
    instance: InstanceGeneration,
}

impl PoolJustBeforeTheAdmissionRaise {
    /// 盘上此刻的样子：起点盘面加上第四次挂载以来录下的全部写（读故障只在读那一侧）。
    fn image_now(&self) -> MemoryPool {
        let mut image = self.image_before_the_fourth_mount.clone();
        image.apply(&self.stream.retained_operations());
        image
    }
}

fn violations_of(image: &MemoryPool, invariants_judged: &[&str]) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(invariant, _)| invariants_judged.contains(invariant))
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

/// 造盘面到准入抬 F 之前：实例 1 覆盖写到 `last`，`hole_count` 次挂载崩在写行根槽之前，第三次（洞之后那一次）挂载写下 X 再空发布两次，
/// 第四次挂载（X 那一槽按 `x_slot_reads` 读坏）之后空发布推到锚。
fn pool_just_before_the_admission_raise(
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


/// 探针（正推腿，K1）：与原第二条用例同一盘面（X = 21 那一槽第四次挂载起一直读不出），但准入抬 F 走「抬到上限」入口。
#[test]
fn probe_admission_raise_to_the_ceiling_over_an_unreadable_x() {
    let mut pool = pool_just_before_the_admission_raise(17, 3, FailingReadsOfARange::Every);
    let parameters = parameters_of_the_ring_hole_pools();
    let floor_before = pool.current.root.rollback_floor;
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.current,
        ShadowLedger::On,
    );
    let summary = match &raised {
        Ok(RaiseToTheAdmissionCeiling::Raised(r)) => format!("Raised ceiling={:?}", r.ceiling),
        Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
            format!("AlreadyAtCeiling floor={floor:?} ceiling={ceiling:?}")
        }
        Err(e) => format!("Err {e:?}"),
    };
    println!("PROBE_RAISED floor_before={floor_before:?} anchor_txg={} => {summary}", pool.current.root.checkpoint_txg.0);
    println!("PROBE_ROOT floor_after={:?}", pool.current.root.rollback_floor);
    let violations = violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN);
    println!("PROBE_VIOLATIONS count={}", violations.len());
    for v in &violations {
        println!("PROBE_VIOLATION {v}");
    }
}

/// 探针（正推腿，K1）：与原第二条用例同一盘面（X = 21 那一槽第四次挂载起一直读不出），但准入抬 F 走「抬到上限」入口。
#[test]
fn probe_admission_raise_to_the_ceiling_with_x_readable() {
    let mut pool = pool_just_before_the_admission_raise(17, 3, FailingReadsOfARange::Never);
    let parameters = parameters_of_the_ring_hole_pools();
    let floor_before = pool.current.root.rollback_floor;
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &parameters,
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.current,
        ShadowLedger::On,
    );
    let summary = match &raised {
        Ok(RaiseToTheAdmissionCeiling::Raised(r)) => format!("Raised ceiling={:?}", r.ceiling),
        Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
            format!("AlreadyAtCeiling floor={floor:?} ceiling={ceiling:?}")
        }
        Err(e) => format!("Err {e:?}"),
    };
    println!("PROBEREADABLE_RAISED floor_before={floor_before:?} anchor_txg={} => {summary}", pool.current.root.checkpoint_txg.0);
    println!("PROBEREADABLE_ROOT floor_after={:?}", pool.current.root.rollback_floor);
    let violations = violations_of(&pool.image_now(), &INVARIANTS_THE_RAISE_KEEPS_GREEN);
    println!("PROBEREADABLE_VIOLATIONS count={}", violations.len());
    for v in &violations {
        println!("PROBEREADABLE_VIOLATION {v}");
    }
}
