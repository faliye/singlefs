use super::*;

// ───────────── 第 4 次重跑：写法与拆分段的硬件 CRC、九种写法的取值与系数 ─────────────

fn hardware_crc_of_this_machine() -> HardwareCrc {
    HardwareCrc::when_the_instructions_are_present(&unit_lengths_of_the_segment())
        .expect("这台机器有 sse4.2（登记第四节第 6 行：两台都有）")
}

#[test]
fn single_lane_hardware_crc_gives_the_check_value_of_the_nine_digits() {
    let hardware = hardware_crc_of_this_machine();
    assert_eq!(
        hardware.crc(HardwareCrcLanes::Single, CHECK_STRING),
        Some(0xE306_9283)
    );
    assert_eq!(
        hardware.crc(HardwareCrcLanes::ThreeWay, CHECK_STRING),
        Some(0xE306_9283)
    );
}

#[test]
fn three_way_hardware_crc_hits_the_modulus_anchors_of_both_unit_lengths() {
    let hardware = hardware_crc_of_this_machine();
    assert_eq!(
        hardware.crc(HardwareCrcLanes::ThreeWay, &modulus_bytes(32_768)),
        Some(0x2AD6_0827)
    );
    assert_eq!(
        hardware.crc(HardwareCrcLanes::ThreeWay, &modulus_bytes(16_384)),
        Some(0xEAFC_A51D)
    );
    assert_eq!(
        hardware.crc(HardwareCrcLanes::Single, &modulus_bytes(32_768)),
        Some(0x2AD6_0827)
    );
    assert_eq!(
        MODULUS_ANCHORS,
        [(32_768, 0x2AD6_0827), (16_384, 0xEAFC_A51D)]
    );
}

#[test]
fn three_way_lanes_are_multiples_of_eight_with_the_rest_on_the_third() {
    assert_eq!(three_way_lanes(32_768), [10_920, 10_920, 10_928]);
    assert_eq!(three_way_lanes(16_384), [5_456, 5_456, 5_472]);
    assert_eq!(three_way_lanes(9), [0, 0, 9]);
}

#[test]
fn both_hardware_writings_agree_with_the_crates_on_random_lengths_up_to_sixty_four_bytes() {
    let hardware = hardware_crc_of_this_machine();
    let strings = random_length_strings();
    assert_eq!(strings.len(), RANDOM_LENGTH_CHECKS);
    assert!(
        strings
            .iter()
            .any(|bytes| bytes.len() % BYTES_PER_HARDWARE_CRC_STEP != 0),
        "有长度不是 8 的倍数的串，尾巴那一圈跑得到"
    );
    assert_eq!(hardware_crc_check_failures(&hardware), Vec::new());
    assert_eq!(hardware_crc_mismatches(&hardware, &strings), 0);
}

#[test]
fn zero_byte_shift_matches_feeding_zero_bytes_through_the_register() {
    let state = 0x1234_5678_u32;
    let shifted = ZeroByteShift::over_zero_bytes(10_920).applied_to(state);
    // 软件那一圈：反射 CRC 每吃一个零字节，寄存器照多项式右移 8 位
    let mut register = state;
    for _byte in 0..10_920 {
        for _bit in 0..8 {
            register = if register & 1 == 1 {
                (register >> 1) ^ CASTAGNOLI_REFLECTED_POLYNOMIAL
            } else {
                register >> 1
            };
        }
    }
    assert_eq!(shifted, register);
}

#[test]
fn release_check_crc_pieces_read_the_release_check_copies_not_the_scan_units() {
    let hardware = hardware_crc_of_this_machine();
    let units = vec![modulus_bytes(32_768)];
    let copies = vec![modulus_bytes(16_384); 3];
    let pieces = hardware_crc_pieces(&hardware, &units, &copies);
    assert_eq!(pieces.crc.map(|(_cpu, bytes)| bytes), [32_768, 32_768]);
    assert_eq!(
        pieces.release_check_crc.map(|(_cpu, bytes)| bytes),
        [49_152, 49_152]
    );
    assert!(pieces
        .crc
        .iter()
        .chain(pieces.release_check_crc.iter())
        .all(|(cpu, _bytes)| cpu.is_some()));
}

