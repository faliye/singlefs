"""m4-i315-r2 攻方腿：在仓副本上打「没有位置项可比」那一格的臂开关、甲二外给隔离清单与实现缺陷开关（只改副本）。
用法：python3 -I apply_arms.py <副本根>
- checker（walk.rs）：线程局部开关 attack_set_noloc_exempt(bool)——假时这一格退回自证（今天），真时这一份当对不上（当隔离豁免）；
  attack_set_external_quarantine(单元集)——甲二，I-3.1 / I-3.11 / I-3.15 三处都把这些单元当隔离；attack_take_noloc_hits() 取走这一格被走到的记录。
- 实现（transaction.rs）：attack_take_quarantine_log() 取走释放前读盘核核出对不上、交回去隔离的每一份；
  环境变量 ATTACK_DEFECT：one_device（隔离只留核出对不上的那一份，对得上那块盘照常释放）、
  extra_leak（这次发布有隔离时，另把一个没隔离的换下落点在两块盘上都留在已分配，不进隔离清单）、
  leak_every（txg 模 7 余 3 的发布把第一个没隔离的换下落点在两块盘上都留在已分配，不要故障）。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])

def once(text, old, new, label, count=1):
    n = text.count(old)
    if n != count:
        raise SystemExit(f"{label}: 旧串命中 {n} 次（要 {count} 次）")
    return text.replace(old, new)

walk = root / "crates/singlefs-checker/src/walk.rs"
t = walk.read_text()
anchor = "\n/// 一个单元（起点槽、跨度，两盘同槽）记着它的那几块盘上，有一份 checker 自己读出来读不出、或与指着它的位置项里的单元校验和对不上：\n"
helpers = '''
thread_local! {
    static ATTACK_NOLOC_EXEMPT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static ATTACK_EXTERNAL_QUARANTINE: std::cell::RefCell<BTreeSet<(u64, u64)>> = const { std::cell::RefCell::new(BTreeSet::new()) };
    static ATTACK_NOLOC_HITS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}
/// 攻方腿副本：这一线程上「没有位置项可比」那一格取当隔离豁免（真）还是退回自证（假）。
pub fn attack_set_noloc_exempt(exempt: bool) {
    ATTACK_NOLOC_EXEMPT.with(|cell| cell.set(exempt));
}
/// 攻方腿副本：这一线程上之后的判定把这几个单元（起点槽、跨度）当隔离的单元（甲二）。
pub fn attack_set_external_quarantine(units: BTreeSet<(u64, u64)>) {
    ATTACK_EXTERNAL_QUARANTINE.with(|set| *set.borrow_mut() = units);
}
/// 攻方腿副本：取走这一线程上「没有位置项可比」那一格被走到的记录。
pub fn attack_take_noloc_hits() -> Vec<String> {
    ATTACK_NOLOC_HITS.with(|hits| std::mem::take(&mut *hits.borrow_mut()))
}
fn attack_externally_quarantined(slot: u64, span_slots: u64) -> bool {
    ATTACK_EXTERNAL_QUARANTINE.with(|set| set.borrow().contains(&(slot, span_slots)))
}
fn attack_noloc_copy_matches(unit: &[u8], device: u32, slot: u64, which: &str) -> bool {
    let self_check_passes = check_unit(unit).is_ok();
    let exempt = ATTACK_NOLOC_EXEMPT.with(std::cell::Cell::get);
    ATTACK_NOLOC_HITS.with(|hits| {
        hits.borrow_mut()
            .push(format!("{which}|dev={device}|slot={slot}|self_check_passes={self_check_passes}"));
    });
    if exempt { false } else { self_check_passes }
}
'''
t = once(t, anchor, helpers + anchor, "helpers")
t = once(t, "                // 这一格条款没写，照改之前的读法退回自证（magic、头校验和、载荷 CRC）——交主 agent（实七报告）。\n                None => check_unit(&unit).is_ok(),",
         "                // 这一格条款没写，照改之前的读法退回自证（magic、头校验和、载荷 CRC）——交主 agent（实七报告）。\n                None => attack_noloc_copy_matches(&unit, device, slot, \"Q31\"),", "q31-noloc")
t = once(t, "            None => check_unit(&unit).is_ok(),\n        };\n        !copy_matches",
         "            None => attack_noloc_copy_matches(&unit, *device, slot, \"I315\"),\n        };\n        !copy_matches", "i315-noloc")
t = once(t, "        let some_copy_fails = devices_of_the_unit\n            .iter()\n            .any(|device| !copy_matches_its_location_entry(*device));",
         "        let some_copy_fails = attack_externally_quarantined(slot, span_slots)\n            || devices_of_the_unit\n                .iter()\n                .any(|device| !copy_matches_its_location_entry(*device));", "q31-ext")
t = once(t, "            if some_copy_of_the_unit_fails_its_location_entries(\n                reader,\n                &devices_of_the_unit,",
         "            if attack_externally_quarantined(slot, span_slots)\n                || some_copy_of_the_unit_fails_its_location_entries(\n                reader,\n                &devices_of_the_unit,", "i315-ext")
walk.write_text(t)

tx = root / "crates/singlefs-core/src/transaction.rs"
t = tx.read_text()
t = once(t, '''    if !some_copy_failed {
        return Vec::new();
    }
    // 两条位置项各指池里一块不同的盘''', '''    if !some_copy_failed {
        return Vec::new();
    }
    ATTACK_QUARANTINE_LOG.with(|log| {
        log.borrow_mut().extend(
            reading_of_each_copy
                .iter()
                .map(|(location, reading)| (location.device.0, location.slot.0, identity.span_slots(), format!("{reading:?}"))),
        );
    });
    let reading_of_each_copy: Vec<(LocationEntry, QuarantinedCopyReading)> =
        if std::env::var("ATTACK_DEFECT").as_deref() == Ok("one_device") {
            reading_of_each_copy
                .into_iter()
                .filter(|(_, reading)| !matches!(reading, QuarantinedCopyReading::IntactButAnotherCopyFailed))
                .collect()
        } else {
            reading_of_each_copy
        };
    // 两条位置项各指池里一块不同的盘''', "core-log")
t = once(t, '''    for placement in release {
        let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> = quarantine
            .iter()
            .filter(|copy| copy.placement == *placement)
            .map(|copy| copy.device)
            .collect();
        allocator.release_leaving_the_record_allocated_on(
            *placement,
            txg,
            &devices_whose_copy_failed_the_checksum,
        );
    }''', '''    let attack_defect = std::env::var("ATTACK_DEFECT").unwrap_or_default();
    let mut attack_leak_done = false;
    for placement in release {
        let mut devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> = quarantine
            .iter()
            .filter(|copy| copy.placement == *placement)
            .map(|copy| copy.device)
            .collect();
        let leak_here = devices_whose_copy_failed_the_checksum.is_empty()
            && !attack_leak_done
            && match attack_defect.as_str() {
                "extra_leak" => !quarantine.is_empty(),
                "leak_every" => txg.0 % 7 == 3,
                _ => false,
            };
        if leak_here {
            attack_leak_done = true;
            devices_whose_copy_failed_the_checksum = vec![DeviceIdentity(0), DeviceIdentity(1)];
            ATTACK_LEAK_LOG.with(|log| log.borrow_mut().push((placement.slot.0, placement.span, txg.0)));
        }
        allocator.release_leaving_the_record_allocated_on(
            *placement,
            txg,
            &devices_whose_copy_failed_the_checksum,
        );
    }''', "release-loop")
t += '''
thread_local! {
    static ATTACK_QUARANTINE_LOG: std::cell::RefCell<Vec<(u32, u64, u64, String)>> = const { std::cell::RefCell::new(Vec::new()) };
    static ATTACK_LEAK_LOG: std::cell::RefCell<Vec<(u64, u64, u64)>> = const { std::cell::RefCell::new(Vec::new()) };
}
/// 攻方腿副本：取走这一线程上释放前读盘核核出对不上、交回去隔离的每一份（盘、槽、跨度、读出来的样子）。
pub fn attack_take_quarantine_log() -> Vec<(u32, u64, u64, String)> {
    ATTACK_QUARANTINE_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
/// 攻方腿副本：取走这一线程上缺陷开关留下的泄漏（槽、跨度、txg）。
pub fn attack_take_leak_log() -> Vec<(u64, u64, u64)> {
    ATTACK_LEAK_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
'''
tx.write_text(t)
print("ok")
