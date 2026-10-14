
// ===== 攻方腿 m3-c554-c572-r1（Opus）B3 原型：只在草稿副本里，接在 mount.rs 末尾 =====
#[cfg(test)]
mod attack_b3_isolation_keyed_by_start_slot {
    use super::{
        isolate_slots_referenced_only_by_abandoned_roots, BeforeTheOneReread,
        PlacementsOfAbandonedRoots,
    };
    use crate::address::{
        CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    };
    use crate::allocator::{
        AllocationRecord, DeviceFreeMap, PlacementOnDevice, PoolAllocator, UnitAreaStart,
    };
    use crate::block_device::{
        BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
    };
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};
    use std::collections::BTreeMap;

    /// 一读就 panic 的盘：这几格的输入全在内存里，被测函数读盘就说明走到了别的支。
    struct NoReads;
    impl BlockDevice for NoReads {
        fn read_at(&self, _o: DeviceOffsetInBytes, _b: &mut [u8]) -> Result<(), BlockDeviceError> {
            panic!("这一格不该读盘")
        }
        fn write_at(
            &mut self,
            _o: DeviceOffsetInBytes,
            _b: &[u8],
            _d: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            panic!("这一格不该写盘")
        }
        fn write_zeroes_at(&mut self, _o: DeviceOffsetInBytes, _l: u64) -> Result<(), BlockDeviceError> {
            panic!("这一格不该写盘")
        }
        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            PhysicalBlockSizeInBytes(512)
        }
        fn size_in_bytes(&self) -> u64 {
            device_bytes()
        }
    }

    const D0: DeviceIdentity = DeviceIdentity(0);

    fn unit_area_start() -> UnitAreaStart {
        UnitAreaStart::following_the_journal_ring(6 << 20).expect("6 MiB 环之后落在段边界")
    }
    fn device_bytes() -> u64 {
        (unit_area_start().slot().0 + 384) * singlefs_format::SLOT_BYTES
    }
    fn s() -> u64 {
        unit_area_start().slot().0 + 10
    }
    fn root(instance: u32, txg: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: [7; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(txg),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
        }
    }
    fn p(slot: u64, span: u64) -> PlacementOnDevice {
        PlacementOnDevice { device: D0, slot: SlotNumber(slot), span }
    }

    /// 跑一次被测函数，交回 s、s+1 两槽各自 is_free（true = 发得出去）。
    fn run(
        abandoned: &[(RootRecord, Vec<PlacementOnDevice>)],
        current_records: &[AllocationRecord],
        named_by_unapplied: &[PlacementOnDevice],
    ) -> (bool, bool) {
        let devices = vec![(D0, NoReads)];
        let mut allocator = PoolAllocator::new(vec![DeviceFreeMap::with_unit_area_start(
            D0,
            device_bytes(),
            unit_area_start(),
        )]);
        for record in current_records.iter().filter(|r| !r.is_released) {
            allocator.devices[0].mark_allocated(record.slot, u64::from(record.span_slots));
        }
        let roots: Vec<RootRecord> = abandoned.iter().map(|(r, _)| *r).collect();
        let recorded: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>> =
            abandoned
                .iter()
                .map(|(r, ps)| ((r.instance, r.checkpoint_txg), ps.clone()))
                .collect();
        let mut reread = BeforeTheOneReread::RereadImmediately;
        isolate_slots_referenced_only_by_abandoned_roots(
            &devices,
            unit_area_start(),
            &mut allocator,
            &roots,
            &|_root| true,
            CheckpointTxg(0),
            current_records,
            named_by_unapplied,
            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(
                &recorded,
            ),
            &mut reread,
            &[],
        )
        .expect("全在内存里，不拒");
        let free = |slot: u64| allocator.devices[0].is_free(SlotNumber(slot));
        println!(
            "OPUS-B3 slot_s_free={} slot_s_plus_1_free={} isolated_slots={}",
            free(s()),
            free(s() + 1),
            allocator.devices[0].isolated_slots()
        );
        (free(s()), free(s() + 1))
    }

    /// 形一：两条被抛弃根，A1 引用 (s, 跨 1)、A2 引用 (s, 跨 2)，A1 在 `roots` 里排前。
    #[test]
    fn opus_b3_two_abandoned_roots_narrow_first_leaves_the_second_slot_of_the_wide_one_free() {
        let narrow_first = run(
            &[(root(1, 8), vec![p(s(), 1)]), (root(2, 12), vec![p(s(), 2)])],
            &[],
            &[],
        );
        let wide_first = run(
            &[(root(2, 12), vec![p(s(), 2)]), (root(1, 8), vec![p(s(), 1)])],
            &[],
            &[],
        );
        println!("OPUS-B3-FORM1 narrow_first={narrow_first:?} wide_first={wide_first:?}");
        assert_eq!(narrow_first, (false, true), "窄的先进：s+1 发得出去");
        assert_eq!(wide_first, (false, false), "宽的先进：两槽都罩着");
    }

    /// 形二：现行账里 (s, 跨 1) 仍分配；被抛弃根引用 (s, 跨 2)。豁免按起点槽认，整份跳过，s+1 没人罩。
    #[test]
    fn opus_b3_current_account_record_at_the_same_start_slot_exempts_the_whole_wider_placement() {
        let record = AllocationRecord {
            device: D0,
            slot: SlotNumber(s()),
            span_slots: 1,
            generation: CheckpointTxg(5),
            is_released: false,
        };
        let got = run(&[(root(1, 8), vec![p(s(), 2)])], &[record], &[]);
        println!("OPUS-B3-FORM2 got={got:?}");
        assert_eq!(got, (false, true), "s 分配着，s+1 发得出去");
    }

    /// 形三：C 那一圈（水位之上没施加的记录点名）(s, 跨 2)，现行账里 (s, 跨 1) 仍分配：同样整份跳过。
    #[test]
    fn opus_b3_c_ring_placement_with_a_current_record_at_the_same_start_slot_is_skipped_whole() {
        let record = AllocationRecord {
            device: D0,
            slot: SlotNumber(s()),
            span_slots: 1,
            generation: CheckpointTxg(5),
            is_released: false,
        };
        let got = run(&[], &[record], &[p(s(), 2)]);
        println!("OPUS-B3-FORM3 got={got:?}");
        assert_eq!(got, (false, true), "s 分配着，s+1 发得出去");
    }
}
