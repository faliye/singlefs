import os, subprocess, sys
root = "/home/fy5090/code/singlefs"
table = {
    "row129": [("crates/singlefs-core/src/allocator.rs", "            existing.span_slots = u16::try_from(placement.span).expect(\"跨度 2 字节\");\n            existing.generation = generation;\n            existing.is_released = false;", "            records.push(AllocationRecord {\n                device,\n                slot: placement.slot,\n                span_slots: u16::try_from(placement.span).expect(\"跨度 2 字节\"),\n                generation,\n                is_released: false,\n            });")],
    "row130": [("crates/singlefs-core/src/allocator.rs", "            records.retain(|record| !(record.device == device && record.slot == record_slot));", "            // 变异：罩住的已回收记录不删")],
    "row140": [("crates/singlefs-core/src/allocator.rs", "            existing.generation = generation;\n            existing.is_released = false;", "            existing.is_released = false;")],
    "row141": [("crates/singlefs-core/src/unit.rs", "    if bytes[payload_end..].iter().any(|byte| *byte != 0) {", "    if bytes[payload_end - 1..].iter().any(|byte| *byte != 0) {")],
    "b1": [("crates/singlefs-core/src/transaction.rs", "if file.content.len() > data_unit_capacity {", "if file.content.len() >= data_unit_capacity {")],
    "a1": [("crates/singlefs-core/src/allocator.rs", ".filter(|record| record.is_released && record.generation <= floor)", ".filter(|record| record.is_released && record.generation < floor)")],
    "a1row139": [("crates/singlefs-core/src/allocator.rs", ".filter(|record| record.is_released && record.generation <= floor)", ".filter(|record| record.is_released && record.generation < floor)"),
                 ("crates/singlefs-harness/src/history.rs", "        && observation.raised_floor_lands_only_on_abandoned_roots == Some(true)\n        && !observation.root_ring_has_turned()", "        && !observation.root_ring_has_turned()")],
}
for name in sys.argv[1:]:
    destination = f"/tmp/claude-1000/m2-supp3-item1/r3/{name}/repo"
    os.makedirs(destination, exist_ok=True)
    subprocess.run(["rsync", "-a", "--delete", "--exclude", "target", "--exclude", ".git", "--exclude", "research", f"{root}/", f"{destination}/"], check=True)
    for path, old, new in table.get(name, []):
        full = os.path.join(destination, path)
        text = open(full, encoding="utf-8").read()
        assert text.count(old) == 1, (name, path, text.count(old))
        open(full, "w", encoding="utf-8").write(text.replace(old, new))
    print(name, "ready", len(table.get(name, [])), "edits")
