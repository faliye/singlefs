
/// 卸载重挂出路的判别子探针（只拼进冻结副本的一份拷贝，不进仓）：抬 F 之前先算判出放不下
/// （`push_one_floor_raise_for_a_refusal_for_space` 里 `judge_before_pushing_a_floor_raise_for_a_refusal` 交回
/// `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）那一刻，读分配器算几个候选判别子，再在只进内存的盘上把「正常卸载、再可写挂载、
/// 发同一个改动」走一遍，记进线程本地的表；探针 crate 在产品那一臂发布之前开记、发布之后取。
/// 只在开着记的时候算：记过一次就关（一次发布里判出放不下就交回，之后不再判），走的那一遍里再判出放不下不记、不再走。
/// 自证用的开关（全进程一份）把每个判别子一律读成真或一律读成假，算法照跑、只换记下的值。
pub mod remount_outlet_probe {
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::time::Instant;

    use singlefs_format::CLUSTER_SEGMENT_SLOTS;

    use super::{
        most_a_floor_raise_could_make_available, refusal_precheck_oracle_probe,
        DeviceKeepingWritesInMemory, MountedSession,
        NoSpaceEvenWithEverythingAFloorRaiseCouldRelease, RemainderOfTheUserChange,
        SlotsAFloorRaiseCouldRelease, UserChangeRefused,
    };
    use crate::address::{DataUnitIndexInFile, DeviceIdentity, SlotNumber};
    use crate::admission::{
        AvailableBytesOnOneDevice, BytesOnOneDevice, JudgementThatFoundTheDeviceShort,
        SpaceAdmission,
    };
    use crate::allocator::{DeviceFreeMap, PoolAllocator};
    use crate::block_device::BlockDevice;
    use crate::make_filesystem::MakeFilesystemParameters;
    use crate::mount::{mount_writable_with_space_admission, unmount, MountError, ShadowLedger};
    use crate::transaction::{PoolVersion, PublishError, TransactionOutput, TransactionUnit};

    /// 一个整段（64 槽）里的槽对数。
    const PAIRS_OF_A_FULL_SEGMENT: u64 = CLUSTER_SEGMENT_SLOTS / 2;

    /// 「几乎全空的整段」：段里至少一槽罩在未释放记录下的槽对不多于这么几对。
    pub const NEARLY_EMPTY_SEGMENT_BLOCKED_PAIRS_AT_MOST: u64 = 2;

    /// 只进内存的卸载做成了。
    pub const UNMOUNT_WENT_THROUGH: &str = "went-through";

    /// 判出放不下那一刻读出的候选判别子，与在只进内存的盘上走的那一遍重挂。
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct DiscriminatorsAtTheRefusal {
        /// 甲一：判短的每块盘，不排聚簇段的「两槽都不罩在未释放记录下」的槽对数乘一个数据单元的槽数换成字节，不小于那块盘的需求。
        pub slot_pairs_once_the_cluster_segments_are_forgotten_cover_the_demand: bool,
        /// 甲二：甲一成立，并且判短的盘里由式子那一判判短的每一块，式子那一判的上界（可用加 defer 加隔离的字节）不小于它的需求。
        pub and_the_formula_bound_covers_the_demand: bool,
        /// 乙一：全空段少于 `N₀` 的每块盘上，这次挂载开过的聚簇段里至少有一段是凑满 64 槽、按记录口径全空的整段；没有这样的盘时为假。
        pub an_opened_cluster_segment_is_empty_by_the_records: bool,
        /// 乙二：全空段少于 `N₀` 的每块盘上，至少有一个整段里罩在未释放记录下的槽对不多于
        /// [`NEARLY_EMPTY_SEGMENT_BLOCKED_PAIRS_AT_MOST`] 对；没有这样的盘时为假。
        pub a_nearly_empty_full_segment_on_every_device_below_the_floor: bool,
        /// 丙一：甲二成立，并且每块盘此刻的全空聚簇段数（`DeviceFreeMap::empty_segments`）不少于整理开关里的 `N₀`（合取 7 读的那两个数）。
        pub formula_bound_and_empty_segments_at_the_floor: bool,
        /// 丙二：丙一成立，并且在只进内存的盘上正常卸载（`mount::unmount`：抬 F 那一串的整串预演、先写系统配置、推空发布）做成了。
        pub and_the_unmount_walked_in_memory_goes_through: bool,
        /// 丁：在只进内存的盘上正常卸载、再可写挂载（分配器经 `PoolAllocator::rebuild_from_records` 重建）、发同一个改动，写进了。
        pub remount_walked_in_memory_publishes: bool,
        /// 甲一、甲二判的盘有几块（0 块时两条照「每一块都够」空成立）。
        pub devices_judged: usize,
        /// 全空段少于 `N₀` 的每块盘上罩着记录的槽对最少的那个整段的对数，取各盘里最大的；没有这样的盘时 `None`。
        pub fewest_blocked_pairs_in_a_full_segment: Option<u64>,
        /// 只进内存的卸载：做成了是 [`UNMOUNT_WENT_THROUGH`]，报错是 `failed:unmount <错的成员名>`。
        pub unmount_walked_in_memory: String,
        /// 走的那一遍重挂的结局，写法同探针重挂那一臂：`published`、`failed:unmount …`、`failed:mount …`、被拒的成员名。
        pub remount_walked_in_memory: String,
        /// 只进内存的卸载那一段的挂钟（纳秒）。
        pub unmount_nanoseconds: u64,
        /// 走一遍重挂（卸载、挂载、发改动）的挂钟（纳秒）。
        pub walk_nanoseconds: u64,
        /// 每块盘的单元区槽数（按分配器里盘的次序）。
        pub unit_area_slots_of_each_device: Vec<u64>,
    }

