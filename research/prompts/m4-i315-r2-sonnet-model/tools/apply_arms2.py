"""辩方腿（m4-i315-r2）第二个补丁：在 apply_arms.py 之后打。给「没有位置项可比」的回退点记一行日志（NOENTRY），
带自证结果、单元头里的诞生代号、记录的分配代。只改副本。用法：python3 -I apply_arms2.py <副本根>"""
import sys, pathlib
root = pathlib.Path(sys.argv[1])
walk = root / "crates/singlefs-checker/src/walk.rs"
t = walk.read_text()
old = """fn defense_copy_matches_without_a_location_entry(unit: &[u8], record_generation: Option<u64>) -> bool {
    match defense_arm() {"""
new = """fn defense_copy_matches_without_a_location_entry(unit: &[u8], record_generation: Option<u64>) -> bool {
    defense_log(format!(
        "NOENTRY|self_check={}|header_birth={:?}|record_generation={record_generation:?}",
        check_unit(unit).is_ok(),
        unit.get(..UNIT_HEADER_SCAN_BYTES)
            .and_then(birth_txg_in_a_usable_unit_header)
    ));
    match defense_arm() {"""
if t.count(old) != 1:
    raise SystemExit(f"旧串命中 {t.count(old)} 次")
walk.write_text(t.replace(old, new))
print("ok")
