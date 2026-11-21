//! 分配记录每个落点每盘各一条，新池新建文件之后每块盘上的落点数按池的几何现算，连同那一步预演被拒的原因。

use super::*;

/// 分配记录「每个落点每盘各一条」（两盘同槽、同一批字段）：同一块盘上一个槽只许一条记录，每块盘各自的（槽, 跨度, 代, 已释放）集合相同，
/// 每盘不少于 `placements_on_each_device_at_least` 个落点（走读传的是这套几何下新池新建文件之后每盘的落点数，
/// [`placements_on_each_device_after_the_new_pool_file_creation`]）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
/// 走读自己不判 key 严格递增，这里逐盘核槽号不重复。
///
/// # Panics
/// `device_identities` 为空：调用方要交一张非空的盘表。产品调用方交的是读者的盘表（`PoolReader::device_identities`），
/// 零块盘的池走到这里之前在读实例表单元那一步已经报错返回；空盘表在这里没有「每盘一条」可判，不交回 `true` 也不交回 `false`。
#[must_use]
pub fn allocation_records_are_one_per_device(
    records: &[AllocationRecord],
    device_identities: &[DeviceIdentity],
    placements_on_each_device_at_least: PlacementsOnEachDevice,
) -> bool {
    assert!(
        !device_identities.is_empty(),
        "盘表不许为空：产品调用方交读者的盘表，零块盘的池在读实例表单元那一步已经报错返回"
    );
    let mut placements_per_device: BTreeMap<DeviceIdentity, BTreeSet<(u64, u16, u64, bool)>> =
        BTreeMap::new();
    let mut slots_per_device: BTreeMap<DeviceIdentity, BTreeSet<u64>> = BTreeMap::new();
    for record in records {
        if !slots_per_device
            .entry(record.device)
            .or_default()
            .insert(record.slot.0)
        {
            return false;
        }
        placements_per_device
            .entry(record.device)
            .or_default()
            .insert((
                record.slot.0,
                record.span_slots,
                record.generation.0,
                record.is_released,
            ));
    }
    if placements_per_device.len() != device_identities.len()
        || device_identities
            .iter()
            .any(|identity| !placements_per_device.contains_key(identity))
    {
        return false;
    }
    let mut placement_sets = placements_per_device.values();
    let first_device_placements = placement_sets
        .next()
        .expect("开头断言过盘表非空，上面又核过盘表里每块盘都有记录：至少有一块盘的落点集合");
    first_device_placements.len() >= placements_on_each_device_at_least.0
        && placement_sets.all(|placements| placements == first_device_placements)
}

/// 每块盘上的落点数（分配记录条数按盘数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlacementsOnEachDevice(pub usize);

