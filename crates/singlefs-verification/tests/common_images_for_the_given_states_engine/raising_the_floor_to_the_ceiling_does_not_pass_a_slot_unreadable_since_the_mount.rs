//! harness 测试文件 `crates/singlefs-verification/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_given_states_engine/mod.rs`）。那份 harness 测试文件与给定状态那一份
//! `crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "引它的测试目标各只调其中一部分")]

use crate::common::{
    with_unreadable_ranges, DeviceWithUnreadableRanges, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use crate::common_images_for_the_given_states_engine::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_root_ring_holes::{
    crash_a_mount_before_its_row_root, devices_of, parameters_of_the_ring_hole_pools, pool_of,
    pool_overwritten_through, publish_empty, readable_root_at_txg, recording_of,
    SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
};
use crate::common_shared_by_harness_scenarios::EMPTY_PUBLISHES_AFTER_X;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling,
    ring_slots_bad_or_reading_back_all_zero_at_mount_by, RaiseToTheAdmissionCeiling, ShadowLedger,
};
use singlefs_core::recovery::allocation_records_under_root;
use singlefs_core::root_ring::RootRingSlot;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{PoolWriter, TransactionOutput};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 回退下界 F 不高于抬 F 的上限。
pub const ROLLBACK_FLOOR_NOT_ABOVE_THE_RAISE_CEILING: &str = "I-7.9";

/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）、F 不高于上限（I-7.9）。
pub const INVARIANTS_THE_RAISE_TO_THE_CEILING_KEEPS_GREEN: [&str; 6] = [
    "I-3.1",
    "I-3.11",
    "I-2.1",
    "I-7.4",
    "I-4.8",
    ROLLBACK_FLOOR_NOT_ABOVE_THE_RAISE_CEILING,
];

/// 有洞的那一份：实例 1 覆盖写到的 txg。
pub const LAST_TXG_OF_INSTANCE_ONE_BEFORE_THE_HOLES: u64 = 17;

/// 有洞的那一份：崩在写行根槽之前的挂载次数（洞 18、19、20）。
pub const HOLES: usize = 3;

/// 有洞的那一份：X 的 txg。
pub const X_TXG_AFTER_THE_HOLES: u64 = 21;

/// 有洞的那一份 RGall 之后的上限：洞里留着的 txg 6、7、8 与 22 那四个不同状态里第 4 新的那条。
pub const CEILING_OVER_THE_HOLES: CheckpointTxg = CheckpointTxg(6);

/// 没洞的那一份：X 的 txg（实例 1 最后一次覆盖写）。
pub const X_TXG_WITHOUT_HOLES: u64 = 17;

pub type DevicesOfTheRaisingMount = Vec<(
    DeviceIdentity,
    DeviceWithUnreadableRanges<RecordingBlockDevice<SparseBlockDevice>>,
)>;

/// 准入抬 F 之前的那一刻：抬 F 那一次可写挂载的盘（X 那一槽按 `x_slot_reads`、`x_slot_read_back` 读坏）、录下的流与它的起点盘面、分配器、现行那一版。
pub struct PoolJustBeforeTheAdmissionRaiseToTheCeiling {
    pub holes: Vec<u64>,
    pub x_txg: u64,
    pub image_before_the_raising_mount: MemoryPool,
    pub stream: SharedStream,
    pub devices: DevicesOfTheRaisingMount,
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
}

impl PoolJustBeforeTheAdmissionRaiseToTheCeiling {
    /// 盘上此刻的样子：起点盘面加上抬 F 那一次挂载以来录下的全部写（读故障只在读那一侧）。
    fn image_now(&self) -> MemoryPool {
        let mut image = self.image_before_the_raising_mount.clone();
        image.apply(&self.stream.retained_operations());
        image
    }
}

