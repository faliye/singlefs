"""m4-i315-r3 攻方腿：在仓副本的实现（transaction.rs）与故障注入层（fault_injection.rs）上打日志与缺陷开关（只改副本）。
用法：python3 -I apply_r3_implementation.py <副本根>
- attack_take_quarantine_log()：释放前读盘核核出对不上、交回去隔离的每一份（盘、槽、跨度、读出来的样子）。发布、删快照、整理、
  管理员回退四条路都经 copies_of_a_released_unit_failing_the_release_checksum_check，所以这一份清单是跨重挂、四条路收齐的（甲二要的那一份）。
- 环境变量 ATTACK_DEFECT（第二轮攻方的四种照搬）：one_device（隔离只留核出对不上的那一份）、extra_leak（有隔离的那次发布另把一个
  没隔离的换下落点在两块盘上都留在已分配，不进隔离清单）、leak_every（txg 模 7 余 3 的发布各泄漏一个）、leak_once（只在 txg 5 泄漏一个）。
- 环境变量 ATTACK_SWALLOW_PARTNER：吞掉一次写之后，另一块盘上同偏移、同长度的下一次写也吞掉（两块盘同时丢写；插回去那张盘两次都插）。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])

def once(text, old, new, label):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"{label}: 旧串命中 {n} 次（要 1 次）")
    return text.replace(old, new)

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
                "leak_once" => txg.0 == 5,
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

fi = root / "crates/singlefs-harness/src/fault_injection.rs"
t = fi.read_text()
t = once(t, "    writes_forwarded_across_the_pool: usize,\n    swallowed_writes: Vec<SwallowedWrite>,\n}",
         "    writes_forwarded_across_the_pool: usize,\n    swallowed_writes: Vec<SwallowedWrite>,\n    attack_partner: Option<(DeviceIdentity, DeviceOffsetInBytes, u64)>,\n}", "state-field")
t = once(t, "            writes_forwarded_across_the_pool: 0,\n            swallowed_writes: Vec::new(),\n        })))",
         "            writes_forwarded_across_the_pool: 0,\n            swallowed_writes: Vec::new(),\n            attack_partner: None,\n        })))", "state-init")
t = once(t, '''        let schedule = state.schedule?;
        if schedule.fault.call_kind() != kind || !schedule.matches(device, offset) {''', '''        if kind == FaultCallKind::Write && state.attack_partner == Some((device, offset, length)) {
            state.attack_partner = None;
            return Some(InjectedFault::WriteIsSwallowed);
        }
        let schedule = state.schedule?;
        if schedule.fault.call_kind() != kind || !schedule.matches(device, offset) {''', "partner-take")
t = once(t, '''        let call_ordinal_across_the_pool = state.calls.of(kind);
        state.fired.push(FiredFault {''', '''        let call_ordinal_across_the_pool = state.calls.of(kind);
        if schedule.fault == InjectedFault::WriteIsSwallowed && std::env::var_os("ATTACK_SWALLOW_PARTNER").is_some() {
            state.attack_partner = Some((DeviceIdentity(u32::from(device.0 == 0)), offset, length));
        }
        state.fired.push(FiredFault {''', "partner-arm")
fi.write_text(t)
print("ok implementation")
