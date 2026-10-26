"""m4-i315-r3 攻方腿：在仓副本的池级 checker（walk.rs）上打「没有位置项可比」那一格的四种判法开关、甲二外给隔离单元、
违例点名的（盘, 槽）收集（只改副本）。用法：python3 -I apply_r3_checker.py <副本根>
线程局部开关 attack_set_arm(n)：0 退回自证（今天）；1 自证加代号核对（自证过、头可用、头里诞生代号等于这一份那条未释放记录的分配代）；
2 丁 两份互比（自证过，且另一块盘同槽同跨度那一份读得出、整单元 CRC-32C 与这一份相同）；3 两者叠用（1 与 2 都过）。
attack_set_external_quarantine(单元集)：甲二，I-3.1 / I-3.11 / I-3.15 三处都把这些（起点槽, 跨度）当隔离单元。
attack_take_noloc_hits()：这一格每走到一次记一行（自证、头里诞生代号、记录分配代、另一份同不同）。
attack_take_culprits()：违例点名的（不变量, 盘, 槽）——I-3.1 / I-3.11 是没被豁免的「已分配、没有根引用」单元（按调用认是哪一条），
I-3.15 是判红的那条记录。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])

def once(text, old, new, label):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"{label}: 旧串命中 {n} 次（要 1 次）")
    return text.replace(old, new)

walk = root / "crates/singlefs-checker/src/walk.rs"
t = walk.read_text()
anchor = "\n/// 隔离的记录（D19（块指针的结构与宽度预算） 已定项 5：释放之前读盘核出对不上的那个单元，每块盘上的分配记录都留在已分配，\n"
helpers = '''
thread_local! {
    static ATTACK_ARM: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
    static ATTACK_Q31_TAG: std::cell::Cell<&'static str> = const { std::cell::Cell::new("I-3.1") };
    static ATTACK_EXTERNAL_QUARANTINE: std::cell::RefCell<BTreeSet<(u64, u64)>> = const { std::cell::RefCell::new(BTreeSet::new()) };
    static ATTACK_NOLOC_HITS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    static ATTACK_CULPRITS: std::cell::RefCell<Vec<(&'static str, u32, u64)>> = const { std::cell::RefCell::new(Vec::new()) };
}
/// 攻方腿副本：这一线程上「没有位置项可比」那一格取哪一种判法（0 自证、1 加代号核对、2 两份互比、3 叠用）。
pub fn attack_set_arm(arm: u8) {
    ATTACK_ARM.with(|cell| cell.set(arm));
}
fn attack_set_q31_tag(tag: &'static str) {
    ATTACK_Q31_TAG.with(|cell| cell.set(tag));
}
/// 攻方腿副本：这一线程上之后的判定把这几个单元（起点槽、跨度）当隔离的单元（甲二）。
pub fn attack_set_external_quarantine(units: BTreeSet<(u64, u64)>) {
    ATTACK_EXTERNAL_QUARANTINE.with(|set| *set.borrow_mut() = units);
}
/// 攻方腿副本：取走这一线程上「没有位置项可比」那一格被走到的记录。
pub fn attack_take_noloc_hits() -> Vec<String> {
    ATTACK_NOLOC_HITS.with(|hits| std::mem::take(&mut *hits.borrow_mut()))
}
/// 攻方腿副本：取走这一线程上违例点名的（不变量, 盘, 槽）。
pub fn attack_take_culprits() -> Vec<(&'static str, u32, u64)> {
    ATTACK_CULPRITS.with(|culprits| std::mem::take(&mut *culprits.borrow_mut()))
}
fn attack_note_culprit(invariant: &'static str, device: u32, slot: u64) {
    ATTACK_CULPRITS.with(|culprits| culprits.borrow_mut().push((invariant, device, slot)));
}
fn attack_externally_quarantined(slot: u64, span_slots: u64) -> bool {
    ATTACK_EXTERNAL_QUARANTINE.with(|set| set.borrow().contains(&(slot, span_slots)))
}
fn attack_noloc_copy_matches(
    reader: &dyn ImageReader,
    unit: &[u8],
    device: u32,
    slot: u64,
    span_slots: u64,
    record_generation: Option<u64>,
    which: &str,
) -> bool {
    let self_check_passes = check_unit(unit).is_ok();
    let birth = unit
        .get(..UNIT_HEADER_SCAN_BYTES)
        .and_then(birth_txg_in_a_usable_unit_header);
    let generation_matches = self_check_passes && birth.is_some() && birth == record_generation;
    let other_device = u32::from(device == 0);
    let other_copy = usize::try_from(span_slots * SLOT_BYTES)
        .ok()
        .and_then(|unit_bytes| reader.read(other_device, slot * SLOT_BYTES, unit_bytes));
    let other_same = other_copy
        .as_deref()
        .is_some_and(|other| crc32_castagnoli_table(other) == crc32_castagnoli_table(unit));
    let copies_match = self_check_passes && other_same;
    ATTACK_NOLOC_HITS.with(|hits| {
        hits.borrow_mut().push(format!(
            "{which}|dev={device}|slot={slot}|span={span_slots}|self={self_check_passes}|birth={birth:?}|gen={record_generation:?}|other_same={other_same}"
        ));
    });
    match ATTACK_ARM.with(std::cell::Cell::get) {
        1 => generation_matches,
        2 => copies_match,
        3 => generation_matches && copies_match,
        _ => self_check_passes,
    }
}
'''
t = once(t, anchor, helpers + anchor, "helpers")
# I-3.1 / I-3.11 那一份：记录分配代、None 臂、甲二、没豁免的单元记成违例点名的槽
t = once(t, "    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();\n",
         "    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();\n    let mut attack_generation: BTreeMap<(u32, u64), u64> = BTreeMap::new();\n    let attack_tag = ATTACK_Q31_TAG.with(std::cell::Cell::get);\n", "q31-decl")
t = once(t, "            unreferenced_units\n                .entry((record.slot, record.span_slots))\n                .or_default()\n                .push(record.device);\n",
         "            attack_generation.insert((record.device, record.slot), record.generation);\n            unreferenced_units\n                .entry((record.slot, record.span_slots))\n                .or_default()\n                .push(record.device);\n", "q31-gen")
t = once(t, "                // 这一格条款没写，照改之前的读法退回自证（magic、头校验和、载荷 CRC）——交主 agent（实七报告）。\n                None => check_unit(&unit).is_ok(),",
         "                // 这一格条款没写，照改之前的读法退回自证（magic、头校验和、载荷 CRC）——交主 agent（实七报告）。\n                None => attack_noloc_copy_matches(reader, &unit, device, slot, span_slots, attack_generation.get(&(device, slot)).copied(), \"Q31\"),", "q31-none")
t = once(t, "        let some_copy_fails = devices_of_the_unit\n            .iter()\n            .any(|device| !copy_matches_its_location_entry(*device));\n        if some_copy_fails {\n",
         "        let some_copy_fails = attack_externally_quarantined(slot, span_slots)\n            || devices_of_the_unit\n                .iter()\n                .any(|device| !copy_matches_its_location_entry(*device));\n        if !some_copy_fails {\n            for device in &devices_of_the_unit {\n                attack_note_culprit(attack_tag, *device, slot);\n            }\n        }\n        if some_copy_fails {\n", "q31-ext")
t = once(t, "    let quarantined_exempted_against_every_walked_version = quarantined_slots_exempted_per_device(\n",
         "    attack_set_q31_tag(\"I-3.1\");\n    let quarantined_exempted_against_every_walked_version = quarantined_slots_exempted_per_device(\n", "tag-all")
t = once(t, "    let quarantined_exempted_against_the_newest_root = quarantined_slots_exempted_per_device(\n",
         "    attack_set_q31_tag(\"I-3.11\");\n    let quarantined_exempted_against_the_newest_root = quarantined_slots_exempted_per_device(\n", "tag-newest")
# I-3.15 那一份：函数多收一张（盘, 槽）→ 分配代的表，None 臂换成同一个判法；调用处加甲二与违例点名的槽
t = once(t, '''    location_entry_checksums: &mut LocationEntryChecksumsForQuarantine<'_>,
) -> bool {
    devices_of_the_unit.iter().any(|device| {''', '''    location_entry_checksums: &mut LocationEntryChecksumsForQuarantine<'_>,
    attack_generation: &BTreeMap<(u32, u64), u64>,
) -> bool {
    devices_of_the_unit.iter().any(|device| {''', "i315-sig")
t = once(t, '''            Some(checksums) => checksums.contains(&crc32_castagnoli_table(&unit)),
            None => check_unit(&unit).is_ok(),
        };
        !copy_matches''', '''            Some(checksums) => checksums.contains(&crc32_castagnoli_table(&unit)),
            None => attack_noloc_copy_matches(
                reader,
                &unit,
                *device,
                slot,
                span_slots,
                attack_generation.get(&(*device, slot)).copied(),
                "I315",
            ),
        };
        !copy_matches''', "i315-none")
t = once(t, '''            if some_copy_of_the_unit_fails_its_location_entries(
                reader,
                &devices_of_the_unit,
                slot,
                span_slots,
                location_entry_checksums,
            ) {
                continue;
            }
            for record in records_of_the_unit {
                judged_records += 1;''', '''            let attack_generation: BTreeMap<(u32, u64), u64> = account
                .records
                .iter()
                .filter(|record| !record.is_released)
                .map(|record| ((record.device, record.slot), record.generation))
                .collect();
            if attack_externally_quarantined(slot, span_slots)
                || some_copy_of_the_unit_fails_its_location_entries(
                    reader,
                    &devices_of_the_unit,
                    slot,
                    span_slots,
                    location_entry_checksums,
                    &attack_generation,
                )
            {
                continue;
            }
            for record in records_of_the_unit {
                attack_note_culprit("I-3.15", record.device, slot);
                judged_records += 1;''', "i315-call")
walk.write_text(t)
print("ok checker")