/// 在 `image` 上做抬 F 的那一次可写挂载（X 那一槽从这一次挂载起按 `x_slot_reads`、`x_slot_read_back` 读坏），之后空发布推到 `anchor`。
pub fn mount_reading_the_x_slot_bad_and_publish_empty_through(
    image: MemoryPool,
    holes: Vec<u64>,
    x_txg: u64,
    x_slot_reads: FailingReadsOfARange,
    x_slot_read_back: UnreadableRangeReadBack,
    anchor: u64,
) -> PoolJustBeforeTheAdmissionRaiseToTheCeiling {
    let parameters = parameters_of_the_ring_hole_pools();
    let slots_per_region = parameters.geometry.root_ring_slots_per_region;
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
        x_slot_read_back,
    );
    let stream = SharedStream::retaining_contents();
    let mut devices = with_unreadable_ranges(recording_of(&image, &stream), &unreadable_x_slot);
    let mounted = mount_writable(&parameters, &mut devices).expect("抬 F 那一次可写挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.into_file_version().expect("这一版带文件");
    // 迭代上界：txg 逐次加一推到锚。
    while current.root.checkpoint_txg.0 < anchor {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
    }
    PoolJustBeforeTheAdmissionRaiseToTheCeiling {
        holes,
        x_txg,
        image_before_the_raising_mount: image,
        stream,
        devices,
        allocator,
        current,
    }
}

/// 有洞的那一份，造到准入抬 F 之前（与 `raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 同一个做法另写一份）。
pub fn pool_over_the_holes_just_before_the_admission_raise(
    x_slot_reads: FailingReadsOfARange,
    x_slot_read_back: UnreadableRangeReadBack,
) -> PoolJustBeforeTheAdmissionRaiseToTheCeiling {
    let parameters = parameters_of_the_ring_hole_pools();
    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
    let mut image = pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE_BEFORE_THE_HOLES);
    let mut holes = Vec::new();
    // 迭代上界：`HOLES` 次挂载，每次一个洞；跨轮携带的是崩溃之后的盘面。
    for _ in 0..HOLES {
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
    let anchor = holes[0] + ring_length - 1;
    mount_reading_the_x_slot_bad_and_publish_empty_through(
        image,
        holes,
        x_txg,
        x_slot_reads,
        x_slot_read_back,
        anchor,
    )
}

/// 没洞的那一份：实例 1 覆盖写到 X = 17、进程退出，抬 F 那一次可写挂载起 X 那一槽读坏，空发布推到 X + 3S − 1（X 成了环里最旧的根）。
pub fn pool_without_holes_just_before_the_admission_raise(
    x_slot_read_back: UnreadableRangeReadBack,
) -> PoolJustBeforeTheAdmissionRaiseToTheCeiling {
    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
    mount_reading_the_x_slot_bad_and_publish_empty_through(
        pool_overwritten_through(X_TXG_WITHOUT_HOLES),
        Vec::new(),
        X_TXG_WITHOUT_HOLES,
        FailingReadsOfARange::Every,
        x_slot_read_back,
        X_TXG_WITHOUT_HOLES + ring_length - 1,
    )
}

/// 根环表里 `ring_slot` 按挂载时读坏记着的、它能住的最大 txg（没这样记着时 `None`）。
pub fn highest_txg_the_table_records_for(
    pool: &PoolJustBeforeTheAdmissionRaiseToTheCeiling,
    ring_slot: RootRingSlot,
) -> Option<CheckpointTxg> {
    pool.allocator
        .root_ring_occupancy()
        .expect("可写挂载装了根环表")
        .ring_slots_bad_at_mount()
        .get(&ring_slot)
        .map(|txgs_it_can_hold| txgs_it_can_hold.highest_txg_it_can_hold())
}

/// 准入抬 F 的上限要并的 `ring_slot` 能住的最大 txg（挂载时读坏或读回全零、还记在根环表里的，`mount::ring_slots_bad_or_reading_back_all_zero_at_mount_by`；
/// 没这样记着时 `None`）。
pub fn highest_txg_merged_into_the_admission_ceiling_for(
    pool: &PoolJustBeforeTheAdmissionRaiseToTheCeiling,
    ring_slot: RootRingSlot,
) -> Option<CheckpointTxg> {
    ring_slots_bad_or_reading_back_all_zero_at_mount_by(&pool.allocator)
        .get(&ring_slot)
        .copied()
}