/// 新池新建文件之后每块盘上的落点数，按这个池的几何现算（C692 取 G2「按几何现算」，用户 2026-10-07 定）：分配记录每盘落点数的下限
/// （[`allocation_records_are_one_per_device`]），之后每次发布只多不少。几何是每块盘的字节数（`device_sizes_in_bytes`，分配记录树的根层级
/// 由它定，[`AllocationRecordTreeGeometry::of_device_sizes_in_bytes`]）与单元区起点（系统配置里的环长现算的，`unit_area_start`）。
/// 算法是在一个空分配器上照新池新建文件那一版分一遍落点（D3（空间分配） 已定项 8 / 已定项 10 的分配器本身，不另写一份落点规则）：
/// mkfs 写在单元区里的两个单元（实例表第 0 片在单元区起点、两槽，第 0 版树表紧跟其后、一槽，与 `make_filesystem` 同一个摆法），
/// 一个数据单元（第一个文件至少一个数据单元，`write_request_split::data_unit_count_of_a_sequential_write`），再照提交内生块的 bump 次序
/// （D3（空间分配） 已定项 10 ⑤：按树 ID 升序、树内先叶后根、中央映射树倒数第二、树表单元最末）分 extent 树根、inode 叶容器、inode 树根、
/// 分配记录树的节点、记账树根、中央映射树根、树表单元。分配记录树要几个节点取决于这些记录落在几片叶里（按位置寻址，D8（核心索引结构） 已定项 14），
/// 而节点自己的落点也有记录：节点数从「每块盘一条根到叶的路径加根」起，按分完之后的记录现算的节点数调，到两者相等为止。
/// 默认几何（4 GiB 盘、默认环）每盘 14（字节表五：28 条分配记录）；1024 MiB 环那套几何（单元区起点 66560）提交内生块多两个、每盘 16。
///
/// 盘的字节数是块设备报的、或崩溃镜像自述的，不是不变量（正推「发现 6」，判决 `research/prompts/m3-batch33t-code-r1-main-verification.md`
/// 第三节第 1 条「排演函数小盘不 panic」）：建排演用的空闲图之前逐盘判够不够长，不够就交错误。
///
/// # Errors
/// 按 `device_sizes_in_bytes` 次序第一块完整槽数不到单元区起点的盘 ⇒ [`NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart`]；
/// 空分配器上分不出这几个落点（单元区装不下新池新建文件那十几个单元，这套几何上写不出带文件的一版）⇒
/// [`NewPoolFileCreationRehearsalRefused::UnitAreaCannotHoldTheNewPoolFileCreation`]。
///
/// # Panics
/// 节点数调了 [`ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS`] 轮还不相等。
pub fn placements_on_each_device_after_the_new_pool_file_creation(
    device_sizes_in_bytes: &[(DeviceIdentity, u64)],
    unit_area_start: UnitAreaStart,
) -> Result<PlacementsOnEachDevice, NewPoolFileCreationRehearsalRefused> {
    if let Some((device, device_bytes)) = device_sizes_in_bytes.iter().find(|(_, device_bytes)| {
        unit_area_slots_of_device_starting_at(*device_bytes, unit_area_start).is_err()
    }) {
        return Err(
            NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart {
                device: *device,
                device_bytes: *device_bytes,
            },
        );
    }
    let geometry = AllocationRecordTreeGeometry::of_device_sizes_in_bytes(device_sizes_in_bytes);
    let new_pool_file_creation_txg = CheckpointTxg(1);
    // 新池新建文件的落点都在单元区开头的两段里（mkfs 与数据单元在第一段，提交内生块开在最低的全空段、即第二段）：空闲图只建到那两段为止，
    // 走读每次都调它，不为整块盘建一张图（4 GiB 盘是二十多万槽）。分配记录树的几何仍按整块盘的字节数（上面那一行）。
    let rehearsal_bytes_on_each_device =
        (unit_area_start.slot().0 + 2 * CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES;
    let mut allocator = crate::allocator::PoolAllocator::new(
        device_sizes_in_bytes
            .iter()
            .map(|(device, device_bytes)| {
                crate::allocator::DeviceFreeMap::with_unit_area_start(
                    *device,
                    (*device_bytes).min(rehearsal_bytes_on_each_device),
                    unit_area_start,
                )
            })
            .collect(),
    );
    let instance_table_span = TransactionUnit::InstanceTable.span_slots();
    allocator.mark_format_time_units(
        crate::allocator::Placement {
            slot: unit_area_start.slot(),
            span: instance_table_span,
        },
        crate::allocator::Placement {
            slot: SlotNumber(unit_area_start.slot().0 + instance_table_span),
            span: TransactionUnit::TreeTable.span_slots(),
        },
    );
    let allocate = |rehearsing: &mut crate::allocator::PoolAllocator, unit: TransactionUnit| {
        let placement = match unit.placement() {
            crate::transaction::PlacementRule::UserData => {
                rehearsing.allocate_user_data(new_pool_file_creation_txg)
            }
            crate::transaction::PlacementRule::CommitGenerated(footprint) => {
                rehearsing.allocate_commit_generated(footprint, new_pool_file_creation_txg)
            }
        };
        placement
            .map(|_placement| ())
            .ok_or(NewPoolFileCreationRehearsalRefused::UnitAreaCannotHoldTheNewPoolFileCreation)
    };
    for unit in [
        TransactionUnit::Data(DataUnitIndexInFile(0)),
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
        TransactionUnit::InodeRoot,
    ] {
        allocate(&mut allocator, unit)?;
    }
    let units_before_the_allocation_record_tree = allocator.clone();
    let device_count = device_sizes_in_bytes.len();
    // 迭代上界 ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS；跨轮只带猜的节点数，每一轮从同一个分配器拷贝重分。
    let mut allocation_record_tree_nodes = device_count * usize::from(geometry.root_level()) + 1;
    for _round in 0..ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS {
        let mut rehearsal = units_before_the_allocation_record_tree.clone();
        for _node in 0..allocation_record_tree_nodes {
            allocate(&mut rehearsal, TransactionUnit::AllocationTree)?;
        }
        for unit in [
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ] {
            allocate(&mut rehearsal, unit)?;
        }
        let nodes_holding_the_records =
            crate::allocation_record_tree::nodes_holding_records(&geometry, rehearsal.records())
                .len();
        if nodes_holding_the_records == allocation_record_tree_nodes {
            return Ok(PlacementsOnEachDevice(
                rehearsal.records().len() / device_count,
            ));
        }
        allocation_record_tree_nodes = nodes_holding_the_records;
    }
    panic!(
        "分配记录树的节点数调了 {ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS} 轮还不相等：每一轮多分的节点只会让记录多落进一两片叶，几轮就该停"
    );
}

/// [`placements_on_each_device_after_the_new_pool_file_creation`] 排演不出来的两种情形（调用方据此报恢复失败，不 panic）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewPoolFileCreationRehearsalRefused {
    /// 这块盘的完整槽数不到单元区起点（盘的字节数是块设备报的、或崩溃镜像自述的）。
    DeviceEndsBeforeTheUnitAreaStart {
        device: DeviceIdentity,
        device_bytes: u64,
    },
    /// 空分配器上分不出新池新建文件那十几个单元：这套几何上写不出带文件的一版。
    UnitAreaCannotHoldTheNewPoolFileCreation,
}