    /// 0 照算，1 一律真，2 一律假。
    static FORCED: AtomicU8 = AtomicU8::new(0);

    thread_local! {
        static NOTED: RefCell<Vec<DiscriminatorsAtTheRefusal>> = const { RefCell::new(Vec::new()) };
        static NOTING_ARMED: Cell<bool> = const { Cell::new(false) };
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

    /// 产品那一臂发布之前：清掉上一次留下的，开记。
    pub fn arm_and_clear_the_noted() {
        NOTED.with(|noted| noted.borrow_mut().clear());
        NOTING_ARMED.with(|armed| armed.set(true));
    }

    /// 产品那一臂发布之后：关记，交回记下的。
    pub fn disarm_and_take_the_noted() -> Vec<DiscriminatorsAtTheRefusal> {
        NOTING_ARMED.with(|armed| armed.set(false));
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

    /// 每块盘此刻的全空聚簇段数都不少于整理开关里的 `N₀`（合取 7 读的那两个数，
    /// 与 `empty_cluster_segments_below_the_floor_before_any_series` 读的同一处）。
    fn empty_segments_at_the_floor_on_every_device(allocator: &PoolAllocator) -> bool {
        let floor = allocator
            .compaction()
            .switches
            .empty_cluster_segments_floor
            .segments();
        allocator
            .devices
            .iter()
            .all(|device_map| device_map.empty_segments() >= floor)
    }

    /// 错误文字的头一段（到第一个括号、花括号或空白为止），与探针 crate 的 `pool::leading_text` 同一个切法。
    fn leading_text_of_the_member(text: &str) -> String {
        text.split(|character: char| character == '(' || character == '{' || character.is_whitespace())
            .next()
            .unwrap_or("")
            .to_string()
    }

    /// 被拒的成员名，写法同探针 crate 的 `shapes::refusal_kind`（重挂那一臂记的那一种）。
    #[allow(
        unreachable_patterns,
        reason = "只认几个成员名；核心层加了成员这里照样编得过、落到 other"
    )]
    fn kind_of_the_refusal(refusal: &UserChangeRefused) -> String {
        match refusal {
            UserChangeRefused::Publish { cause, .. } => {
                format!("Publish:{}", leading_text_of_the_member(&format!("{cause:?}")))
            }
            UserChangeRefused::NoSpaceAfterRaisingTheFloor(_) => {
                "NoSpaceAfterRaisingTheFloor".to_string()
            }
            UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(_) => {
                "NoSpaceEvenWithEverythingAFloorRaiseCouldRelease".to_string()
            }
            UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(_) => {
                "FloorRaiseFailedWhilePushingForSpace".to_string()
            }
            UserChangeRefused::NoFileVersionToChange { .. } => "NoFileVersionToChange".to_string(),
            other => leading_text_of_the_member(&format!("{other:?}")),
        }
    }

