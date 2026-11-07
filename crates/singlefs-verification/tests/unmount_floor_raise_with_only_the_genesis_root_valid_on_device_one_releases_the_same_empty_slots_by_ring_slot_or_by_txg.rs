//! 正常卸载那一遍给每条读出的根认盘：按读出它的那一槽所在区域认（`singlefs_core::mount` 的 `roots_on_the_devices_of_their_ring_slots`，
//! 产品今天的写法），与按落点公式从 txg 推槽再认，在「某块盘上唯一的有效根就是它那个区域槽 0 的第 0 代根」这一形上，每块盘最新有效根
//! 两边不同（按槽认是 0，按 txg 认是「没有」），而卸载当场放开的那一截两边逐项相同、都空（欠账 C972「盘上唯一有效根是第 0 代根」那一形；判决
//! `research/prompts/m3-close-b19-r3fix-code-r1-main-verification.md` 第二节 Q2.3 与第一节攻方第二样本的
//! `newest_on_every_device_differs=483840`、`immediate_reclaim_differs=0`）。等价性只在「放开的那一截」上成立，不在「各盘最新有效根」上；
//! 另一形（那块盘上另有更新的有效根，三样都相同）在 `unmount_floor_raise_with_a_genesis_root_read_from_region_one_releases_the_same_slots_by_ring_slot_or_by_txg.rs`。
//!
//! 这一形里撑住「放开的那一截相同」的是：
//! - P1 读者核落点（`singlefs_core::root_ring::root_sits_in_the_ring_slot_the_formula_gives`）：读出的非第 0 代根都住公式那一槽，两种认法认到同一块盘；
//! - P4 没有释放代为 0 的已释放记录：按槽认时各盘最新有效根取最小是 0，当场放开释放代不高于 0 的那一截；按 txg 认时有一块盘一条都没有，
//!   当场一个不放。两边都空靠的是那一截本来就空。用例在盘上的记录之外另添一条释放代为 0 的已释放记录再算一遍，钉住这时两边分开。
//!
//! 盘面（根环有洞的小池，S = 4、环长 12，区域 0、2 在盘 0，区域 1 在盘 1）：mkfs → 可写挂载（零单元的发布 1、2）→ 第一个文件（txg 3），
//! 现行那一版是 txg 3（公式那一槽在区域 0、盘 0）；下一条落在区域 1 的根是 txg 4，这段历史到不了。之后在盘 1 上动两槽：
//! - 区域 1 槽 0 写回 mkfs 写下的第 0 代根（txg 1 那条根的写被设备吞掉、那一槽留着 mkfs 的那一份）。盘 1 上于是只有这一条有效根：
//!   按槽认在盘 1，按 txg 认在盘 0。
//! - 区域 1 槽 1（这次挂载那一刻读回全零、之后没写过）抄进现行那一版（txg 3，公式那一槽是区域 0 槽 1）的根槽字节：槽号与公式那一槽相同、
//!   区域不同的一份错位的根。读者核落点时它当读坏；核落点只比槽号时按槽认会把 txg 3 算到盘 1 上。
mod common_root_ring_holes;

use std::collections::{BTreeMap, BTreeSet};