/// [`placements_on_each_device_after_the_new_pool_file_creation`] 调分配记录树节点数的轮数上界：猜的节点数每一轮只会因为节点自己的落点
/// 多落进一片叶而加几个，推的，没量过会不会到第三轮。
const ALLOCATION_RECORD_TREE_NODE_COUNT_SETTLING_ROUNDS: usize = 8;

/// 每个落点每盘一条（两盘同槽）：新池新建文件之后每盘的落点数按这个池的几何现算（默认几何 14 × 盘数，
/// [`placements_on_each_device_after_the_new_pool_file_creation`]），之后每次发布只多不少（换下的那些改写、不删）。
/// 只核总数是盘数的整数倍拦不住「一盘多一条、另一盘少一条」——发布 B 三方第一轮正推腿打中，改成逐盘核同一批（槽, 跨度）
/// （[`allocation_records_are_one_per_device`]）。
///
/// # Panics
/// 读者列出的盘说不出自己的字节数：每个读者都从同一张表里取（`PoolReader::device_size_in_bytes`）。
pub(super) fn allocation_records_are_one_placement_per_device_on_every_device(
    reader: &dyn PoolReader,
    unit_area_start: UnitAreaStart,
    records: &[AllocationRecord],
    device_identities: &[DeviceIdentity],
) -> Result<(), RecoveryFailure> {
    let device_sizes_in_bytes: Vec<(DeviceIdentity, u64)> = device_identities
        .iter()
        .map(|device| {
            (
                *device,
                reader
                    .device_size_in_bytes(*device)
                    .expect("读者列出的盘说得出自己的字节数"),
            )
        })
        .collect();
    let not_one_placement_per_device = RecoveryFailure::InvariantViolated {
        invariant: "E142 走读同款",
        detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数",
    };
    // 先判记录本身（下限取 0），再排演下限：排演要按盘的字节数建空闲图，记录判空、各盘不同这些不用它就判得出（正推「发现 6」）。
    if !allocation_records_are_one_per_device(records, device_identities, PlacementsOnEachDevice(0))
    {
        return Err(not_one_placement_per_device);
    }
    let placements_on_each_device_at_least =
        match placements_on_each_device_after_the_new_pool_file_creation(
            &device_sizes_in_bytes,
            unit_area_start,
        ) {
            Ok(placements_on_each_device_at_least) => placements_on_each_device_at_least,
            Err(NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart {
                device,
                device_bytes,
            }) => {
                return Err(RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                    device,
                    device_bytes,
                })
            }
            // 这套几何上写不出带文件的一版，分配记录就不够这一版的下限。
            Err(NewPoolFileCreationRehearsalRefused::UnitAreaCannotHoldTheNewPoolFileCreation) => {
                return Err(not_one_placement_per_device)
            }
        };
    if !allocation_records_are_one_per_device(
        records,
        device_identities,
        placements_on_each_device_at_least,
    ) {
        return Err(not_one_placement_per_device);
    }
    Ok(())
}

