"""m4-i315-r3 辩方腿：在仓副本（HEAD）上给「没有位置项可比」那一格打九个臂的开关、日志与外给隔离清单（只改副本）。
用法：python3 -I apply_arms_r3.py <副本根>，再 python3 -I apply_defects_r3.py <副本根>。
臂（环境变量 I315_ARM，或探针调 defense_set_arm；读不到按 today）：
  today    今天：自证（magic、头校验和、载荷 CRC）过就算对得上
  exempt   一律对不上（当隔离豁免，第二轮已淘汰，只当参照）
  gen      自证加代号核对：自证过，且头里诞生代号等于罩住它的未释放记录的分配代
  genplus  gen 再加两样头里已有的东西：头里 fsid 等于本池、头里写序按已发布谓词判成已发布
  cmp      丁 两份互比：自证过，且另一块盘同槽那一份的整单元 CRC-32C 相同
  both     gen 与 cmp 都过
  bothplus genplus 与 cmp 都过
  jrn      journal 点名项比：环里有点名项指着这一槽、诞生代号等于记录的分配代时，拿它带的整单元校验和比；没有这样的点名项退回自证
  jrngen   同 jrn，没有点名项时退回 gen
外给隔离清单（甲二）：线程局部，探针经 defense_set_external_quarantine 设。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])

def once(text, old, new, label):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"{label}: 旧串命中 {n} 次")
    return text.replace(old, new)

walk = root / "crates/singlefs-checker/src/walk.rs"
t = walk.read_text()

helpers = r'''
thread_local! {
    static DEFENSE_LOG: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    static DEFENSE_EXTERNAL_QUARANTINE: std::cell::RefCell<std::collections::BTreeSet<(u64, u64)>> = const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
    static DEFENSE_ARM_OVERRIDE: std::cell::Cell<Option<DefenseArm>> = const { std::cell::Cell::new(None) };
    static DEFENSE_JOURNAL: std::cell::RefCell<BTreeMap<(u32, u64), Vec<(u64, u32)>>> = const { std::cell::RefCell::new(BTreeMap::new()) };
    static DEFENSE_POOL: std::cell::RefCell<(u64, u32, u64, Vec<InstanceTableRow>)> = const { std::cell::RefCell::new((0, 0, 0, Vec::new())) };
}
/// 辩方腿副本：取走这一线程记下的日志。
pub fn defense_take_log() -> Vec<String> {
    DEFENSE_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
fn defense_log(line: String) {
    if std::env::var_os("DEFENSE_DUMP").is_some() {
        DEFENSE_LOG.with(|log| log.borrow_mut().push(line));
    }
}
/// 辩方腿副本：这一线程上之后的判定把这几个单元（起点槽、跨度）当隔离的单元（甲二）。
pub fn defense_set_external_quarantine(units: std::collections::BTreeSet<(u64, u64)>) {
    DEFENSE_EXTERNAL_QUARANTINE.with(|set| *set.borrow_mut() = units);
}
fn defense_externally_quarantined(slot: u64, span: u64) -> bool {
    DEFENSE_EXTERNAL_QUARANTINE.with(|set| set.borrow().contains(&(slot, span)))
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DefenseArm {
    Today,
    Exempt,
    Gen,
    GenPlus,
    Cmp,
    Both,
    BothPlus,
    Jrn,
    JrnGen,
}
pub fn defense_arm_named(name: &str) -> DefenseArm {
    match name {
        "exempt" => DefenseArm::Exempt,
        "gen" => DefenseArm::Gen,
        "genplus" => DefenseArm::GenPlus,
        "cmp" => DefenseArm::Cmp,
        "both" => DefenseArm::Both,
        "bothplus" => DefenseArm::BothPlus,
        "jrn" => DefenseArm::Jrn,
        "jrngen" => DefenseArm::JrnGen,
        _ => DefenseArm::Today,
    }
}
/// 辩方腿副本：这一线程上之后的判定取这个臂（探针在一个进程里把同一张盘按每个臂各判一遍）。
pub fn defense_set_arm(name: &str) {
    DEFENSE_ARM_OVERRIDE.with(|cell| cell.set(Some(defense_arm_named(name))));
}
fn defense_arm() -> DefenseArm {
    DEFENSE_ARM_OVERRIDE
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| defense_arm_named(std::env::var("I315_ARM").as_deref().unwrap_or("today")))
}
/// journal 环里自证过的记录的点名项：(盘, 槽) → [(诞生代号, 位置条目带的整单元校验和)]。
fn defense_note_journal(records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)]) {
    let mut table: BTreeMap<(u32, u64), Vec<(u64, u32)>> = BTreeMap::new();
    for (_, records) in records_by_device {
        for record in records.values() {
            for named in &record.named_entries {
                for (device, slot, checksum) in named.locations {
                    table.entry((device, slot)).or_default().push((named.birth_txg, checksum));
                }
            }
        }
    }
    DEFENSE_JOURNAL.with(|cell| *cell.borrow_mut() = table);
}
fn defense_note_pool(fsid_low: u64, instance: u32, txg: u64, rows: &[InstanceTableRow]) {
    DEFENSE_POOL.with(|cell| *cell.borrow_mut() = (fsid_low, instance, txg, rows.to_vec()));
}
/// 头里 fsid 等于本池，而且头里写序按已发布谓词判成已发布（都是单元头里已有的字段，不动格式）。
fn defense_unit_fits_the_pool(unit: &[u8]) -> bool {
    let (fsid_low, instance, txg, rows) = DEFENSE_POOL.with(|cell| cell.borrow().clone());
    let fsid_offset = match unit.get(6) {
        Some(1) => 83,
        Some(2) => 60 + 2 * usize::from(unit[51]),
        Some(3) => 81,
        _ => return false,
    };
    if unit.len() < UNIT_HEADER_SCAN_BYTES || read_u64(unit, fsid_offset) != fsid_low {
        return false;
    }
    let Some(identity) = birth_identity_in_the_header(unit) else { return false };
    PublishedPredicate {
        mount_root_instance: instance,
        mount_root_checkpoint_txg: txg,
        instance_table_rows: &rows,
    }
    .holds_for(identity.write_order)
}
/// 没有位置项可比时，一份单元算不算「对得上」。九个臂见文件头。
fn defense_copy_matches_without_a_location_entry(
    reader: &dyn ImageReader,
    unit: &[u8],
    device: u32,
    slot: u64,
    span_slots: u64,
    record_generation: Option<u64>,
) -> bool {
    let arm = defense_arm();
    let self_ok = check_unit(unit).is_ok();
    let birth = unit
        .get(..UNIT_HEADER_SCAN_BYTES)
        .and_then(birth_txg_in_a_usable_unit_header);
    let gen_ok = self_ok && birth.is_some() && birth == record_generation;
    let fits = gen_ok && defense_unit_fits_the_pool(unit);
    let own_crc = crc32_castagnoli_table(unit);
    let cmp_ok = self_ok && {
        let other = reader.devices().into_iter().find(|candidate| *candidate != device);
        other
            .and_then(|other_device| {
                usize::try_from(span_slots * SLOT_BYTES)
                    .ok()
                    .and_then(|unit_bytes| reader.read(other_device, slot * SLOT_BYTES, unit_bytes))
            })
            .is_some_and(|other_copy| crc32_castagnoli_table(&other_copy) == own_crc)
    };
    let journaled: Option<bool> = record_generation.and_then(|generation| {
        DEFENSE_JOURNAL.with(|cell| {
            let table = cell.borrow();
            let expected: Vec<u32> = table
                .get(&(device, slot))
                .map(|entries| {
                    entries
                        .iter()
                        .filter(|(birth_txg, _)| *birth_txg == generation)
                        .map(|(_, checksum)| *checksum)
                        .collect()
                })
                .unwrap_or_default();
            (!expected.is_empty()).then(|| expected.contains(&own_crc))
        })
    });
    defense_log(format!(
        "NOENTRY|dev={device}|slot={slot}|self_check={self_ok}|header_birth={birth:?}|record_generation={record_generation:?}|gen={gen_ok}|fits={fits}|cmp={cmp_ok}|journal={journaled:?}"
    ));
    match arm {
        DefenseArm::Today => self_ok,
        DefenseArm::Exempt => false,
        DefenseArm::Gen => gen_ok,
        DefenseArm::GenPlus => fits,
        DefenseArm::Cmp => cmp_ok,
        DefenseArm::Both => gen_ok && cmp_ok,
        DefenseArm::BothPlus => fits && cmp_ok,
        DefenseArm::Jrn => journaled.unwrap_or(self_ok),
        DefenseArm::JrnGen => journaled.unwrap_or(gen_ok),
    }
}

/// 隔离的记录（D19（块指针的结构与宽度预算） 已定项 5：释放之前读盘核出对不上的那个单元，每块盘上的分配记录都留在已分配，
'''
t = once(t, "\n/// 隔离的记录（D19（块指针的结构与宽度预算） 已定项 5：释放之前读盘核出对不上的那个单元，每块盘上的分配记录都留在已分配，\n", helpers, "helpers")

# 站点 A：I-3.1 / I-3.11 的隔离豁免
old_push = '''            unreferenced_units
                .entry((record.slot, record.span_slots))
                .or_default()
                .push(record.device);'''
new_push = old_push + '''
            generation_of_each_unreferenced_record.insert((record.device, record.slot), record.generation);'''
t = once(t, old_push, new_push, "pushA")
t = once(t, '''    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();
    for entry in &tree_table.entries {''', '''    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();
    let mut generation_of_each_unreferenced_record: BTreeMap<(u32, u64), u64> = BTreeMap::new();
    for entry in &tree_table.entries {''', "declA")
t = once(t, "                None => check_unit(&unit).is_ok(),\n            }\n        };\n        let some_copy_fails = devices_of_the_unit\n            .iter()\n            .any(|device| !copy_matches_its_location_entry(*device));",
 '''                None => defense_copy_matches_without_a_location_entry(
                    reader,
                    &unit,
                    device,
                    slot,
                    span_slots,
                    generation_of_each_unreferenced_record
                        .get(&(device, slot))
                        .copied(),
                ),
            }
        };
        let some_copy_fails = defense_externally_quarantined(slot, span_slots)
            || devices_of_the_unit
                .iter()
                .any(|device| !copy_matches_its_location_entry(*device));
        defense_log(format!(
            "Q31_GROUP|root_txg={}|slot={slot}|span={span_slots}|devices={devices_of_the_unit:?}|exempt={some_copy_fails}",
            root.checkpoint_txg
        ));''', "bodyA")

# 站点 B：I-3.15 的隔离判据
old_b = '''fn some_copy_of_the_unit_fails_its_location_entries(
    reader: &dyn ImageReader,
    devices_of_the_unit: &[u32],
    slot: u64,
    span_slots: u64,
    location_entry_checksums: &mut LocationEntryChecksumsForQuarantine<'_>,
) -> bool {'''
new_b = '''fn some_copy_of_the_unit_fails_its_location_entries(
    reader: &dyn ImageReader,
    devices_of_the_unit: &[u32],
    slot: u64,
    span_slots: u64,
    location_entry_checksums: &mut LocationEntryChecksumsForQuarantine<'_>,
    generation_of_each_unreleased_record: &BTreeMap<(u32, u64), u64>,
) -> bool {
    if defense_externally_quarantined(slot, span_slots) {
        return true;
    }'''
t = once(t, old_b, new_b, "sigB")
t = once(t, '''            None => check_unit(&unit).is_ok(),
        };
        !copy_matches
    })''', '''            None => defense_copy_matches_without_a_location_entry(
                reader,
                &unit,
                *device,
                slot,
                span_slots,
                generation_of_each_unreleased_record
                    .get(&(*device, slot))
                    .copied(),
            ),
        };
        !copy_matches
    })''', "bodyB")
old_call = '''            if some_copy_of_the_unit_fails_its_location_entries(
                reader,
                &devices_of_the_unit,
                slot,
                span_slots,
                location_entry_checksums,
            ) {
                continue;
            }
            for record in records_of_the_unit {
                judged_records += 1;'''
new_call = '''            let generation_of_each_unreleased_record: BTreeMap<(u32, u64), u64> = account
                .records
                .iter()
                .filter(|record| !record.is_released)
                .map(|record| ((record.device, record.slot), record.generation))
                .collect();
            let quarantined = some_copy_of_the_unit_fails_its_location_entries(
                reader,
                &devices_of_the_unit,
                slot,
                span_slots,
                location_entry_checksums,
                &generation_of_each_unreleased_record,
            );
            defense_log(format!(
                "I315_GROUP|{}|slot={slot}|span={span_slots}|no_key={no_mapping_key_of_the_version_names_the_slot}|group={devices_of_the_unit:?}|quarantined={quarantined}",
                version.name
            ));
            if quarantined {
                continue;
            }
            for record in records_of_the_unit {
                defense_log(format!(
                    "I315_RED|{}|dev={}|slot={slot}|span={span_slots}",
                    version.name, record.device
                ));
                judged_records += 1;'''
t = once(t, old_call, new_call, "callB")

# 钩子：journal 点名项表与池的事实
t = once(t, '''    let journal_records_by_device =
        scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low);
''', '''    let journal_records_by_device =
        scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low);
    defense_note_journal(&journal_records_by_device);
''', "journal-hook")
t = once(t, '''    let content_units =
        scanned_content_units(reader, &geometry, filesystem_identifier_low, &published);
''', '''    defense_note_pool(
        filesystem_identifier_low,
        roots[newest_index].2.instance,
        roots[newest_index].2.checkpoint_txg,
        &instance_table_rows,
    );
    let content_units =
        scanned_content_units(reader, &geometry, filesystem_identifier_low, &published);
''', "pool-hook")
walk.write_text(t)
print("ok walk")
