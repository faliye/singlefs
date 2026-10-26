//! 验证侧的盘面读法。解析与校验走 checker 的独立实现；core 的类型只用于与被测调用方交换字段。
//! 不调用 core 的解析器、校验和、择根或恢复辅助函数。

use singlefs_format::{
    DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
};

/// 码 1 的有效载荷容量，按字段表计算，不问实现。
pub(crate) fn data_payload_capacity() -> usize {
    (DATA_UNIT_BYTES - DATA_UNIT_HEADER_BYTES - NONCE_MAC_ALGORITHM_RESERVED_BYTES) as usize
}

/// 校验数据单元并取声明长度的载荷；补齐区也必须全零。
pub(crate) fn data_payload(bytes: &[u8]) -> Result<&[u8], &'static str> {
    if singlefs_checker::check_unit(bytes).map_err(|_| "数据单元校验失败")? != 1 || bytes[42] != 1
    {
        return Err("不是码 1 数据单元");
    }
    let start = (DATA_UNIT_HEADER_BYTES + NONCE_MAC_ALGORITHM_RESERVED_BYTES) as usize;
    let length = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
    let end = start.checked_add(length).ok_or("载荷长度溢出")?;
    let payload = bytes.get(start..end).ok_or("载荷越过单元末尾")?;
    if bytes[end..].iter().any(|byte| *byte != 0) {
        return Err("数据单元补齐非零");
    }
    Ok(payload)
}

use singlefs_checker::image::{ImageReader, SystemConfigurationSlotReading};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::instance_table::{InstanceRow, InstanceTableRecords};
use singlefs_core::journal::{
    JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, NamedUnit,
};
use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use singlefs_core::recovery::PoolReader;
use singlefs_core::root_record::{PinnedRingSlot, RootRecord, UnmountMarker};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    InodeRecordTruncationFieldsOfThePool, SystemConfiguration, SystemImmutableConfiguration,
    SystemImmutableSizes, SystemMutableConfiguration, SystemRuntimeConfiguration,
    SystemRuntimeQuantities,
};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, NODE_BYTES, ROOT_RECORD_BYTES,
    ROOT_RING_BASE_SLOT, ROOT_RING_CHUNK_BYTES, ROOT_RING_PRIME_STEP, SLOT_BYTES,
    SYSTEM_CONFIGURATION_SLOT_BYTES,
};
use std::collections::{BTreeMap, BTreeSet};

fn number(bytes: &[u8], offset: usize, width: usize) -> u64 {
    let mut word = [0; 8];
    word[..width].copy_from_slice(&bytes[offset..offset + width]);
    u64::from_le_bytes(word)
}

/// 这里只接设备字节，不经 core 的任何内容解析或扫描候选生成。
struct Reader<'a>(&'a dyn PoolReader);
impl ImageReader for Reader<'_> {
    fn devices(&self) -> Vec<u32> {
        self.0
            .device_identities()
            .into_iter()
            .map(|d| d.0)
            .collect()
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        self.0.device_size_in_bytes(DeviceIdentity(device))
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        self.0
            .read(DeviceIdentity(device), DeviceOffsetInBytes(offset), length)
    }
    fn candidate_unit_slots(&self, _: u32) -> Option<Vec<u64>> {
        None
    }
    fn candidate_journal_slots(&self, _: u32) -> Option<Vec<u64>> {
        None
    }
}

pub(crate) fn parse_system_configuration(slot: &[u8]) -> Result<SystemConfiguration, ()> {
    let view = singlefs_checker::check_system_configuration_slot(slot).map_err(|_| ())?;
    let geometry = singlefs_checker::image::geometry_of(slot, &view).map_err(|_| ())?;
    configuration_from_view(&view, &geometry)
}

