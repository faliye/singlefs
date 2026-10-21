//! 整理的选段与计划（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G2；锚点取自正推报告
//! `research/prompts/m4-gc-r1-sonnet-output.md`「单测怎么钉绝对值」，锚点 4 照这一批的派发只钉条数与 key 集合；
//! 资格、被拒的段与空批两种照判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 S4）：
//! - 锚点 1：段 50176 / 50240 / 50304 活槽 [60, 10, 0]，greedy 选段 50240，全空的段 50304 扫过之后不选；
//! - 锚点 2：段 50240 里有一个钉住的槽 ⇒ 改选段 50176；
//! - 锚点 3：段 50240 在这次挂载开过的聚簇段里 ⇒ 改选段 50176；有资格的全被排除 ⇒ 没有合格段；
//! - 活意图的 R 不再选；同分取起点槽号最小的；
//! - 资格是「R 里至少一个能搬的单元」：活槽全是已死、不一致单元的段不选；
//! - 跨段的坏记录让两段的扫描整次拒：记进这次挂载的不合格段，不再选它们（选别的，或报没有合格段）；
//! - 锚点 4：K = 3、R 里三个可搬的数据单元 ⇒ 一批三条，key 集合就是那三把；
//! - 守恒：一批批搬下去，每批 ≤ K、批间不交、并集 = R 里可搬的 key 集合，钉住 / 隔离 / 已死 / 不一致的一个都不进；
//! - 一个能搬的都没有时分「搬空」（R 里没有未释放的记录）与「搬不动」（还有）。
//!
//! - 签发、停机与预算的几个判定（判决 G4、G8；攻方 K-c、K-g、K-h）：一个槽既钉住又在隔离集里，扫描判钉住、丢意图那一判照样看得到它；
//!   签发前判数的空洞只在 R 外、装着用户数据的段里；停机谓词看 R 里还有没有未释放的记录；全空段按记录口径数；预算 `3S − 4 − 已过去的`、至少 1；
//! - R 排他（D26（后台整理与放置回收） 已定项 1 第 3 条）：活意图排他时用户数据与新开的聚簇段都不落进 R，关掉排他时落得进。
//!
//! 选段经 [`choose_after_scanning`] 照 `choose_region` 的约定走到底：答「先扫」就扫那一段再选。期望值按锚点手写，不从被测函数反推。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::{
    AllocationRecord, DeviceFreeMap, Placement, PoolAllocator, UnitAreaStart, UnitFootprint,
};
use singlefs_core::compaction_plan::{
    choose_region, empty_cluster_segments_by_the_records, greedy_score, holes_admit_the_issue,
    holes_outside_the_region_in_segments_holding_user_data, plan_batch,
    publish_budget_of_a_compaction_run, region_holds_a_quarantined_slot,
    region_is_empty_by_the_records, the_completed_intent_yielded, CompactionBatchSize,
    CompactionBatchUnits, CompactionRegionExclusion, CompactionSwitches, PlannedBatch,
    PlannedRelocation, RegionChoice, RegionChoiceCriterion, SegmentsRefusedByTheRegionScan,
    UnitAreaOfOneDevice,
};
use singlefs_core::logged_operations::{CompactionIntent, CompactionRegion};
use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use singlefs_core::records::{mapping_key_for_data, mapping_key_for_node};
use singlefs_core::region_scan::AllocationInRegion;
use singlefs_core::region_scan::{scan_region, CentralMappingEntry, RegionScan, RegionScanRefusal};
use singlefs_core::unit::{
    build_data_unit, build_index_node, build_packed_unit, DataUnitIdentity, PackedIdentity,
    WriteOrder, PACKED_TYPE_INODE, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use singlefs_format::{TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE};

const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);
const BIRTH_TXG: CheckpointTxg = CheckpointTxg(7);
const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
const FILESYSTEM_IDENTIFIER: [u8; 16] = [7u8; 16];

/// 单元区三个整段 [50176, 50240)、[50240, 50304)、[50304, 50368)。
const FIRST_SEGMENT: u64 = 50176;
const SECOND_SEGMENT: u64 = 50240;
const THIRD_SEGMENT: u64 = 50304;
const SEGMENTS_IN_THE_UNIT_AREA: usize = 3;

fn unit_area_of_three_segments() -> UnitAreaOfOneDevice {
    let start = UnitAreaStart::of_the_default_journal_ring();
    assert_eq!(
        start.slot(),
        SlotNumber(FIRST_SEGMENT),
        "默认环下单元区从 50176 起"
    );
    UnitAreaOfOneDevice {
        device: DEVICE_ZERO,
        start,
        slots: 3 * 64,
    }
}

fn region_at(start_slot: u64) -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(DEVICE_ZERO, SlotNumber(start_slot))
        .expect("三个段的起点都是 64 的倍数")
}

