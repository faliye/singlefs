import re,sys
p='tree-core/crates/singlefs-core/src/mount.rs'
s=open(p,encoding='utf-8').read()
old='fn placements_a_rollback_publish_revived_released_in_the_current_account<Device: BlockDevice>('
assert s.count(old)==1
new_impl='fn placements_a_rollback_publish_revived_released_in_the_current_account_impl<Device: BlockDevice>('
s=s.replace(old,new_impl)
wrapper='''
#[allow(clippy::ptr_arg)]
fn placements_a_rollback_publish_revived_released_in_the_current_account<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    newer_root: &RootRecord,
    readable_records_of_its_publish: &[&crate::journal::JournalRecord],
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    current_records: &[AllocationRecord],
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    let arm = std::env::var("SINGLEFS_PROBE_ARM").unwrap_or_default();
    match arm.as_str() {
        "triref" => triref(devices, unit_area_start, newer_root, readable_records_of_its_publish, roots, is_abandoned, current_records),
        "diff" => {
            let i = placements_a_rollback_publish_revived_released_in_the_current_account_impl(devices, unit_area_start, newer_root, readable_records_of_its_publish, roots, is_abandoned, current_records);
            let r = triref(devices, unit_area_start, newer_root, readable_records_of_its_publish, roots, is_abandoned, current_records);
            let si: BTreeSet<_> = i.iter().cloned().collect();
            let sr: BTreeSet<_> = r.iter().cloned().collect();
            let extra: Vec<_> = si.difference(&sr).map(|(d,s,sp)| (d.0, s.0, *sp)).collect();
            let miss: Vec<_> = sr.difference(&si).map(|(d,s,sp)| (d.0, s.0, *sp)).collect();
            eprintln!("TRIDIFF txg={} equal={} impl={} ref={} extra_in_impl={:?} missing_in_impl={:?}", newer_root.checkpoint_txg.0, extra.is_empty() && miss.is_empty(), si.len(), sr.len(), extra, miss);
            i
        }
        "narrow" => {
            // pre-tri: the code before the lm patch, as in jk-round "implemented"
            narrow(devices, unit_area_start, newer_root, readable_records_of_its_publish, roots, is_abandoned, current_records)
        }
        _ => placements_a_rollback_publish_revived_released_in_the_current_account_impl(devices, unit_area_start, newer_root, readable_records_of_its_publish, roots, is_abandoned, current_records),
    }
}

fn triref_state<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    txg: CheckpointTxg,
    records: &[&crate::journal::JournalRecord],
) -> (Option<CheckpointTxg>, bool) {
    let packed_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mut unreadable = false;
    let mut found = None;
    for named in records.iter().flat_map(|record| record.named.iter()).filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED) {
        let Ok(bytes) = read_unit_via_locations(devices, &named.locations, packed_unit_bytes) else { unreadable = true; continue; };
        let Ok(packed) = crate::unit::parse_packed_unit(&bytes) else { unreadable = true; continue; };
        if packed.identity.record_type != crate::unit::PACKED_TYPE_LINEAGE { continue; }
        for record_bytes in packed.records {
            if let Some(lineage) = LineageRecord::parse(&record_bytes) {
                if lineage.rollback_txg == txg && found.is_none() { found = Some(lineage.rollback_target_txg); }
            }
        }
    }
    (found, unreadable)
}

fn triref<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    newer_root: &RootRecord,
    records: &[&crate::journal::JournalRecord],
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    current_records: &[AllocationRecord],
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    let all: Vec<(DeviceIdentity, SlotNumber, u64)> = current_records.iter().filter(|r| r.is_released).map(|r| (r.device, r.slot, u64::from(r.span_slots))).collect();
    let (named_target, unreadable) = triref_state(devices, newer_root.checkpoint_txg, records);
    let target = match named_target {
        Some(t) => t,
        None => return if unreadable { all } else { Vec::new() },
    };
    let r_old_account = roots.iter().filter(|root| !is_abandoned(root) && root.checkpoint_txg == target).find_map(|rt| placements_referenced_by_root(devices, unit_area_start, rt));
    if r_old_account.is_none() { return all; }
    narrow_tail(devices, unit_area_start, target, roots, is_abandoned, current_records)
}

fn narrow_tail<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    target: CheckpointTxg,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    current_records: &[AllocationRecord],
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    let released: BTreeSet<(DeviceIdentity, SlotNumber)> = current_records.iter().filter(|r| r.is_released).map(|r| (r.device, r.slot)).collect();
    roots.iter().filter(|root| !is_abandoned(root) && root.checkpoint_txg == target).filter_map(|rt| placements_referenced_by_root(devices, unit_area_start, rt)).flatten().filter(|(d, s, _)| released.contains(&(*d, *s))).collect()
}

fn narrow<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    newer_root: &RootRecord,
    records: &[&crate::journal::JournalRecord],
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    current_records: &[AllocationRecord],
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    let (named_target, _unreadable) = triref_state(devices, newer_root.checkpoint_txg, records);
    let Some(target) = named_target else { return Vec::new(); };
    narrow_tail(devices, unit_area_start, target, roots, is_abandoned, current_records)
}
'''
# insert wrapper before the doc comment? simply before "/// 一次发布读得出的记录点名的一个打包单元" line
marker='/// 一次发布读得出的记录点名的一个打包单元（类标签 3）读成了什么。'
assert s.count(marker)==1
s=s.replace(marker, wrapper+'\n'+marker)
open(p,'w',encoding='utf-8').write(s)