#[test]
fn hardware_crc_writing_keeps_the_scan_increment_of_today_minus_the_crc_of_today() {
    let pieces = synthetic_cpu_writing_pieces();
    assert_eq!(
        pieces.stage_of_the_writing(CpuWriting::Today, WritingStage::ScanIncrement),
        Some(6.0)
    );
    assert_eq!(
        pieces.stage_of_the_writing(CpuWriting::HardwareCrcSingle, WritingStage::ScanIncrement),
        Some(6.0),
        "乙段增量里那一份 CRC 是 parse_data_unit 调的 crates CRC，乙不改它"
    );
    assert_eq!(
        pieces.stage_of_the_writing(CpuWriting::HardwareCrcSingle, WritingStage::Crc),
        Some(1.0)
    );
    assert_eq!(
        pieces.stage_of_the_writing(
            CpuWriting::HardwareCrcThreeWay,
            WritingStage::ReleaseCheckCrc
        ),
        Some(0.3)
    );
}

#[test]
fn identity_index_writing_counts_the_build_only_at_the_with_build_end() {
    let pieces = synthetic_cpu_writing_pieces();
    let with_build = pieces
        .stage_of_the_writing(
            CpuWriting::IdentityIndexWithBuild,
            WritingStage::ScanIncrement,
        )
        .expect("读得到");
    let without_build = pieces
        .stage_of_the_writing(
            CpuWriting::IdentityIndexWithoutBuild,
            WritingStage::ScanIncrement,
        )
        .expect("读得到");
    assert_eq!(
        with_build + 4.0,
        1.125,
        "计入：查 0.125 加建 1，减今天的 CRC 4"
    );
    assert_eq!(without_build + 4.0, 0.125, "不计：只有查的 0.125");
    assert_eq!(
        pieces.stage_of_the_writing(CpuWriting::BothThreeWayWithBuild, WritingStage::Crc),
        Some(0.5)
    );
    assert_eq!(
        pieces.stage_of_the_writing(CpuWriting::IdentityIndexWithBuild, WritingStage::Crc),
        Some(4.0)
    );
    let mut unreadable = pieces;
    unreadable.identity_index_build = None;
    assert_eq!(
        unreadable
            .stage_of_the_writing(CpuWriting::BothSingleWithBuild, WritingStage::ScanIncrement),
        None,
        "读不到不当 0"
    );
}

#[test]
fn writing_factor_takes_the_median_of_the_rounds_not_the_mean() {
    assert_eq!(
        writing_factor_of_the_rounds(&[Some(1.0), Some(1.0), Some(1.0), Some(1.0), Some(5.0)]),
        Some(1.0)
    );
    assert_eq!(
        writing_factor_of_the_rounds(&[None, Some(2.0), None]),
        Some(2.0)
    );
    assert_eq!(writing_factor_of_the_rounds(&[None, None]), None);
}

#[test]
fn writing_factor_sums_the_crc_over_issue_and_batches_and_the_release_check_over_batches_only() {
    let mut issue = synthetic_cpu_writing_pieces();
    issue.release_check_crc_today = Some(100.0);
    issue.release_check_crc_hardware_single = Some(100.0);
    let batch = synthetic_cpu_writing_pieces();
    let calls = vec![
        (CallKind::Issue, issue),
        (CallKind::Batch, batch),
        (CallKind::FinalBatch, batch),
    ];
    assert_eq!(
        stage_sum_of_the_intent(&calls, CpuWriting::Today, WritingStage::Crc),
        Some(12.0)
    );
    assert_eq!(
        stage_sum_of_the_intent(&calls, CpuWriting::Today, WritingStage::ReleaseCheckCrc),
        Some(4.0),
        "签发那一次的释放前读盘核不进 D 的式子"
    );
    assert_eq!(
        writing_factor_of_the_intent(&calls, CpuWriting::HardwareCrcSingle, WritingStage::Crc),
        Some(0.25)
    );
    assert_eq!(
        writing_factor_of_the_intent(
            &calls,
            CpuWriting::HardwareCrcSingle,
            WritingStage::ReleaseCheckCrc
        ),
        Some(0.3)
    );
    for stage in WritingStage::ALL {
        assert_eq!(
            writing_factor_of_the_intent(&calls, CpuWriting::Today, stage),
            Some(1.0),
            "写法甲恒为 1"
        );
    }
}