/// 两块盘上同一个落点各一条未释放记录（两盘同槽）。
fn records_on_both_devices(slot: u64, span_slots: u16) -> [AllocationRecord; 2] {
    [DEVICE_ZERO, DEVICE_ONE].map(|device| AllocationRecord {
        device,
        slot: SlotNumber(slot),
        span_slots,
        generation: BIRTH_TXG,
        is_released: false,
    })
}

/// 一块镜像：两盘同槽各一份单元、各一条记录，加中央映射。计划那几条用例的整理 R 固定取段 50240。
#[derive(Default)]
struct PoolUnderTest {
    units: BTreeMap<SlotNumber, Vec<u8>>,
    records: Vec<AllocationRecord>,
    mapping_entries: Vec<CentralMappingEntry>,
    /// 下一个可搬数据单元用的事务号：每个单元的写序不同，映射 key 就不同。
    next_transaction: u64,
}

fn locations_at(slot: SlotNumber) -> [LocationEntry; 2] {
    [DEVICE_ZERO, DEVICE_ONE].map(|device| LocationEntry {
        device,
        slot,
        unit_checksum: 0,
    })
}

impl PoolUnderTest {
    fn place(&mut self, slot: u64, span_slots: u16, unit: Vec<u8>) {
        self.records
            .extend(records_on_both_devices(slot, span_slots));
        self.units.insert(SlotNumber(slot), unit);
    }

    /// 只放记录、不放单元：盘上读来的坏记录（跨段的那种）。
    fn place_record_without_a_unit(&mut self, slot: u64, span_slots: u16) {
        self.records
            .extend(records_on_both_devices(slot, span_slots));
    }

    /// 两盘各一条已释放的记录：不算活槽。
    fn place_released(&mut self, slot: u64, span_slots: u16) {
        for mut released in records_on_both_devices(slot, span_slots) {
            released.is_released = true;
            self.records.push(released);
        }
    }

    fn map(&mut self, key: &[u8], slot: u64) {
        self.mapping_entries.push(CentralMappingEntry {
            key: key.to_vec(),
            locations: locations_at(SlotNumber(slot)),
        });
    }

    /// 放一个可搬的单元：记录、字节、映射条目指着它。
    fn place_movable(
        &mut self,
        slot: u64,
        span_slots: u16,
        (unit, key): (Vec<u8>, Vec<u8>),
    ) -> Vec<u8> {
        self.place(slot, span_slots, unit);
        self.map(&key, slot);
        key
    }

    /// 从 `first_slot` 起连着放 `units` 个可搬的两槽数据单元（事务号各不相同）。
    fn place_movable_data_units(&mut self, first_slot: u64, units: u64) {
        for unit in 0..units {
            self.next_transaction += 1;
            let transaction = 1000 + self.next_transaction;
            self.place_movable(first_slot + 2 * unit, 2, data_unit(transaction));
        }
    }

    fn scan_of(
        &self,
        region: CompactionRegion,
        pinned_slots: &BTreeSet<SlotNumber>,
        quarantined_slots: &BTreeSet<SlotNumber>,
    ) -> Result<RegionScan, RegionScanRefusal> {
        scan_region(
            region,
            &self.records,
            &self.mapping_entries,
            pinned_slots,
            quarantined_slots,
            |_device, slot, unit_bytes| {
                self.units
                    .get(&slot)
                    .filter(|bytes| bytes.len() == unit_bytes)
                    .cloned()
            },
        )
    }

    fn scan(
        &self,
        pinned_slots: &BTreeSet<SlotNumber>,
        quarantined_slots: &BTreeSet<SlotNumber>,
    ) -> RegionScan {
        self.scan_of(region_at(SECOND_SEGMENT), pinned_slots, quarantined_slots)
            .expect("没有跨 R 边界的记录")
    }

    /// 这一批发布之后的样子（模拟，不经发布路径）：每个搬走的 key 的映射条目改指 `destination_slots` 里依次取的槽（R 外），
    /// 源落点两盘的记录改成已释放。
    fn relocate(
        &mut self,
        batch: &[PlannedRelocation],
        destination_slots: &mut impl Iterator<Item = u64>,
    ) {
        for relocation in batch {
            let destination = destination_slots.next().expect("R 外的目的槽够用");
            let entry = self
                .mapping_entries
                .iter_mut()
                .find(|entry| entry.key == relocation.mapping_key)
                .expect("计划里的 key 都在映射里");
            entry.locations = locations_at(SlotNumber(destination));
            for record in self
                .records
                .iter_mut()
                .filter(|record| record.slot == relocation.source.slot)
            {
                record.is_released = true;
            }
        }
    }
}

/// 照 `choose_region` 的约定选到底之后的样子：最后的答案、依次被要求先扫的段、这一次挂载的不合格段。
struct ChoiceAfterScanning {
    choice: RegionChoice,
    segments_scanned_on_request: Vec<SlotNumber>,
    refused_segments: SegmentsRefusedByTheRegionScan,
}