#[cfg(test)]
mod allocation_records_per_device_tests {
    use super::{
        allocation_records_are_one_per_device,
        allocation_records_are_one_placement_per_device_on_every_device,
        placements_on_each_device_after_the_new_pool_file_creation,
        NewPoolFileCreationRehearsalRefused, PlacementsOnEachDevice, PoolReader, RecoveryFailure,
    };
    use crate::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
    use crate::allocator::{AllocationRecord, UnitAreaStart};

    const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

    /// 默认几何与 1024 MiB 环那套几何的盘都是 4 GiB（`.claude/kb/layout/01-first-txn.md` 字节表五；
    /// `crates/singlefs-verification/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的一 GiB 环那一条）。
    const FOUR_GIBIBYTES: u64 = 4 << 30;
    const ONE_GIBIBYTE_JOURNAL_RING_BYTES: u64 = 1 << 30;

    fn both_devices_of_four_gibibytes() -> Vec<(DeviceIdentity, u64)> {
        BOTH_DEVICES
            .iter()
            .map(|device| (*device, FOUR_GIBIBYTES))
            .collect()
    }

    fn placements_on_each_device_of_the_default_geometry() -> PlacementsOnEachDevice {
        placements_on_each_device_after_the_new_pool_file_creation(
            &both_devices_of_four_gibibytes(),
            UnitAreaStart::of_the_default_journal_ring(),
        )
        .expect("4 GiB 盘够长到默认环的单元区起点、装得下新池新建文件")
    }

    fn unit_area_start_after_a_one_gibibyte_journal_ring() -> UnitAreaStart {
        UnitAreaStart::following_the_journal_ring(ONE_GIBIBYTE_JOURNAL_RING_BYTES)
            .expect("1024 MiB 环末尾的下一个槽（66560）落在聚簇段边界上")
    }

    fn record(device: DeviceIdentity, slot: u64, is_released: bool) -> AllocationRecord {
        AllocationRecord {
            device,
            slot: SlotNumber(slot),
            span_slots: 2,
            generation: CheckpointTxg(3),
            is_released,
            quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        }
    }

    /// 按下限造的形状：每盘恰好 `placements_on_each_device` 个落点、同一批槽号，从 `first_slot` 起；已释放的记录照样算一个落点。
    fn shape_with_placements_on_each_device(
        first_slot: u64,
        placements_on_each_device: PlacementsOnEachDevice,
    ) -> Vec<AllocationRecord> {
        let mut records = Vec::new();
        for placement_index in 0..placements_on_each_device.0 {
            let slot = first_slot + 2 * u64::try_from(placement_index).expect("落点序号");
            for device in BOTH_DEVICES {
                records.push(record(device, slot, placement_index == 0));
            }
        }
        records
    }

    fn new_pool_file_creation_shape() -> Vec<AllocationRecord> {
        shape_with_placements_on_each_device(
            50176,
            placements_on_each_device_of_the_default_geometry(),
        )
    }

    #[test]
    fn the_default_geometry_has_fourteen_placements_on_each_device_after_the_new_pool_file_creation(
    ) {
        assert_eq!(
            placements_on_each_device_of_the_default_geometry(),
            PlacementsOnEachDevice(14),
            "字节表五：28 条分配记录 = m1、m2、t1..t12 十四个单元 × 2 盘"
        );
    }

    #[test]
    fn the_one_gibibyte_journal_ring_geometry_has_sixteen_placements_on_each_device_after_the_new_pool_file_creation(
    ) {
        assert_eq!(
            placements_on_each_device_after_the_new_pool_file_creation(
                &both_devices_of_four_gibibytes(),
                unit_area_start_after_a_one_gibibyte_journal_ring(),
            ),
            Ok(PlacementsOnEachDevice(16)),
            "单元区起点 66560：mkfs 与数据单元在槽 81 那片叶（W = 812），提交内生块在槽 82 那片叶，分配记录树每盘多一片叶、多两个节点，提交内生块 13 个"
        );
    }

    /// 记录与盘表都空：盘表非空是这个函数的前提（文档注释的 panic 条件），在开头那一句断言上 panic、消息说的就是这个前提。
    /// 改之前走到取第一块盘的落点集合那一句 `expect`，消息写的「上面核过每块盘都有记录」在空输入上不成立（外部代码审查第一条）。
    #[test]
    #[should_panic(expected = "盘表不许为空")]
    fn an_empty_device_table_with_no_records_panics_on_the_non_empty_device_table_precondition() {
        let _ = allocation_records_are_one_per_device(&[], &[], PlacementsOnEachDevice(0));
    }