#[test]
fn one_failed_timed_piece_voids_exactly_the_writings_that_use_it() {
    assert_eq!(
        TimedPiece::TodayCrc.writings_voided(),
        CpuWriting::ALL.to_vec()
    );
    assert_eq!(
        TimedPiece::HardwareCrc(HardwareCrcLanes::ThreeWay).writings_voided(),
        vec![
            CpuWriting::HardwareCrcThreeWay,
            CpuWriting::BothThreeWayWithBuild,
            CpuWriting::BothThreeWayWithoutBuild
        ]
    );
    assert_eq!(
        TimedPiece::IdentityIndexBuild.writings_voided(),
        vec![
            CpuWriting::IdentityIndexWithBuild,
            CpuWriting::BothSingleWithBuild,
            CpuWriting::BothThreeWayWithBuild
        ]
    );
    assert_eq!(
        TimedPiece::IdentityIndexLookupScan.writings_voided().len(),
        6
    );
}

#[test]
fn cpu_writing_controls_judge_two_milliseconds_and_twice_the_work_at_the_registered_edges() {
    assert_eq!(
        cpu_writing_control_judgements(Some(1e6), Some(3e6), Some(2e6)),
        (Some(2e6), Some(2.0), true, true)
    );
    let (_increment, _doubling, spin_holds, double_holds) =
        cpu_writing_control_judgements(Some(1e6), Some(3.4e6), Some(2.5e6));
    assert!(!spin_holds && !double_holds, "2.4 ms 与 2.5 倍都在线外");
    let (_unread_increment, _unread_doubling, unread_spin_holds, unread_double_holds) =
        cpu_writing_control_judgements(None, Some(3e6), Some(2e6));
    assert!(!unread_spin_holds && !unread_double_holds, "读不到不算过");
}

#[test]
fn cpu_writings_of_a_call_stop_on_a_crc_mismatch_a_scan_mismatch_or_different_bytes() {
    let agreeing = synthetic_cpu_writings_of_a_call();
    assert_eq!(agreeing.stop_finding(3), None);
    assert!(!agreeing.is_unreadable());
    let mut crc_differs = synthetic_cpu_writings_of_a_call();
    crc_differs.hardware_crc_mismatches = 1;
    assert_eq!(
        crc_differs.stop_finding(3).map(|(clause, _finding)| clause),
        Some("K-S1")
    );
    let mut scan_differs = synthetic_cpu_writings_of_a_call();
    scan_differs.identity_index_scan_matches = Some(false);
    assert_eq!(
        scan_differs
            .stop_finding(3)
            .map(|(clause, _finding)| clause),
        Some("K-S2")
    );
    let mut bytes_differ = synthetic_cpu_writings_of_a_call();
    bytes_differ.hardware_release_check_crc_bytes = [32_768, 16_384];
    assert_eq!(
        bytes_differ
            .stop_finding(3)
            .map(|(clause, _finding)| clause),
        Some("K-S1")
    );
    let mut empty_index = synthetic_cpu_writings_of_a_call();
    empty_index.identity_index_entries = 0;
    assert!(empty_index.is_unreadable());
}

// ───────────── 第 4 次重跑：抄的区域扫描（两张索引）在手搭的池上与 crates `scan_region` 逐个结局相同（PW3、停机 K-S2） ─────────────

const SCAN_CASE_DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
const SCAN_CASE_DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);

/// 单元区里起点 50240 的那个聚簇段 [50240, 50304)（同 crates `region_scan.rs` 单测那一个）。
fn scan_case_region() -> CompactionRegion {
    CompactionRegion::of_one_cluster_segment(
        SCAN_CASE_DEVICE_ZERO,
        SlotNumber(singlefs_format::UNIT_AREA_START_SLOT + LOCAL_CLUSTER_SEGMENT_SLOTS),
    )
    .expect("50240 = 785 × 64")
}