/// 照 `choose_region` 的约定选到底：答「先扫」就扫那一段（扫成了并进扫描结果，被拒了记进不合格段），再选一次。
/// 每一轮要么交回最后的答案，要么多一段有结论；被要求先扫的段若是扫过或被拒过的，就是约定被破坏了（不收敛），当场判红。
fn choose_after_scanning(
    pool: &PoolUnderTest,
    cluster_segments: &BTreeSet<SlotNumber>,
    live_intent_region: Option<CompactionRegion>,
    pinned_slots: &BTreeSet<SlotNumber>,
) -> ChoiceAfterScanning {
    let mut region_scans: Vec<RegionScan> = Vec::new();
    let mut refused_segments = SegmentsRefusedByTheRegionScan::default();
    let mut segments_scanned_on_request = Vec::new();
    // 迭代上界：每答一次「先扫」就多一段有结论，三段至多三次，再加最后那一次答案。
    for _round in 0..=SEGMENTS_IN_THE_UNIT_AREA {
        let choice = choose_region(
            &pool.records,
            unit_area_of_three_segments(),
            cluster_segments,
            live_intent_region,
            pinned_slots,
            &region_scans,
            &refused_segments,
            RegionChoiceCriterion::Greedy,
        );
        let RegionChoice::ScanBeforeChoosing(region) = choice else {
            return ChoiceAfterScanning {
                choice,
                segments_scanned_on_request,
                refused_segments,
            };
        };
        assert!(
            !segments_scanned_on_request.contains(&region.start_slot()),
            "段 {} 已经扫过或被扫描拒过，又被要求先扫：选段不收敛",
            region.start_slot().0
        );
        segments_scanned_on_request.push(region.start_slot());
        match pool.scan_of(region, pinned_slots, &BTreeSet::new()) {
            Ok(scan) => region_scans.push(scan),
            Err(refusal) => refused_segments.note_refusal(region, refusal),
        }
    }
    panic!(
        "{} 段的单元区选了 {} 轮还在要求先扫",
        SEGMENTS_IN_THE_UNIT_AREA,
        SEGMENTS_IN_THE_UNIT_AREA + 1
    );
}

/// 锚点 1 的池：段 50176 里 30 个可搬的两槽单元（活槽 60）、段 50240 里 5 个（活槽 10）、段 50304 里只有一条已释放的（活槽 0）。
fn pool_with_live_slots_sixty_ten_and_zero() -> PoolUnderTest {
    let mut pool = PoolUnderTest::default();
    pool.place_movable_data_units(FIRST_SEGMENT, 30);
    pool.place_movable_data_units(SECOND_SEGMENT, 5);
    pool.place_released(THIRD_SEGMENT, 2);
    pool
}

/// 锚点 1：活槽 [60, 10, 0]，段 50304 分最低、先扫，扫出来一个能搬的都没有，不选；选活槽最少而有能搬单元的段 50240。
#[test]
fn greedy_chooses_the_segment_with_the_fewest_live_slots_but_not_an_empty_one() {
    assert_eq!(greedy_score(10), 10, "greedy 的分就是活槽数");
    let after = choose_after_scanning(
        &pool_with_live_slots_sixty_ten_and_zero(),
        &BTreeSet::new(),
        None,
        &BTreeSet::new(),
    );
    assert_eq!(
        after.choice,
        RegionChoice::Chosen(region_at(SECOND_SEGMENT))
    );
    assert_eq!(
        after.segments_scanned_on_request,
        vec![SlotNumber(THIRD_SEGMENT), SlotNumber(SECOND_SEGMENT)],
        "按分先扫全空的段 50304，再扫段 50240"
    );
}

/// 锚点 2：段 50240 里有一个钉住的槽 ⇒ 它没资格，改选段 50176。
#[test]
fn a_segment_holding_a_pinned_slot_is_not_chosen() {
    let after = choose_after_scanning(
        &pool_with_live_slots_sixty_ten_and_zero(),
        &BTreeSet::new(),
        None,
        &BTreeSet::from([SlotNumber(SECOND_SEGMENT + 20)]),
    );
    assert_eq!(after.choice, RegionChoice::Chosen(region_at(FIRST_SEGMENT)));
}

/// 锚点 3：段 50240 是这次挂载开过的聚簇段 ⇒ 改选段 50176；这两段都开过、段 50304 没有能搬的 ⇒ 一段都没资格。
#[test]
fn a_segment_opened_in_this_mount_is_not_chosen_and_none_is_left_when_every_segment_is_excluded() {
    let pool = pool_with_live_slots_sixty_ten_and_zero();
    assert_eq!(
        choose_after_scanning(
            &pool,
            &BTreeSet::from([SlotNumber(SECOND_SEGMENT)]),
            None,
            &BTreeSet::new(),
        )
        .choice,
        RegionChoice::Chosen(region_at(FIRST_SEGMENT))
    );
    assert_eq!(
        choose_after_scanning(
            &pool,
            &BTreeSet::from([SlotNumber(FIRST_SEGMENT), SlotNumber(SECOND_SEGMENT)]),
            None,
            &BTreeSet::new(),
        )
        .choice,
        RegionChoice::NoEligibleSegment
    );
}