    #[test]
    fn exactly_the_placements_of_the_default_geometry_are_accepted_and_one_fewer_is_rejected() {
        let placements_on_each_device = placements_on_each_device_of_the_default_geometry();
        assert!(allocation_records_are_one_per_device(
            &new_pool_file_creation_shape(),
            &BOTH_DEVICES,
            placements_on_each_device,
        ));
        let one_fewer = shape_with_placements_on_each_device(
            50176,
            PlacementsOnEachDevice(placements_on_each_device.0 - 1),
        );
        assert!(
            !allocation_records_are_one_per_device(
                &one_fewer,
                &BOTH_DEVICES,
                placements_on_each_device
            ),
            "每盘比新池新建文件少一个落点：判红"
        );
    }

    #[test]
    fn exactly_the_placements_of_the_one_gibibyte_journal_ring_geometry_are_accepted_and_one_fewer_is_rejected(
    ) {
        let unit_area_start = unit_area_start_after_a_one_gibibyte_journal_ring();
        let placements_on_each_device = placements_on_each_device_after_the_new_pool_file_creation(
            &both_devices_of_four_gibibytes(),
            unit_area_start,
        )
        .expect("4 GiB 盘够长到 1024 MiB 环的单元区起点、装得下新池新建文件");
        assert!(allocation_records_are_one_per_device(
            &shape_with_placements_on_each_device(
                unit_area_start.slot().0,
                placements_on_each_device
            ),
            &BOTH_DEVICES,
            placements_on_each_device,
        ));
        let one_fewer = shape_with_placements_on_each_device(
            unit_area_start.slot().0,
            PlacementsOnEachDevice(placements_on_each_device.0 - 1),
        );
        assert!(
            !allocation_records_are_one_per_device(
                &one_fewer,
                &BOTH_DEVICES,
                placements_on_each_device
            ),
            "每盘比这套几何下的新池新建文件少一个落点：判红（默认几何的 14 放得过它）"
        );
    }

    #[test]
    fn placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even() {
        // 盘 0 多一条、盘 1 少一条：总数仍是盘数的整数倍，旧的「总数是盘数的整数倍」判定放过它。
        let mut records = new_pool_file_creation_shape();
        let moved = records
            .iter()
            .position(|record| record.device == DeviceIdentity(1))
            .expect("有盘 1 的记录");
        records[moved].device = DeviceIdentity(0);
        records[moved].slot = SlotNumber(50300);
        assert_eq!(
            records.len(),
            2 * placements_on_each_device_of_the_default_geometry().0
        );
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
    }