/// 根环表里 `ring_slot` 按挂载时读回全零记着、钉着的 txg（没这样记着时 `None`）。
pub fn txg_the_table_pins_for_the_slot_reading_back_all_zero(
    pool: &PoolJustBeforeTheAdmissionRaiseToTheCeiling,
    ring_slot: RootRingSlot,
) -> Option<CheckpointTxg> {
    pool.allocator
        .root_ring_occupancy()
        .expect("可写挂载装了根环表")
        .ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held()
        .get(&ring_slot)
        .copied()
}

/// txg `checkpoint_txg` 那一版的账里仍分配、而分配器上空着（可再分配）的槽：(盘, 槽)。
pub fn slots_of_the_version_left_free(
    pool: &PoolJustBeforeTheAdmissionRaiseToTheCeiling,
    checkpoint_txg: u64,
) -> Vec<(DeviceIdentity, SlotNumber)> {
    let image = pool.image_now();
    let root = readable_root_at_txg(&image, checkpoint_txg).expect("那一版的根在盘上、读得出");
    allocation_records_under_root(&image, &root)
        .expect("那一版的账读得出")
        .iter()
        .filter(|record| !record.is_released)
        .flat_map(|record| {
            (record.slot.0..record.slot.0 + u64::from(record.span_slots))
                .map(move |slot| (record.device, SlotNumber(slot)))
        })
        .filter(|(device, slot)| {
            pool.allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == *device)
                .expect("账里的盘在分配器里")
                .is_free(*slot)
        })
        .collect()
}

/// 抬到上限，交回抬到的上限与抬完之后现行那一版带的 F。
pub fn raise_to_the_admission_ceiling(
    pool: &mut PoolJustBeforeTheAdmissionRaiseToTheCeiling,
) -> (CheckpointTxg, CheckpointTxg) {
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &parameters_of_the_ring_hole_pools(),
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.current,
        ShadowLedger::On,
    );
    let ceiling = match raised {
        Ok(RaiseToTheAdmissionCeiling::Raised(raised_floor)) => raised_floor.ceiling,
        Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
            panic!("前提：F = {floor:?} 低于上限 {ceiling:?}，要抬")
        }
        Err(error) => panic!("准入抬 F 做成：{error:?}"),
    };
    (ceiling, pool.current.root.rollback_floor)
}

/// 场景：有洞的那一份，X 那一槽抬 F 那一次挂载起一直读不出（根环表按它能住的最大 txg 21 记着）。
/// 预期：上限 6（洞里的旧根定的，X 那一槽并进来也不改），F 抬到 6，抬完那几条不变量不红。
/// 照一圈窗口取上限时洞里的旧根不算，上限是 21（并进 X 那一槽之后）。
pub fn run_the_scenario_where_admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_over_the_holes_just_before_the_admission_raise(
        FailingReadsOfARange::Every,
        UnreadableRangeReadBack::DeviceError,
    );
    assert_eq!(
        (pool.holes.clone(), pool.x_txg),
        (vec![18, 19, 20], X_TXG_AFTER_THE_HOLES),
        "三个洞紧跟着 X"
    );
    let x_slot = target_for_publish(
        CheckpointTxg(X_TXG_AFTER_THE_HOLES),
        parameters_of_the_ring_hole_pools()
            .geometry
            .root_ring_slots_per_region,
    );
    assert_eq!(
        highest_txg_the_table_records_for(&pool, x_slot),
        Some(CheckpointTxg(X_TXG_AFTER_THE_HOLES)),
        "前提：根环表把 X 那一槽记成挂载时读坏、能住的最大 txg 21"
    );
    assert_eq!(
        raise_to_the_admission_ceiling(&mut pool),
        (CEILING_OVER_THE_HOLES, CEILING_OVER_THE_HOLES),
        "上限与抬到的 F 都是洞里旧根定的 6"
    );
    images.hand_out(
        "抬完之后",
        || pool.image_now(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_RAISE_TO_THE_CEILING_KEEPS_GREEN),
    );
}

/// 场景：有洞的那一份，X 那一槽一直读得出。
/// 预期：上限照读出来的根取 6（洞里的 txg 6、7、8 与 X 是四个不同状态），F 抬到 6，抬完那几条不变量不红。
pub fn run_the_scenario_where_admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_over_the_holes_just_before_the_admission_raise(
        FailingReadsOfARange::Never,
        UnreadableRangeReadBack::DeviceError,
    );
    assert_eq!(
        raise_to_the_admission_ceiling(&mut pool),
        (CEILING_OVER_THE_HOLES, CEILING_OVER_THE_HOLES),
        "X 读得出时上限也是 6"
    );
    images.hand_out(
        "抬完之后",
        || pool.image_now(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_RAISE_TO_THE_CEILING_KEEPS_GREEN),
    );
}