/// 段 50240 就是活意图的 R ⇒ 改选段 50176（判决 G2「不在活 R」）。
#[test]
fn the_live_intent_region_is_not_chosen_again() {
    let after = choose_after_scanning(
        &pool_with_live_slots_sixty_ten_and_zero(),
        &BTreeSet::new(),
        Some(region_at(SECOND_SEGMENT)),
        &BTreeSet::new(),
    );
    assert_eq!(after.choice, RegionChoice::Chosen(region_at(FIRST_SEGMENT)));
}

/// 段 50176 与段 50304 活槽都是 4：同分取起点槽号最小的段 50176；段 50240 有 6 个活槽、分更高。
#[test]
fn a_tie_goes_to_the_segment_with_the_lowest_start_slot() {
    let mut pool = PoolUnderTest::default();
    pool.place_movable_data_units(FIRST_SEGMENT, 2);
    pool.place_movable_data_units(SECOND_SEGMENT, 3);
    pool.place_movable_data_units(THIRD_SEGMENT, 2);
    assert_eq!(
        choose_after_scanning(&pool, &BTreeSet::new(), None, &BTreeSet::new()).choice,
        RegionChoice::Chosen(region_at(FIRST_SEGMENT))
    );
}

/// 资格是「R 里至少一个能搬的单元」（判决 G2，攻方 K-b）：段 50240 活槽最少（4），可那两个单元一个已死（映射里没人指着、
/// 头里的 key 映射里也没有）、一个不一致（指着它的映射条目是别的 key），一个能搬的都没有 ⇒ 不选；选有能搬单元的段 50176。
#[test]
fn a_segment_whose_live_slots_are_all_dead_or_inconsistent_units_is_not_chosen() {
    let mut pool = PoolUnderTest::default();
    pool.place_movable_data_units(FIRST_SEGMENT, 5);
    let (dead, _dead_key) = data_unit(11);
    pool.place(SECOND_SEGMENT, 2, dead);
    let (mismatched, _mismatched_key) = data_unit(12);
    let (_other, other_key) = data_unit(13);
    pool.place(SECOND_SEGMENT + 2, 2, mismatched);
    pool.map(&other_key, SECOND_SEGMENT + 2);

    let after = choose_after_scanning(&pool, &BTreeSet::new(), None, &BTreeSet::new());

    assert_eq!(after.choice, RegionChoice::Chosen(region_at(FIRST_SEGMENT)));
    assert_eq!(
        after.segments_scanned_on_request,
        vec![
            SlotNumber(THIRD_SEGMENT),
            SlotNumber(SECOND_SEGMENT),
            SlotNumber(FIRST_SEGMENT)
        ],
        "段 50304（0）、段 50240（4）扫过都没有能搬的，才轮到段 50176（10）"
    );
}

/// 盘上一条坏记录从段 50176 的末槽跨进段 50240（起点 50239、跨 2）：两段的区域扫描都整次拒。段 50240 分最低（活槽 5）、先扫、被拒，
/// 记进这次挂载的不合格段、不再选；接着扫段 50304（活槽 10）、选它；段 50176（活槽 61）轮不到（判决 m4-batch2-code-r1 S4）。
#[test]
fn a_segment_the_region_scan_refuses_over_a_record_crossing_segments_is_not_chosen_and_another_is()
{
    let mut pool = PoolUnderTest::default();
    pool.place_movable_data_units(FIRST_SEGMENT, 30);
    pool.place_record_without_a_unit(SECOND_SEGMENT - 1, 2);
    pool.place_movable_data_units(SECOND_SEGMENT + 2, 2);
    pool.place_movable_data_units(THIRD_SEGMENT, 5);

    let after = choose_after_scanning(&pool, &BTreeSet::new(), None, &BTreeSet::new());

    assert_eq!(after.choice, RegionChoice::Chosen(region_at(THIRD_SEGMENT)));
    assert_eq!(
        after.segments_scanned_on_request,
        vec![SlotNumber(SECOND_SEGMENT), SlotNumber(THIRD_SEGMENT)]
    );
    assert!(
        after.refused_segments.contains(SlotNumber(SECOND_SEGMENT)),
        "被扫描拒的段 50240 记进不合格段"
    );
    assert!(
        !after.refused_segments.contains(SlotNumber(FIRST_SEGMENT)),
        "段 50176 没扫过，不记"
    );
}

