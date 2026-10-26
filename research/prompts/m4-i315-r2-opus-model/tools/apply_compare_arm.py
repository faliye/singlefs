"""在 apply_arms.py 打过的副本上加第三种判法「两份互比」（攻方腿自己提的，被攻过零轮）：没有位置项可比时，这一份自证过了还要与
另一块盘同槽那一份逐字节相同才算对得上；读不出或不同算对不上。探针在 ATTACK_COMPARE_ARM 设了时多跑这一臂（arm=compare），没设时输出不变。
用法：python3 -I apply_compare_arm.py <副本根>（先跑 apply_arms.py，再把 attack_probe.rs 拷进 tests/ 之后跑）"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])
def once(text, old, new, label):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"{label}: 旧串命中 {n} 次")
    return text.replace(old, new)
walk = root / "crates/singlefs-checker/src/walk.rs"
t = walk.read_text()
t = once(t, "fn attack_noloc_copy_matches(unit: &[u8], device: u32, slot: u64, which: &str) -> bool {", '''thread_local! {
    static ATTACK_NOLOC_COMPARE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
/// 攻方腿副本：这一线程上「没有位置项可比」那一格取两份互比（真）。
pub fn attack_set_noloc_compare(compare: bool) {
    ATTACK_NOLOC_COMPARE.with(|cell| cell.set(compare));
}
fn attack_noloc_copy_matches_with_reader(
    reader: &dyn ImageReader,
    unit: &[u8],
    device: u32,
    slot: u64,
    span_slots: u64,
    which: &str,
) -> bool {
    let matches = attack_noloc_copy_matches(unit, device, slot, which);
    if !ATTACK_NOLOC_COMPARE.with(std::cell::Cell::get) || !matches {
        return matches;
    }
    let other_device = if device == 0 { 1 } else { 0 };
    usize::try_from(span_slots * SLOT_BYTES)
        .ok()
        .and_then(|unit_bytes| reader.read(other_device, slot * SLOT_BYTES, unit_bytes))
        .is_some_and(|other_copy| other_copy == unit)
}
fn attack_noloc_copy_matches(unit: &[u8], device: u32, slot: u64, which: &str) -> bool {''', "fn")
t = once(t, 'None => attack_noloc_copy_matches(&unit, device, slot, "Q31"),',
         'None => attack_noloc_copy_matches_with_reader(reader, &unit, device, slot, span_slots, "Q31"),', "q31")
t = once(t, 'None => attack_noloc_copy_matches(&unit, *device, slot, "I315"),',
         'None => attack_noloc_copy_matches_with_reader(reader, &unit, *device, slot, span_slots, "I315"),', "i315")
walk.write_text(t)
probe = root / "crates/singlefs-harness/tests/attack_probe.rs"
p = probe.read_text()
p = once(p, "attack_set_external_quarantine, attack_set_noloc_exempt,", "attack_set_external_quarantine, attack_set_noloc_compare, attack_set_noloc_exempt,", "use")
p = once(p, '''    for exempt in [false, true] {
        attack_set_noloc_exempt(exempt);
        attack_set_external_quarantine(BTreeSet::new());
        let arm = if exempt { "exempt" } else { "selfcheck" };''', '''    let arms: &[&str] = if std::env::var_os("ATTACK_COMPARE_ARM").is_some() {
        &["selfcheck", "exempt", "compare"]
    } else {
        &["selfcheck", "exempt"]
    };
    for arm in arms.iter().copied() {
        attack_set_noloc_exempt(arm == "exempt");
        attack_set_noloc_compare(arm == "compare");
        attack_set_external_quarantine(BTreeSet::new());''', "arms")
p = once(p, "    attack_set_noloc_exempt(false);\n}", "    attack_set_noloc_exempt(false);\n    attack_set_noloc_compare(false);\n}", "reset")
probe.write_text(p)
print("ok")
