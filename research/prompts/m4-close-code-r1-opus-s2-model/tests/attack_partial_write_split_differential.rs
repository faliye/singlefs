//! 攻方腿 s2 的部分写切分差分：净荷边界、跨单元、写到尾、带洞与 u64 顶端的取样点，同一串输入喂给改前与改后的 core，逐行比。
use std::io::Write;
use std::num::NonZeroU64;
use std::panic::{catch_unwind, AssertUnwindSafe};

use singlefs_core::address::{DataUnitIndexInFile, FileOffsetInBytes};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::write_request_split::*;

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 }
    fn below(&mut self, bound: u64) -> u64 { self.next() % bound.max(1) }
    /// 偏向净荷边界：k × 净荷 + {−2…+2}，或随机，或 u64 顶端附近。
    fn near_a_boundary(&mut self, capacity: u64, units: u64) -> u64 {
        match self.below(6) {
            0 => self.next() >> self.below(64),
            1 => u64::MAX - self.below(3 * capacity),
            _ => (self.below(units + 2) * capacity).wrapping_add(self.below(5)).wrapping_sub(2),
        }
    }
}

#[test]
fn partial_write_split_outputs_for_the_differential() {
    let path = std::env::var("ATTACK_OUT").expect("ATTACK_OUT");
    let mut out = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
    std::panic::set_hook(Box::new(|_| {}));
    let capacity = data_unit_payload_capacity() as u64;
    let mut random = Random(0x2545_F491_4F6C_DD1D);
    for case in 0..300000u64 {
        let units_before = match random.below(5) { 0 => 0, 1 => 1, _ => random.below(6) };
        // size 与单元数一致（最后一个单元满或不满），或带洞（size 越过单元），或 size 0。
        let size_before = match random.below(4) {
            0 if units_before > 0 => (units_before - 1) * capacity + 1 + random.below(capacity),
            1 => units_before * capacity + random.below(3 * capacity),
            2 => units_before * capacity,
            _ => random.near_a_boundary(capacity, units_before + 2),
        };
        let offset = random.near_a_boundary(capacity, units_before + 3);
        let length = match random.below(5) { 0 => 1, 1 => capacity, 2 => capacity + 1, 3 => random.below(4 * capacity) + 1, _ => random.near_a_boundary(capacity, 4).max(1) };
        let built = catch_unwind(|| PartialWriteOfAFile::new(units_before, size_before, FileOffsetInBytes(offset), NonZeroU64::new(length).unwrap()));
        let mut line = format!("{case} new {units_before} {size_before} {offset} {length} {built:?}");
        if let Ok(Some(write)) = built {
            let facts = catch_unwind(AssertUnwindSafe(|| (write.end_exclusive(), write.size_after_in_bytes(), write.first_written_unit(), write.last_written_unit(),
                write.data_units_after(), write.first_rewritten_unit(), write.rewritten_units())));
            line.push_str(&format!(" facts={facts:?}"));
            if let Ok((_, _, _, _, _, _, rewritten)) = facts {
                let shown_end = rewritten.end.min(rewritten.start.saturating_add(8));
                for unit in rewritten.start..shown_end {
                    let unit = DataUnitIndexInFile(unit);
                    let each = catch_unwind(AssertUnwindSafe(|| (write.data_unit(unit), write.declared_length_after_the_write(unit), write.bytes_written_into(unit))));
                    line.push_str(&format!(" u{}={each:?}", unit.0));
                }
                for probe in [rewritten.start.saturating_sub(1), rewritten.end] {
                    let each = catch_unwind(AssertUnwindSafe(|| write.bytes_written_into(DataUnitIndexInFile(probe))));
                    line.push_str(&format!(" probe{probe}={each:?}"));
                }
            }
        }
        let units_on_disk = random.below(6);
        let read = catch_unwind(|| (data_unit_count_of_a_sequential_write(size_before),
            data_units_read_within_the_file_size(size_before, units_on_disk, DataUnitsOfTheInode::NoneWritten),
            data_units_read_within_the_file_size(size_before, units_on_disk, DataUnitsOfTheInode::SomeWritten)));
        line.push_str(&format!(" read={read:?}"));
        if size_before < 6 * capacity {
            let split = catch_unwind(|| split_sequential_write_into_one_unit_transactions(size_before, 7).len());
            line.push_str(&format!(" split={split:?}"));
        }
        writeln!(out, "{line}").unwrap();
    }
    out.flush().unwrap();
}