    fn nanoseconds_since(started: Instant) -> u64 {
        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    /// 在只进内存的盘上走的那一遍重挂。
    struct RemountWalkedInMemory {
        unmount: String,
        remount: String,
        unmount_nanoseconds: u64,
        walk_nanoseconds: u64,
    }

    /// 在只进内存的盘（`DeviceKeepingWritesInMemory`：写只进内存，读没写过的块穿到底下那块盘）上，拿分配器与现行那一版各一份拷贝
    /// 照探针重挂那一臂走一遍：`mount::unmount`（抬 F 到现行 txg 那一串：影子账重算、回收、整串预演、先写系统配置、推空发布），
    /// `mount::mount_writable_with_space_admission`（恢复、按记录重建分配器、取号、写行、暖机），再发这次改动剩下的。
    /// 底下那几块盘一字不动，会话的分配器与现行那一版不动。
    fn walk_the_remount_in_memory<Device: BlockDevice>(
        parameters: &MakeFilesystemParameters,
        devices: &[(DeviceIdentity, Device)],
        allocator: &PoolAllocator,
        current: &TransactionOutput,
        shadow_ledger: ShadowLedger,
        remainder: RemainderOfTheUserChange<'_>,
    ) -> RemountWalkedInMemory {
        let started = Instant::now();
        let mut devices_keeping_writes_in_memory: Vec<(
            DeviceIdentity,
            DeviceKeepingWritesInMemory<'_>,
        )> = devices
            .iter()
            .map(|(identity, device)| {
                let disk: &dyn BlockDevice = device;
                (*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))
            })
            .collect();
        let mut walked_allocator = allocator.clone();
        let mut walked_version = PoolVersion::of_a_version_of_trees(current.clone());
        let unmounted = unmount(
            parameters,
            &mut devices_keeping_writes_in_memory,
            &mut walked_allocator,
            &mut walked_version,
            shadow_ledger,
        );
        let unmount_nanoseconds = nanoseconds_since(started);
        if let Err(error) = unmounted {
            let failed = format!(
                "failed:unmount {}",
                leading_text_of_the_member(&format!("{error:?}"))
            );
            // 丙二那一行另带预演被拒的那一次是第几次、报的什么（重挂那一臂只记错的成员名，`remount` 照它的写法、不带这些）。
            let unmount = match &error {
                MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                    refused_publish_in_the_sequence,
                    publishes_in_the_sequence,
                    cause,
                } => format!(
                    "{failed} refused-publish={refused_publish_in_the_sequence}/{publishes_in_the_sequence} cause={}",
                    leading_text_of_the_member(&format!("{cause:?}"))
                ),
                _ => failed.clone(),
            };
            return RemountWalkedInMemory {
                unmount,
                remount: failed,
                unmount_nanoseconds,
                walk_nanoseconds: nanoseconds_since(started),
            };
        }
        let remount = match mount_writable_with_space_admission(
            parameters,
            &mut devices_keeping_writes_in_memory,
            SpaceAdmission::JudgedByTheFormula,
        ) {
            Err(error) => format!(
                "failed:mount {}",
                leading_text_of_the_member(&format!("{error:?}"))
            ),
            Ok(mounted) => {
                let (_output, mut session) = MountedSession::of_the_mount(mounted);
                match remainder {
                    RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
                        match session.publish_user_change(&mut devices_keeping_writes_in_memory, change) {
                            Ok(_) => "published".to_string(),
                            Err(refused) => kind_of_the_refusal(&refused),
                        }
                    }
                    // 只剩走空（unlink 的意图已发成）：重挂那一臂从改动之前的池发整个 unlink，这里没有同一个改动可发，不判。
                    RemainderOfTheUserChange::WalkTheQueue => {
                        "remainder-is-the-queue-walk-only".to_string()
                    }
                }
            }
        };
        RemountWalkedInMemory {
            unmount: UNMOUNT_WENT_THROUGH.to_string(),
            remount,
            unmount_nanoseconds,
            walk_nanoseconds: nanoseconds_since(started),
        }
    }

    fn discriminators_of(
        allocator: &PoolAllocator,
        no_space: &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
        walked: RemountWalkedInMemory,
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
        let formula_bound_and_empty_segments_at_the_floor = slot_pairs_cover
            && formula_covers
            && empty_segments_at_the_floor_on_every_device(allocator);
        let computed = DiscriminatorsAtTheRefusal {
            slot_pairs_once_the_cluster_segments_are_forgotten_cover_the_demand: slot_pairs_cover,
            and_the_formula_bound_covers_the_demand: slot_pairs_cover && formula_covers,
            an_opened_cluster_segment_is_empty_by_the_records: opened_segment_empty,
            a_nearly_empty_full_segment_on_every_device_below_the_floor: nearly_empty,
            formula_bound_and_empty_segments_at_the_floor,
            and_the_unmount_walked_in_memory_goes_through: formula_bound_and_empty_segments_at_the_floor
                && walked.unmount == UNMOUNT_WENT_THROUGH,
            remount_walked_in_memory_publishes: walked.remount == "published",
            devices_judged: judged.len(),
            fewest_blocked_pairs_in_a_full_segment,
            unmount_walked_in_memory: walked.unmount,
            remount_walked_in_memory: walked.remount,
            unmount_nanoseconds: walked.unmount_nanoseconds,
            walk_nanoseconds: walked.walk_nanoseconds,
            unit_area_slots_of_each_device: allocator
                .devices
                .iter()
                .map(DeviceFreeMap::unit_area_slots)
                .collect(),
        };
        match forced() {
            None => computed,
            Some(value) => DiscriminatorsAtTheRefusal {
                slot_pairs_once_the_cluster_segments_are_forgotten_cover_the_demand: value,
                and_the_formula_bound_covers_the_demand: value,
                an_opened_cluster_segment_is_empty_by_the_records: value,
                a_nearly_empty_full_segment_on_every_device_below_the_floor: value,
                formula_bound_and_empty_segments_at_the_floor: value,
                and_the_unmount_walked_in_memory_goes_through: value,
                remount_walked_in_memory_publishes: value,
                ..computed
            },
        }
    }

    /// 判出放不下那一刻调（拼进 `push_one_floor_raise_for_a_refusal_for_space` 先算那一步之后）：开着记时先关记，
    /// 取走推串记录与合取 7 那一判的记录、在只进内存的盘上走一遍重挂、放回取走的（走的那一遍记下的丢掉），再读分配器算候选，记一条。
    #[allow(
        clippy::too_many_arguments,
        reason = "走一遍重挂要的参数与盘表、分配器、现行那一版、影子账那一臂、这次改动还剩什么，加上判出的那一份拒绝，照推串那一处原样借"
    )]
    pub(super) fn note_the_discriminators_at_the_refusal<Device: BlockDevice>(
        parameters: &MakeFilesystemParameters,
        devices: &[(DeviceIdentity, Device)],
        allocator: &PoolAllocator,
        current: &TransactionOutput,
        shadow_ledger: ShadowLedger,
        remainder: RemainderOfTheUserChange<'_>,
        no_space: &NoSpaceEvenWithEverythingAFloorRaiseCouldRelease,
    ) {
        if !NOTING_ARMED.with(Cell::get) {
            return;
        }
        NOTING_ARMED.with(|armed| armed.set(false));
        let series_before_the_walk = refusal_precheck_oracle_probe::take_series();
        let conjunct_before_the_walk = refusal_precheck_oracle_probe::take_conjunct_before_any_series();
        let walked = walk_the_remount_in_memory(
            parameters,
            devices,
            allocator,
            current,
            shadow_ledger,
            remainder,
        );
        refusal_precheck_oracle_probe::put_back_series(series_before_the_walk);
        refusal_precheck_oracle_probe::put_back_conjunct_before_any_series(conjunct_before_the_walk);
        let discriminators = discriminators_of(allocator, no_space, walked);
        NOTED.with(|noted| noted.borrow_mut().push(discriminators));
    }
}