/// 同一条跨段的坏记录，而段 50304 全空：段 50304 扫过没有能搬的，段 50240、段 50176 先后被扫描拒、记进不合格段 ⇒ 报没有合格段，
/// 不在被拒的段上打转（判决 m4-batch2-code-r1 S4，攻方 T5 的卡死）。
#[test]
fn segments_the_region_scan_refuses_leave_no_eligible_segment_instead_of_being_chosen_again() {
    let mut pool = PoolUnderTest::default();
    pool.place_movable_data_units(FIRST_SEGMENT, 30);
    pool.place_record_without_a_unit(SECOND_SEGMENT - 1, 2);
    pool.place_movable_data_units(SECOND_SEGMENT + 2, 2);

    let after = choose_after_scanning(&pool, &BTreeSet::new(), None, &BTreeSet::new());

    assert_eq!(after.choice, RegionChoice::NoEligibleSegment);
    assert_eq!(
        after.segments_scanned_on_request,
        vec![
            SlotNumber(THIRD_SEGMENT),
            SlotNumber(SECOND_SEGMENT),
            SlotNumber(FIRST_SEGMENT)
        ]
    );
    assert!(after.refused_segments.contains(SlotNumber(SECOND_SEGMENT)));
    assert!(after.refused_segments.contains(SlotNumber(FIRST_SEGMENT)));
    assert!(
        !after.refused_segments.contains(SlotNumber(THIRD_SEGMENT)),
        "段 50304 扫成了，只是没有能搬的，不算被拒"
    );
}

fn data_unit(transaction: u64) -> (Vec<u8>, Vec<u8>) {
    let write_order = WriteOrder {
        instance: WRITING_INSTANCE,
        transaction,
    };
    let bytes = build_data_unit(
        DataUnitIdentity {
            tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            object: 1,
            object_birth: CheckpointTxg(3),
            anchor_offset: 0,
        },
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER,
        write_order,
        &[0x5A; 100],
    );
    let key = mapping_key_for_data(
        PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            birth_txg: BIRTH_TXG,
        },
        write_order,
    );
    (bytes, key)
}

fn node_pointer(tree: u64, birth_sequence: u32) -> NodePointer {
    NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(tree),
            birth_txg: BIRTH_TXG,
        },
        locations: NodePointer::empty_root().locations,
        instance: WRITING_INSTANCE,
        birth_sequence: BirthSequence(birth_sequence),
    }
}

fn index_node(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
    let bytes = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_EXTENT),
        0,
        24,
        &[0u8; 24],
        &[0u8; 24],
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER,
        WRITING_INSTANCE,
        BirthSequence(birth_sequence),
        112,
        &[],
    );
    let key = mapping_key_for_node(
        UNIT_CLASS_INDEX_NODE,
        node_pointer(TREE_IDENTIFIER_EXTENT, birth_sequence),
    );
    (bytes, key)
}

fn packed_unit(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
    let bytes = build_packed_unit(
        PackedIdentity {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
            record_type: PACKED_TYPE_INODE,
            container: 1,
            container_birth: BIRTH_TXG,
        },
        140,
        &[vec![1u8; 140]],
        BIRTH_TXG,
        &FILESYSTEM_IDENTIFIER,
        WriteOrder {
            instance: WRITING_INSTANCE,
            transaction: 9,
        },
        BirthSequence(birth_sequence),
    );
    let key = mapping_key_for_node(
        UNIT_CLASS_PACKED,
        node_pointer(TREE_IDENTIFIER_INODE, birth_sequence),
    );
    (bytes, key)
}

fn three_units() -> CompactionBatchUnits {
    CompactionBatchUnits::new(3).expect("3 不是 0")
}

/// 计划交回要搬的那几项；不是「要搬」时判红。
fn relocations_of(planned: PlannedBatch) -> Vec<PlannedRelocation> {
    match planned {
        PlannedBatch::Relocate(relocations) => relocations,
        PlannedBatch::RegionEmptied | PlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
            panic!("R 里还有可搬的，计划却没交要搬的：{planned:?}")
        }
    }
}

/// 锚点 4（只钉条数与 key 集合）：K = 3、R 里三个可搬的两槽数据单元 ⇒ 一批恰好三条，key 集合就是那三把，源落点是它们的落点。
#[test]
fn a_batch_of_three_over_three_movable_data_units_carries_exactly_their_keys() {
    let mut pool = PoolUnderTest::default();
    let mut expected_keys = BTreeSet::new();
    for unit in 0..3 {
        expected_keys.insert(pool.place_movable(SECOND_SEGMENT + 2 * unit, 2, data_unit(unit + 1)));
    }

    let batch = relocations_of(plan_batch(
        &pool.scan(&BTreeSet::new(), &BTreeSet::new()),
        three_units(),
    ));

    assert_eq!(batch.len(), 3, "一批三条");
    assert_eq!(
        batch
            .iter()
            .map(|relocation| relocation.mapping_key.clone())
            .collect::<BTreeSet<_>>(),
        expected_keys
    );
    assert_eq!(
        batch
            .iter()
            .map(|relocation| relocation.source)
            .collect::<Vec<_>>(),
        [0, 2, 4].map(|offset| Placement {
            slot: SlotNumber(SECOND_SEGMENT + offset),
            span: 2,
        }),
        "源落点按槽升序"
    );
    assert_eq!(CompactionBatchUnits::new(0), None, "一批 0 个造不出来");
}