fn configuration_from_view(
    view: &singlefs_checker::SystemConfigurationView,
    geometry: &singlefs_checker::image::PoolGeometry,
) -> Result<SystemConfiguration, ()> {
    if singlefs_checker::image::journal_ring_geometry_field_that_disagrees(geometry).is_some() {
        return Err(());
    }
    Ok(SystemConfiguration {
        immutable: SystemImmutableConfiguration {
            filesystem_identifier: view.filesystem_identifier,
            this_device: DeviceIdentity(view.this_device),
            device_count: view.device_count,
            region_devices: geometry.region_devices.map(DeviceIdentity),
            sizes: SystemImmutableSizes {
                physical_block_size: geometry.physical_block_size,
                minimum_input_output_bytes: geometry.declared_minimum_input_output_bytes,
                fixed_structure_slot_spacing: u32::try_from(geometry.slot_spacing)
                    .map_err(|_| ())?,
                journal_ring_bytes: geometry.journal_ring_bytes,
                root_ring_slots_per_region:
                    RootRingSlotsPerRegion::from_system_configuration_field(
                        geometry.slots_per_region,
                    )
                    .map_err(|_| ())?,
            },
            journal_ring_start_slot: SlotNumber(geometry.journal_ring_start_slot),
            root_ring_base_slot: SlotNumber(geometry.base_slot),
            inode_record_truncation_fields: match view.inode_record_truncation_field {
                singlefs_checker::InodeRecordTruncationField::AbsentWithoutIncompatBitTwo => {
                    InodeRecordTruncationFieldsOfThePool::AllZero
                }
                singlefs_checker::InodeRecordTruncationField::CarriedUnderIncompatBitTwo => {
                    InodeRecordTruncationFieldsOfThePool::Carried
                }
            },
        },
        mutable: SystemMutableConfiguration,
        runtime: SystemRuntimeConfiguration,
        quantities: SystemRuntimeQuantities {
            slot_generation: view.slot_generation,
            journal_tail: view.journal_tail,
            journal_instance: InstanceGeneration(view.journal_instance),
            rollback_floor: CheckpointTxg(view.rollback_floor),
        },
    })
}

pub(crate) fn choose_system_configuration(
    reader: &dyn PoolReader,
) -> Result<SystemConfiguration, ()> {
    let mut chosen: Option<SystemConfiguration> = None;
    for (_, readings) in
        singlefs_checker::image::system_configuration_slot_readings(&Reader(reader))
    {
        let mut best: Option<SystemConfiguration> = None;
        for reading in readings {
            match reading {
                SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => {
                    return Err(())
                }
                SystemConfigurationSlotReading::Usable(view, geometry) => {
                    let candidate = configuration_from_view(&view, &geometry)?;
                    if best.as_ref().is_none_or(|current| {
                        candidate.quantities.slot_generation > current.quantities.slot_generation
                    }) {
                        best = Some(candidate);
                    }
                }
                SystemConfigurationSlotReading::NotSelfVerified
                | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => {}
            }
        }
        if let Some(candidate) = best {
            if let Some(current) = chosen {
                if current.immutable.filesystem_identifier
                    != candidate.immutable.filesystem_identifier
                    || current.immutable.device_count != candidate.immutable.device_count
                {
                    return Err(());
                }
            } else {
                chosen = Some(candidate);
            }
        }
    }
    chosen.ok_or(())
}

pub(crate) fn verified_system_configuration_slots(
    reader: &dyn PoolReader,
    device: DeviceIdentity,
    spacing: u64,
    fsid: &[u8; 16],
) -> Vec<SystemConfiguration> {
    [0, spacing]
        .into_iter()
        .filter_map(|offset| {
            let bytes = reader.read(
                device,
                DeviceOffsetInBytes(offset),
                SYSTEM_CONFIGURATION_SLOT_BYTES as usize,
            )?;
            parse_system_configuration(&bytes)
                .ok()
                .filter(|slot| slot.immutable.filesystem_identifier == *fsid)
        })
        .collect()
}

pub(crate) fn chosen_system_configuration_slot_bytes(reader: &dyn PoolReader) -> Option<Vec<u8>> {
    let chosen = choose_system_configuration(reader).ok()?;
    [
        0,
        u64::from(chosen.immutable.sizes.fixed_structure_slot_spacing),
    ]
    .into_iter()
    .find_map(|offset| {
        let bytes = reader.read(
            chosen.immutable.this_device,
            DeviceOffsetInBytes(offset),
            SYSTEM_CONFIGURATION_SLOT_BYTES as usize,
        )?;
        (parse_system_configuration(&bytes).ok() == Some(chosen)).then_some(bytes)
    })
}

fn locations(bytes: &[u8]) -> [LocationEntry; 2] {
    [0, 14].map(|offset| LocationEntry {
        device: DeviceIdentity(number(bytes, offset, 4) as u32),
        slot: SlotNumber(number(bytes, offset + 4, 6)),
        unit_checksum: number(bytes, offset + 10, 4) as u32,
    })
}

