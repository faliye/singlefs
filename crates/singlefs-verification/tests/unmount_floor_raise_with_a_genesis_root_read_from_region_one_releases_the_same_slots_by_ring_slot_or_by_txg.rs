//! 正常卸载那一遍给每条读出的根认盘：按读出它的那一槽所在区域认（`singlefs_core::mount` 的 `roots_on_the_devices_of_their_ring_slots`，
//! 产品今天的写法），与按落点公式从 txg 推槽再认，在卸载走得到的盘面上算出的每块盘最新有效根、当场放开的那一截逐项相同（欠账 C972；
//! 判决 `research/prompts/m3-close-b19-r3fix-code-r1-main-verification.md` 第二节 Q2.3）。把调用点那一处实参改成按 txg 推槽是等价变异，
//! 这一条把等价性留档（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」等价变异那一行）。
//!
//! 两种认法只在第 0 代根上不同：mkfs 在三个区域的槽 0 各种一份 txg 0 的根，公式把三份都推到区域 0 的槽 0。撑住「算出的数相同」的是：
//! - 读者核落点（`singlefs_core::root_ring::root_sits_in_the_ring_slot_the_formula_gives`，读根环的 `recovery::read_root_ring_slot`
//!   把不住在公式那一槽的根当读坏）：读出的非第 0 代根都住公式那一槽，两种认法认到同一块盘；
//! - 第 0 代根按实例表不被抛弃（`recovery::root_is_abandoned_by_the_instance_table`）；
//! - 没有释放代为 0 的已释放记录：第 0 代根算进哪块盘，至多让那块盘的最新有效根在 0 与「没有」之间换，当场放开的那一截两边都空。
//!
//! 盘面（根环有洞的小池，S = 4、环长 12，区域 0、2 在盘 0，区域 1 在盘 1）：mkfs → 可写挂载（零单元的发布 1、2）→ 第一个文件（txg 3）
//! → 覆盖写 txg 4、5、6。之后在盘 1 上动两槽：
//! - 区域 1 槽 0 写回 mkfs 写下的第 0 代根（txg 1 那条根的写被设备吞掉、那一槽留着 mkfs 的那一份）。卸载那一遍于是从区域 1 读出第 0 代根：
//!   按槽认在盘 1，按 txg 认在盘 0。
//! - 区域 1 槽 3（这次挂载那一刻读回全零、之后没写过，这个进程不知道那里住着根）抄进现行那一版（txg 6，公式那一槽在区域 0、盘 0）的根槽字节：
//!   一份错位的根。读者核落点时它当读坏；不核时按槽认会把 txg 6 算到盘 1 上，盘 1 的最新有效根从 4 变成 6，两种认法分开。
mod common_root_ring_holes;

use std::collections::{BTreeMap, BTreeSet};