/// 守恒：R 里 7 个可搬（码 1 × 4、码 2 × 2、码 3 × 1）与钉住、隔离、已死、不一致各一个，K = 3 一批批搬：
/// 批大小 3、3、1，每批 ≤ K、批间不交，并集 = 7 把可搬的 key；搬完之后再计划是「搬不动」，剩下的四类原样还在。
#[test]
fn batches_never_overlap_and_their_union_is_every_movable_key_of_the_region() {
    let mut pool = PoolUnderTest::default();
    let mut movable_keys = BTreeSet::new();
    for unit in 0..4 {
        movable_keys.insert(pool.place_movable(SECOND_SEGMENT + 2 * unit, 2, data_unit(unit + 1)));
    }
    movable_keys.insert(pool.place_movable(SECOND_SEGMENT + 8, 1, index_node(0)));
    movable_keys.insert(pool.place_movable(SECOND_SEGMENT + 9, 1, index_node(1)));
    movable_keys.insert(pool.place_movable(SECOND_SEGMENT + 10, 2, packed_unit(0)));
    let (pinned, _pinned_key) = index_node(2);
    pool.place(SECOND_SEGMENT + 12, 1, pinned);
    let (quarantined, _quarantined_key) = data_unit(10);
    pool.place(SECOND_SEGMENT + 14, 2, quarantined);
    let (dead, _dead_key) = data_unit(11);
    pool.place(SECOND_SEGMENT + 16, 2, dead);
    let (mismatched, _mismatched_key) = data_unit(12);
    let (_other, other_key) = data_unit(13);
    pool.place(SECOND_SEGMENT + 18, 2, mismatched);
    pool.map(&other_key, SECOND_SEGMENT + 18);
    let pinned_slots = BTreeSet::from([SlotNumber(SECOND_SEGMENT + 12)]);
    let quarantined_slots = BTreeSet::from([SlotNumber(SECOND_SEGMENT + 14)]);
    let allocations_left_in_the_region = pool
        .scan(&pinned_slots, &quarantined_slots)
        .allocations
        .len()
        - movable_keys.len();
    let mut destination_slots = (0..).map(|index| THIRD_SEGMENT + 2 * index);

    let mut planned_keys = BTreeSet::new();
    let mut batch_sizes = Vec::new();
    let mut last_plan = None;
    // 迭代上界：7 个可搬、每批至少一个 ⇒ 至多 7 批，再加最后那一次不是「要搬」的计划。
    for _batch_index in 0..8 {
        let batch = match plan_batch(&pool.scan(&pinned_slots, &quarantined_slots), three_units()) {
            PlannedBatch::Relocate(batch) => batch,
            not_a_relocation @ (PlannedBatch::RegionEmptied
            | PlannedBatch::NothingMovableButTheRegionIsNotEmpty) => {
                last_plan = Some(not_a_relocation);
                break;
            }
        };
        assert!(
            (1..=3).contains(&batch.len()),
            "每批 1 到 K = 3 个，这一批 {}",
            batch.len()
        );
        for relocation in &batch {
            assert!(
                planned_keys.insert(relocation.mapping_key.clone()),
                "批间不交：一把 key 只进一批"
            );
        }
        batch_sizes.push(batch.len());
        pool.relocate(&batch, &mut destination_slots);
    }

    assert_eq!(batch_sizes, vec![3, 3, 1], "7 个可搬、K = 3");
    assert_eq!(planned_keys, movable_keys, "并集 = R 里可搬的 key 集合");
    assert_eq!(
        last_plan,
        Some(PlannedBatch::NothingMovableButTheRegionIsNotEmpty),
        "搬完可搬的，R 里还剩搬不动的四个：搬不动，不是搬空"
    );
    let final_scan = pool.scan(&pinned_slots, &quarantined_slots);
    assert_eq!(
        final_scan.movable_units().count(),
        0,
        "搬完之后 R 里没有可搬的"
    );
    assert_eq!(
        final_scan.allocations.len(),
        allocations_left_in_the_region,
        "钉住、隔离、已死、不一致四个原样还在 R 里"
    );
    assert_eq!(allocations_left_in_the_region, 4);
}

/// R 里两个可搬的、别的什么都没有，K = 3：第一批搬两个，搬完 R 里一条未释放的记录都没有，再计划是「搬空」（判决 m4-batch2-code-r1 S4）。
#[test]
fn planning_after_the_last_movable_unit_left_reports_the_region_emptied() {
    let mut pool = PoolUnderTest::default();
    pool.place_movable_data_units(SECOND_SEGMENT, 2);
    let mut destination_slots = (0..).map(|index| THIRD_SEGMENT + 2 * index);

    let first_batch = relocations_of(plan_batch(
        &pool.scan(&BTreeSet::new(), &BTreeSet::new()),
        three_units(),
    ));
    assert_eq!(first_batch.len(), 2);
    pool.relocate(&first_batch, &mut destination_slots);

    assert_eq!(
        plan_batch(
            &pool.scan(&BTreeSet::new(), &BTreeSet::new()),
            three_units()
        ),
        PlannedBatch::RegionEmptied
    );
}