pub(crate) fn node_pointer(bytes: &[u8]) -> Option<NodePointer> {
    if bytes.len() < 86 || bytes[..34].iter().any(|b| *b != 0) {
        return None;
    }
    Some(NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(number(bytes, 34, 8)),
            birth_txg: CheckpointTxg(number(bytes, 42, 8)),
        },
        locations: locations(&bytes[50..78]),
        instance: InstanceGeneration(number(bytes, 78, 4) as u32),
        birth_sequence: BirthSequence(number(bytes, 82, 4) as u32),
    })
}

fn marker(flags: u64) -> Option<UnmountMarker> {
    match flags {
        0 => Some(UnmountMarker::NotWrittenByTheUnmountSequence),
        1 => Some(UnmountMarker::WrittenByTheUnmountSequence),
        _ => None,
    }
}

fn root_from_slot(bytes: &[u8], fsid: &[u8; 16]) -> Option<RootRecord> {
    let view = singlefs_checker::check_root_slot(bytes, fsid).ok()?;
    Some(RootRecord {
        filesystem_identifier: *fsid,
        unmount_marker: marker(number(bytes, 20, 4))?,
        instance: InstanceGeneration(view.instance),
        checkpoint_txg: CheckpointTxg(view.checkpoint_txg),
        tree_table: node_pointer(&bytes[36..122])?,
        tree_identifier_watermark: view.tree_identifier_watermark,
        rollback_floor: CheckpointTxg(view.rollback_floor),
        instance_table: node_pointer(&bytes[170..256])?,
        mapping_root: node_pointer(&bytes[256..342])?,
        allocation_record_tree_root: node_pointer(&bytes[342..428])?,
        reclaim_threshold: CheckpointTxg(view.reclaim_threshold),
        pinned_ring_slot: PinnedRingSlot(view.pinned_ring_slot),
    })
}

fn ring_slots(
    devices: &[DeviceIdentity; 3],
    sizes: &SystemImmutableSizes,
) -> Vec<(DeviceIdentity, u64)> {
    (0..3)
        .flat_map(|region| {
            (0..sizes.root_ring_slots_per_region.count()).map(move |slot| {
                (
                    devices[region],
                    ROOT_RING_BASE_SLOT * SLOT_BYTES
                        + region as u64 * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES
                        + slot * u64::from(sizes.fixed_structure_slot_spacing),
                )
            })
        })
        .collect()
}

pub(crate) fn readable_roots(
    reader: &dyn PoolReader,
    devices: &[DeviceIdentity; 3],
    sizes: &SystemImmutableSizes,
    fsid: &[u8; 16],
) -> Vec<RootRecord> {
    ring_slots(devices, sizes)
        .into_iter()
        .filter_map(|(device, offset)| {
            let bytes = reader.read(
                device,
                DeviceOffsetInBytes(offset),
                sizes.physical_block_size as usize,
            )?;
            root_from_slot(&bytes, fsid)
        })
        .collect()
}

