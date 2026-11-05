//! 攻方腿 s2 的编解码差分：同一份种子、同一份输入喂给改前（HEAD）与改后（冻结副本）的 core，逐行打出结局，两边逐行比。
//! 只调两边签名相同的公开函数；panic 记成 PANIC 一行，不让它停整份。输出路径取环境变量 ATTACK_OUT。
use std::fmt::Debug;
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
use singlefs_core::bytes::ByteReader;
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::{JournalRecord, NamedUnit, JOURNAL_HEADER_CHECKSUM_OFFSET};
use singlefs_core::logged_operations::{logged_operation_entries_of_the_root_entries, LoggedOperationEntry};
use singlefs_core::pointer::{BirthSequence, DataPointer, LocationEntry, NodePointer, PointerHeadFieldsJudgement};
use singlefs_core::records::*;
use singlefs_core::root_record::{RootRecord, ROOT_CHECKSUM_OFFSET};
use singlefs_core::unit::*;

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
    fn byte(&mut self) -> u8 {
        match self.below(20) {
            0..=10 => 0,
            11..=13 => 0xFF,
            14..=16 => self.below(4) as u8,
            _ => self.next() as u8,
        }
    }
    fn bytes(&mut self, length: usize) -> Vec<u8> {
        (0..length).map(|_| self.byte()).collect()
    }
    /// 在给定的底子上改 1–4 个字节，取值偏向边界。
    fn mutated(&mut self, base: &[u8]) -> Vec<u8> {
        let mut bytes = base.to_vec();
        if bytes.is_empty() {
            return bytes;
        }
        for _ in 0..=self.below(4) {
            let at = self.below(bytes.len() as u64) as usize;
            bytes[at] = self.byte();
        }
        bytes
    }
}

struct Out(std::io::BufWriter<std::fs::File>, u64);
impl Out {
    fn line<T: Debug>(&mut self, name: &str, input: &[u8], call: impl FnOnce() -> T) {
        self.1 += 1;
        let result = catch_unwind(AssertUnwindSafe(call));
        let shown = match result {
            Ok(value) => format!("{value:?}"),
            Err(_) => "PANIC".to_string(),
        };
        writeln!(self.0, "{} {} {} {}", self.1, name, hex(input), shown).unwrap();
    }
}

