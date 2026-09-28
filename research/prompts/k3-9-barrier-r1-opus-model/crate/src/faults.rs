//! 方向 ③：点名单元读故障、单盘掉与缺席叠加。第一截的每个状态上换几种读故障跑恢复（看 journal），按签名比两臂。
use std::collections::BTreeMap;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::recovery::{recover, JournalPolicy, PoolReader};
use singlefs_harness::memory_pool::CrashImage;
use singlefs_harness::segments::StepKind;

use crate::common::*;
use crate::scenario;

struct Faulty<'a> {
    inner: &'a CrashImage<'a>,
    hidden: Vec<(u32, u64, u64)>,
    dropped_device: Option<u32>,
}

impl PoolReader for Faulty<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.inner.device_identities().into_iter().filter(|d| Some(d.0) != self.dropped_device).collect()
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        if Some(device.0) == self.dropped_device { return None; }
        self.inner.device_size_in_bytes(device)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        if Some(device.0) == self.dropped_device { return None; }
        let end = offset.0 + length as u64;
        if self.hidden.iter().any(|(d, o, l)| *d == device.0 && offset.0 < o + l && *o < end) {
            return None;
        }
        self.inner.read(device, offset, length)
    }
    fn journal_record_offsets_hint(&self, device: DeviceIdentity, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        if Some(device.0) == self.dropped_device { return Some(Vec::new()); }
        self.inner.journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

pub fn run(args: &[String]) {
    let name = &args[2];
    let width = crate::width_of(&args[3]);
    let full_limit: usize = args[4].parse().unwrap();
    let random: usize = args[5].parse().unwrap();
    let built = scenario::build(name, width);
    let (writes, segments) = split(&built.publish_ops, width);
    let (states, _) = crate::crash1_states(&built, full_limit, random);
    let mut signatures: BTreeMap<String, u64> = BTreeMap::new();
    let mut count = 0u64;
    for (segment_index, mask) in &states {
        let persisted = persisted_of(&segments, writes.len(), *segment_index, mask);
        let image = CrashImage { base: &built.base, writes: &writes, persisted: persisted.clone() };
        let persisted_units: Vec<usize> = (0..writes.len()).filter(|w| persisted[*w] && writes[*w].kind == StepKind::UnitWrite).collect();
        let loc = |w: usize| (writes[w].device.0, writes[w].offset.0, writes[w].length_in_bytes());
        let mut models: Vec<(String, Vec<(u32, u64, u64)>, Option<u32>)> = vec![("none".into(), vec![], None)];
        if let Some(w) = persisted_units.iter().find(|w| writes[**w].device.0 == 0) { models.push(("unit-copy-dev0-unreadable".into(), vec![loc(*w)], None)); }
        if let Some(w) = persisted_units.iter().find(|w| writes[**w].device.0 == 1) { models.push(("unit-copy-dev1-unreadable".into(), vec![loc(*w)], None)); }
        models.push(("device1-dropped".into(), vec![], Some(1)));
        models.push(("device0-dropped".into(), vec![], Some(0)));
        models.push(("floor-root-slot-unreadable".into(), vec![built.floor_root_location], None));
        models.push(("floor-records-unreadable".into(), built.floor_record_locations.clone(), None));
        let mut both = built.floor_record_locations.clone();
        both.push(built.floor_root_location);
        models.push(("floor-root-and-records-unreadable".into(), both, None));
        for (model, hidden, dropped) in models {
            let reader = Faulty { inner: &image, hidden, dropped_device: dropped };
            let report = recover(&reader, JournalPolicy::Consult);
            let sig = signature(&report, &built.versions, Some(built.floor), None, &[], (false, false));
            *signatures.entry(format!("{model} {sig}")).or_insert(0) += 1;
            count += 1;
        }
    }
    for (sig, n) in &signatures {
        println!("SIG {n} {sig}");
    }
    println!("FAULT-RECOVERIES {count} states={}", states.len());
}
