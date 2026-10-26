#!/usr/bin/env bash
# K3 merge-and-verify 那一步的探针：从 merge-and-verify.sh 原样截出 PY_SELECT 那段 python，在真树（只读）上喂造出来的补丁目录，看 CHECKER 那一行判 affected 还是 unaffected。
# 用法：bash merge_select_probe.sh <草稿目录> <被判的 merge-and-verify.sh> <lib_rebuilt_packages.py> <真树根（只读）>
set -uo pipefail
work="$1/merge-select"; script="$2"; library="$3"; tree="$4"; mkdir -p "$work"
awk "/<<'PY_SELECT'/{f=1;next} /^PY_SELECT\$/{f=0} f" "$script" > "$work/select.py"
patch_for() { printf 'diff --git a/%s b/%s\n--- a/%s\n+++ b/%s\n@@ -1 +1 @@\n-a\n+b\n' "$1" "$1" "$1" "$1"; }
mkdir -p "$work/unit-graph" "$work/experiments" "$work/empty" "$work/harness-tests"
patch_for crates/singlefs-unit-graph/src/lib.rs > "$work/unit-graph/crates.patch"
patch_for scripts/experiment-apparatus/src/lib.rs > "$work/experiments/crates.patch"
: > "$work/empty/crates.patch"
patch_for crates/singlefs-harness/tests/checker_known_bad_images.rs > "$work/harness-tests/crates.patch"
for name in unit-graph experiments empty harness-tests; do
  printf '%s → ' "$name"
  (cd "$tree" && python3 "$work/select.py" "$tree" "$library" "" "singlefs-format singlefs-core" singlefs-checker-tier "$work/$name" 2>&1 | grep -m1 $'^CHECKER')
done
python3 - "$tree" <<'PY'
import os, sys, tomllib
root = sys.argv[1]
members = tomllib.load(open(os.path.join(root, "Cargo.toml"), "rb"))["workspace"]["members"]
names = {}
for member in members:
    table = tomllib.load(open(os.path.join(root, member, "Cargo.toml"), "rb")); names[table["package"]["name"]] = table
needs = {name: {value.get("package", key) if isinstance(value, dict) else key for section in ("dependencies", "dev-dependencies", "build-dependencies")
                for key, value in (table.get(section) or {}).items()} & set(names) for name, table in names.items()}
closure, frontier = {"singlefs-checker-tier"}, ["singlefs-checker-tier"]
while frontier:
    for needed in needs[frontier.pop()]:
        if needed not in closure:
            closure.add(needed); frontier.append(needed)
print("工作区成员", len(names), "；checker 档包依赖闭包", len(closure), "；闭包外的成员", sorted(set(names) - closure))
PY