fn scan_case_slot(offset_in_slots: u64) -> SlotNumber {
    SlotNumber(scan_case_region().start_slot().0 + offset_in_slots)
}

/// 一块手搭的池：两块盘上每个落点各一份字节、各一条记录（两盘同槽），加中央映射的条目。
#[derive(Default)]
struct ScanCasePool {
    units: BTreeMap<(DeviceIdentity, SlotNumber), Vec<u8>>,
    records: Vec<AllocationRecord>,
    mapping_entries: Vec<CentralMappingEntry>,
}

impl ScanCasePool {
    fn place(&mut self, slot: SlotNumber, span_slots: u16, unit: Option<&[u8]>) {
        for device in [SCAN_CASE_DEVICE_ZERO, SCAN_CASE_DEVICE_ONE] {
            self.records.push(AllocationRecord {
                device,
                slot,
                span_slots,
                generation: singlefs_core::address::CheckpointTxg(7),
                is_released: false,
                quarantine_mark:
                    singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
            });
            if let Some(bytes) = unit {
                self.units.insert((device, slot), bytes.to_vec());
            }
        }
    }

    pub(crate) fn map(&mut self, key: Vec<u8>, locations: [(DeviceIdentity, SlotNumber); 2]) {
        self.mapping_entries.push(CentralMappingEntry {
            key,
            locations: locations.map(|(device, slot)| singlefs_core::pointer::LocationEntry {
                device,
                slot,
                unit_checksum: 0,
            }),
        });
    }

    /// crates 那一份与抄的那一份各扫一次，交回（crates 的、抄的）。
    fn both_scans(
        &self,
        pinned_slots: &BTreeSet<SlotNumber>,
        quarantined_slots: &BTreeSet<SlotNumber>,
    ) -> (
        Result<RegionScan, singlefs_core::region_scan::RegionScanRefusal>,
        Result<RegionScan, singlefs_core::region_scan::RegionScanRefusal>,
    ) {
        let read = |device: DeviceIdentity, slot: SlotNumber, unit_bytes: usize| {
            self.units
                .get(&(device, slot))
                .filter(|bytes| bytes.len() == unit_bytes)
                .cloned()
        };
        let index = IdentityIndexOfTheMapping::of_the_entries(&self.mapping_entries);
        (
            scan_region(
                scan_case_region(),
                &self.records,
                &self.mapping_entries,
                pinned_slots,
                quarantined_slots,
                read,
            ),
            scan_region_with_the_identity_index(
                scan_case_region(),
                &self.records,
                &self.mapping_entries,
                &index,
                pinned_slots,
                quarantined_slots,
                read,
            ),
        )
    }

    /// 两份扫描整个 `==`，交回那一份的结果（不同就红在这里）。
    fn agreeing_scan(
        &self,
        pinned_slots: &BTreeSet<SlotNumber>,
        quarantined_slots: &BTreeSet<SlotNumber>,
    ) -> Result<RegionScan, singlefs_core::region_scan::RegionScanRefusal> {
        let (crates_scan, copied_scan) = self.both_scans(pinned_slots, quarantined_slots);
        assert_eq!(
            copied_scan, crates_scan,
            "抄的扫描与 crates 那一份不同（停机 K-S2）"
        );
        crates_scan
    }

    fn class_at_the_region_start(&self, pinned: &[u64], quarantined: &[u64]) -> AllocationInRegion {
        let slots = |offsets: &[u64]| {
            offsets
                .iter()
                .map(|offset| scan_case_slot(*offset))
                .collect::<BTreeSet<_>>()
        };
        let scan = self
            .agreeing_scan(&slots(pinned), &slots(quarantined))
            .expect("这几种结局都扫得成");
        scan.allocations
            .iter()
            .find(|allocation| allocation.placement.slot == scan_case_slot(0))
            .map(|allocation| allocation.class.clone())
            .expect("R 起点那一条在扫描结果里")
    }
}

