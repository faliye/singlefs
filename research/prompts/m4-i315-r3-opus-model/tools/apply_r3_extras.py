"""m4-i315-r3 攻方腿：在 apply_r3_checker.py、apply_r3_implementation.py 之后打（只改副本）。用法：python3 -I apply_r3_extras.py <副本根>
- checker（walk.rs，I-3.1 / I-3.11 那一份）：有位置项可比而这一份对不上时记一行 LOC（自证、头里诞生代号、记录分配代）：
  位置项还在的隔离单元里旧住户的代号是不是恰等于记录的分配代——「自证加代号核对」推的盲区在转出根环之前就数得到。
- 故障注入（fault_injection.rs）：环境变量 ATTACK_ORDINAL_EARLIER 设了时，按种子抽到的「吞掉第 n 次写」改成第 n−1 次（n 为 1 时不动）；
  与 ATTACK_SWALLOW_PARTNER 一起用，抽到的是一个单元第二份的写时就从第一份吞起、两份一起吞（两块盘同时丢写）。"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])

def once(text, old, new, label):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"{label}: 旧串命中 {n} 次（要 1 次）")
    return text.replace(old, new)

walk = root / "crates/singlefs-checker/src/walk.rs"
t = walk.read_text()
t = once(t, '''                Some(checksums) => checksums.contains(&crc32_castagnoli_table(&unit)),
                // 走过的版本与根环里别的根都没有一条位置项指着这一份''', '''                Some(checksums) => {
                    let matches = checksums.contains(&crc32_castagnoli_table(&unit));
                    if !matches {
                        let birth = unit
                            .get(..UNIT_HEADER_SCAN_BYTES)
                            .and_then(birth_txg_in_a_usable_unit_header);
                        let generation = attack_generation.get(&(device, slot)).copied();
                        ATTACK_NOLOC_HITS.with(|hits| {
                            hits.borrow_mut().push(format!(
                                "LOC|dev={device}|slot={slot}|span={span_slots}|loc_self={}|birth={birth:?}|gen={generation:?}",
                                check_unit(&unit).is_ok()
                            ));
                        });
                    }
                    matches
                }
                // 走过的版本与根环里别的根都没有一条位置项指着这一份''', "loc-log")
walk.write_text(t)

fi = root / "crates/singlefs-harness/src/fault_injection.rs"
t = fi.read_text()
t = once(t, "        FaultSchedule::the_nth_call_across_the_pool(drawn.fault, drawn.call_ordinal),",
         '''        FaultSchedule::the_nth_call_across_the_pool(
            drawn.fault,
            if drawn.fault == InjectedFault::WriteIsSwallowed
                && drawn.call_ordinal > 1
                && std::env::var_os("ATTACK_ORDINAL_EARLIER").is_some()
            {
                drawn.call_ordinal - 1
            } else {
                drawn.call_ordinal
            },
        ),''', "ordinal-earlier")
fi.write_text(t)
print("ok extras")
