#!/usr/bin/env python3
"""用法：edit.py <crate 目录> <源 crate 目录> <种类>：拷源到目标，再做一处与核无关的改动。
种类：dummy-spec（verus! 末尾加一个没人用的 spec fn）、shift（文件头加 10 行注释）、dummy-proof（末尾加一个没人用的 proof fn）"""
import shutil, sys, pathlib
dst, src, kind = sys.argv[1:4]
shutil.rmtree(dst, ignore_errors=True); shutil.copytree(src, dst)
lib = pathlib.Path(dst) / "src/lib.rs"; t = lib.read_text()
end = "} // verus!"
assert t.count(end) == 1
if kind == "dummy-spec":
    t = t.replace(end, "pub open spec fn attack_probe_unrelated(x: int) -> bool { x == x }\n\n" + end)
elif kind == "dummy-proof":
    t = t.replace(end, "proof fn attack_probe_unrelated_proof(x: int) ensures x + 0 == x {}\n\n" + end)
elif kind == "shift":
    t = "// attack probe\n" * 10 + t
else:
    sys.exit(2)
lib.write_text(t); print(f"{dst} {kind}")