/// 场景：攻方 `k2-zeros` 那一形——有洞的那一份，X 那一槽抬 F 那一次挂载起每次读都报成功、字节全 0。
/// 预期：根环表把那一槽记成挂载时读回全零（不记成读坏）：回收门槛里钉在它可能住过的最旧 txg 9，准入抬 F 的上限按它能住的最大 txg 21 并
/// （D16（发布语义） 已定项 1「抬 F 的上限」那一行：并）。上限照读出来的根取洞里旧根定的 6，
/// 并进 21 不改，F 抬到 6、不越过 21，抬完那几条不变量（含 I-7.9）都不红。这一形上 F 不越过 X 靠的是 RGall 之后洞里那三条旧根：
/// 攻方当初量到的「上限 22、I-7.9 红」是上限只取一圈窗口那一轮的数（`research/prompts/m3-root-ring-code-r3-opus-model/results/rs3z.log`）；
/// 并进 X 那一槽改上限的那一形在没洞的那一份上钉着。
pub fn run_the_scenario_where_admission_raising_the_floor_over_the_holes_with_a_slot_reading_back_zeros_since_the_mount_stops_at_the_roots_left_in_the_holes(
    images: &mut PoolCheckerImages,
) {
    let mut pool = pool_over_the_holes_just_before_the_admission_raise(
        FailingReadsOfARange::Every,
        UnreadableRangeReadBack::Zeros,
    );
    let x_slot = target_for_publish(
        CheckpointTxg(X_TXG_AFTER_THE_HOLES),
        parameters_of_the_ring_hole_pools()
            .geometry
            .root_ring_slots_per_region,
    );
    assert_eq!(
        (
            highest_txg_the_table_records_for(&pool, x_slot),
            txg_the_table_pins_for_the_slot_reading_back_all_zero(&pool, x_slot),
            highest_txg_merged_into_the_admission_ceiling_for(&pool, x_slot)
        ),
        (
            None,
            Some(CheckpointTxg(
                X_TXG_AFTER_THE_HOLES % (3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS)
            )),
            Some(CheckpointTxg(X_TXG_AFTER_THE_HOLES))
        ),
        "前提：X 那一槽记成读回全零、不记成读坏；回收门槛里钉在它可能住过的最旧 txg 9，准入抬 F 的上限按它能住的最大 txg 21 并"
    );
    assert_eq!(
        raise_to_the_admission_ceiling(&mut pool),
        (CEILING_OVER_THE_HOLES, CEILING_OVER_THE_HOLES),
        "上限与抬到的 F 都是洞里旧根定的 6，不越过 X 的 txg 21"
    );
    images.hand_out(
        "抬完之后（含 I-7.9）",
        || pool.image_now(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_RAISE_TO_THE_CEILING_KEEPS_GREEN),
    );
}

/// 场景：没洞的那一份，X 那一槽抬 F 那一次挂载起一直读不出（设备报错）。
/// 预期：上限 17（X 那一槽按它能住的最大 txg 并进来），F 抬到 17；抬完那几条不变量不红——I-7.9 不红（checker 读盘上完好的 X，按它算上限 17）。
/// 不并 X 那一槽时上限是 18、F 越过 X（I-7.9 红）。抬 F 那一遍的回收门槛并不并 X 那一槽（RR4）在这份盘面上分不出：X 是环里最旧的根，
/// 抬 F 那一串的空发布先把它那一槽写掉，扣到生效才放开的那几个槽放开时 X 已不在环里。
pub fn run_the_scenario_where_admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount(
    images: &mut PoolCheckerImages,
) {
    let mut pool =
        pool_without_holes_just_before_the_admission_raise(UnreadableRangeReadBack::DeviceError);
    assert_eq!(
        raise_to_the_admission_ceiling(&mut pool),
        (
            CheckpointTxg(X_TXG_WITHOUT_HOLES),
            CheckpointTxg(X_TXG_WITHOUT_HOLES)
        ),
        "上限与抬到的 F 都停在 X 那一槽能住的 txg 17"
    );
    images.hand_out(
        "抬完之后",
        || pool.image_now(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_RAISE_TO_THE_CEILING_KEEPS_GREEN),
    );
}