/// R 里只有一个已死的单元（映射里没人指着、头里的 key 映射里也没有）：一个能搬的都没有而 R 不空，计划是「搬不动」，不是「搬空」
/// （判决 m4-batch2-code-r1 S4）。
#[test]
fn planning_over_a_region_holding_only_a_dead_unit_reports_nothing_movable_not_emptied() {
    let mut pool = PoolUnderTest::default();
    let (dead, _dead_key) = data_unit(11);
    pool.place(SECOND_SEGMENT, 2, dead);

    assert_eq!(
        plan_batch(
            &pool.scan(&BTreeSet::new(), &BTreeSet::new()),
            three_units()
        ),
        PlannedBatch::NothingMovableButTheRegionIsNotEmpty
    );
}

/// 丁 1 交判第 4 点：一个槽既钉住又在隔离集里，区域扫描照旧判「钉住」（次序钉住 → 隔离），丢意图那一判直接查隔离集，照样看得到它。
#[test]
fn a_slot_both_pinned_and_quarantined_is_scanned_as_pinned_and_still_drops_the_intent() {
    let mut pool = PoolUnderTest::default();
    pool.place_movable(SECOND_SEGMENT, 1, index_node(3));
    let both = BTreeSet::from([SlotNumber(SECOND_SEGMENT)]);
    let scan = pool.scan(&both, &both);
    assert_eq!(
        scan.allocations[0].class,
        AllocationInRegion::Pinned,
        "扫描判钉住（钉住排在隔离之前）"
    );
    assert!(
        region_holds_a_quarantined_slot(region_at(SECOND_SEGMENT), &both),
        "丢意图那一判直接查隔离集：两者同时成立也丢"
    );
}

/// 签发前判（攻方 K-g）：空洞只数 R 外、装着用户数据的段里没挡的槽；只装提交内生块的段、R 自己都不数。空洞少于要搬的单元数不签。
#[test]
fn the_issue_counts_holes_only_outside_the_region_in_segments_holding_user_data() {
    let mut records: Vec<AllocationRecord> = Vec::new();
    // 段 50176：一个数据单元（50176、50177），其余 62 槽空着。
    records.extend(records_on_both_devices(FIRST_SEGMENT, 2));
    // 段 50240（R）：一个数据单元、一个索引节点。
    records.extend(records_on_both_devices(SECOND_SEGMENT, 2));
    records.extend(records_on_both_devices(SECOND_SEGMENT + 10, 1));
    // 段 50304：只装一个提交内生块（索引节点），其余 63 槽空着。
    records.extend(records_on_both_devices(THIRD_SEGMENT, 1));
    let slots_holding_user_data: BTreeSet<SlotNumber> = [
        FIRST_SEGMENT,
        FIRST_SEGMENT + 1,
        SECOND_SEGMENT,
        SECOND_SEGMENT + 1,
    ]
    .map(SlotNumber)
    .into_iter()
    .collect();
    let occupied: BTreeSet<u64> = [
        FIRST_SEGMENT,
        FIRST_SEGMENT + 1,
        SECOND_SEGMENT,
        SECOND_SEGMENT + 1,
        SECOND_SEGMENT + 10,
        THIRD_SEGMENT,
    ]
    .into_iter()
    .collect();
    let holes = holes_outside_the_region_in_segments_holding_user_data(
        &records,
        &slots_holding_user_data,
        unit_area_of_three_segments(),
        region_at(SECOND_SEGMENT),
        |slot| !occupied.contains(&slot.0),
    );
    assert_eq!(
        holes, 62,
        "只有段 50176 的 62 个空槽：R 不数，只装提交内生块的段 50304 不数"
    );
    assert!(holes_admit_the_issue(holes, 62), "空洞 = 要搬的单元数：签");
    assert!(
        !holes_admit_the_issue(holes, 63),
        "空洞少于要搬的单元数：不签"
    );
}

/// 停机谓词（判决 G4）：R 里还有一条起点在 R 内的未释放记录就不停；只剩已释放的就停，R 外的记录不管。
#[test]
fn the_stop_predicate_holds_only_when_no_unreleased_record_starts_in_the_region() {
    let mut records: Vec<AllocationRecord> = Vec::new();
    records.extend(records_on_both_devices(FIRST_SEGMENT, 2));
    records.extend(records_on_both_devices(SECOND_SEGMENT + 10, 1));
    assert!(
        !region_is_empty_by_the_records(&records, region_at(SECOND_SEGMENT)),
        "R 里还有一条未释放的"
    );
    for record in records
        .iter_mut()
        .filter(|record| record.slot == SlotNumber(SECOND_SEGMENT + 10))
    {
        record.is_released = true;
    }
    assert!(
        region_is_empty_by_the_records(&records, region_at(SECOND_SEGMENT)),
        "R 里只剩已释放的：按记录全空"
    );
}

