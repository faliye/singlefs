#!/usr/bin/env python3
"""数登记表里几份文件头的弄坏开关条数：读法照门禁 code-tooling.sh 的 BREAK_SWITCH_TABLE_LIBRARY（从 <根>/.claude/gate.d/code-tooling.sh 里原样抽出来执行，不另写一份）。
用法：count-switches.py <根> <相对路径>…；每份打一行 name=switch_count。只读，不起子进程。"""
import collections
import os
import re
import sys

root = sys.argv[1]
with open(os.path.join(root, ".claude/gate.d/code-tooling.sh"), encoding="utf-8") as handle:
    stage = handle.read()
found = re.search(r"BREAK_SWITCH_TABLE_LIBRARY=\"\$\(cat <<'PY_BREAK_SWITCH_TABLE'\n(.*?)\nPY_BREAK_SWITCH_TABLE\n", stage, re.S)
if not found:
    print("name=switch_count error=library-not-found")
    sys.exit(1)
namespace = {}
exec(found.group(1), namespace)
for relative in sys.argv[2:]:
    with open(os.path.join(root, relative), encoding="utf-8") as handle:
        entries = namespace["switch_entries"](handle.read())
    by_variable = collections.Counter(entry["name"] for entry in entries)
    distinct = {(entry["name"], entry["value"]) for entry in entries}
    print(f"name=switch_count file={relative} entries={len(entries)} distinct={len(distinct)} "
          + " ".join(f"{variable}={count}" for variable, count in sorted(by_variable.items())))
