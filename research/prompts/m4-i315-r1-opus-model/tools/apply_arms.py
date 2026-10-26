"""在仓副本上打攻方腿的臂开关与日志（只改副本）。用法：python3 -I apply_arms.py <副本根>"""
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
    static ATTACK_LOG: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}
/// 攻方腿副本：取走这一线程记下的日志。
pub fn attack_take_log() -> Vec<String> {
    ATTACK_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
fn attack_log(line: String) {
    if std::env::var_os("ATTACK_DUMP").is_some() {
        ATTACK_LOG.with(|log| log.borrow_mut().push(line));
    }
}
fn attack_arm() -> String {
    std::env::var("ATTACK_ARM").unwrap_or_else(|_| "today".to_string())
}

/// 一个单元（起点槽、跨度，两盘同槽）记着它的那几块盘上，有一份 checker 自己读出来读不出、或与指着它的位置项里的单元校验和对不上：
'''
t = once(t, "\n/// 一个单元（起点槽、跨度，两盘同槽）记着它的那几块盘上，有一份 checker 自己读出来读不出、或与指着它的位置项里的单元校验和对不上：\n", helpers, "helpers")

old_group = '''        for ((slot, span_slots), records_of_the_unit) in records_off_the_mapping_by_unit {
            let devices_of_the_unit: Vec<u32> = records_of_the_unit
                .iter()
                .map(|record| record.device)
                .collect();
            if some_copy_of_the_unit_fails_its_location_entries(
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
new_group = '''        for ((slot, span_slots), records_of_the_unit) in records_off_the_mapping_by_unit {
            let arm = attack_arm();
            let every_unreleased_device_of_the_unit: Vec<u32> = account
                .records
                .iter()
                .filter(|record| {
                    !record.is_released && record.slot == slot && record.span_slots == span_slots
                })
                .map(|record| record.device)
                .collect();
            let some_key_names_the_slot = account
                .records
                .iter()
                .filter(|record| record.slot == slot)
                .any(|record| keys_at_each_placement.contains_key(&(record.device, slot)));
            let devices_of_the_unit: Vec<u32> = match arm.as_str() {
                "b1" => every_unreleased_device_of_the_unit.clone(),
                "b2" if !some_key_names_the_slot => every_unreleased_device_of_the_unit.clone(),
                _ => records_of_the_unit
                    .iter()
                    .map(|record| record.device)
                    .collect(),
            };
            let fallback: Vec<(u32, bool)> = every_unreleased_device_of_the_unit
                .iter()
                .map(|device| (*device, location_entry_checksums.at(*device, slot).is_none()))
                .collect();
            let quarantined = some_copy_of_the_unit_fails_its_location_entries(
                reader,
                &devices_of_the_unit,
                slot,
                span_slots,
                location_entry_checksums,
            );
            attack_log(format!(
                "I315_GROUP|{}|slot={slot}|span={span_slots}|arm={arm}|group={devices_of_the_unit:?}|all={every_unreleased_device_of_the_unit:?}|key_names_slot={some_key_names_the_slot}|no_location_entry={fallback:?}|quarantined={quarantined}",
                version.name
            ));
            if quarantined {
                continue;
            }
            for record in records_of_the_unit {
                attack_log(format!(
                    "I315_RED|{}|dev={}|slot={slot}|span={span_slots}",
                    version.name, record.device
                ));
                judged_records += 1;'''
t = once(t, old_group, new_group, "group")

old_q = '''        let some_copy_fails = devices_of_the_unit
            .iter()
            .any(|device| !copy_matches_its_location_entry(*device));'''
new_q = '''        let some_copy_fails = devices_of_the_unit
            .iter()
            .any(|device| !copy_matches_its_location_entry(*device));
        let fallback: Vec<(u32, bool)> = devices_of_the_unit
            .iter()
            .map(|device| (*device, location_entry_checksums.at(*device, slot).is_none()))
            .collect();
        attack_log(format!(
            "Q31_GROUP|root_txg={}|slot={slot}|span={span_slots}|devices={devices_of_the_unit:?}|no_location_entry={fallback:?}|exempt={some_copy_fails}",
            root.checkpoint_txg
        ));'''
t = once(t, old_q, new_q, "q31")
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
    ATTACK_QUARANTINE_LOG.with(|log| {
        log.borrow_mut().extend(
            reading_of_each_copy
                .iter()
                .map(|(location, reading)| (location.device.0, location.slot.0, identity.span_slots(), format!("{reading:?}"))),
        );
    });
    // 两条位置项各指池里一块不同的盘'''
t = once(t, old_core, new_core, "core")
t += '''
thread_local! {
    static ATTACK_QUARANTINE_LOG: std::cell::RefCell<Vec<(u32, u64, u64, String)>> = const { std::cell::RefCell::new(Vec::new()) };
}
/// 攻方腿副本：取走这一线程上释放前读盘核核出对不上、交回去隔离的每一份（盘、槽、跨度、读出来的样子）。
pub fn attack_take_quarantine_log() -> Vec<(u32, u64, u64, String)> {
    ATTACK_QUARANTINE_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
'''
tx.write_text(t)
print("ok")