use common_root_ring_holes::{
    content_of_the_version, devices_of, parameters_of_the_ring_hole_pools, pool_of,
    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
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
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

/// 盘 1 独占的区域。
const REGION_OF_DEVICE_ONE: u64 = 1;

/// 区域 1 里 mkfs 种第 0 代根的那一槽：盘面上它留着第 0 代根（txg 1 那条根的写被吞掉）。
const GENESIS_RING_SLOT_OF_REGION_ONE: RootRingSlot = RootRingSlot {
    region: REGION_OF_DEVICE_ONE,
    slot: 0,
};

/// 区域 1 的末槽（公式里住 txg 10，这段历史到不了）：盘面上它抄着现行那一版的根，是一份错位的根。
const RING_SLOT_HOLDING_A_MISPLACED_COPY_OF_THE_CURRENT_ROOT: RootRingSlot = RootRingSlot {
    region: REGION_OF_DEVICE_ONE,
    slot: 3,
};

/// 覆盖写推到的那一版：现行那一版的根在区域 0（盘 0），盘 1 上最新的根是 txg 4。
const CURRENT_TXG_OF_THE_HISTORY: CheckpointTxg = CheckpointTxg(6);

/// 盘 1 上最新的有效根（区域 1 槽 1 那一条）。
const NEWEST_VALID_ROOT_ON_DEVICE_ONE: CheckpointTxg = CheckpointTxg(4);

/// 分配器回收交回的落点按这块盘的记录报（两盘同槽，`PoolAllocator::reclaim_released_up_to` 只取池里排第一的那块盘）。
const DEVICE_WHOSE_RECORDS_THE_RECLAIM_REPORTS: DeviceIdentity = DeviceIdentity(0);

/// 卸载那一刻的盘面、这个进程的分配器与现行那一版（txg 6）。
struct PoolAboutToUnmount {
    image: MemoryPool,
    allocator: PoolAllocator,
    current: TransactionOutput,
}

/// 文件头那段历史：mkfs、可写挂载、第一个文件、覆盖写到 txg 6，再把盘 1 上区域 1 的槽 0 写回第 0 代根、槽 3 抄进现行那一版的根。
fn pool_with_a_genesis_root_left_in_region_one_and_a_misplaced_copy_of_the_current_root(
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
    let mut current: TransactionOutput = {
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
    // 迭代上界：txg 逐次加一推到 6（三次覆盖写）；跨轮携带的是现行那一版，每轮它的 txg 加一。
    while current.root.checkpoint_txg < CURRENT_TXG_OF_THE_HISTORY {
        let salt = current.root.checkpoint_txg.0 + 1;
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            instance,
        )
        .expect("覆盖写");
    }
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

/// 场景：文件头那份盘面，卸载那一遍从区域 1 槽 0 读出第 0 代根、区域 1 槽 3 读到一份错位的现行那一版的根，在这个进程里正常卸载。
/// 预期：按槽认与按 txg 认算出的每块盘最新有效根（盘 0 是 6、盘 1 是 4）、各盘取最小（4）与当场放开的那一截（释放代不高于 4 的已释放落点）
/// 逐项相同；卸载做成，回收交回的落点前一截恰是当场放开的那一截、后一截恰是扣到生效的那一截（释放代 5、6）。
/// 判别力：
/// - 读者不核落点（`read_root_ring_slot` 收下错位的根）：按槽认把 txg 6 算到盘 1 上，盘 1 的最新有效根两边是 6 与 4，第一条断言红；
/// - 第 0 代根被当成被抛弃（`root_is_abandoned_by_the_instance_table` 对 txg 0 交真）：「第 0 代根不被抛弃」那一条前提红，
///   或更早在搭池那一步的可写挂载上红。这一形里两种认法照样相同（抛弃在认盘之后滤，第 0 代根两边一起滤掉），它不是卸载这一格等价的支柱，
///   支柱是读者核落点与没有释放代为 0 的已释放记录，见文件头。
/// - 调用点那一处实参改成按 txg 推槽（等价变异，不进 `crates/mutations.tsv`）：这一条照旧绿，合法域上同值。
#[test]
fn unmount_with_a_genesis_root_read_from_region_one_releases_the_same_slots_whether_roots_count_on_the_device_of_their_ring_slot_or_of_their_txg(
) {
    let PoolAboutToUnmount {
        image,
        mut allocator,
        current,
    } = pool_with_a_genesis_root_left_in_region_one_and_a_misplaced_copy_of_the_current_root();
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
    let release_by = |attribution| {
        release_by_one_attribution(
            attribution,
            &roots_read,
            &immutable.region_devices,
            slots_per_region,
            &table,
            &records_before_the_unmount,
            current.root.checkpoint_txg,
        )
    };
    let by_ring_slot = release_by(DeviceOfARootRead::OfTheRingSlotItWasReadFrom);
    let by_txg = release_by(DeviceOfARootRead::OfTheRingSlotTheFormulaGivesItsTxg);
    assert_eq!(
        by_ring_slot.newest_valid_root_on_each_device, by_txg.newest_valid_root_on_each_device,
        "按槽认与按 txg 认，每块盘上最新的有效根逐盘相同"
    );
    assert_eq!(
        by_ring_slot.newest_valid_root_on_every_device, by_txg.newest_valid_root_on_every_device,
        "按槽认与按 txg 认，各盘最新有效根取最小相同"
    );
    assert_eq!(
        by_ring_slot.released_immediately, by_txg.released_immediately,
        "按槽认与按 txg 认，卸载当场放开的那一截逐个落点相同"
    );
    assert_eq!(
        by_ring_slot.newest_valid_root_on_each_device,
        BTreeMap::from([
            (DeviceIdentity(0), Some(CURRENT_TXG_OF_THE_HISTORY)),
            (DeviceIdentity(1), Some(NEWEST_VALID_ROOT_ON_DEVICE_ONE)),
        ]),
        "盘 0 上最新的有效根是现行那一版 txg 6，盘 1 上是 txg 4（区域 1 槽 3 那份错位的 txg 6 不算）"
    );
    assert!(
        roots_read.iter().all(|(ring_slot, root)| {
            root_sits_in_the_ring_slot_the_formula_gives(
                root.checkpoint_txg,
                *ring_slot,
                slots_per_region,
            )
        }),
        "前提（读者核落点）：读出的每一条根都住在它的 txg 该住的那一槽（第 0 代根住 mkfs 种根的槽）：{roots_read:?}"
    );
    assert!(
        !roots_read
            .iter()
            .any(|(ring_slot, _)| *ring_slot
                == RING_SLOT_HOLDING_A_MISPLACED_COPY_OF_THE_CURRENT_ROOT),
        "前提（读者核落点）：区域 1 槽 3 那份错位的根当读坏，不在读出的根里"
    );
    assert!(
        !root_is_abandoned_by_the_instance_table(&genesis_root_read_from_region_one, &table),
        "前提（第 0 代根不被抛弃）：现行那一版的实例表不抛弃第 0 代根"
    );
    assert!(
        records_before_the_unmount
            .iter()
            .filter(|record| record.is_released)
            .all(|record| record.generation != GENESIS_CHECKPOINT_TXG),
        "前提（没有释放代为 0 的已释放记录）：第 0 代根算进哪块盘都放不开一个落点"
    );
    let released_up_to_the_new_floor =
        released_placements_up_to(&records_before_the_unmount, current.root.checkpoint_txg);
    let held_until_the_floor_takes_effect: BTreeSet<(SlotNumber, u64)> =
        released_up_to_the_new_floor
            .difference(&by_ring_slot.released_immediately)
            .copied()
            .collect();
    assert!(
        !by_ring_slot.released_immediately.is_empty() && !held_until_the_floor_takes_effect.is_empty(),
        "前提：当场放开的（释放代不高于 4）与扣到生效的（释放代 5、6）两截都不空，切在哪儿分得出：{:?} / {:?}",
        by_ring_slot.released_immediately,
        held_until_the_floor_takes_effect
    );
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
    assert!(
        reclaimed.len() >= by_ring_slot.released_immediately.len(),
        "回收交回的落点不少于当场放开的那一截：{reclaimed:?}"
    );
    let (reclaimed_immediately, reclaimed_and_held) =
        reclaimed.split_at(by_ring_slot.released_immediately.len());
    assert_eq!(
        reclaimed_immediately
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        by_ring_slot.released_immediately,
        "卸载回收交回的前一截（当场放开）恰是两种认法算出的那一截"
    );
    assert_eq!(
        reclaimed_and_held.iter().copied().collect::<BTreeSet<_>>(),
        held_until_the_floor_takes_effect,
        "卸载回收交回的后一截（扣到生效）恰是释放代高于 4、不高于新 F 的那一截"
    );
}