pub(crate) fn choose_root(
    reader: &dyn PoolReader,
    configuration: &SystemConfiguration,
) -> Option<RootRecord> {
    let immutable = configuration.immutable;
    readable_roots(
        reader,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .reduce(|best, candidate| {
        if (candidate.checkpoint_txg, candidate.instance) > (best.checkpoint_txg, best.instance) {
            candidate
        } else {
            best
        }
    })
}

pub(crate) fn highest_root_instance(
    reader: &dyn PoolReader,
    devices: &[DeviceIdentity; 3],
    sizes: &SystemImmutableSizes,
    fsid: &[u8; 16],
) -> Option<InstanceGeneration> {
    readable_roots(reader, devices, sizes, fsid)
        .into_iter()
        .map(|root| root.instance)
        .max()
}
pub(crate) fn highest_root_txg(
    reader: &dyn PoolReader,
    devices: &[DeviceIdentity; 3],
    sizes: &SystemImmutableSizes,
    fsid: &[u8; 16],
) -> Option<CheckpointTxg> {
    readable_roots(reader, devices, sizes, fsid)
        .into_iter()
        .map(|root| root.checkpoint_txg)
        .max()
}

fn read_locations(
    reader: &dyn PoolReader,
    locations: &[LocationEntry; 2],
    length: usize,
) -> Option<Vec<u8>> {
    locations.iter().find_map(|location| {
        let offset = location.slot.0.checked_mul(SLOT_BYTES)?;
        reader
            .read(location.device, DeviceOffsetInBytes(offset), length)
            .filter(|bytes| {
                singlefs_checker::crc32_castagnoli_table(bytes) == location.unit_checksum
            })
    })
}

pub(crate) fn tree_table_entry_bytes_of_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Result<Vec<Vec<u8>>, ()> {
    let bytes = read_locations(reader, &root.tree_table.locations, NODE_BYTES as usize).ok_or(())?;
    singlefs_checker::index_node_view(&bytes)
        .map(|node| node.entries)
        .map_err(|_| ())
}

pub(crate) struct InstancePage {
    pub rows: Vec<InstanceRow>,
    pub next: Option<NodePointer>,
}

pub(crate) fn instance_table_page(bytes: &[u8], page_index: u64) -> Option<InstancePage> {
    let page = singlefs_checker::packed_unit_view(bytes).ok()?;
    if page.birth_tree != 0
        || page.record_type != 4
        || page.container != page_index
        || page.container_birth != 0
        || page.record_width != 88
    {
        return None;
    }
    let (chain, records) = page.records.split_last()?;
    let mut rows = Vec::new();
    for record in records {
        if record[0] != 0 || record[21] != 0 {
            return None;
        }
        rows.push(InstanceRow {
            instance: InstanceGeneration(number(record, 1, 4) as u32),
            selected_root_txg: CheckpointTxg(number(record, 5, 8)),
            applied_transaction_high_water: number(record, 13, 8),
        });
    }
    if chain[0] != 1 {
        return None;
    }
    let next = match chain[1] {
        0 if chain[2..].iter().all(|b| *b == 0) => None,
        1 if chain[2..].iter().any(|b| *b != 0) => Some(node_pointer(&chain[2..])?),
        _ => return None,
    };
    Some(InstancePage { rows, next })
}

pub(crate) fn instance_table_of_root(
    reader: &dyn PoolReader,
    root: &RootRecord,
) -> Option<InstanceTableRecords> {
    let mut pointer = root.instance_table;
    let mut rows = Vec::new();
    let mut visited = BTreeSet::new();
    loop {
        let identity = (
            pointer.locations[0].device.0,
            pointer.locations[0].slot.0,
            pointer.locations[1].device.0,
            pointer.locations[1].slot.0,
        );
        if !visited.insert(identity) {
            return None;
        }
        let bytes = read_locations(reader, &pointer.locations, DATA_UNIT_BYTES as usize)?;
        let page = instance_table_page(&bytes, (visited.len() - 1) as u64)?;
        rows.extend(page.rows);
        match page.next {
            None => return Some(InstanceTableRecords { rows }),
            Some(next) => pointer = next,
        }
    }
}

pub(crate) fn effective_rollback_floor(
    reader: &dyn PoolReader,
    devices: &[DeviceIdentity; 3],
    sizes: &SystemImmutableSizes,
    fsid: &[u8; 16],
) -> CheckpointTxg {
    let roots = readable_roots(reader, devices, sizes, fsid);
    let newest = roots.iter().fold(None, |best: Option<&RootRecord>, root| {
        if best.is_none_or(|best| {
            (root.checkpoint_txg, root.instance) > (best.checkpoint_txg, best.instance)
        }) {
            Some(root)
        } else {
            best
        }
    });
    let table = newest.and_then(|root| instance_table_of_root(reader, root));
    let mut newest_on_device = BTreeMap::new();
    let mut abandoned_floor = CheckpointTxg(0);
    for (device, offset) in ring_slots(devices, sizes) {
        let Some(root) = reader
            .read(
                device,
                DeviceOffsetInBytes(offset),
                sizes.physical_block_size as usize,
            )
            .and_then(|bytes| root_from_slot(&bytes, fsid))
        else {
            continue;
        };
        if table.as_ref().is_some_and(|table| {
            table.rows.iter().any(|row| {
                row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg
            })
        }) {
            abandoned_floor = abandoned_floor.max(root.rollback_floor);
        } else {
            let best = newest_on_device.entry(device).or_insert(root);
            if (root.checkpoint_txg, root.instance) > (best.checkpoint_txg, best.instance) {
                *best = root;
            }
        }
    }
    let root_floor = newest_on_device
        .values()
        .map(|root| root.rollback_floor)
        .chain([abandoned_floor])
        .max()
        .unwrap_or(CheckpointTxg(0));
    let configuration_floor = reader
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            verified_system_configuration_slots(
                reader,
                device,
                u64::from(sizes.fixed_structure_slot_spacing),
                fsid,
            )
        })
        .map(|slot| slot.quantities.rollback_floor)
        .max()
        .unwrap_or(CheckpointTxg(0));
    root_floor.max(configuration_floor)
}

