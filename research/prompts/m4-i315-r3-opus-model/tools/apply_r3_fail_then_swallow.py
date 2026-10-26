"""m4-i315-r3 攻方腿：在 apply_r3_extras.py 之后打（只改副本）。用法：python3 -I apply_r3_fail_then_swallow.py <副本根>
造「旧住户诞生代号恰等于记录分配代」那一形：环境变量 ATTACK_SWALLOW_AFTER_FAIL 设了时，按种子抽到的 WriteFails 一报错，
记下报错之前已落盘的最近 512 次单元写（盘、偏移、长度）；之后头一次落在同一（盘、偏移、长度）上的单元写被吞掉。
失败的发布连分配器一起退回、重发用同一个 txg（`crates/singlefs-core/src/allocator.rs` 的 `record_root_written_by_this_process` 那条断言的说明），
所以重发那次落在同一槽上的新单元与槽上留着的旧单元诞生代号相同；吞掉新单元那一份，盘上就是一个代号相等、内容不同的旧住户。
注入点的种类仍记成 write_fails（归类器那一侧不认它是说谎的设备），这一档只看 checker 在各张盘上的判定。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])

def once(text, old, new, label):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"{label}: 旧串命中 {n} 次（要 1 次）")
    return text.replace(old, new)

fi = root / "crates/singlefs-harness/src/fault_injection.rs"
t = fi.read_text()
t = once(t, "    attack_partner: Option<(DeviceIdentity, DeviceOffsetInBytes, u64)>,\n}",
         "    attack_partner: Option<(DeviceIdentity, DeviceOffsetInBytes, u64)>,\n    attack_recent_unit_writes: std::collections::VecDeque<(DeviceIdentity, DeviceOffsetInBytes, u64)>,\n    attack_targets_after_fail: Option<Vec<(DeviceIdentity, DeviceOffsetInBytes, u64)>>,\n}", "field")
t = once(t, "            attack_partner: None,\n", "            attack_partner: None,\n            attack_recent_unit_writes: std::collections::VecDeque::new(),\n            attack_targets_after_fail: None,\n", "init")
t = once(t, '''        if kind == FaultCallKind::Write && state.attack_partner == Some((device, offset, length)) {''', '''        if kind == FaultCallKind::Write
            && state
                .attack_targets_after_fail
                .as_ref()
                .is_some_and(|targets| targets.contains(&(device, offset, length)))
        {
            state.attack_targets_after_fail = Some(Vec::new());
            return Some(InjectedFault::WriteIsSwallowed);
        }
        if kind == FaultCallKind::Write
            && state.geometry.classify(&RecordedOperation {
                device,
                kind: RecordedOperationKind::Write,
                offset,
                length,
                content_hash: 0,
            }) == StepKind::UnitWrite
        {
            state.attack_recent_unit_writes.push_back((device, offset, length));
            if state.attack_recent_unit_writes.len() > 512 {
                state.attack_recent_unit_writes.pop_front();
            }
        }
        if kind == FaultCallKind::Write && state.attack_partner == Some((device, offset, length)) {''', "take")
t = once(t, '''        if schedule.fault == InjectedFault::WriteIsSwallowed && std::env::var_os("ATTACK_SWALLOW_PARTNER").is_some() {''', '''        if schedule.fault == InjectedFault::WriteFails && std::env::var_os("ATTACK_SWALLOW_AFTER_FAIL").is_some() {
            let mut recent: Vec<(DeviceIdentity, DeviceOffsetInBytes, u64)> = state.attack_recent_unit_writes.iter().copied().collect();
            if recent.last() == Some(&(device, offset, length)) {
                recent.pop();
            }
            state.attack_targets_after_fail = Some(recent);
        }
        if schedule.fault == InjectedFault::WriteIsSwallowed && std::env::var_os("ATTACK_SWALLOW_PARTNER").is_some() {''', "arm")
fi.write_text(t)
print("ok fail-then-swallow")