/// 一个码 1 数据单元与写者给它的映射 key（同 crates `region_scan.rs` 单测的 `data_unit`）。
fn scan_case_data_unit(transaction: u64) -> (Vec<u8>, Vec<u8>) {
    let write_order = singlefs_core::unit::WriteOrder {
        instance: singlefs_core::address::InstanceGeneration(2),
        transaction,
    };
    let tree = singlefs_core::address::TreeIdentifier(singlefs_format::TREE_IDENTIFIER_EXTENT);
    let bytes = singlefs_core::unit::build_data_unit(
        singlefs_core::unit::DataUnitIdentity {
            tree,
            object: 1,
            object_birth: singlefs_core::address::CheckpointTxg(3),
            anchor_offset: 0,
        },
        singlefs_core::address::CheckpointTxg(7),
        &[7_u8; 16],
        write_order,
        &[0x5A; 100],
    );
    let key = singlefs_core::records::mapping_key_for_data(
        singlefs_core::pointer::PointerHead {
            birth_tree: tree,
            birth_txg: singlefs_core::address::CheckpointTxg(7),
        },
        write_order,
    );
    (bytes, key)
}

/// R 起点放一个数据单元（两盘），交回池与它的 key。
fn scan_case_pool_with_a_data_unit_at_the_start() -> (ScanCasePool, Vec<u8>) {
    let mut pool = ScanCasePool::default();
    let (bytes, key) = scan_case_data_unit(1);
    pool.place(scan_case_slot(0), 2, Some(&bytes));
    (pool, key)
}

fn scan_case_both_devices(offset_in_slots: u64) -> [(DeviceIdentity, SlotNumber); 2] {
    [SCAN_CASE_DEVICE_ZERO, SCAN_CASE_DEVICE_ONE]
        .map(|device| (device, scan_case_slot(offset_in_slots)))
}

#[test]
fn copied_scan_agrees_on_a_movable_unit() {
    let (mut pool, key) = scan_case_pool_with_a_data_unit_at_the_start();
    pool.map(key.clone(), scan_case_both_devices(0));
    assert!(matches!(
        pool.class_at_the_region_start(&[], &[]),
        AllocationInRegion::Movable(unit) if unit.mapping_key == key
    ));
}

#[test]
fn copied_scan_agrees_on_a_pinned_unit() {
    let (mut pool, key) = scan_case_pool_with_a_data_unit_at_the_start();
    pool.map(key, scan_case_both_devices(0));
    assert_eq!(
        pool.class_at_the_region_start(&[1], &[]),
        AllocationInRegion::Pinned
    );
}

#[test]
fn copied_scan_agrees_on_a_quarantined_unit() {
    let (mut pool, key) = scan_case_pool_with_a_data_unit_at_the_start();
    pool.map(key, scan_case_both_devices(0));
    assert_eq!(
        pool.class_at_the_region_start(&[], &[0]),
        AllocationInRegion::Quarantined
    );
}

#[test]
fn copied_scan_agrees_on_a_dead_unit() {
    let (pool, _key) = scan_case_pool_with_a_data_unit_at_the_start();
    assert_eq!(
        pool.class_at_the_region_start(&[], &[]),
        AllocationInRegion::Dead
    );
}

#[test]
fn copied_scan_agrees_on_an_unreadable_header() {
    let mut pool = ScanCasePool::default();
    pool.place(scan_case_slot(0), 2, None);
    assert_eq!(
        pool.class_at_the_region_start(&[], &[]),
        AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
            UnitHeaderUnreadable::DeviceReadFailed
        ))
    );
}

#[test]
fn copied_scan_agrees_on_a_class_tag_differing_from_the_mapping_key() {
    let (mut pool, _key) = scan_case_pool_with_a_data_unit_at_the_start();
    pool.map(
        vec![LOCAL_UNIT_CLASS_INDEX_NODE; 27],
        scan_case_both_devices(0),
    );
    assert_eq!(
        pool.class_at_the_region_start(&[], &[]),
        AllocationInRegion::Inconsistent(
            RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
                header_unit_class: MappedUnitClass::Data,
                mapping_key_class_tag: LOCAL_UNIT_CLASS_INDEX_NODE,
            }
        )
    );
}

