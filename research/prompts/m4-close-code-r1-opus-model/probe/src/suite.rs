// 差分探针的共用一份：main.rs 把它 include! 进两个模块，`c` 一边指改前的 core（singlefs_core_old），一边指改后的 core。
// 每个函数只用两份 core 都有、签名相同的公开项；输出是 Debug 串（崩了写 PANIC），两边逐字比。
#[allow(unused_imports)]
use c::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration, SlotNumber, TreeIdentifier};
use c::bytes::{ByteReader, ByteWriter};
use c::pointer::{BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHead, PointerHeadFieldsJudgement};
use c::unit::{DataUnitIdentity, PackedIdentity, WriteOrder};
use std::panic::{catch_unwind, AssertUnwindSafe};

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }
    // 边界偏重的 u64：0、1、2^k±1、全 1、随机
    pub fn edgy(&mut self) -> u64 {
        match self.below(8) {
            0 => 0,
            1 => self.below(4),
            2 => { let k = self.below(64); (1u64 << k).wrapping_sub(1 - self.below(3) as u64 % 2) }
            3 => u64::MAX - self.below(3),
            4 => (1u64 << 48) - 1 + self.below(3),
            5 => self.below(256),
            _ => self.next(),
        }
    }
    pub fn bytes(&mut self, len: usize) -> Vec<u8> {
        let mode = self.below(3);
        (0..len).map(|_| match mode {
            0 => self.next() as u8,
            1 => if self.below(8) == 0 { self.next() as u8 } else { 0 },
            _ => if self.below(32) == 0 { self.next() as u8 } else { 0 },
        }).collect()
    }
}

pub fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes { h ^= u64::from(*b); h = h.wrapping_mul(0x100_0000_01b3); }
    h
}

pub fn guard<T, F: FnOnce() -> T>(f: F, show: impl FnOnce(&T) -> String) -> String {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => show(&v),
        Err(e) => {
            let m = if let Some(s) = e.downcast_ref::<String>() { s.clone() }
                else if let Some(s) = e.downcast_ref::<&str>() { (*s).to_string() } else { String::from("?") };
            format!("PANIC⟦{}⟧", m.chars().filter(|ch| *ch != '⟦' && *ch != '⟧').take(60).collect::<String>())
        }
    }
}
fn dbg<T: std::fmt::Debug>(v: &T) -> String { format!("{:?}", v) }
fn hb(v: &Vec<u8>) -> String { format!("len={} fnv={:016x}", v.len(), fnv(v)) }

