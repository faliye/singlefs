
/// 卸载重挂出路的判别子探针（只拼进冻结副本的一份拷贝，不进仓）：抬 F 之前先算判出放不下
/// （`judge_before_pushing_a_floor_raise_for_a_refusal` 交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）那一刻，
/// 只读分配器算几个候选判别子，记进线程本地的表；探针 crate 在产品那一臂发布之前清、发布之后取。
/// 自证用的开关（全进程一份）把每个判别子一律读成真或一律读成假，算法照跑、只换记下的值。
pub mod remount_outlet_probe {
    use std::cell::RefCell;
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicU8, Ordering};

    use singlefs_format::CLUSTER_SEGMENT_SLOTS;

    use super::{
        most_a_floor_raise_could_make_available, NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
        SlotsAFloorRaiseCouldRelease,
    };
    use crate::address::{DataUnitIndexInFile, DeviceIdentity, SlotNumber};
    use crate::admission::{
        AvailableBytesOnOneDevice, BytesOnOneDevice, JudgementThatFoundTheDeviceShort,
    };
    use crate::allocator::{DeviceFreeMap, PoolAllocator};
    use crate::transaction::{PublishError, TransactionUnit};

    /// 一个整段（64 槽）里的槽对数。
    const PAIRS_OF_A_FULL_SEGMENT: u64 = CLUSTER_SEGMENT_SLOTS / 2;

    /// 「几乎全空的整段」：段里至少一槽罩在未释放记录下的槽对不多于这么几对。
    pub const NEARLY_EMPTY_SEGMENT_BLOCKED_PAIRS_AT_MOST: u64 = 2;

    /// 判出放不下那一刻读出的候选判别子。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DiscriminatorsAtTheRefusal {
        /// 派发提示的甲一：判短的每块盘，不排聚簇段的「两槽都不罩在未释放记录下」的槽对数乘一个数据单元的槽数换成字节，不小于那块盘的需求。
        pub slot_pairs_once_the_cluster_segments_are_forgotten_cover_the_demand: bool,
        /// 派发提示的甲二：上一条成立，并且判短的盘里由式子那一判判短的每一块，式子那一判的上界（可用加 defer 加隔离的字节）不小于它的需求。
        pub and_the_formula_bound_covers_the_demand: bool,
        /// 派发提示的乙一：全空段少于 `N₀` 的每块盘上，这次挂载开过的聚簇段里至少有一段是凑满 64 槽、按记录口径全空的整段；没有这样的盘时为假。
        pub an_opened_cluster_segment_is_empty_by_the_records: bool,
        /// 加的乙二：全空段少于 `N₀` 的每块盘上，至少有一个整段里罩在未释放记录下的槽对不多于
        /// [`NEARLY_EMPTY_SEGMENT_BLOCKED_PAIRS_AT_MOST`] 对；没有这样的盘时为假。
        pub a_nearly_empty_full_segment_on_every_device_below_the_floor: bool,
        /// 甲一、甲二判的盘有几块（0 块时两条照「每一块都够」空成立）。
        pub devices_judged: usize,
        /// 全空段少于 `N₀` 的每块盘上罩着记录的槽对最少的那个整段的对数，取各盘里最大的；没有这样的盘时 `None`。
        pub fewest_blocked_pairs_in_a_full_segment: Option<u64>,
    }

    /// 0 照算，1 一律真，2 一律假。
    static FORCED: AtomicU8 = AtomicU8::new(0);

    thread_local! {
        static NOTED: RefCell<Vec<DiscriminatorsAtTheRefusal>> = const { RefCell::new(Vec::new()) };
    }

    /// 自证用：`Some(true)` 把每个判别子一律记成真，`Some(false)` 一律记成假，`None` 照算。
    pub fn force_every_discriminator(forced: Option<bool>) {
        let code = match forced {
            None => 0,
            Some(true) => 1,
            Some(false) => 2,
        };
        FORCED.store(code, Ordering::SeqCst);
    }

    fn forced() -> Option<bool> {
        match FORCED.load(Ordering::SeqCst) {
            1 => Some(true),
            2 => Some(false),
            _ => None,
        }
    }

    pub fn clear_the_noted() {
        NOTED.with(|noted| noted.borrow_mut().clear());
    }

    pub fn take_the_noted() -> Vec<DiscriminatorsAtTheRefusal> {
        NOTED.with(|noted| std::mem::take(&mut *noted.borrow_mut()))
    }

    /// 判短的一块盘：判它短的那一判、那一判的需求，那一判是式子时另带式子那一判推抬 F 至多凑得出的字节。
    struct DeviceJudgedShort {
        device: DeviceIdentity,
        demand: BytesOnOneDevice,
        formula_bound: Option<AvailableBytesOnOneDevice>,
    }

    fn device_map_of(allocator: &PoolAllocator, device: DeviceIdentity) -> &DeviceFreeMap {
        allocator
            .devices
            .iter()
            .find(|device_map| device_map.device == device)
            .expect("拒绝里的盘来自这个分配器")
    }

    /// 判短的盘：`devices_short` 非空时就是它（槽对、式子两判判出的格）；空时（合取 7 那一判判出的格）取最后那一次被拒
    /// `SpaceAdmissionRefused` 里不够的每一块，最后那一次是 `PlacementRefused` 时一块都没有。
    fn devices_judged_short(
        allocator: &PoolAllocator,
        no_space: &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
    ) -> Vec<DeviceJudgedShort> {
        if !no_space.devices_short.is_empty() {
            return no_space
                .devices_short
                .iter()
                .map(|short| DeviceJudgedShort {
                    device: short.device,
                    demand: short.demand,
                    formula_bound: match short.judged_short_by {
                        JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand => {
                            Some(short.most_a_floor_raise_could_make_available)
                        }
                        JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits
                        | JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,
                    },
                })
                .collect();
        }
        let PublishError::SpaceAdmissionRefused(refused) = &no_space.last_refusal else {
            return Vec::new();
        };
        refused
            .short_devices
            .iter()
            .map(|short| {
                let formula_bound = match short.judged_short_by {
                    JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand => {
                        most_a_floor_raise_could_make_available(
                            short,
                            SlotsAFloorRaiseCouldRelease::of_the_device(
                                device_map_of(allocator, short.device),
                                allocator.cluster_segments(),
                            ),
                        )
                    }
                    JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits
                    | JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,
                };
                DeviceJudgedShort {
                    device: short.device,
                    demand: short.demand,
                    formula_bound,
                }
            })
            .collect()
    }

    /// 不排聚簇段的「两槽都不罩在未释放记录下」的槽对数，乘一个数据单元的槽数换成字节（`most_a_floor_raise_could_make_available` 槽对那一臂的换法）。
    fn slot_pair_bytes_once_the_cluster_segments_are_forgotten(device_map: &DeviceFreeMap) -> i128 {
        let no_cluster_segments: BTreeSet<SlotNumber> = BTreeSet::new();
        let pairs = device_map
            .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(&no_cluster_segments)
            .0;
        let slots_of_a_data_unit = TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots();
        i128::from(BytesOnOneDevice::of_slots(pairs * slots_of_a_data_unit).0)
    }

    /// 一段里两槽都不罩在未释放记录下的槽对数：不排聚簇段的总数减去只排这一段时的数。
    fn pairs_without_a_record_in_the_segment(device_map: &DeviceFreeMap, segment_start: SlotNumber) -> u64 {
        let no_cluster_segments: BTreeSet<SlotNumber> = BTreeSet::new();
        let only_this_segment: BTreeSet<SlotNumber> = BTreeSet::from([segment_start]);
        device_map
            .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(&no_cluster_segments)
            .0
            - device_map
                .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(&only_this_segment)
                .0
    }

    fn full_segments(device_map: &DeviceFreeMap) -> u64 {
        device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS
    }

    fn is_a_full_segment_of(device_map: &DeviceFreeMap, segment_start: SlotNumber) -> bool {
        let start = device_map.unit_area_start().slot().0;
        segment_start.0 >= start
            && (segment_start.0 - start).is_multiple_of(CLUSTER_SEGMENT_SLOTS)
            && (segment_start.0 - start) / CLUSTER_SEGMENT_SLOTS < full_segments(device_map)
    }

    fn fewest_blocked_pairs_in_a_full_segment_of(device_map: &DeviceFreeMap) -> Option<u64> {
        let start = device_map.unit_area_start().slot().0;
        (0..full_segments(device_map))
            .map(|index| {
                PAIRS_OF_A_FULL_SEGMENT
                    - pairs_without_a_record_in_the_segment(
                        device_map,
                        SlotNumber(start + index * CLUSTER_SEGMENT_SLOTS),
                    )
            })
            .min()
    }

    fn discriminators_of(
        allocator: &PoolAllocator,
        no_space: &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
    ) -> DiscriminatorsAtTheRefusal {
        let judged = devices_judged_short(allocator, no_space);
        let slot_pairs_cover = judged.iter().all(|short| {
            slot_pair_bytes_once_the_cluster_segments_are_forgotten(device_map_of(allocator, short.device))
                >= i128::from(short.demand.0)
        });
        let formula_covers = judged.iter().all(|short| {
            short
                .formula_bound
                .is_none_or(|bound| bound.0 >= i128::from(short.demand.0))
        });
        let devices_below_the_floor: Vec<DeviceIdentity> = no_space
            .empty_cluster_segments_below_the_floor
            .as_ref()
            .map(|below| below.short_devices.iter().map(|device| device.device).collect())
            .unwrap_or_default();
        let opened_segment_empty = !devices_below_the_floor.is_empty()
            && devices_below_the_floor.iter().all(|device| {
                let device_map = device_map_of(allocator, *device);
                allocator.cluster_segments().iter().any(|segment_start| {
                    is_a_full_segment_of(device_map, *segment_start)
                        && pairs_without_a_record_in_the_segment(device_map, *segment_start)
                            == PAIRS_OF_A_FULL_SEGMENT
                })
            });
        let fewest_on_each_device: Vec<Option<u64>> = devices_below_the_floor
            .iter()
            .map(|device| fewest_blocked_pairs_in_a_full_segment_of(device_map_of(allocator, *device)))
            .collect();
        let nearly_empty = !fewest_on_each_device.is_empty()
            && fewest_on_each_device.iter().all(|fewest| {
                fewest.is_some_and(|pairs| pairs <= NEARLY_EMPTY_SEGMENT_BLOCKED_PAIRS_AT_MOST)
            });
        let fewest_blocked_pairs_in_a_full_segment = if fewest_on_each_device.is_empty() {
            None
        } else {
            Some(
                fewest_on_each_device
                    .iter()
                    .map(|fewest| fewest.unwrap_or(PAIRS_OF_A_FULL_SEGMENT))
                    .max()
                    .unwrap_or(PAIRS_OF_A_FULL_SEGMENT),
            )
        };
        let computed = DiscriminatorsAtTheRefusal {
            slot_pairs_once_the_cluster_segments_are_forgotten_cover_the_demand: slot_pairs_cover,
            and_the_formula_bound_covers_the_demand: slot_pairs_cover && formula_covers,
            an_opened_cluster_segment_is_empty_by_the_records: opened_segment_empty,
            a_nearly_empty_full_segment_on_every_device_below_the_floor: nearly_empty,
            devices_judged: judged.len(),
            fewest_blocked_pairs_in_a_full_segment,
        };
        match forced() {
            None => computed,
            Some(value) => DiscriminatorsAtTheRefusal {
                slot_pairs_once_the_cluster_segments_are_forgotten_cover_the_demand: value,
                and_the_formula_bound_covers_the_demand: value,
                an_opened_cluster_segment_is_empty_by_the_records: value,
                a_nearly_empty_full_segment_on_every_device_below_the_floor: value,
                ..computed
            },
        }
    }

    /// 判出放不下那一刻调（拼进 `judge_before_pushing_a_floor_raise_for_a_refusal` 的那一层）：只读分配器，记一条。
    pub fn note_the_discriminators_at_the_refusal(
        allocator: &PoolAllocator,
        no_space: &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
    ) {
        let discriminators = discriminators_of(allocator, no_space);
        NOTED.with(|noted| noted.borrow_mut().push(discriminators));
    }
}