use common_root_ring_holes::{
    content_of_the_version, devices_of, parameters_of_the_ring_hole_pools, pool_of,
    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, AllocationRecordQuarantineMark, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::instance_table::InstanceTableRecords;
use singlefs_core::make_filesystem::{make_filesystem, GENESIS_CHECKPOINT_TXG};
use singlefs_core::mount::{mount_writable, unmount, Mounted, ShadowLedger, Unmounted};
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root, readable_roots_with_ring_slots,
    root_is_abandoned_by_the_instance_table,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{
    root_sits_in_the_ring_slot_the_formula_gives, slot_offset, target_for_publish, RootRingSlot,
    RootRingSlotsPerRegion,
};
use singlefs_core::transaction::{
    publish_first_file, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

/// 盘 1 独占的区域。
const REGION_OF_DEVICE_ONE: u64 = 1;

/// 区域 1 里 mkfs 种第 0 代根的那一槽：盘面上它留着第 0 代根（txg 1 那条根的写被吞掉）。
const GENESIS_RING_SLOT_OF_REGION_ONE: RootRingSlot = RootRingSlot {
    region: REGION_OF_DEVICE_ONE,
    slot: 0,
};

/// 区域 1 槽 1（公式里住 txg 4，这段历史到不了）：盘面上它抄着现行那一版的根，槽号与那条根的公式那一槽相同、区域不同。
const RING_SLOT_HOLDING_A_MISPLACED_COPY_OF_THE_CURRENT_ROOT: RootRingSlot = RootRingSlot {
    region: REGION_OF_DEVICE_ONE,
    slot: 1,
};

/// 第一个文件那一版：这段历史的现行那一版，根在区域 0 槽 1（盘 0）。
const CURRENT_TXG_OF_THE_HISTORY: CheckpointTxg = CheckpointTxg(3);

/// 分配器回收交回的落点按这块盘的记录报（两盘同槽，`PoolAllocator::reclaim_released_up_to` 只取池里排第一的那块盘）。
const DEVICE_WHOSE_RECORDS_THE_RECLAIM_REPORTS: DeviceIdentity = DeviceIdentity(0);

/// 「P4 不成立」那一算另添的已释放记录的落点：只进用例自己的那一算，不写盘、不进分配器。
const SLOT_OF_THE_RECORD_RELEASED_AT_GENERATION_ZERO: SlotNumber = SlotNumber(0);

/// 卸载那一刻的盘面、这个进程的分配器与现行那一版（txg 3）。
struct PoolAboutToUnmount {
    image: MemoryPool,
    allocator: PoolAllocator,
    current: TransactionOutput,
}

/// 文件头那段历史：mkfs、可写挂载、第一个文件（txg 3），再把盘 1 上区域 1 的槽 0 写回第 0 代根、槽 1 抄进现行那一版的根。
fn pool_with_only_the_genesis_root_valid_on_device_one_and_a_misplaced_copy_of_the_current_root(
) -> PoolAboutToUnmount {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let spacing = parameters.geometry.fixed_structure_slot_spacing;
    let device_of_region = |region: u64| {
        parameters.region_devices[usize::try_from(region).expect("区域号小于根环区域数 3")]
    };
    let device_one = device_of_region(REGION_OF_DEVICE_ONE);
    let genesis_root_bytes_of_region_one = pool_of(&devices).devices[&device_one].read(
        slot_offset(GENESIS_RING_SLOT_OF_REGION_ONE, spacing),
        slot_bytes,
    );
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("只做过 mkfs 的池可写挂载");
    assert_eq!(
        current.root().checkpoint_txg,
        CheckpointTxg(2),
        "前提：只做过 mkfs 的池可写挂载推零单元的发布 1、2，到每块盘上都有本实例的根"
    );
    let instance = output.instance;
    let current: TransactionOutput = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
            },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    assert_eq!(current.root.checkpoint_txg, CURRENT_TXG_OF_THE_HISTORY);
    let mut image = pool_of(&devices);
    let ring_slot_of_the_current_root = target_for_publish(
        current.root.checkpoint_txg,
        parameters.geometry.root_ring_slots_per_region,
    );
    let current_root_bytes = image.devices[&device_of_region(ring_slot_of_the_current_root.region)]
        .read(
            slot_offset(ring_slot_of_the_current_root, spacing),
            slot_bytes,
        );
    let device_one_image = image.devices.get_mut(&device_one).expect("盘 1 在池里");
    device_one_image.write(
        slot_offset(GENESIS_RING_SLOT_OF_REGION_ONE, spacing),
        &genesis_root_bytes_of_region_one,
    );
    device_one_image.write(
        slot_offset(
            RING_SLOT_HOLDING_A_MISPLACED_COPY_OF_THE_CURRENT_ROOT,
            spacing,
        ),
        &current_root_bytes,
    );
    PoolAboutToUnmount {
        image,
        allocator,
        current,
    }
}

/// 给一条读出的根认盘的两种认法。
#[derive(Clone, Copy, Debug)]
enum DeviceOfARootRead {
    /// 按读出它的那一槽所在区域认（产品今天的写法）。
    OfTheRingSlotItWasReadFrom,
    /// 按落点公式从它的 txg 推槽、再按那一槽所在区域认（等价变异那一行的写法）。
    OfTheRingSlotTheFormulaGivesItsTxg,
}

/// 一种认法下卸载那一遍算出的东西（与 `singlefs_core::mount` 里卸载那一格同一个算法：每块盘取没被抛弃的根里 txg 最大的，
/// 各盘之间取最小；有一块盘一条都没有就是「没有」，当场放开的那一截为空）。
#[derive(Debug, PartialEq, Eq)]
struct ReleaseByOneAttribution {
    newest_valid_root_on_each_device: BTreeMap<DeviceIdentity, Option<CheckpointTxg>>,
    newest_valid_root_on_every_device: Option<CheckpointTxg>,
    /// 已释放、释放代不高于上面那个数（也不高于新 F）的落点：(槽号, 跨度)。
    released_immediately: BTreeSet<(SlotNumber, u64)>,
}

/// 按 `attribution` 给 `roots_read` 认盘，算出 [`ReleaseByOneAttribution`]。被抛弃按 `table`（现行那一版的实例表）判；
/// `released_records` 是卸载之前分配器里的记录，`new_floor` 是卸载抬到的 F。
fn release_by_one_attribution(
    attribution: DeviceOfARootRead,
    roots_read: &[(RootRingSlot, RootRecord)],
    region_devices: &[DeviceIdentity; 3],
    slots_per_region: RootRingSlotsPerRegion,
    table: &InstanceTableRecords,
    released_records: &[AllocationRecord],
    new_floor: CheckpointTxg,
) -> ReleaseByOneAttribution {
    let device_of_the_root = |ring_slot_it_was_read_from: RootRingSlot, root: &RootRecord| {
        let ring_slot = match attribution {
            DeviceOfARootRead::OfTheRingSlotItWasReadFrom => ring_slot_it_was_read_from,
            DeviceOfARootRead::OfTheRingSlotTheFormulaGivesItsTxg => {
                target_for_publish(root.checkpoint_txg, slots_per_region)
            }
        };
        region_devices[usize::try_from(ring_slot.region).expect("区域号小于根环区域数 3")]
    };
    let devices_of_the_pool: BTreeSet<DeviceIdentity> = region_devices.iter().copied().collect();
    let newest_valid_root_on_each_device: BTreeMap<DeviceIdentity, Option<CheckpointTxg>> =
        devices_of_the_pool
            .iter()
            .map(|device| {
                (
                    *device,
                    roots_read
                        .iter()
                        .filter(|(ring_slot, root)| {
                            !root_is_abandoned_by_the_instance_table(root, table)
                                && device_of_the_root(*ring_slot, root) == *device
                        })
                        .map(|(_, root)| root.checkpoint_txg)
                        .max(),
                )
            })
            .collect();
    let newest_valid_root_on_every_device = newest_valid_root_on_each_device
        .values()
        .copied()
        .collect::<Option<Vec<CheckpointTxg>>>()
        .and_then(|newest_on_each| newest_on_each.into_iter().min());
    let released_immediately = newest_valid_root_on_every_device
        .map(|newest_on_every_device| {
            released_placements_up_to(released_records, newest_on_every_device.min(new_floor))
        })
        .unwrap_or_default();
    ReleaseByOneAttribution {
        newest_valid_root_on_each_device,
        newest_valid_root_on_every_device,
        released_immediately,
    }
}

/// 回收报的那块盘上，已释放、释放代不高于 `floor` 的落点：(槽号, 跨度)。
fn released_placements_up_to(
    records: &[AllocationRecord],
    floor: CheckpointTxg,
) -> BTreeSet<(SlotNumber, u64)> {
    records
        .iter()
        .filter(|record| {
            record.device == DEVICE_WHOSE_RECORDS_THE_RECLAIM_REPORTS
                && record.is_released
                && record.generation <= floor
        })
        .map(|record| (record.slot, u64::from(record.span_slots)))
        .collect()
}

/// 场景：文件头那份盘面，盘 1 上读得出的有效根只有区域 1 槽 0 那一份第 0 代根，区域 1 槽 1 读到一份错位的现行那一版的根，
/// 在这个进程里正常卸载。
/// 预期：按槽认每块盘最新有效根是盘 0 为 3、盘 1 为 0，各盘取最小是 0；按 txg 认盘 1 一条都没有、各盘取最小是「没有」——两边不同；
/// 当场放开的那一截两边都空、逐项相同。盘上没有释放代为 0 的已释放记录（P4）；另添一条这样的记录再算，按槽认当场放开它、按 txg 认不放，
/// 两边分开。卸载做成，回收交回的落点恰是释放代不高于新 F 的那一截（全部扣到生效，当场放开的那一截为空）。
/// 判别力：
/// - 读者核落点只比槽号不比区域（`crates/mutations.tsv`「C972 盘上唯一有效根是第 0 代根：读者核落点只比槽号不比区域」）：区域 1 槽 1 那份 txg 3 收下，
///   按槽认把它算到盘 1 上，盘 1 的最新有效根钉值那一条红；
/// - 读者不核落点（`crates/mutations.tsv`「卸载抬 F 那一遍读者不核落点：收下不住在公式那一槽的根」，点名另一形的用例）：同上，这一条也红；
/// - 盘面上出现释放代为 0 的已释放记录（P4 不成立；P4 是盘面性质，配不出代码变异）：P4 那条前提断言红；
/// - 调用点那一处实参改成按 txg 推槽（等价变异，不进 `crates/mutations.tsv`）：这一条照旧绿，放开的那一截两边都空。
#[test]
fn unmount_with_only_the_genesis_root_valid_on_device_one_releases_the_same_empty_slots_whether_roots_count_on_the_device_of_their_ring_slot_or_of_their_txg(
) {
    let PoolAboutToUnmount {
        image,
        mut allocator,
        current,
    } = pool_with_only_the_genesis_root_valid_on_device_one_and_a_misplaced_copy_of_the_current_root(
    );
    let system_configuration = choose_system_configuration(&image).expect("系统配置读得出");
    let immutable = &system_configuration.immutable;
    let slots_per_region = immutable.sizes.root_ring_slots_per_region;
    let roots_read = readable_roots_with_ring_slots(
        &image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    let genesis_root_read_from_region_one = roots_read
        .iter()
        .find(|(ring_slot, _)| *ring_slot == GENESIS_RING_SLOT_OF_REGION_ONE)
        .map(|(_, root)| *root)
        .expect("前提：卸载那一遍从区域 1 槽 0 读出根");
    assert_eq!(
        genesis_root_read_from_region_one.checkpoint_txg, GENESIS_CHECKPOINT_TXG,
        "前提：区域 1 槽 0 读出的是第 0 代根"
    );
    assert_ne!(
        target_for_publish(GENESIS_CHECKPOINT_TXG, slots_per_region).region,
        REGION_OF_DEVICE_ONE,
        "前提：落点公式把第 0 代根推到区域 1 之外，两种认法把这一条认到不同的盘"
    );
    let table = instance_table_chain_of_root(&image, &current.root)
        .expect("现行那一版的实例表读得出")
        .records;
    let records_before_the_unmount = allocator.records().to_vec();
    let release_by = |attribution, records: &[AllocationRecord]| {
        release_by_one_attribution(
            attribution,
            &roots_read,
            &immutable.region_devices,
            slots_per_region,
            &table,
            records,
            current.root.checkpoint_txg,
        )
    };
    let by_ring_slot = release_by(
        DeviceOfARootRead::OfTheRingSlotItWasReadFrom,
        &records_before_the_unmount,
    );
    let by_txg = release_by(
        DeviceOfARootRead::OfTheRingSlotTheFormulaGivesItsTxg,
        &records_before_the_unmount,
    );
    assert_eq!(
        by_ring_slot.newest_valid_root_on_each_device,
        BTreeMap::from([
            (DeviceIdentity(0), Some(CURRENT_TXG_OF_THE_HISTORY)),
            (DeviceIdentity(1), Some(GENESIS_CHECKPOINT_TXG)),
        ]),
        "按槽认：盘 0 上最新的有效根是现行那一版 txg 3，盘 1 上是它区域 1 槽 0 的第 0 代根（区域 1 槽 1 那份错位的 txg 3 不算）"
    );
    assert_eq!(
        by_txg.newest_valid_root_on_each_device,
        BTreeMap::from([
            (DeviceIdentity(0), Some(CURRENT_TXG_OF_THE_HISTORY)),
            (DeviceIdentity(1), None),
        ]),
        "按 txg 认：第 0 代根三份都推到区域 0（盘 0），盘 1 一条有效根都没有"
    );
    assert_eq!(
        (
            by_ring_slot.newest_valid_root_on_every_device,
            by_txg.newest_valid_root_on_every_device
        ),
        (Some(GENESIS_CHECKPOINT_TXG), None),
        "各盘最新有效根取最小两边不同：按槽认是 0，按 txg 认是「没有」"
    );
    assert_eq!(
        by_ring_slot.released_immediately, by_txg.released_immediately,
        "按槽认与按 txg 认，卸载当场放开的那一截逐个落点相同"
    );
    assert!(
        by_ring_slot.released_immediately.is_empty(),
        "当场放开的那一截两边都空：{:?}",
        by_ring_slot.released_immediately
    );
    assert!(
        roots_read.iter().all(|(ring_slot, root)| {
            root_sits_in_the_ring_slot_the_formula_gives(
                root.checkpoint_txg,
                *ring_slot,
                slots_per_region,
            )
        }),
        "前提（P1 读者核落点）：读出的每一条根都住在它的 txg 该住的那一槽（第 0 代根住 mkfs 种根的槽）：{roots_read:?}"
    );
    assert_eq!(
        roots_read
            .iter()
            .filter(|(ring_slot, _)| ring_slot.region == REGION_OF_DEVICE_ONE)
            .map(|(ring_slot, root)| (*ring_slot, root.checkpoint_txg))
            .collect::<Vec<(RootRingSlot, CheckpointTxg)>>(),
        vec![(GENESIS_RING_SLOT_OF_REGION_ONE, GENESIS_CHECKPOINT_TXG)],
        "前提（P1 读者核落点）：区域 1（盘 1）上读得出的根只有槽 0 那一份第 0 代根，槽 1 那份错位的根当读坏"
    );
    assert!(
        !root_is_abandoned_by_the_instance_table(&genesis_root_read_from_region_one, &table),
        "前提（P2 第 0 代根不被抛弃）：现行那一版的实例表不抛弃区域 1 槽 0 那一份第 0 代根"
    );
    assert!(
        records_before_the_unmount
            .iter()
            .filter(|record| record.is_released)
            .all(|record| record.generation != GENESIS_CHECKPOINT_TXG),
        "前提（P4 没有释放代为 0 的已释放记录）：按槽认当场放到 0，放不开一个落点"
    );
    let records_with_one_released_at_generation_zero: Vec<AllocationRecord> =
        records_before_the_unmount
            .iter()
            .copied()
            .chain([AllocationRecord {
                device: DEVICE_WHOSE_RECORDS_THE_RECLAIM_REPORTS,
                slot: SLOT_OF_THE_RECORD_RELEASED_AT_GENERATION_ZERO,
                span_slots: 1,
                generation: GENESIS_CHECKPOINT_TXG,
                is_released: true,
                quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
            }])
            .collect();
    assert_eq!(
        (
            release_by(
                DeviceOfARootRead::OfTheRingSlotItWasReadFrom,
                &records_with_one_released_at_generation_zero,
            )
            .released_immediately,
            release_by(
                DeviceOfARootRead::OfTheRingSlotTheFormulaGivesItsTxg,
                &records_with_one_released_at_generation_zero,
            )
            .released_immediately,
        ),
        (
            BTreeSet::from([(SLOT_OF_THE_RECORD_RELEASED_AT_GENERATION_ZERO, 1)]),
            BTreeSet::new(),
        ),
        "P4 不成立时（另添一条释放代为 0 的已释放记录）两边分开：按槽认当场放开它，按 txg 认一个不放——这一形的等价靠 P4"
    );
    let released_up_to_the_new_floor =
        released_placements_up_to(&records_before_the_unmount, current.root.checkpoint_txg);
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(&image);
    let mut current_version = PoolVersion::WithFile(std::sync::Arc::new(current));
    let unmounted = unmount(
        &parameters,
        &mut devices,
        &mut allocator,
        &mut current_version,
        ShadowLedger::On,
    );
    let Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) = &unmounted else {
        panic!(
            "现行那一版带文件，正常卸载做成、推抬 F 的空发布：{:?}",
            unmounted.as_ref().err()
        );
    };
    assert_eq!(raised.rollback_floor, CURRENT_TXG_OF_THE_HISTORY);
    let reclaimed: Vec<(SlotNumber, u64)> = raised
        .reclaimed
        .iter()
        .map(|placement| (placement.slot, placement.span))
        .collect();
    assert_eq!(
        reclaimed.len(),
        released_up_to_the_new_floor.len(),
        "卸载回收交回的落点个数恰是释放代不高于新 F 的那一截（当场放开的那一截为空，全是扣到生效的）：{reclaimed:?}"
    );
    assert_eq!(
        reclaimed.iter().copied().collect::<BTreeSet<_>>(),
        released_up_to_the_new_floor,
        "卸载回收交回的落点恰是释放代不高于新 F 的那一截"
    );
}