pub const FSID: [u8; 16] = [7, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

// 位置条目大多造成合法的（设备身份严格升序、两条同槽，I-2.5），少数照随机字节读出来的原样留着
fn fix_locations(locations: &mut [LocationEntry; 2], r: &mut Rng) {
    if r.below(8) == 0 { return; }
    let device = r.below(4) as u32;
    locations[0].device = DeviceIdentity(device);
    locations[1].device = DeviceIdentity(device + 1 + r.below(3) as u32);
    locations[1].slot = locations[0].slot;
}
fn node_pointer(r: &mut Rng) -> NodePointer {
    let b = r.bytes(256);
    let mut p = NodePointer::read_from(&mut ByteReader::at(&b, 0));
    fix_locations(&mut p.locations, r);
    p
}
fn data_pointer(r: &mut Rng) -> DataPointer {
    let b = r.bytes(256);
    let mut p = DataPointer::read_from(&mut ByteReader::at(&b, 0));
    fix_locations(&mut p.locations, r);
    p
}
fn checksum_arguments(input: &[u8], extra: u64) -> (usize, usize) {
    let cover_end = if extra & 1 == 0 { input.len() } else { (extra >> 1 & 0xffff) as usize };
    let field_offset = if (extra >> 17) & 3 != 0 { (extra >> 20) as usize % cover_end.max(1) } else { (extra >> 20 & 0xffff) as usize };
    (cover_end, field_offset)
}
fn logged_entry(r: &mut Rng) -> Option<c::logged_operations::LoggedOperationEntry> {
    let issued = if r.below(3) == 0 { CheckpointTxg(r.below(4)) } else { txg(r) };
    let op = if r.below(2) == 0 {
        let slot = if r.below(2) == 0 { SlotNumber(r.below(1000) * 64) } else { SlotNumber(r.edgy()) };
        let region = c::logged_operations::CompactionRegion::of_one_cluster_segment(DeviceIdentity(r.below(4) as u32), slot).ok()?;
        c::logged_operations::LoggedOperation::Compaction(c::logged_operations::CompactionIntent { region, allocation_generation_at_issue: txg(r), empty_cluster_segments_at_issue: r.edgy() })
    } else {
        c::logged_operations::LoggedOperation::Truncation(c::logged_operations::TruncationIntent { inode: InodeNumber(r.edgy()), object_birth: txg(r), target_size_in_bytes: r.edgy() })
    };
    Some(c::logged_operations::LoggedOperationEntry { issued_txg: issued, operation: op })
}
fn tree_table_entry(r: &mut Rng) -> c::records::TreeTableEntry {
    c::records::TreeTableEntry { kind: r.below(5) as u16, tree: TreeIdentifier(r.edgy()), root: node_pointer(r), previous_snapshot_txg: txg(r), birth_txg: txg(r), head_identifier: r.edgy() }
}
fn txg(r: &mut Rng) -> CheckpointTxg { CheckpointTxg(r.edgy()) }
fn packed_identity(r: &mut Rng) -> PackedIdentity {
    PackedIdentity { birth_tree: TreeIdentifier(r.edgy()), record_type: r.edgy() as u16, container: r.edgy(), container_birth: txg(r) }
}
fn write_order(r: &mut Rng) -> WriteOrder {
    let t = if r.below(10) == 0 { r.edgy() } else { r.below(1u64 << 48) };
    WriteOrder { instance: InstanceGeneration(r.edgy() as u32), transaction: t }
}

/// 解码一侧：同一段字节喂给读者，交出 Debug 串；读者收下的，再把收下的值交给同一边的写者，写出来的与原字节比。
pub fn parse_case(case: &str, input: &[u8], extra: u64) -> String {
    match case {
        "unit_index" => guard(|| c::unit::parse_index_node(input), dbg),
        "unit_packed" => guard(|| c::unit::parse_packed_unit(input), dbg),
        "unit_data" => guard(|| c::unit::parse_data_unit(input), dbg),
        "unit_declared_length" => guard(|| c::unit::declared_length(input), dbg),
        "unit_payload" => guard(|| c::unit::data_unit_payload(input, extra as u16).map(|p| (p.len(), fnv(p))), dbg),
        "root_slot" => guard(|| {
            let p = c::root_record::RootRecord::parse_slot(input, &FSID);
            let back = p.as_ref().map(|x| { let s = x.to_slot(input.len()); (s == input, c::root_record::RootRecord::parse_slot(&s, &FSID) == p) });
            (p, back)
        }, dbg),
        "inode_record" => guard(|| { let p = c::records::InodeRecord::parse(input); let back = p.as_ref().map(|x| x.to_bytes() == input); (p, back) }, dbg),
        "tombstone" => guard(|| { let p = c::records::TombstoneRecord::parse(input); let back = p.as_ref().map(|x| x.to_bytes() == input); (p, back) }, dbg),
        "lineage" => guard(|| { let p = c::records::LineageRecord::parse(input); let back = p.as_ref().map(|x| x.to_bytes() == input); (p, back) }, dbg),
        "retirement" => guard(|| { let p = c::records::RetirementRecord::parse(input); let back = p.as_ref().map(|x| x.to_bytes() == input); (p, back) }, dbg),
        "accounting" => guard(|| { let p = c::records::AccountingEntry::parse(input); let back = p.as_ref().map(|x| (x.to_bytes() == input, x.key_bytes(), x.sort_key())); (p, back) }, dbg),
        "deadlist" => guard(|| { let p = c::records::DeadlistEntry::parse(input); let back = p.as_ref().map(|x| (x.key_bytes(), x.birth_txg())); (p, back) }, dbg),
        "tree_table" => guard(|| { let p = c::records::TreeTableEntry::parse(input); let back = p.as_ref().map(|x| x.to_bytes() == input); (p, back) }, dbg),
        "tree_table_mark" => guard(|| { let p = c::records::TreeTableEntry::parse_with_destruction_mark(input); let back = p.as_ref().map(|(x, m)| x.to_bytes_with_destruction_mark(*m) == input); (p, back) }, dbg),
        "root_pointer_bytes" => guard(|| c::records::TreeTableEntry::root_pointer_bytes(input).to_vec(), dbg),
        "inode_internal" => guard(|| c::records::parse_inode_internal_entry(input), dbg),
        "inode_internal_refusal" => guard(|| { let p = c::records::parse_inode_internal_entry_or_refusal(input); let back = p.as_ref().ok().map(|(k, i, n)| c::records::build_inode_internal_entry(*k, *i, *n) == input); (p, back) }, dbg),
        "extent_record" => guard(|| c::records::parse_extent_record(input), dbg),
        "mapping_entry" => guard(|| { let p = c::records::parse_mapping_entry(input); let back = p.as_ref().map(|(k, l)| c::records::build_mapping_entry(k, *l) == input); (p, back) }, dbg),
        "mapping_sort_key" => guard(|| c::records::mapping_key_sort_key(input), dbg),
        "birth_txg_of_mapping_key" => guard(|| c::records::birth_txg_of_a_mapping_key(input), dbg),
        "logged_entry" => guard(|| { let p = c::logged_operations::LoggedOperationEntry::parse(input); let back = p.as_ref().ok().map(|x| (x.to_bytes() == input, x.key().sort_key(), x.key())); (p, back) }, dbg),
        "journal" => guard(|| {
            let fsid = c::unit::unit_filesystem_identifier(&FSID);
            let p = c::journal::JournalRecord::parse(input, fsid);
            let back = p.as_ref().map(|x| x.to_bytes() == input[..4096]);
            (p, back)
        }, dbg),
        "back_chain" => guard(|| c::journal::back_chain_of(input), dbg),
        "wide_checksum_holds" => guard(|| { let (end, at) = checksum_arguments(input, extra); c::checksum::wide_checksum_field_holds(input, end, at) }, dbg),
        "wide_checksum_zeroed" => guard(|| { let (end, at) = checksum_arguments(input, extra); c::checksum::wide_checksum_with_field_zeroed(input, end, at) }, dbg),
        "node_pointer_judging" => guard(|| { let mut rd = ByteReader::at(input, 0); let p = NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(&mut rd); (p, rd.position()) }, dbg),
        "data_pointer_judging" => guard(|| { let mut rd = ByteReader::at(input, 0); let p = DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(&mut rd); (p, rd.position()) }, dbg),
        "head_judging" => guard(|| { let mut rd = ByteReader::at(input, 0); let p = PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from(&mut rd); (p, rd.position()) }, dbg),
        "node_pointer_judgement_shadow" => guard(|| { let mut rd = ByteReader::at(input, 0); let p = NodePointer::read_with_head_fields_judgement_from(&mut rd, PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger); (p, rd.position()) }, dbg),
        "node_pointer_judgement_first" => guard(|| { let mut rd = ByteReader::at(input, 0); let p = NodePointer::read_with_head_fields_judgement_from(&mut rd, PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion); (p, rd.position()) }, dbg),
        "location_entries" => guard(|| { let mut rd = ByteReader::at(input, 0); let a = LocationEntry::read_from(&mut rd); let b = LocationEntry::read_from(&mut rd); (c::pointer::location_entries_ascend_by_device(&[a, b]).map_err(|e| format!("{:?}", e)), c::pointer::slot_shared_by_both_location_entries(&[a, b]).map_err(|e| format!("{:?}", e))) }, dbg),
        "reader_ops" => guard(|| {
            let mut rd = ByteReader::at(input, (extra % 8) as usize);
            let mut out: Vec<u64> = Vec::new();
            let mut ops = extra >> 3;
            for _ in 0..12 {
                match ops % 6 { 0 => out.push(u64::from(rd.get_u8())), 1 => out.push(u64::from(rd.get_u16())), 2 => out.push(u64::from(rd.get_u32())),
                    3 => out.push(rd.get_six_byte_unsigned()), 4 => out.push(rd.get_u64()), _ => { rd.skip((ops % 5) as usize); out.push(rd.position() as u64) } }
                ops /= 6;
            }
            out
        }, dbg),
        _ => format!("UNKNOWN CASE {case}"),
    }
}

pub const PARSE_CASES: &[&str] = &[
    "unit_index", "unit_packed", "unit_data", "unit_declared_length", "unit_payload", "root_slot", "inode_record", "tombstone",
    "lineage", "retirement", "accounting", "deadlist", "tree_table", "tree_table_mark", "root_pointer_bytes", "inode_internal",
    "inode_internal_refusal", "extent_record", "mapping_entry", "mapping_sort_key", "birth_txg_of_mapping_key", "logged_entry",
    "journal", "back_chain", "wide_checksum_holds", "wide_checksum_zeroed", "node_pointer_judging", "data_pointer_judging",
    "head_judging", "node_pointer_judgement_shadow", "node_pointer_judgement_first", "location_entries", "reader_ops",
];

/// 写者一侧：同一个种子在两边造出同一份值（只经两边相同的读法从随机字节读出指针），交给写者，再交给同一边的读者读回。
pub fn write_case(case: &str, seed: u64) -> String {
    let mut r = Rng(seed | 1);
    let r = &mut r;
    match case {
        "w_journal" => {
            let named_count = match r.below(4) { 0 => r.below(3), 1 => 64 + r.below(3) - 1, _ => r.below(70) } as usize;
            let mut named = Vec::new();
            for _ in 0..named_count { let b = r.bytes(64); named.push(c::journal::NamedUnit::read_from(&mut ByteReader::at(&b, 0))); }
            let place = if r.below(2) == 0 { c::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish } else { c::journal::JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow };
            let marker = if r.below(2) == 0 { c::root_record::UnmountMarker::WrittenByTheUnmountSequence } else { c::root_record::UnmountMarker::NotWrittenByTheUnmountSequence };
            let counter = if r.below(6) == 0 { r.edgy() } else { r.below(1u64 << 48) };
            let rec = c::journal::JournalRecord {
                instance: InstanceGeneration(r.edgy() as u32), counter, checkpoint_txg: txg(r), transaction: r.edgy(), is_commit: r.below(2) == 0,
                ordinal_within_publish: c::journal::JournalRecordOrdinalWithinPublish(r.edgy() as u32), place_in_publish: place,
                back_chain: r.edgy() as u32, filesystem_identifier: c::unit::unit_filesystem_identifier(&FSID),
                new_tree_table: node_pointer(r), new_mapping_root: node_pointer(r), new_tree_identifier_watermark: r.edgy(),
                new_rollback_floor: txg(r), new_instance_table: node_pointer(r), new_allocation_record_tree_root: node_pointer(r),
                new_unmount_marker: marker, new_reclaim_threshold: txg(r), new_pinned_ring_slot: c::root_record::PinnedRingSlot(r.edgy() as u8), named,
            };
            let ordinal = rec.ordinal_within_publish.0;
            let written = guard(|| { let b = rec.to_bytes(); let back = c::journal::JournalRecord::parse(&b, rec.filesystem_identifier); (hb(&b), back.as_ref() == Some(&rec), back.is_some()) }, dbg);
            format!("{written} | ordinal={ordinal}")
        }
        "w_root" => {
            let marker = if r.below(2) == 0 { c::root_record::UnmountMarker::WrittenByTheUnmountSequence } else { c::root_record::UnmountMarker::NotWrittenByTheUnmountSequence };
            let rec = c::root_record::RootRecord {
                filesystem_identifier: FSID, unmount_marker: marker, instance: InstanceGeneration(r.edgy() as u32), checkpoint_txg: txg(r),
                tree_table: node_pointer(r), tree_identifier_watermark: r.edgy(), rollback_floor: txg(r), instance_table: node_pointer(r),
                mapping_root: node_pointer(r), allocation_record_tree_root: node_pointer(r), reclaim_threshold: txg(r),
                pinned_ring_slot: c::root_record::PinnedRingSlot(r.edgy() as u8),
            };
            let slot = match r.below(5) { 0 => 465 + r.below(3) as usize, 1 => 4096, 2 => 16384, 3 => r.below(600) as usize, _ => 466 + r.below(5000) as usize };
            guard(|| { let b = rec.to_slot(slot); let back = c::root_record::RootRecord::parse_slot(&b, &FSID); (hb(&b), back == Some(rec), back.is_some()) }, dbg)
        }
        "w_packed" => {
            let width = match r.below(4) { 0 => 0u16, 1 => r.below(8) as u16, 2 => r.below(400) as u16, _ => r.edgy() as u16 };
            let cap = if width == 0 { 5 } else { (32768 - 107 - 29) / usize::from(width) };
            let count = match r.below(4) { 0 => 0, 1 => cap + r.below(3) as usize, 2 => r.below(cap as u64 + 1) as usize, _ => 1 + r.below(4) as usize };
            let count = if usize::from(width) > 400 { count.min(3) } else { count.min(400) };
            let skew = r.below(20) == 0;
            let records: Vec<Vec<u8>> = (0..count).map(|i| { let w = usize::from(width) + usize::from(skew && i == 0); r.bytes(w) }).collect();
            let id = packed_identity(r);
            let bt = txg(r);
            let wo = write_order(r);
            let bs = BirthSequence(r.edgy() as u32);
            guard(|| { let b = c::unit::build_packed_unit(id, width, &records, bt, &FSID, wo, bs); (hb(&b), c::unit::parse_packed_unit(&b), c::unit::declared_length(&b)) }, dbg)
        }
        "w_index" => {
            let kw = match r.below(4) { 0 => 0usize, 1 => r.below(40) as usize, 2 => 200 + r.below(80) as usize, _ => r.below(9) as usize * 8 };
            let ew = match r.below(5) { 0 => 0u16, 1 => kw as u16, 2 => (kw as u16).wrapping_sub(1), 3 => r.below(300) as u16, _ => kw as u16 + r.below(100) as u16 };
            let cap_guess = if ew == 0 { 3 } else { 16384usize.saturating_sub(115 + 2 * kw) / usize::from(ew) };
            let count = match r.below(4) { 0 => 0, 1 => cap_guess + r.below(3) as usize, 2 => r.below(cap_guess as u64 + 1) as usize, _ => 1 + r.below(3) as usize }.min(600);
            let key_skew = r.below(20) == 0;
            let small = r.bytes(kw + usize::from(key_skew));
            let large = r.bytes(kw);
            let skew = r.below(20) == 0;
            let entries: Vec<Vec<u8>> = (0..count).map(|i| r.bytes(usize::from(ew) + usize::from(skew && i == count - 1))).collect();
            let tree = TreeIdentifier(r.edgy());
            let level = r.edgy() as u8;
            let bt = txg(r);
            let inst = InstanceGeneration(r.edgy() as u32);
            let bs = BirthSequence(r.edgy() as u32);
            guard(|| { let b = c::unit::build_index_node(tree, level, kw, &small, &large, bt, &FSID, inst, bs, ew, &entries); (hb(&b), c::unit::parse_index_node(&b), c::unit::declared_length(&b)) }, dbg)
        }
        "w_index_capacity" => {
            let kw = r.below(300) as usize;
            let ew = r.below(400) as usize;
            guard(|| (c::unit::index_node_header_bytes(kw), c::unit::index_node_entry_capacity(kw, ew)), dbg)
        }
        "w_data" => {
            let len = match r.below(4) { 0 => 0usize, 1 => 32634 + r.below(3) as usize - 1, 2 => r.below(32634) as usize, _ => r.below(64) as usize };
            let payload = r.bytes(len);
            let id = DataUnitIdentity { tree: TreeIdentifier(r.edgy()), object: r.edgy(), object_birth: txg(r), anchor_offset: r.edgy() };
            let bt = txg(r);
            let wo = write_order(r);
            let ask = r.edgy() as u16;
            guard(|| { let b = c::unit::build_data_unit(id, bt, &FSID, wo, &payload); let h = c::unit::parse_data_unit(&b);
                let dl = c::unit::declared_length(&b);
                (hb(&b), h, c::unit::data_unit_payload(&b, dl).map(|p| p == payload.as_slice()), c::unit::data_unit_payload(&b, ask).map(|p| p.len()), c::unit::data_unit_payload_capacity()) }, dbg)
        }
        "w_records" => {
            let dp = data_pointer(r);
            let np = node_pointer(r);
            let id = packed_identity(r);
            let sep = r.edgy();
            let inode = r.edgy();
            let offset = r.edgy();
            let wo = write_order(r);
            let class = r.edgy() as u8;
            let bs = BirthSequence(r.edgy() as u32);
            let inst = InstanceGeneration(r.edgy() as u32);
            let key_len = if r.below(4) == 0 { r.below(40) as usize } else { 27 };
            let key = r.bytes(key_len);
            let b1 = guard(|| { let e = c::records::build_inode_internal_entry(sep, id, np); (hb(&e), c::records::parse_inode_internal_entry(&e)) }, dbg);
            let b2 = guard(|| { let e = c::records::build_extent_record(inode, offset, dp); (hb(&e), c::records::parse_extent_record(&e)) }, dbg);
            let b3 = guard(|| { let e = c::records::build_mapping_entry(&key, dp.locations); (hb(&e), c::records::parse_mapping_entry(&e)) }, dbg);
            let b4 = guard(|| { let k = c::records::mapping_key_for_data(dp.head, wo); (k.clone(), c::records::mapping_key_sort_key(&k), c::records::birth_txg_of_a_mapping_key(&k)) }, dbg);
            let b5 = guard(|| { let k = c::records::mapping_key_for_node(class, np); (k.clone(), c::records::mapping_key_sort_key(&k), c::records::birth_txg_of_a_mapping_key(&k)) }, dbg);
            let b6 = guard(|| (c::records::data_key_tail(wo), c::records::node_key_tail(inst, bs)), dbg);
            format!("{b1} | {b2} | {b3} | {b4} | {b5} | {b6}")
        }
        "w_logged_root" => {
            let n = r.below(5) as usize;
            let mut entries = Vec::new();
            for _ in 0..n {
                let issued = if r.below(3) == 0 { CheckpointTxg(r.below(4)) } else { txg(r) };
                let op = if r.below(2) == 0 {
                    let slot = if r.below(2) == 0 { SlotNumber(r.below(1000) * 64) } else { SlotNumber(r.edgy()) };
                    match c::logged_operations::CompactionRegion::of_one_cluster_segment(DeviceIdentity(r.below(4) as u32), slot) {
                        Ok(region) => c::logged_operations::LoggedOperation::Compaction(c::logged_operations::CompactionIntent { region, allocation_generation_at_issue: txg(r), empty_cluster_segments_at_issue: r.edgy() }),
                        Err(_) => continue,
                    }
                } else {
                    c::logged_operations::LoggedOperation::Truncation(c::logged_operations::TruncationIntent { inode: InodeNumber(r.edgy()), object_birth: txg(r), target_size_in_bytes: r.edgy() })
                };
                entries.push(c::logged_operations::LoggedOperationEntry { issued_txg: issued, operation: op });
            }
            guard(|| { let roots = c::logged_operations::logged_operations_root_entries(&entries); let back = c::logged_operations::logged_operation_entries_of_the_root_entries(&roots);
                let ok_same = back.as_ref().map(|v| *v == entries).unwrap_or(false);
                (roots.iter().map(hb).collect::<Vec<_>>(), back, ok_same, c::logged_operations::live_compaction_intent_of_the_entries(&entries), c::logged_operations::live_truncation_intent_of_the_entries(&entries)) }, dbg)
        }
        "w_writer_ops" => {
            let total = r.below(40) as usize;
            let ops: Vec<(u64, u64)> = (0..10).map(|_| (r.below(7), r.edgy())).collect();
            guard(|| {
                let mut w = ByteWriter::new(total);
                for (op, v) in &ops {
                    match op { 0 => w.put_u8(*v as u8), 1 => w.put_u16(*v as u16), 2 => w.put_u32(*v as u32), 3 => w.put_six_byte_unsigned(*v),
                        4 => w.put_u64(*v), 5 => w.skip((*v % 6) as usize), _ => w.put(&[1, 2, 3][..(*v % 4).min(3) as usize]) }
                }
                let p = w.position();
                (p, w.into_bytes())
            }, dbg)
        }
        _ => format!("UNKNOWN CASE {case}"),
    }
}

pub const WRITE_CASES: &[&str] = &["w_journal", "w_root", "w_packed", "w_index", "w_index_capacity", "w_data", "w_records", "w_logged_root", "w_writer_ops"];

pub fn deleted_inodes_case(seed: u64) -> String {
    let mut r = Rng(seed | 1);
    let n = match r.below(4) { 0 => 0, 1 => 2031 + r.below(3) as usize - 1, _ => r.below(40) as usize };
    let mut keys: Vec<InodeNumber> = Vec::new();
    let mut k = r.below(5);
    for _ in 0..n { k += if r.below(30) == 0 { 0 } else { 1 + r.below(3) }; keys.push(InodeNumber(k)); }
    if r.below(10) == 0 && keys.len() > 1 { keys.swap(0, 1); }
    let entry_lens: Vec<usize> = (0..r.below(4)).map(|_| r.below(12) as usize).collect();
    let raw: Vec<Vec<u8>> = entry_lens.iter().map(|l| r.bytes(*l)).collect();
    let a = guard(|| { let e = c::deleted_inodes::deleted_inodes_root_entries(&keys); let back = e.as_ref().ok().map(|v| c::deleted_inodes::inode_numbers_of_the_deleted_inodes_root_entries(v) == keys); (e.map(|v| v.len()), back) }, dbg);
    let b = guard(|| c::deleted_inodes::inode_numbers_of_the_deleted_inodes_root_entries(&raw), dbg);
    let cap = guard(|| c::deleted_inodes::deleted_inodes_one_root_node_holds(), dbg);
    format!("{a} | {b} | {cap}")
}

/// 只给 main 的解码格造底稿用（只在改前那一边调）：参数都取在写者的前提之内，崩了交 None。
pub fn valid_bytes(case: &str, seed: u64) -> Option<Vec<u8>> {
    let mut r = Rng(seed | 1);
    let r = &mut r;
    let built = catch_unwind(AssertUnwindSafe(|| match case {
        "w_index" => {
            let kw = match r.below(3) { 0 => 8usize, 1 => 16, _ => r.below(41) as usize };
            let ew = (kw + r.below(60) as usize).max(1);
            let cap = c::unit::index_node_entry_capacity(kw, ew);
            let count = match r.below(3) { 0 => cap, 1 => 0, _ => r.below(cap as u64 + 1) as usize };
            let entries: Vec<Vec<u8>> = (0..count).map(|_| r.bytes(ew)).collect();
            let small = r.bytes(kw);
            let large = r.bytes(kw);
            c::unit::build_index_node(TreeIdentifier(r.edgy()), r.below(4) as u8, kw, &small, &large, txg(r), &FSID, InstanceGeneration(r.edgy() as u32), BirthSequence(r.edgy() as u32), ew as u16, &entries)
        }
        "w_packed" => {
            let width = 1 + r.below(200) as usize;
            let cap = (32768 - 107 - 29) / width;
            let count = match r.below(3) { 0 => cap, 1 => 0, _ => r.below(cap as u64 + 1) as usize };
            let records: Vec<Vec<u8>> = (0..count).map(|_| r.bytes(width)).collect();
            let wo = WriteOrder { instance: InstanceGeneration(r.edgy() as u32), transaction: r.below(1u64 << 48) };
            c::unit::build_packed_unit(packed_identity(r), width as u16, &records, txg(r), &FSID, wo, BirthSequence(r.edgy() as u32))
        }
        "w_data" => {
            let len = match r.below(3) { 0 => 32634usize, 1 => 0, _ => r.below(32635) as usize };
            let payload = r.bytes(len);
            let wo = WriteOrder { instance: InstanceGeneration(r.edgy() as u32), transaction: r.below(1u64 << 48) };
            let id = DataUnitIdentity { tree: TreeIdentifier(r.edgy()), object: r.edgy(), object_birth: txg(r), anchor_offset: r.edgy() };
            c::unit::build_data_unit(id, txg(r), &FSID, wo, &payload)
        }
        "w_journal" => {
            let named_count = r.below(65) as usize;
            let mut named = Vec::new();
            for _ in 0..named_count { let b = r.bytes(64); named.push(c::journal::NamedUnit::read_from(&mut ByteReader::at(&b, 0))); }
            let rec = c::journal::JournalRecord {
                instance: InstanceGeneration(r.edgy() as u32), counter: r.below(1u64 << 48), checkpoint_txg: txg(r), transaction: r.edgy(), is_commit: r.below(2) == 0,
                ordinal_within_publish: c::journal::JournalRecordOrdinalWithinPublish(1 + r.below(5) as u32),
                place_in_publish: if r.below(2) == 0 { c::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish } else { c::journal::JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow },
                back_chain: r.edgy() as u32, filesystem_identifier: c::unit::unit_filesystem_identifier(&FSID),
                new_tree_table: node_pointer(r), new_mapping_root: node_pointer(r), new_tree_identifier_watermark: r.edgy(),
                new_rollback_floor: txg(r), new_instance_table: node_pointer(r), new_allocation_record_tree_root: node_pointer(r),
                new_unmount_marker: c::root_record::UnmountMarker::NotWrittenByTheUnmountSequence, new_reclaim_threshold: txg(r),
                new_pinned_ring_slot: c::root_record::PinnedRingSlot(r.below(256) as u8), named,
            };
            let mut b = rec.to_bytes();
            let tail = r.below(3) as usize * 7;
            b.extend(std::iter::repeat(0u8).take(tail));
            b
        }
        "w_root" => {
            let rec = c::root_record::RootRecord {
                filesystem_identifier: FSID, unmount_marker: c::root_record::UnmountMarker::WrittenByTheUnmountSequence, instance: InstanceGeneration(r.edgy() as u32), checkpoint_txg: txg(r),
                tree_table: node_pointer(r), tree_identifier_watermark: r.edgy(), rollback_floor: txg(r), instance_table: node_pointer(r),
                mapping_root: node_pointer(r), allocation_record_tree_root: node_pointer(r), reclaim_threshold: txg(r),
                pinned_ring_slot: c::root_record::PinnedRingSlot(r.below(256) as u8),
            };
            rec.to_slot(466 + r.below(4096 - 466) as usize)
        }
        "w_logged_entry" => match logged_entry(r) { Some(entry) => entry.to_bytes(), None => Vec::new() },
        "w_tree_table" => { let entry = tree_table_entry(r); if r.below(2) == 0 { entry.to_bytes() } else {
            entry.to_bytes_with_destruction_mark(if r.below(2) == 0 { c::records::SnapshotDestructionMark::BeingDestroyed } else { c::records::SnapshotDestructionMark::NotBeingDestroyed }) } }
        _ => Vec::new(),
    }));
    built.ok().filter(|b| !b.is_empty())
}

pub fn reseal(case: &str, bytes: &mut Vec<u8>, header_end: usize) {
    let _ = catch_unwind(AssertUnwindSafe(|| match case {
        "w_journal" => { let f = c::checksum::wide_checksum_with_field_zeroed(bytes, 4096, 46); bytes[46..78].copy_from_slice(&f); }
        "w_root" => { let n = bytes.len(); let f = c::checksum::wide_checksum_with_field_zeroed(bytes, n, 138); bytes[138..170].copy_from_slice(&f); }
        _ => c::unit::seal_header_checksum(bytes, header_end),
    }));
}