fn journal_record(bytes: &[u8], fsid: u64) -> Option<JournalRecord> {
    let view = singlefs_checker::check_journal_record(bytes, fsid).ok()?;
    if view.commit_marker_byte > 1 || view.record_flags_byte > 1 {
        return None;
    }
    let segment = &view.new_root_segment;
    Some(JournalRecord {
        instance: InstanceGeneration(view.instance),
        counter: view.counter,
        checkpoint_txg: CheckpointTxg(view.checkpoint_txg),
        transaction: view.transaction,
        is_commit: view.is_commit,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish(view.ordinal_within_publish),
        place_in_publish: if view.record_flags_byte == 1 {
            JournalRecordPlaceInPublish::LastRecordOfThePublish
        } else {
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
        },
        back_chain: view.back_chain,
        filesystem_identifier: fsid,
        new_tree_table: node_pointer(&segment[..86])?,
        new_mapping_root: node_pointer(&segment[86..172])?,
        new_tree_identifier_watermark: view.new_tree_identifier_watermark,
        new_rollback_floor: CheckpointTxg(view.new_rollback_floor),
        new_instance_table: node_pointer(&segment[188..274])?,
        new_allocation_record_tree_root: node_pointer(&segment[274..360])?,
        new_unmount_marker: marker(number(segment, 360, 4))?,
        new_reclaim_threshold: CheckpointTxg(number(segment, 364, 8)),
        new_pinned_ring_slot: PinnedRingSlot(segment[372]),
        named: view
            .named
            .into_iter()
            .map(|named| NamedUnit {
                locations: named
                    .locations
                    .map(|(device, slot, checksum)| LocationEntry {
                        device: DeviceIdentity(device),
                        slot: SlotNumber(slot),
                        unit_checksum: checksum,
                    }),
                unit_class: named.unit_class,
                birth_tree: TreeIdentifier(named.birth_tree),
                birth_txg: CheckpointTxg(named.birth_txg),
                key_tail: named.key_tail,
            })
            .collect(),
    })
}

pub(crate) fn scan_journal(
    reader: &dyn PoolReader,
    configuration: &SystemConfiguration,
) -> BTreeMap<(InstanceGeneration, u64), JournalRecord> {
    let mut records = BTreeMap::new();
    let fsid = number(&configuration.immutable.filesystem_identifier, 0, 8);
    let start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let ring_bytes = configuration.immutable.sizes.journal_ring_bytes;
    for device in reader.device_identities() {
        let end = reader
            .device_size_in_bytes(device)
            .unwrap_or(0)
            .saturating_sub(start)
            .min(ring_bytes);
        let offsets: Box<dyn Iterator<Item = DeviceOffsetInBytes>> = match reader
            .journal_record_offsets_hint(device, DeviceOffsetInBytes(start), ring_bytes)
        {
            Some(offsets) => Box::new(offsets.into_iter()),
            None => Box::new(
                (0..end / JOURNAL_RECORD_BYTES)
                    .map(|index| DeviceOffsetInBytes(start + index * JOURNAL_RECORD_BYTES)),
            ),
        };
        for offset in offsets {
            if let Some(record) = reader
                .read(device, offset, JOURNAL_RECORD_BYTES as usize)
                .and_then(|bytes| journal_record(&bytes, fsid))
            {
                records
                    .entry((record.instance, record.counter))
                    .or_insert(record);
            }
        }
    }
    records
}

fn put(bytes: &mut [u8], offset: usize, value: u64, width: usize) {
    bytes[offset..offset + width].copy_from_slice(&value.to_le_bytes()[..width]);
}
fn pointer_bytes(pointer: &NodePointer) -> [u8; 86] {
    let mut bytes = [0; 86];
    put(&mut bytes, 34, pointer.head.birth_tree.0, 8);
    put(&mut bytes, 42, pointer.head.birth_txg.0, 8);
    for (index, location) in pointer.locations.iter().enumerate() {
        let offset = 50 + index * 14;
        put(&mut bytes, offset, u64::from(location.device.0), 4);
        put(&mut bytes, offset + 4, location.slot.0, 6);
        put(
            &mut bytes,
            offset + 10,
            u64::from(location.unit_checksum),
            4,
        );
    }
    put(&mut bytes, 78, u64::from(pointer.instance.0), 4);
    put(&mut bytes, 82, u64::from(pointer.birth_sequence.0), 4);
    bytes
}