fn hex(bytes: &[u8]) -> String {
    if bytes.len() > 200 {
        return format!("len{}crc{:08x}", bytes.len(), crc32_castagnoli(bytes));
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 定长记录与条目的读者：每个宽度附近的长度各取一批随机字节，正好宽度的再从读回成功的样本上改字节。
fn fixed_width_readers(out: &mut Out, random: &mut Random) {
    type Reader = (&'static str, usize, fn(&[u8]) -> String);
    let readers: Vec<Reader> = vec![
        ("inode", 140, |b| format!("{:?}", InodeRecord::parse(b).map(|r| (r, r.to_bytes())))),
        ("tombstone", 40, |b| format!("{:?}", TombstoneRecord::parse(b).map(|r| (r, r.to_bytes())))),
        ("lineage", 24, |b| format!("{:?}", LineageRecord::parse(b).map(|r| (r, r.to_bytes())))),
        ("retirement", 34, |b| format!("{:?}", RetirementRecord::parse(b).map(|r| (r, r.to_bytes())))),
        ("accounting", 34, |b| format!("{:?}", AccountingEntry::parse(b).map(|r| (r, r.to_bytes(), r.key_bytes(), r.sort_key())))),
        ("tree_table", 200, |b| format!("{:?}", TreeTableEntry::parse(b).map(|r| (r, r.to_bytes())))),
        ("tree_table_mark", 200, |b| format!("{:?}", TreeTableEntry::parse_with_destruction_mark(b))),
        ("root_pointer_bytes", 200, |b| format!("{:?}", if b.len() == 200 { TreeTableEntry::root_pointer_bytes(b).to_vec() } else { vec![] })),
        ("inode_internal", 120, |b| format!("{:?} {:?}", parse_inode_internal_entry(b), parse_inode_internal_entry_or_refusal(b))),
        ("extent", 112, |b| format!("{:?}", parse_extent_record(b))),
        ("mapping_entry", 55, |b| format!("{:?}", parse_mapping_entry(b))),
        ("mapping_key", 27, |b| format!("{:?} {:?}", birth_txg_of_a_mapping_key(b), if b.len() == 27 { Some(mapping_key_sort_key(b)) } else { None })),
        ("logged_op", 44, |b| format!("{:?}", LoggedOperationEntry::parse(b).map(|r| (r, r.to_bytes())))),
        ("deadlist", 43, |b| format!("{:?}", DeadlistEntry::parse(b).map(|r| (r.clone(), r.key_bytes(), r.birth_txg())))),
        ("location", 14, |b| format!("{:?}", if b.len() >= 14 { Some(LocationEntry::read_from(&mut ByteReader::at(b, 0))) } else { None })),
        ("node_pointer", 120, |b| {
            if b.len() < 120 { return "short".into(); }
            let mut a = ByteReader::at(b, 0);
            let mut c = ByteReader::at(b, 0);
            let mut d = ByteReader::at(b, 0);
            format!("{:?} {:?} {:?} {}",
                NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(&mut a),
                NodePointer::read_with_head_fields_judgement_from(&mut c, PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion),
                DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(&mut d),
                a.position())
        }),
    ];
    for (name, width, reader) in readers {
        let mut successes: Vec<Vec<u8>> = vec![vec![0u8; width]];
        if width == 200 { let mut seed = vec![0u8; 200]; seed[8] = 200; successes.push(seed); }
        for length in 0..=width + 3 {
            let tries = if length == width { 4000 } else { 4 };
            for _ in 0..tries {
                let bytes = random.bytes(length);
                let shown = catch_unwind(AssertUnwindSafe(|| reader(&bytes))).unwrap_or_else(|_| "PANIC".into());
                if length == width && (shown.contains("Some(") || shown.contains("Ok(")) {
                    successes.push(bytes.clone());
                }
                out.line(name, &bytes, || shown);
            }
        }
        for _ in 0..40000 {
            let base = successes[random.below(successes.len() as u64) as usize].clone();
            let bytes = random.mutated(&base);
            let shown = catch_unwind(AssertUnwindSafe(|| reader(&bytes))).unwrap_or_else(|_| "PANIC".into());
            if (shown.contains("Some(") || shown.contains("Ok(")) && successes.len() < 2000 {
                successes.push(bytes.clone());
            }
            out.line(name, &bytes, || shown);
        }
    }
}

fn seal_journal(bytes: &mut [u8]) {
    let named = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    if named <= 64 {
        let crc = crc32_castagnoli(&bytes[496..496 + 56 * named]);
        bytes[95..99].copy_from_slice(&crc.to_le_bytes());
    }
    let digest = wide_checksum_with_field_zeroed(bytes, 4096, JOURNAL_HEADER_CHECKSUM_OFFSET);
    bytes[46..78].copy_from_slice(&digest);
}

fn journal_records(out: &mut Out, random: &mut Random) {
    let mut successes: Vec<Vec<u8>> = Vec::new();
    for round in 0..60000 {
        let mut bytes = if round % 2 == 1 && !successes.is_empty() {
            let base = successes[random.below(successes.len() as u64) as usize].clone();
            random.mutated(&base)
        } else {
            let mut fresh = random.bytes(4096);
            fresh[0..4].copy_from_slice(b"SFSJ");
            fresh[4..6].copy_from_slice(&(random.below(3) as u16).to_le_bytes());
            fresh[7] = random.below(4) as u8;
            fresh[8..12].copy_from_slice(&(if random.below(10) == 0 { random.next() as u32 } else { 4096 }).to_le_bytes());
            let named: u32 = match random.below(8) { 0 => 0, 1 => 1, 2 => 63, 3 => 64, 4 => 65, 5 => u32::MAX, _ => random.below(66) as u32 };
            fresh[12..16].copy_from_slice(&named.to_le_bytes());
            fresh[86] = random.below(3) as u8;
            fresh[87..91].copy_from_slice(&(random.below(3) as u32).to_le_bytes());
            for byte in &mut fresh[99..472] { *byte = 0; }
            for _ in 0..random.below(3) { let at = 99 + random.below(373) as usize; fresh[at] = random.byte(); }
            if random.below(2) == 0 { let v = random.below(3) as u8; fresh[4] = 1; fresh[5] = 0; fresh[86] = v.min(1); }
            fresh
        };
        if random.below(8) != 0 {
            seal_journal(&mut bytes);
        }
        let fsid = if random.below(5) == 0 { random.next() } else { u64::from_le_bytes(bytes[472..480].try_into().unwrap()) };
        let parsed = catch_unwind(AssertUnwindSafe(|| JournalRecord::parse(&bytes, fsid)));
        let shown = match &parsed {
            Ok(Some(record)) => {
                successes.push(bytes.clone());
                let again = catch_unwind(AssertUnwindSafe(|| record.to_bytes()));
                format!("{record:?} same_bytes={:?} keys={:?}", again.map(|b| b == bytes),
                    record.named.iter().map(NamedUnit::mapping_key).collect::<Vec<_>>())
            }
            Ok(None) => "None".into(),
            Err(_) => "PANIC".into(),
        };
        out.line("journal", &bytes, || shown);
        out.line("back_chain", &bytes, || singlefs_core::journal::back_chain_of(&bytes[..random.below(4097) as usize]));
    }
    for _ in 0..20000 {
        let ring = match random.below(4) { 0 => random.below(1 << 20), 1 => random.next(), 2 => 4096 * random.below(1000), _ => u64::MAX - random.below(10) };
        let counter = random.next() >> random.below(64);
        out.line("ring", &ring.to_le_bytes(), || (
            catch_unwind(|| singlefs_core::journal::slot_after_the_journal_ring(ring)).map_err(|_| "PANIC"),
            catch_unwind(|| singlefs_core::journal::record_offset(counter, ring)).map_err(|_| "PANIC"), counter));
    }
}

fn root_slots(out: &mut Out, random: &mut Random) {
    let mut successes: Vec<(Vec<u8>, [u8; 16])> = Vec::new();
    for round in 0..60000 {
        let (mut bytes, expected) = if round % 2 == 1 && !successes.is_empty() {
            let (base, expected) = successes[random.below(successes.len() as u64) as usize].clone();
            (random.mutated(&base), expected)
        } else {
            let length = [466usize, 467, 512, 4096, 465][random.below(5) as usize];
            let mut fresh = random.bytes(length);
            if length >= 4 { fresh[0..4].copy_from_slice(b"SFSR"); }
            if length >= 24 { fresh[20..24].copy_from_slice(&(random.below(3) as u32).to_le_bytes()); }
            for byte in &mut fresh[24.min(length)..] { *byte = 0; }
            for _ in 0..random.below(4) { if length > 24 { let at = 24 + random.below(length as u64 - 24) as usize; fresh[at] = random.byte(); } }
            let mut expected = [0u8; 16];
            if length >= 20 { expected.copy_from_slice(&fresh[4..20]); }
            (fresh, expected)
        };
        if bytes.len() >= ROOT_CHECKSUM_OFFSET + 32 && random.below(8) != 0 {
            let digest = wide_checksum_with_field_zeroed(&bytes, bytes.len(), ROOT_CHECKSUM_OFFSET);
            bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        }
        let mut asked = expected;
        if random.below(6) == 0 { asked[random.below(16) as usize] ^= 1; }
        let parsed = catch_unwind(AssertUnwindSafe(|| RootRecord::parse_slot(&bytes, &asked)));
        let shown = match &parsed {
            Ok(Some(root)) => {
                successes.push((bytes.clone(), asked));
                format!("{root:?} same_bytes={:?}", catch_unwind(AssertUnwindSafe(|| root.to_slot(bytes.len()) == bytes)))
            }
            Ok(None) => "None".into(),
            Err(_) => "PANIC".into(),
        };
        out.line("root_slot", &bytes, || shown);
    }
}

fn fsid(random: &mut Random) -> [u8; 16] {
    let mut fsid = [0u8; 16];
    for byte in &mut fsid { *byte = random.next() as u8; }
    fsid
}

fn reseal_and_parse(out: &mut Out, random: &mut Random, name: &str, unit: &[u8], header_end: usize, parse: fn(&[u8]) -> String) {
    out.line(name, unit, || parse(unit));
    out.line("declared", unit, || (declared_length(unit), catch_unwind(|| data_unit_payload(unit, declared_length(unit)).map(|p| crc32_castagnoli(p))).map_err(|_| "PANIC")));
    for _ in 0..6 {
        let mut bytes = unit.to_vec();
        for _ in 0..=random.below(3) {
            let at = random.below(header_end.min(bytes.len()) as u64) as usize;
            if !(14..46).contains(&at) { bytes[at] = random.byte(); }
        }
        if random.below(10) == 0 { bytes.truncate(random.below(bytes.len() as u64 + 1) as usize); }
        let sealed = catch_unwind(AssertUnwindSafe(|| { let mut copy = bytes.clone(); let end = header_end.min(copy.len()); seal_header_checksum(&mut copy, end); copy }));
        if let Ok(sealed) = sealed {
            out.line(name, &sealed, || parse(&sealed));
            out.line("declared", &sealed, || (catch_unwind(|| declared_length(&sealed)).map_err(|_| "PANIC"), catch_unwind(|| data_unit_payload(&sealed, 32000)).map(|p| p.map(|p| p.len())).map_err(|_| "PANIC")));
        }
    }
}

fn units(out: &mut Out, random: &mut Random) {
    for _ in 0..6000 {
        let fsid = fsid(random);
        let order = WriteOrder { instance: InstanceGeneration(random.next() as u32), transaction: random.next() >> random.below(64) };
        out.line("fsid8", &fsid, || (unit_filesystem_identifier(&fsid), data_key_tail(order), node_key_tail(order.instance, BirthSequence(random.next() as u32))));
        let capacity = data_unit_payload_capacity();
        let payload_length = match random.below(4) { 0 => 0, 1 => capacity, 2 => capacity + 1, _ => random.below(capacity as u64) as usize };
        let payload = random.bytes(payload_length);
        let identity = DataUnitIdentity { tree: TreeIdentifier(random.next()), object: random.next(), object_birth: CheckpointTxg(random.next()), anchor_offset: random.next() };
        let built = catch_unwind(AssertUnwindSafe(|| build_data_unit(identity, CheckpointTxg(random.next()), &fsid, order, &payload)));
        out.line("build_data", &payload, || built.as_ref().map(|b| (b.len(), crc32_castagnoli(b))).map_err(|_| "PANIC"));
        if let Ok(unit) = built { reseal_and_parse(out, random, "parse_data", &unit, 105, |b| format!("{:?}", parse_data_unit(b))); }

        let width = [0u16, 1, 8, 24, 34, 40, 140, 4000, 32661][random.below(9) as usize];
        let count = match random.below(4) { 0 => 0, 1 => 1, _ => random.below(1 + 32661 / u64::from(width.max(1)) + 2) as usize };
        let records: Vec<Vec<u8>> = (0..count).map(|_| { let l = if random.below(50) == 0 { width as usize + 1 } else { width as usize }; random.bytes(l) }).collect();
        let identity = PackedIdentity { birth_tree: TreeIdentifier(random.next()), record_type: random.below(5) as u16, container: random.next(), container_birth: CheckpointTxg(random.next()) };
        let built = catch_unwind(AssertUnwindSafe(|| build_packed_unit(identity, width, &records, CheckpointTxg(random.next()), &fsid, order, BirthSequence(random.next() as u32))));
        out.line("build_packed", &[width as u8, count as u8], || built.as_ref().map(|b| (b.len(), crc32_castagnoli(b))).map_err(|_| "PANIC"));
        if let Ok(unit) = built { reseal_and_parse(out, random, "parse_packed", &unit, 107, |b| format!("{:?}", parse_packed_unit(b))); }

        let key_width = [0usize, 1, 8, 16, 27, 43, 64, 255, 16000][random.below(9) as usize];
        let entry_width = match random.below(3) { 0 => key_width as u16, 1 => (key_width + random.below(100) as usize) as u16, _ => random.below(300) as u16 };
        out.line("capacity", &[], || (catch_unwind(|| index_node_header_bytes(key_width)).map_err(|_| "PANIC"), catch_unwind(|| index_node_entry_capacity(key_width, entry_width as usize)).map_err(|_| "PANIC")));
        let count = match random.below(4) { 0 => 0, 1 => 1, _ => random.below(1 + 32000 / u64::from(entry_width.max(1))) as usize };
        let entries: Vec<Vec<u8>> = (0..count).map(|_| { let l = if random.below(50) == 0 { entry_width as usize + 1 } else { entry_width as usize }; random.bytes(l) }).collect();
        let smallest_length = if random.below(30) == 0 { key_width + 1 } else { key_width }; let smallest = random.bytes(smallest_length);
        let largest = random.bytes(key_width);
        let built = catch_unwind(AssertUnwindSafe(|| build_index_node(TreeIdentifier(random.next()), random.below(4) as u8, key_width, &smallest, &largest,
            CheckpointTxg(random.next()), &fsid, order.instance, BirthSequence(random.next() as u32), entry_width, &entries)));
        out.line("build_index", &[key_width as u8, entry_width as u8, count as u8], || built.as_ref().map(|b| (b.len(), crc32_castagnoli(b))).map_err(|_| "PANIC"));
        if let Ok(unit) = built { reseal_and_parse(out, random, "parse_index", &unit, 86 + 2 * key_width, |b| format!("{:?}", parse_index_node(b))); }
    }
}

fn logged_operation_roots(out: &mut Out, random: &mut Random) {
    let mut entries: Vec<Vec<u8>> = Vec::new();
    for _ in 0..200000 {
        let mut bytes = random.bytes(44);
        bytes[0] = random.below(4) as u8;
        bytes[1] = 0;
        if LoggedOperationEntry::parse(&bytes).is_ok() { entries.push(bytes); }
        if entries.len() > 300 { break; }
    }
    for _ in 0..20000 {
        let count = random.below(8) as usize;
        let mut chosen: Vec<Vec<u8>> = (0..count).map(|_| entries[random.below(entries.len().max(1) as u64) as usize % entries.len().max(1)].clone()).collect();
        if random.below(4) == 0 { chosen.sort(); }
        if random.below(10) == 0 { let l = random.below(50) as usize; chosen.push(random.bytes(l)); }
        let parsed = catch_unwind(AssertUnwindSafe(|| logged_operation_entries_of_the_root_entries(&chosen)));
        out.line("logged_root", &chosen.concat(), || parsed.map_err(|_| "PANIC"));
    }
}

#[test]
fn codec_outputs_for_the_differential() {
    let path = std::env::var("ATTACK_OUT").expect("ATTACK_OUT");
    let mut out = Out(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), 0);
    std::panic::set_hook(Box::new(|_| {}));
    let mut random = Random(0x9E37_79B9_7F4A_7C15);
    fixed_width_readers(&mut out, &mut random);
    journal_records(&mut out, &mut random);
    root_slots(&mut out, &mut random);
    units(&mut out, &mut random);
    logged_operation_roots(&mut out, &mut random);
    out.0.flush().unwrap();
}
