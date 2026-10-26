"""副本 walk.rs 加「外给的隔离清单」（甲二：插回去那张盘照实现交出的隔离单元再判一遍）。用法：python3 -I apply_external.py <副本根>"""
import sys, pathlib
walk = pathlib.Path(sys.argv[1]) / "crates/singlefs-checker/src/walk.rs"
t = walk.read_text()
def once(text, old, new, label):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"{label}: 旧串命中 {n} 次")
    return text.replace(old, new)
t = once(t, "fn attack_arm() -> String {", '''thread_local! {
    static ATTACK_EXTERNAL_QUARANTINE: std::cell::RefCell<std::collections::BTreeSet<(u64, u64)>> = const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
}
/// 攻方腿副本：这一线程上之后的判定把这几个单元（起点槽、跨度）当隔离的单元（甲二）。
pub fn attack_set_external_quarantine(units: std::collections::BTreeSet<(u64, u64)>) {
    ATTACK_EXTERNAL_QUARANTINE.with(|set| *set.borrow_mut() = units);
}
fn attack_externally_quarantined(slot: u64, span: u64) -> bool {
    ATTACK_EXTERNAL_QUARANTINE.with(|set| set.borrow().contains(&(slot, span)))
}
fn attack_arm() -> String {''', "ext")
t = once(t, '''            let quarantined = some_copy_of_the_unit_fails_its_location_entries(
                reader,
                &devices_of_the_unit,
                slot,
                span_slots,
                location_entry_checksums,
            );''', '''            let quarantined = attack_externally_quarantined(slot, span_slots)
                || some_copy_of_the_unit_fails_its_location_entries(
                    reader,
                    &devices_of_the_unit,
                    slot,
                    span_slots,
                    location_entry_checksums,
                );''', "i315ext")
t = once(t, '''        let some_copy_fails = devices_of_the_unit
            .iter()
            .any(|device| !copy_matches_its_location_entry(*device));''', '''        let some_copy_fails = attack_externally_quarantined(slot, span_slots)
            || devices_of_the_unit
                .iter()
                .any(|device| !copy_matches_its_location_entry(*device));''', "q31ext")
walk.write_text(t)
print("ok")