/// 全空聚簇段数按记录口径（D3（空间分配） 已定项 10 ①，C372）：只有已释放记录的段算全空，有一条未释放记录的段不算。
#[test]
fn empty_cluster_segments_are_counted_by_the_records() {
    let mut records: Vec<AllocationRecord> = Vec::new();
    records.extend(records_on_both_devices(FIRST_SEGMENT, 2));
    for mut released in records_on_both_devices(SECOND_SEGMENT, 2) {
        released.is_released = true;
        records.push(released);
    }
    assert_eq!(
        empty_cluster_segments_by_the_records(&records, unit_area_of_three_segments()),
        2,
        "段 50240 只有已释放的记录、段 50304 没有记录：两段全空"
    );
}

/// 一条意图完成而全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为假就不签下一条（攻方 K-h）。
#[test]
fn no_next_intent_is_issued_after_one_that_did_not_add_an_empty_segment() {
    assert!(the_completed_intent_yielded(5, 4), "多了一段：签下一条");
    assert!(!the_completed_intent_yielded(4, 4), "没多：不签");
    assert!(!the_completed_intent_yielded(3, 4), "少了：不签");
}

/// 一次整理运行的发布预算（判决 G8 `B_gc`）：`3S − 4 − 已经过去的发布数`，至少 1。
#[test]
fn the_publish_budget_of_a_run_is_three_s_minus_four_minus_the_publishes_passed_and_at_least_one() {
    assert_eq!(publish_budget_of_a_compaction_run(8, 0), 20);
    assert_eq!(publish_budget_of_a_compaction_run(8, 5), 15);
    assert_eq!(
        publish_budget_of_a_compaction_run(8, 25),
        1,
        "减到 0 以下取 1"
    );
}

/// 只装着两块盘、三个整段单元区、没有活意图之外的东西的分配器，活意图的 R 取段 `region_start`。
fn allocator_with_a_live_intent_on(
    region_start: u64,
    exclusion: CompactionRegionExclusion,
) -> PoolAllocator {
    let device_bytes = (FIRST_SEGMENT + 3 * 64) * 16384;
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DEVICE_ZERO, device_bytes),
        DeviceFreeMap::new(DEVICE_ONE, device_bytes),
    ]);
    let product = CompactionSwitches::PRODUCT;
    allocator.set_compaction_switches(CompactionSwitches {
        batch_size: CompactionBatchSize::NotDecidedByTheClauses,
        forced_region: product.forced_region,
        run_forcing: product.run_forcing,
        region_exclusion: exclusion,
        destination_policy: product.destination_policy,
        release_checksum_mismatch_after_issue: product.release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: product.empty_cluster_segments_floor,
        relocation_publish_order: product.relocation_publish_order,
    });
    allocator.compaction_mut().issue(CompactionIntent {
        region: region_at(region_start),
        allocation_generation_at_issue: BIRTH_TXG,
        empty_cluster_segments_at_issue: 3,
    });
    allocator
}

/// R 排他（D26 已定项 1 第 3 条、D3 已定项 7 登记表第 5 行）：活意图的 R 是最低的那一段时，用户数据落到 R 之外的最低空槽对。
#[test]
fn user_data_does_not_land_in_the_live_intent_region_while_it_is_excluded() {
    let mut allocator = allocator_with_a_live_intent_on(
        FIRST_SEGMENT,
        CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
    );
    let placement = allocator
        .try_allocate_user_data(BIRTH_TXG)
        .expect("R 外有空槽对");
    assert_eq!(
        placement.slot,
        SlotNumber(SECOND_SEGMENT),
        "跳过 R，落在下一段的起点"
    );
}

/// 只供测试的开关关掉 R 排他（判决 G6）：用户数据照最低空槽对落进 R。
#[test]
fn user_data_lands_in_the_live_intent_region_when_the_exclusion_is_switched_off_for_tests() {
    let mut allocator = allocator_with_a_live_intent_on(
        FIRST_SEGMENT,
        CompactionRegionExclusion::SwitchedOffForTests,
    );
    let placement = allocator
        .try_allocate_user_data(BIRTH_TXG)
        .expect("有空槽对");
    assert_eq!(
        placement.slot,
        SlotNumber(FIRST_SEGMENT),
        "R 不排他：最低的空槽对就在 R 里"
    );
}

/// R 排他：提交内生块开新聚簇段时不开 R 那一段，开 R 之外最低的全空段。
#[test]
fn a_new_cluster_segment_is_not_opened_in_the_live_intent_region() {
    let mut allocator = allocator_with_a_live_intent_on(
        FIRST_SEGMENT,
        CompactionRegionExclusion::ExcludedWhileTheIntentIsLive,
    );
    let placement = allocator
        .try_allocate_commit_generated(UnitFootprint::OneSlot, BIRTH_TXG)
        .expect("R 外有全空段");
    assert_eq!(
        placement.slot,
        SlotNumber(SECOND_SEGMENT),
        "开的是 R 之外最低的全空段"
    );
    assert!(
        !allocator
            .cluster_segments()
            .contains(&SlotNumber(FIRST_SEGMENT)),
        "R 那一段没被开成聚簇段"
    );
}
