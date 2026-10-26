#!/usr/bin/env python3
"""K1 用的四份脚本副本：<工作目录>/copy-lfu0（仓里原样）、copy-lfu1（lib_function_units.py 一处：Rust 单元的末行多算一行，E1）、
copy-lfu0-fix、copy-lfu1-fix（再在 crates-mutation-rows.py 的覆盖号里加判法单元指纹，零轮改法）。用法：make_copies.py <仓根> <工作目录>"""
import os, shutil, subprocess, sys

repo, work = sys.argv[1], sys.argv[2]
E1_OLD = '        unit = Unit("rust", kind, "::".join(context + [name]), self.path, self.lines[start], self.lines[min(end, self.count) - 1],\n'
E1_NEW = '        unit = Unit("rust", kind, "::".join(context + [name]), self.path, self.lines[start], self.lines[min(end, self.count) - 1] + 1,\n'
FIX_OLD = '        coverage_id = mutation_units.key_of_lines([("form", [COVERAGE_KEY_FORM]), ("test-unit", [test.unit.fingerprint]),\n'
FIX_NEW = '        coverage_id = mutation_units.key_of_lines([("form", [COVERAGE_KEY_FORM]), ("judging", [self.judging]), ("test-unit", [test.unit.fingerprint]),\n'


def replace_once(path, old, new):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    if text.count(old) != 1:
        sys.exit(f"{path}: 要换的那一行命中 {text.count(old)} 次，要恰好 1 次")
    with open(path + ".new", "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))
    os.replace(path + ".new", path)


base = os.path.join(work, "copy-lfu0")
os.makedirs(os.path.join(base, ".claude", "gate.d"), exist_ok=True)
for source, target in (("scripts", "."), (".claude/scripts", ".claude"), (".claude/singlefs-ai-sop", ".claude")):
    subprocess.run(["rsync", "-a", "--exclude", "__pycache__", "--exclude", ".git", os.path.join(repo, source), os.path.join(base, target) + "/"], check=True)
for name in (".claude/gate.d/stage-inputs.tsv", ".claude/heavy-thread-default", "Cargo.toml", "Cargo.lock"):
    shutil.copy(os.path.join(repo, name), os.path.join(base, name))
for name in ("copy-lfu1", "copy-lfu0-fix", "copy-lfu1-fix"):
    subprocess.run(["rsync", "-a", base + "/", os.path.join(work, name) + "/"], check=True)
for name in ("copy-lfu1", "copy-lfu1-fix"):
    replace_once(os.path.join(work, name, "scripts", "gate", "lib_function_units.py"), E1_OLD, E1_NEW)
for name in ("copy-lfu0-fix", "copy-lfu1-fix"):
    replace_once(os.path.join(work, name, "scripts", "mutation", "crates-mutation-rows.py"), FIX_OLD, FIX_NEW)
print(f"name=copies work={work} made=4")
