//! 造的池：每个记录槽交回的日志记录、系统配置，按填满档与点名项数现造记录的读池。
use super::*;

// ============================================================================================
// 造记录与系统配置（登记第五节「装置的池」）。记录由 `JournalRecord::to_bytes` 现编，过 `parse` 往返。
// ============================================================================================

/// 一条过得了 `to_bytes` / `parse` 往返的节点指针：两条位置条目分在 0、1 号盘同一个槽。
fn node_pointer_at_slot(slot: u64, birth_txg: u64) -> NodePointer {
    let location_on = |device: u32| LocationEntry {
        device: DeviceIdentity(device),
        slot: SlotNumber(slot),
        unit_checksum: u32::try_from(slot % 0xffff_fff1).expect("取过模，装得进 32 位"),
    };
    NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(11),
            birth_txg: CheckpointTxg(birth_txg),
        },
        locations: [location_on(0), location_on(1)],
        instance: InstanceGeneration(PRODUCTION_INSTANCE),
        birth_sequence: BirthSequence(1),
    }
}

/// 计数器 `counter`、事务号 `transaction`、`named_count` 个点名项的一条记录；同一参数造出同一条记录（每块盘同一槽字节相同）。
pub(super) fn journal_record_for(
    instance: u32,
    counter: u64,
    transaction: u64,
    named_count: usize,
) -> JournalRecord {
    let named = (0..named_count)
        .map(|index| {
            let slot = 100_000 + counter * 64 + u64::try_from(index).expect("下标装得进 u64");
            let location_on = |device: u32| LocationEntry {
                device: DeviceIdentity(device),
                slot: SlotNumber(slot),
                unit_checksum: u32::try_from(slot % 0xffff_fff1).expect("取过模，装得进 32 位"),
            };
            NamedUnit {
                locations: [location_on(0), location_on(1)],
                unit_class: 3,
                birth_tree: TreeIdentifier(11),
                birth_txg: CheckpointTxg(counter),
                key_tail: [u8::try_from(index % 251).expect("取过模 251"); 10],
            }
        })
        .collect();
    JournalRecord {
        instance: InstanceGeneration(instance),
        counter,
        checkpoint_txg: CheckpointTxg(counter),
        transaction,
        is_commit: true,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
        back_chain: 0,
        filesystem_identifier: unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER),
        new_tree_table: node_pointer_at_slot(70_000 + counter, counter),
        new_mapping_root: node_pointer_at_slot(80_000 + counter, counter),
        new_tree_identifier_watermark: 1000 + counter,
        new_rollback_floor: CheckpointTxg(10 + counter),
        new_instance_table: node_pointer_at_slot(60_000, 1),
        new_allocation_record_tree_root: node_pointer_at_slot(60_001, 1),
        new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        new_reclaim_threshold: CheckpointTxg(0),
        new_pinned_ring_slot: PinnedRingSlot::NO_SLOT_PINNED,
        named,
    }
}

/// 喂给 `scan_journal` 的系统配置：fsid 取 [`FILESYSTEM_IDENTIFIER`]，journal 环长取 `ring_bytes`，其余字段 `scan_journal` 不读（值随便取，只求造得出结构体）。
pub(super) fn system_configuration_for(ring_bytes: u64, device_count: u32) -> SystemConfiguration {
    SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: FILESYSTEM_IDENTIFIER,
            this_device: DeviceIdentity(0),
            device_count,
            region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
            sizes: SystemImmutableSizes {
                physical_block_size: 4096,
                minimum_input_output_bytes: 4096,
                fixed_structure_slot_spacing: 4096,
                journal_ring_bytes: ring_bytes,
                root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
            },
            journal_ring_start_slot: SlotNumber(JOURNAL_RING_START_SLOT),
            root_ring_base_slot: SlotNumber(0),
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: 1,
            journal_tail: 0,
            journal_instance: InstanceGeneration(PRODUCTION_INSTANCE),
            rollback_floor: CheckpointTxg(0),
        },
    }
}

// ============================================================================================
// 装置的池：只按 (盘, 偏移) 现算字节、不存镜像、不缓存任何一槽的字节（登记第五节「装置的池」）。
// ============================================================================================

/// 某块盘（盘次序里的下标）的某个环内槽上放着哪条记录；`None` 是从没写过的全 0 槽。
pub(super) type SlotSource = Box<dyn Fn(u32, u64) -> Option<JournalRecord>>;

pub(super) fn slot_source_of(
    source: impl Fn(u32, u64) -> Option<JournalRecord> + 'static,
) -> SlotSource {
    Box::new(source)
}

/// 各臂共用的生产槽：每块盘同一槽字节相同；前 `filled_slots` 个槽是计数器 = 槽号 + 1、带 `named_count` 个点名项的记录，别处全 0。
pub(super) fn production_slot_source(filled_slots: u64, named_count: usize) -> SlotSource {
    Box::new(move |_device, slot| {
        if slot < filled_slots {
            Some(journal_record_for(
                PRODUCTION_INSTANCE,
                slot + 1,
                slot + 1,
                named_count,
            ))
        } else {
            None
        }
    })
}

impl GeneratedPool {
    pub(super) fn new(
        device_count: u32,
        device_bytes: u64,
        ring_slots: u64,
        slot_source: SlotSource,
    ) -> Self {
        GeneratedPool {
            device_count,
            device_bytes,
            ring_slots,
            slot_source,
            reads: Cell::new(0),
            resident_kib_at_each_device_end: RefCell::new(Vec::with_capacity(
                usize::try_from(device_count).expect("盘数装得进 usize"),
            )),
        }
    }
}

impl PoolReader for GeneratedPool {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        (0..self.device_count).map(DeviceIdentity).collect()
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        (device.0 < self.device_count).then_some(self.device_bytes)
    }

    fn physical_block_size_probed_on_the_device(
        &self,
        _device: DeviceIdentity,
    ) -> Option<PhysicalBlockSizeInBytes> {
        unreachable!("生成的池只交给扫环那一段读 journal 环，不挂载、不探测块宽")
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.reads.set(self.reads.get() + 1);
        let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
        if device.0 >= self.device_count || offset.0 < ring_start {
            return None;
        }
        let offset_in_ring = offset.0 - ring_start;
        let is_a_whole_record_slot = offset_in_ring.is_multiple_of(JOURNAL_RECORD_BYTES)
            && u64::try_from(length).ok() == Some(JOURNAL_RECORD_BYTES);
        let slot = offset_in_ring / JOURNAL_RECORD_BYTES;
        if !is_a_whole_record_slot || slot >= self.ring_slots {
            return None;
        }
        if slot + 1 == self.ring_slots {
            self.resident_kib_at_each_device_end
                .borrow_mut()
                .push(vm_rss_kib_now());
        }
        match (self.slot_source)(device.0, slot) {
            Some(record) => Some(record.to_bytes()),
            None => Some(vec![0u8; length]),
        }
    }

    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        // 提示交 None：扫整环（与文件后端的真盘同，登记第五节）。
        None
    }
}

#[cfg(test)]
mod tests;
