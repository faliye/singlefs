"""m4-i315-r3 辩方腿：在仓副本上给实现的隔离路径加缺陷开关与隔离清单日志（只改副本，transaction.rs）。
ATTACK_DEFECT 取值：one_device（隔离只留核出对不上那一份，对得上那块盘照常释放）、
extra_leak（有隔离的那次发布另把一个没隔离的换下落点在两块盘上都留在已分配，不进清单）、
leak_every（txg 模 7 余 3 的发布泄漏一个）、leak_once（只在 txg 5 泄漏一个）。
用法：python3 -I apply_defects_r3.py <副本根>"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])

def once(text, old, new, label):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"{label}: 旧串命中 {n} 次")
    return text.replace(old, new)

tx = root / "crates/singlefs-core/src/transaction.rs"
t = tx.read_text()
t = once(t, '''    if !some_copy_failed {
        return Vec::new();
    }
    // 两条位置项各指池里一块不同的盘''', '''    if !some_copy_failed {
        return Vec::new();
    }
    DEFENSE_QUARANTINE_LOG.with(|log| {
        log.borrow_mut().extend(reading_of_each_copy.iter().map(|(location, reading)| {
            (location.device.0, location.slot.0, identity.span_slots(), format!("{reading:?}"))
        }));
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
        allocator.release_leaving_the_record_allocated_on(''', '''    let defense_defect = std::env::var("ATTACK_DEFECT").unwrap_or_default();
    let mut defense_leak_done = false;
    for placement in release {
        let mut devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> = quarantine
            .iter()
            .filter(|copy| copy.placement == *placement)
            .map(|copy| copy.device)
            .collect();
        let leak_here = devices_whose_copy_failed_the_checksum.is_empty()
            && !defense_leak_done
            && match defense_defect.as_str() {
                "extra_leak" => !quarantine.is_empty(),
                "leak_every" => txg.0 % 7 == 3,
                "leak_once" => txg.0 == 5,
                _ => false,
            };
        if leak_here {
            defense_leak_done = true;
            devices_whose_copy_failed_the_checksum = vec![DeviceIdentity(0), DeviceIdentity(1)];
            DEFENSE_LEAK_LOG.with(|log| log.borrow_mut().push((placement.slot.0, placement.span, txg.0)));
        }
        allocator.release_leaving_the_record_allocated_on(''', "release-loop")
t += """
thread_local! {
    static DEFENSE_QUARANTINE_LOG: std::cell::RefCell<Vec<(u32, u64, u64, String)>> = const { std::cell::RefCell::new(Vec::new()) };
    static DEFENSE_LEAK_LOG: std::cell::RefCell<Vec<(u64, u64, u64)>> = const { std::cell::RefCell::new(Vec::new()) };
}
/// 辩方腿副本：取走这一线程上释放前读盘核核出对不上、交回去隔离的每一份（盘、槽、跨度、读出来的样子）。
pub fn defense_take_quarantine_log() -> Vec<(u32, u64, u64, String)> {
    DEFENSE_QUARANTINE_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
/// 辩方腿副本：取走这一线程上缺陷开关留下的泄漏（槽、跨度、txg）。
pub fn defense_take_leak_log() -> Vec<(u64, u64, u64)> {
    DEFENSE_LEAK_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}
"""
tx.write_text(t)
print("ok defects")