#[test]
fn copied_scan_agrees_when_the_header_key_is_mapped_at_another_slot_only() {
    let (mut pool, key) = scan_case_pool_with_a_data_unit_at_the_start();
    pool.map(key, scan_case_both_devices(10));
    assert_eq!(
        pool.class_at_the_region_start(&[], &[]),
        AllocationInRegion::Inconsistent(
            RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation
        )
    );
}

#[test]
fn copied_scan_agrees_on_a_record_straddling_the_region_start() {
    let (mut pool, _key) = scan_case_pool_with_a_data_unit_at_the_start();
    pool.place(SlotNumber(scan_case_slot(0).0 - 1), 2, None);
    let (crates_scan, copied_scan) = pool.both_scans(&BTreeSet::new(), &BTreeSet::new());
    assert_eq!(copied_scan, crates_scan);
    assert!(matches!(
        crates_scan,
        Err(RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported { .. })
    ));
}

#[test]
fn copied_scan_agrees_on_two_records_movable_under_one_key() {
    let (mut pool, key) = scan_case_pool_with_a_data_unit_at_the_start();
    let (bytes, _same_key) = scan_case_data_unit(1);
    pool.place(scan_case_slot(2), 2, Some(&bytes));
    pool.map(
        key,
        [
            (SCAN_CASE_DEVICE_ZERO, scan_case_slot(0)),
            (SCAN_CASE_DEVICE_ZERO, scan_case_slot(2)),
        ],
    );
    let (crates_scan, copied_scan) = pool.both_scans(&BTreeSet::new(), &BTreeSet::new());
    assert_eq!(copied_scan, crates_scan);
    assert!(matches!(
        crates_scan,
        Err(RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey { .. })
    ));
}

#[test]
fn copied_scan_looks_up_the_slot_together_with_its_device() {
    let (mut pool, _key) = scan_case_pool_with_a_data_unit_at_the_start();
    let (_other_bytes, other_key) = scan_case_data_unit(2);
    pool.map(
        other_key,
        [
            (SCAN_CASE_DEVICE_ONE, scan_case_slot(0)),
            (DeviceIdentity(2), scan_case_slot(0)),
        ],
    );
    assert_eq!(
        pool.class_at_the_region_start(&[], &[]),
        AllocationInRegion::Dead,
        "同槽号、别的盘上的条目不指着 R 那块盘上的这一槽"
    );
}

#[test]
fn copied_scan_reports_the_first_disagreeing_entry_in_the_mapping_order() {
    let (mut pool, _key) = scan_case_pool_with_a_data_unit_at_the_start();
    // 条目 0 指着这一槽的是第二条位置项、条目 1 是第一条：建表时条目 1 先进去，查的时候要排回映射的次序
    pool.map(
        vec![LOCAL_UNIT_CLASS_INDEX_NODE; 27],
        [
            (SCAN_CASE_DEVICE_ONE, scan_case_slot(0)),
            (SCAN_CASE_DEVICE_ZERO, scan_case_slot(0)),
        ],
    );
    pool.map(vec![LOCAL_UNIT_CLASS_PACKED; 27], scan_case_both_devices(0));
    assert_eq!(
        pool.class_at_the_region_start(&[], &[]),
        AllocationInRegion::Inconsistent(
            RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
                header_unit_class: MappedUnitClass::Data,
                mapping_key_class_tag: LOCAL_UNIT_CLASS_INDEX_NODE,
            }
        )
    );
}

#[test]
fn identity_index_counts_its_keys_and_its_locations() {
    let (mut pool, key) = scan_case_pool_with_a_data_unit_at_the_start();
    pool.map(key, scan_case_both_devices(0));
    pool.map(vec![LOCAL_UNIT_CLASS_PACKED; 27], scan_case_both_devices(4));
    let index = IdentityIndexOfTheMapping::of_the_entries(&pool.mapping_entries);
    assert_eq!(index.keys(), 2);
    assert_eq!(index.location_keys(), 4);
}