/// 没有根槽的 journal 版本按字段表重建原样；校验和用验证侧 CRC。
pub(crate) fn encoded_root(root: &RootRecord, slot_bytes: usize) -> Vec<u8> {
    let mut bytes = vec![0; slot_bytes];
    bytes[..4].copy_from_slice(b"SFSR");
    bytes[4..20].copy_from_slice(&root.filesystem_identifier);
    put(
        &mut bytes,
        20,
        match root.unmount_marker {
            UnmountMarker::WrittenByTheUnmountSequence => 1,
            UnmountMarker::NotWrittenByTheUnmountSequence => 0,
        },
        4,
    );
    put(&mut bytes, 24, u64::from(root.instance.0), 4);
    put(&mut bytes, 28, root.checkpoint_txg.0, 8);
    for (offset, pointer) in [
        (36, root.tree_table),
        (170, root.instance_table),
        (256, root.mapping_root),
        (342, root.allocation_record_tree_root),
    ] {
        bytes[offset..offset + 86].copy_from_slice(&pointer_bytes(&pointer));
    }
    put(&mut bytes, 122, root.tree_identifier_watermark, 8);
    put(&mut bytes, 130, root.rollback_floor.0, 8);
    put(&mut bytes, 428, root.reclaim_threshold.0, 8);
    bytes[436] = root.pinned_ring_slot.0;
    let crc = singlefs_checker::crc32_castagnoli_table(&bytes);
    put(&mut bytes, 138, u64::from(crc), 4);
    bytes
}

pub(crate) fn encoded_tree_table_entry(entry: &singlefs_core::records::TreeTableEntry) -> Vec<u8> {
    let mut bytes = vec![0; singlefs_format::TREE_TABLE_ENTRY_BYTES as usize];
    put(&mut bytes, 0, entry.tree.0, 8);
    put(&mut bytes, 8, singlefs_format::TREE_TABLE_ENTRY_BYTES, 2);
    put(&mut bytes, 10, u64::from(entry.kind), 2);
    bytes[14..100].copy_from_slice(&pointer_bytes(&entry.root));
    put(&mut bytes, 100, entry.previous_snapshot_txg.0, 8);
    put(&mut bytes, 108, entry.birth_txg.0, 8);
    put(&mut bytes, 116, entry.head_identifier, 8);
    bytes
}

pub(crate) fn root_record_bytes_of_the_version(
    reader: &dyn PoolReader,
    devices: &[DeviceIdentity; 3],
    sizes: &SystemImmutableSizes,
    root: &RootRecord,
) -> Vec<u8> {
    ring_slots(devices, sizes)
        .into_iter()
        .find_map(|(device, offset)| {
            let bytes = reader.read(
                device,
                DeviceOffsetInBytes(offset),
                sizes.physical_block_size as usize,
            )?;
            (root_from_slot(&bytes, &root.filesystem_identifier) == Some(*root))
                .then(|| bytes[..ROOT_RECORD_BYTES as usize].to_vec())
        })
        .unwrap_or_else(|| {
            encoded_root(root, sizes.physical_block_size as usize)[..ROOT_RECORD_BYTES as usize]
                .to_vec()
        })
}

/// 只按被测恢复报告给的身份读取该版本的盘上字段，不以实现的字段重建辅助函数充当参照。
/// 身份来自实际结果，根字段必须来自独立校验的根槽或 journal 新根段。
pub(crate) fn root_of_observed_version(
    reader: &dyn PoolReader,
    configuration: &SystemConfiguration,
    version: (InstanceGeneration, CheckpointTxg),
) -> Option<RootRecord> {
    let immutable = configuration.immutable;
    readable_roots(
        reader,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| (root.instance, root.checkpoint_txg) == version)
    .or_else(|| {
        scan_journal(reader, configuration)
            .into_values()
            .filter(|record| {
                record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
            })
            .find(|record| (record.instance, record.checkpoint_txg) == version)
            .map(|record| RootRecord {
                filesystem_identifier: immutable.filesystem_identifier,
                unmount_marker: record.new_unmount_marker,
                instance: record.instance,
                checkpoint_txg: record.checkpoint_txg,
                tree_table: record.new_tree_table,
                tree_identifier_watermark: record.new_tree_identifier_watermark,
                rollback_floor: record.new_rollback_floor,
                instance_table: record.new_instance_table,
                mapping_root: record.new_mapping_root,
                allocation_record_tree_root: record.new_allocation_record_tree_root,
                reclaim_threshold: record.new_reclaim_threshold,
                pinned_ring_slot: record.new_pinned_ring_slot,
            })
    })
}

