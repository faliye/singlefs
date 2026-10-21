#!/usr/bin/env python3
"""倒推钩子的两版：今天的 lib_heavy_tests.py、heavy-test-guard.sh
  → 倒掉 cargo run 那一批换上（hook-swap-cargo-run-spec.json，规格里的 new 换回 old，倒序）得到开工快照那一版（R2、R3 换上之后）；
  → 再照 R2、R3 那一批 hook-swap.py 留下的两份 reverse 规格（先第二次、再第一次）得到 R2、R3 换上之前那一版（第二轮攻方量的那一版）。
每处要恰好命中一次；快照那一版的 sha256 与冻结副本 defs-sha256.txt 里记的比。今天的钩子再被换过，这里会报命中次数不对、停下。
用法：rebuild-hooks.py <仓根> <输入目录> <冻结副本的 defs-sha256.txt> <输出目录>"""
import hashlib, json, os, sys
repo, inputs, frozen_list, output = sys.argv[1:5]
paths = {".claude/hooks/lib_heavy_tests.py": "lib_heavy_tests.py", ".claude/hooks/heavy-test-guard.sh": "heavy-test-guard.sh"}
text = {path: open(os.path.join(repo, path), encoding="utf-8").read() for path in paths}
def replace(edits, before, after, label):
    for edit in edits:
        count = text[edit["file"]].count(edit[before])
        if count != 1:
            sys.exit(f"name=rebuild_hooks step={label} file={edit['file']} count={count}（应当恰好 1 次）")
        text[edit["file"]] = text[edit["file"]].replace(edit[before], edit[after])
def write(version):
    os.makedirs(os.path.join(output, version), exist_ok=True)
    for path, name in paths.items():
        with open(os.path.join(output, version, name), "w", encoding="utf-8") as handle:
            handle.write(text[path])
        yield path, hashlib.sha256(text[path].encode()).hexdigest()
recorded = dict(reversed(line.split()) for line in open(frozen_list, encoding="utf-8") if line.strip())
replace(reversed(json.load(open(os.path.join(inputs, "hook-swap-cargo-run-spec.json"), encoding="utf-8"))), "new", "old", "snapshot")
for path, digest in write("snapshot"):
    print(f"name=rebuilt_hook version=snapshot file={path} sha256={digest} matches_frozen={'yes' if recorded.get(path) == digest else 'no'}")
for reverse in ("hook-swap-reverse-second.json", "hook-swap-reverse-first.json"):
    replace(json.load(open(os.path.join(inputs, reverse), encoding="utf-8")), "old", "new", "pre-swap")
for path, digest in write("pre-swap"):
    print(f"name=rebuilt_hook version=pre-swap file={path} sha256={digest}")
