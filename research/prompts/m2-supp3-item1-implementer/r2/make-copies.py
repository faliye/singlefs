import os, shutil, subprocess, sys
root = "/home/fy5090/code/singlefs"
table = {
    "m121": ("crates/singlefs-core/src/allocator.rs", "            records.retain(|record| !(record.device == device && record.slot == record_slot));", "            // 变异：罩住的已回收记录不删"),
    "n2": ("crates/singlefs-core/src/allocator.rs", "            existing.generation = generation;\n            existing.is_released = false;", "            existing.is_released = false;"),
    "b6": ("crates/singlefs-core/src/unit.rs", "    if bytes[payload_end..].iter().any(|byte| *byte != 0) {", "    if bytes[payload_end - 1..].iter().any(|byte| *byte != 0) {"),
    "b1": ("crates/singlefs-core/src/transaction.rs", "if file.content.len() > data_unit_capacity {", "if file.content.len() >= data_unit_capacity {"),
    "a1": ("crates/singlefs-core/src/allocator.rs", ".filter(|record| record.is_released && record.generation <= floor)", ".filter(|record| record.is_released && record.generation < floor)"),
}
names = sys.argv[1:]
for name in names:
    destination = f"/tmp/claude-1000/m2-supp3-item1/r2/{name}/repo"
    os.makedirs(destination, exist_ok=True)
    subprocess.run(["rsync", "-a", "--delete", "--exclude", "target", "--exclude", ".git", "--exclude", "research", f"{root}/", f"{destination}/"], check=True)
    if name == "base":
        print(name, "copied"); continue
    path, old, new = table[name]
    full = os.path.join(destination, path)
    text = open(full, encoding="utf-8").read()
    assert text.count(old) == 1, (name, text.count(old))
    open(full, "w", encoding="utf-8").write(text.replace(old, new))
    print(name, "applied", path)