/// 场景：没洞的那一份，X 那一槽抬 F 那一次挂载起每次读都报成功、字节全 0（盘上那一槽住着完好的非空有效根 X = 17）。
/// 预期：读回全零的槽同读坏的槽一样并进准入抬 F 的上限（D16（发布语义） 已定项 1「抬 F 的上限」那一行）：
/// 根环表把那一槽记成挂载时读回全零、能住的最大 txg 17，上限 = min(照读得出的根算的 18, 17) = 17，F 抬到 17、
/// 不越过盘上完好的 X。抬完那几条不变量都不红——I-7.9 与设备报错那一形（[`run_the_scenario_where_admission_raising_the_floor_without_holes_stops_at_a_slot_unreadable_since_the_mount`]）
/// 同一个结局：抬 F 那一串的第一次空发布（txg 29 = X + 3S）写掉了 X 那一槽，丁借 journal 重建抬 F 那一刻的根环，住 17 那一种成立、
/// 住 5 那一种红，两种判得不一致、不红（推的，那一形的期望照旧）。
/// 回收门槛里那一槽照「自证不过」那一类钉在它可能住过的最旧 txg 5：抬 F 之前（txg 28）X 那一版引用的单元一个都不空闲。
/// 判别力：全零不并进上限时上限是读得出的最旧有效根 18，F 越过 X、这一条红
/// （`crates/mutations.tsv`「D16（发布语义） 已定项 1 全零并：读回全零的槽不并进准入抬 F 的上限」）。
pub fn run_the_scenario_where_admission_raising_the_floor_without_holes_stops_at_a_slot_reading_back_zeros_since_the_mount(
    images: &mut PoolCheckerImages,
) {
    let mut pool =
        pool_without_holes_just_before_the_admission_raise(UnreadableRangeReadBack::Zeros);
    let x_slot = target_for_publish(
        CheckpointTxg(X_TXG_WITHOUT_HOLES),
        parameters_of_the_ring_hole_pools()
            .geometry
            .root_ring_slots_per_region,
    );
    assert_eq!(
        (
            highest_txg_the_table_records_for(&pool, x_slot),
            txg_the_table_pins_for_the_slot_reading_back_all_zero(&pool, x_slot),
            highest_txg_merged_into_the_admission_ceiling_for(&pool, x_slot)
        ),
        (
            None,
            Some(CheckpointTxg(
                X_TXG_WITHOUT_HOLES % (3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS)
            )),
            Some(CheckpointTxg(X_TXG_WITHOUT_HOLES))
        ),
        "前提：X 那一槽记成读回全零、不记成读坏；回收门槛里钉在它可能住过的最旧 txg 5，准入抬 F 的上限按它能住的最大 txg 17 并"
    );
    assert!(
        readable_root_at_txg(&pool.image_now(), X_TXG_WITHOUT_HOLES).is_some(),
        "前提：抬 F 之前盘上 X 那一槽里是完好的 txg 17 的根"
    );
    assert_eq!(
        slots_of_the_version_left_free(&pool, X_TXG_WITHOUT_HOLES),
        Vec::new(),
        "抬 F 之前：X 那一槽钉在 5，X 那一版引用的单元一个都不空闲"
    );
    assert_eq!(
        raise_to_the_admission_ceiling(&mut pool),
        (
            CheckpointTxg(X_TXG_WITHOUT_HOLES),
            CheckpointTxg(X_TXG_WITHOUT_HOLES)
        ),
        "上限与抬到的 F 都停在 X 那一槽能住的 txg 17，不越过盘上完好的 X"
    );
    images.hand_out(
        "抬完之后（含 I-7.9：X 那一槽已被抬 F 那一串写掉）",
        || pool.image_now(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_THE_RAISE_TO_THE_CEILING_KEEPS_GREEN),
    );
}
