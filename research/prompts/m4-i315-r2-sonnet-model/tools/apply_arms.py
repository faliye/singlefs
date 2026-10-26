"""辩方腿（m4-i315-r2）：在仓副本上打「没有位置项可比」那一格的候选开关与日志（只改副本）。
用法：python3 -I apply_arms.py <副本根>
开关（环境变量 I315_ARM，读不到或不认识按 today）：
  today   今天：没有位置项可比 -> 退回自证（check_unit 过就算对得上）
  exempt  当隔离豁免（朴素版）：没有位置项可比 -> 这一份算对不上，单元当隔离
  gen     当隔离豁免（带守卫版）：没有位置项可比 -> 自证过、头可用、头里诞生代号等于罩住它的未释放记录的分配代，才算对得上；
          头可用而代对不上（被吞的写落在复用槽上，旧住户的完整单元）算对不上，单元当隔离
外给隔离清单（甲二 / 丙的模型）：线程局部，探针经 defense_set_external_quarantine 设。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])

def once(text, old, new, label):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"{label}: 旧串命中 {n} 次")
    return text.replace(old, new)

walk = root / "crates/singlefs-checker/src/walk.rs"
t = walk.read_text()

helpers = '''
thread_local! {
    static DEFENSE_LOG: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    static DEFENSE_EXTERNAL_QUARANTINE: std::cell::RefCell<std::collections::BTreeSet<(u64, u64)>> = const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
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
/// 辩方腿副本：这一线程上之后的判定把这几个单元（起点槽、跨度）当隔离的单元（甲二 / 丙的模型）。
pub fn defense_set_external_quarantine(units: std::collections::BTreeSet<(u64, u64)>) {
    DEFENSE_EXTERNAL_QUARANTINE.with(|set| *set.borrow_mut() = units);
}
fn defense_externally_quarantined(slot: u64, span: u64) -> bool {
    DEFENSE_EXTERNAL_QUARANTINE.with(|set| set.borrow().contains(&(slot, span)))
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum DefenseArm {
    Today,
    Exempt,
    Gen,
}
fn defense_arm() -> DefenseArm {
    match std::env::var("I315_ARM").as_deref() {
        Ok("exempt") => DefenseArm::Exempt,
        Ok("gen") => DefenseArm::Gen,
        _ => DefenseArm::Today,
    }
}
/// 没有位置项可比时，一份单元算不算「对得上」（今天：自证；exempt：一律对不上；gen：自证加头里诞生代号等于记录的分配代）。
fn defense_copy_matches_without_a_location_entry(unit: &[u8], record_generation: Option<u64>) -> bool {
    match defense_arm() {
        DefenseArm::Today => check_unit(unit).is_ok(),
        DefenseArm::Exempt => false,
        DefenseArm::Gen => {
            check_unit(unit).is_ok()
                && unit
                    .get(..UNIT_HEADER_SCAN_BYTES)
                    .and_then(birth_txg_in_a_usable_unit_header)
                    .is_some_and(|birth| Some(birth) == record_generation)
        }
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
new_push = '''            unreferenced_units
                .entry((record.slot, record.span_slots))
                .or_default()
                .push(record.device);
            generation_of_each_unreferenced_record.insert((record.device, record.slot), record.generation);'''
t = once(t, old_push, new_push, "pushA")
t = once(t, '''    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();
    for entry in &tree_table.entries {''', '''    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();
    let mut generation_of_each_unreferenced_record: BTreeMap<(u32, u64), u64> = BTreeMap::new();
    for entry in &tree_table.entries {''', "declA")
t = once(t, "                None => check_unit(&unit).is_ok(),\n            }\n        };\n        let some_copy_fails = devices_of_the_unit\n            .iter()\n            .any(|device| !copy_matches_its_location_entry(*device));",
 '''                None => defense_copy_matches_without_a_location_entry(
                    &unit,
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
                &unit,
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
walk.write_text(t)

tx = root / "crates/singlefs-core/src/transaction.rs"
t = tx.read_text()
old_core = '''    if !some_copy_failed {
        return Vec::new();
    }
    // 两条位置项各指池里一块不同的盘'''
new_core = '''    if !some_copy_failed {
        return Vec::new();
    }
    DEFENSE_QUARANTINE_LOG.with(|log| {
        log.borrow_mut().extend(reading_of_each_copy.iter().map(|(location, reading)| {
            (location.device.0, location.slot.0, identity.span_slots(), format!("{reading:?}"))
        }));
    });
    // 两条位置项各指池里一块不同的盘'''
t = once(t, old_core, new_core, "core")
t += """
thread_local! {
    static DEFENSE_QUARANTINE_LOG: std::cell::RefCell<Vec<(u32, u64, u64, String)>> = const { std::cell::RefCell::new(Vec::new()) };
}
/// 辩方腿副本：取走这一线程上释放前读盘核核出对不上、交回去隔离的每一份（盘、槽、跨度、读出来的样子）。
pub fn defense_take_quarantine_log() -> Vec<(u32, u64, u64, String)> {
    DEFENSE_QUARANTINE_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
"""
tx.write_text(t)
print("ok")