pub(crate) fn first_file_content(
    reader: &dyn PoolReader,
    root: &RootRecord,
    sizes: &SystemImmutableSizes,
) -> Option<Option<Vec<u8>>> {
    let bytes = encoded_root(root, sizes.physical_block_size as usize);
    singlefs_checker::walk::first_file_content_of_a_root_record(&Reader(reader), &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_pool::{MemoryPool, SparseBlockDevice};
    use crate::{RecordingBlockDevice, SharedStream};
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::transaction::TransactionOutput;

    fn pool_with_file() -> (MemoryPool, TransactionOutput) {
        let stream = SharedStream::retaining_contents();
        let mut devices = [0, 1].map(|device| {
            (
                DeviceIdentity(device),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device),
                    SparseBlockDevice::new(4 << 30, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        });
        let parameters = crate::scenario::e142_parameters(512, 512);
        let run = crate::scenario::run_new_pool_file_creation(
            &parameters,
            &mut devices,
            &stream,
            |_, _| {},
        )
        .expect("产品写出已知文件的盘面");
        let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 4 << 30);
        pool.apply(&stream.retained_operations());
        (pool, run.output)
    }

    fn seal(bytes: &mut [u8], field: usize, end: usize) {
        bytes[field..field + 32].fill(0);
        let crc = singlefs_checker::crc32_castagnoli_bitwise(&bytes[..end]);
        bytes[field..field + 4].copy_from_slice(&crc.to_le_bytes());
    }

    fn data_unit() -> Vec<u8> {
        let mut bytes = vec![0; DATA_UNIT_BYTES as usize];
        bytes[..4].copy_from_slice(b"SFSU");
        put(&mut bytes, 4, 1, 2);
        bytes[6] = 1;
        put(&mut bytes, 8, 3, 2);
        bytes[42] = 1;
        bytes[134..137].copy_from_slice(b"abc");
        seal_data(&mut bytes);
        bytes
    }
    fn seal_data(bytes: &mut [u8]) {
        let crc = singlefs_checker::crc32_castagnoli_bitwise(&bytes[105..]);
        put(bytes, 101, u64::from(crc), 4);
        seal(bytes, 10, 105);
    }

    #[test]
    fn independent_data_reader_checks_declared_length_padding_and_both_checksums() {
        let bytes = data_unit();
        assert_eq!(data_payload(&bytes), Ok(b"abc".as_slice()));
        let mut corrupt = bytes.clone();
        corrupt[134] ^= 1;
        assert!(data_payload(&corrupt).is_err(), "载荷 CRC 必须核对");
        let mut corrupt = bytes.clone();
        corrupt[8] ^= 1;
        assert!(data_payload(&corrupt).is_err(), "头校验和必须核对");
        let mut corrupt = bytes.clone();
        corrupt[137] = 1;
        seal_data(&mut corrupt);
        assert!(
            data_payload(&corrupt).is_err(),
            "校验和重签仍须拒绝非零补齐"
        );
        let mut corrupt = bytes.clone();
        put(&mut corrupt, 8, 65535, 2);
        seal_data(&mut corrupt);
        assert!(data_payload(&corrupt).is_err(), "坏长度不许越界");
        let mut corrupt = bytes;
        corrupt[105] = 1;
        seal_data(&mut corrupt);
        assert!(
            data_payload(&corrupt).is_err(),
            "校验和重签仍须拒绝加密预留位"
        );
    }

    #[test]
    fn independent_readers_decode_product_written_fields_and_content() {
        let (pool, output) = pool_with_file();
        let configuration = choose_system_configuration(&pool).expect("验证侧择配置");
        let root = choose_root(&pool, &configuration).expect("验证侧择根");
        assert_eq!(root, output.root);
        assert_eq!(
            first_file_content(&pool, &root, &configuration.immutable.sizes),
            Some(Some(crate::scenario::first_file_content()))
        );
        assert!(instance_table_of_root(&pool, &root).is_some());
        let entries = tree_table_entry_bytes_of_root(&pool, &root).expect("独立树表解析");
        assert_eq!(
            entries,
            output
                .tree_table_entries
                .iter()
                .map(encoded_tree_table_entry)
                .collect::<Vec<_>>()
        );
        let records = scan_journal(&pool, &configuration);
        let record = records
            .get(&(output.record.instance, output.record.counter))
            .expect("独立 journal 解析");
        assert_eq!(record, &output.record);
        let observed =
            root_of_observed_version(&pool, &configuration, (root.instance, root.checkpoint_txg));
        assert_eq!(observed, Some(root));
    }

    #[test]
    fn candidate_scan_keeps_written_units_when_a_checksummed_configuration_lies_about_the_start() {
        let (mut pool, _) = pool_with_file();
        let device = DeviceIdentity(0);
        let expected = ImageReader::candidate_unit_slots(&pool, device.0).expect("稀疏候选");
        assert!(!expected.is_empty());
        let mut slot = PoolReader::read(
            &pool,
            device,
            DeviceOffsetInBytes(0),
            SYSTEM_CONFIGURATION_SLOT_BYTES as usize,
        )
        .expect("配置槽");
        let invalid_start = expected.iter().max().unwrap() + 64;
        put(&mut slot, 417, invalid_start, 8);
        let slot_len = slot.len();
        seal(&mut slot, 155, slot_len);
        assert!(
            singlefs_checker::check_system_configuration_slot(&slot).is_ok(),
            "这是一份校验和正确但几何自述错误的槽"
        );
        pool.devices
            .get_mut(&device)
            .expect("盘 0")
            .write(DeviceOffsetInBytes(0), &slot);
        let actual: BTreeSet<_> = ImageReader::candidate_unit_slots(&pool, device.0)
            .expect("保守候选")
            .into_iter()
            .collect();
        assert!(
            expected.into_iter().all(|slot| actual.contains(&slot)),
            "坏配置不能裁掉原有单元候选"
        );
    }

    #[test]
    fn independently_decoded_roots_reject_reserved_pointer_bits_even_after_resigning() {
        let (pool, output) = pool_with_file();
        let configuration = choose_system_configuration(&pool).unwrap();
        let mut bytes = encoded_root(
            &output.root,
            configuration.immutable.sizes.physical_block_size as usize,
        );
        for pointer_offset in [36, 170, 256, 342] {
            bytes[pointer_offset + 32] = 1;
            let length = bytes.len();
            seal(&mut bytes, 138, length);
            assert!(
                root_from_slot(&bytes, &output.root.filesystem_identifier).is_none(),
                "extent 偏移非零的根指针不可用"
            );
            bytes[pointer_offset + 32] = 0;
        }
    }
    #[test]
    fn independent_floor_uses_the_newest_valid_root_on_each_device() {
        let (mut pool, output) = pool_with_file();
        let configuration = choose_system_configuration(&pool).unwrap();
        let immutable = configuration.immutable;
        let positions = ring_slots(&immutable.region_devices, &immutable.sizes);
        for (device, offset) in &positions {
            pool.devices.get_mut(device).unwrap().write(
                DeviceOffsetInBytes(*offset),
                &vec![0; immutable.sizes.physical_block_size as usize],
            );
        }
        let first_on_device_one = *positions.iter().find(|(device, _)| device.0 == 1).unwrap();
        for ((device, offset), txg, floor) in [
            (positions[0], 100, 50),
            (positions[1], 101, 2),
            (first_on_device_one, 99, 3),
        ] {
            let root = RootRecord {
                instance: InstanceGeneration(77),
                checkpoint_txg: CheckpointTxg(txg),
                rollback_floor: CheckpointTxg(floor),
                ..output.root
            };
            pool.devices.get_mut(&device).unwrap().write(
                DeviceOffsetInBytes(offset),
                &encoded_root(&root, immutable.sizes.physical_block_size as usize),
            );
        }
        assert_eq!(
            effective_rollback_floor(
                &pool,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier
            ),
            CheckpointTxg(3),
            "同盘较旧但未被抛弃的根不能压过该盘最新根的 F"
        );
    }
}