    #[test]
    fn unknown_device_or_missing_device_is_rejected() {
        let mut records = new_pool_file_creation_shape();
        records.push(record(DeviceIdentity(2), 50176, false));
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
        let only_device_zero: Vec<AllocationRecord> = new_pool_file_creation_shape()
            .into_iter()
            .filter(|record| record.device == DeviceIdentity(0))
            .collect();
        assert!(!allocation_records_are_one_per_device(
            &only_device_zero,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
    }

    /// 两盘同一个落点、一盘改写成已释放而另一盘没有：槽号集合相同，字段不同，同样拒。
    #[test]
    fn release_rewritten_on_one_device_only_is_rejected() {
        let mut records = new_pool_file_creation_shape();
        let released_on_device_one = records
            .iter()
            .position(|record| record.device == DeviceIdentity(1) && record.is_released)
            .expect("盘 1 有一条已释放");
        records[released_on_device_one].is_released = false;
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
    }

    /// 同一块盘上同一个槽两条记录（代不同、两盘对称）：槽号集合相同、字段集合也相同，靠「同盘槽号唯一」拦。
    #[test]
    fn two_records_for_the_same_slot_on_the_same_device_are_rejected() {
        let mut records = new_pool_file_creation_shape();
        for device in BOTH_DEVICES {
            let mut second_record_for_first_slot = record(device, 50176, false);
            second_record_for_first_slot.generation = CheckpointTxg(9);
            records.push(second_record_for_first_slot);
        }
        assert!(!allocation_records_are_one_per_device(
            &records,
            &BOTH_DEVICES,
            placements_on_each_device_of_the_default_geometry(),
        ));
    }

    /// 1 MiB 的盘：默认环的单元区起点（槽 50176）在它末尾之后。
    const ONE_MEBIBYTE: u64 = 1 << 20;

    /// 只交得出「池里有哪几块盘、每块多大」的读者：盘 0 是 4 GiB，盘 1 只有 1 MiB。
    struct PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart;

    impl PoolReader for PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart {
        fn device_identities(&self) -> Vec<DeviceIdentity> {
            BOTH_DEVICES.to_vec()
        }
        fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
            match device {
                DeviceIdentity(0) => Some(FOUR_GIBIBYTES),
                DeviceIdentity(1) => Some(ONE_MEBIBYTE),
                DeviceIdentity(_) => None,
            }
        }
        fn physical_block_size_probed_on_the_device(
            &self,
            _device: DeviceIdentity,
        ) -> Option<crate::block_device::PhysicalBlockSizeInBytes> {
            unreachable!("分配记录每盘一条那一道判只读盘的身份与大小，不探测块宽")
        }
        fn read(
            &self,
            _device: DeviceIdentity,
            _offset: DeviceOffsetInBytes,
            _length: usize,
        ) -> Option<Vec<u8>> {
            unreachable!("分配记录每盘一条那一道判只读盘的身份与大小，不读盘上的字节")
        }
        fn journal_record_offsets_hint(
            &self,
            _device: DeviceIdentity,
            _ring_start: DeviceOffsetInBytes,
            _ring_bytes: u64,
        ) -> Option<Vec<DeviceOffsetInBytes>> {
            unreachable!("分配记录每盘一条那一道判不扫 journal 环")
        }
    }

    /// 正推「发现 6」：一块盘比单元区起点短时，排演新池新建文件交回那块盘，不在建空闲图那一步 panic。
    /// 判别力：排演不先判盘长（变异「排演不判盘短于单元区起点」）时这一条 panic、红。
    #[test]
    fn rehearsal_on_a_device_shorter_than_the_unit_area_start_returns_that_device_instead_of_panicking(
    ) {
        assert_eq!(
            placements_on_each_device_after_the_new_pool_file_creation(
                &[
                    (DeviceIdentity(0), FOUR_GIBIBYTES),
                    (DeviceIdentity(1), ONE_MEBIBYTE)
                ],
                UnitAreaStart::of_the_default_journal_ring(),
            ),
            Err(
                NewPoolFileCreationRehearsalRefused::DeviceEndsBeforeTheUnitAreaStart {
                    device: DeviceIdentity(1),
                    device_bytes: ONE_MEBIBYTE,
                }
            ),
            "盘 1 只有 1 MiB，到不了默认环的单元区起点"
        );
    }

    /// 同一块短盘上走读判分配记录每盘一条：记录是空的先判记录（不必排演就判得出），每盘一条而盘短报那块盘到不了单元区起点；两样都不 panic。
    /// 判别力：同上一条的变异，第二个断言 panic、红；排演挪回判记录之前（变异「排演先于判记录」）时第一个断言报的是盘短、红——
    /// 「不作为实参先求值」由第一个断言钉着。
    #[test]
    fn walk_judging_allocation_records_on_a_device_shorter_than_the_unit_area_start_returns_an_error_instead_of_panicking(
    ) {
        assert_eq!(
            allocation_records_are_one_placement_per_device_on_every_device(
                &PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart,
                UnitAreaStart::of_the_default_journal_ring(),
                &[],
                &BOTH_DEVICES,
            ),
            Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于这套几何下新池新建文件之后每盘的落点数",
            }),
            "一条分配记录都没有：先判记录，报每盘一条不成立"
        );
        assert_eq!(
            allocation_records_are_one_placement_per_device_on_every_device(
                &PoolWhoseSecondDeviceEndsBeforeTheUnitAreaStart,
                UnitAreaStart::of_the_default_journal_ring(),
                &[
                    record(DeviceIdentity(0), 50176, false),
                    record(DeviceIdentity(1), 50176, false)
                ],
                &BOTH_DEVICES,
            ),
            Err(RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                device: DeviceIdentity(1),
                device_bytes: ONE_MEBIBYTE,
            }),
            "每盘一条而盘 1 短于单元区起点：报那块盘，不 panic"
        );
    }
}
