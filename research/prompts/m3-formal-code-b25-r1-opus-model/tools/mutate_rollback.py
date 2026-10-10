#!/usr/bin/env python3
"""在攻方副本的 model.rs 上施加 / 还原 25-1 回退那一支的两条手造变异（只改副本）。
用法：mutate_rollback.py <副本根> <none|drop-retain|drop-extend>"""
import sys, pathlib
root, which = sys.argv[1], sys.argv[2]
p = pathlib.Path(root) / "crates/singlefs-harness/src/model.rs"
orig = pathlib.Path(str(p) + ".orig")
if not orig.exists():
    orig.write_bytes(p.read_bytes())
s = orig.read_text()
retain = """                role_written_at.retain(|role, _| {
                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))
                });
"""
extend_head = "                role_written_at.extend(\n                    target_root\n"
assert s.count(retain) == 1 and s.count(extend_head) == 1
if which == "drop-retain":
    s = s.replace(retain, "")
elif which == "drop-extend":
    s = s.replace(extend_head, "                let _ = (&role_written_at, &target_root);\n                Vec::<(ModelUnitRole, ModelCheckpointTxg)>::new().extend(\n                    target_root\n")
elif which != "none":
    sys.exit("which?")
tmp = pathlib.Path(str(p) + ".tmp")
tmp.write_text(s)
tmp.replace(p)
print(f"applied={which}")
