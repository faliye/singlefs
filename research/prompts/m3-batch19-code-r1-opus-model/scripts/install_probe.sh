#!/usr/bin/env bash
# 用法：install_probe.sh <副本根> <原型源目录>：把原型 crate 放进副本、加进工作区成员（只在副本上）。
set -eu
root=$1; src=$2
mkdir -p "$root/crates/attack-b19-probe/src"
cp "$src/Cargo.toml" "$root/crates/attack-b19-probe/Cargo.toml"
cp "$src/main.rs" "$src/n7.rs" "$src/extra.rs" "$src/lone.rs" "$root/crates/attack-b19-probe/src/"
grep -q '"crates/attack-b19-probe"' "$root/Cargo.toml" || python3 - "$root/Cargo.toml" <<'PY'
import sys
p = sys.argv[1]; t = open(p, encoding='utf-8').read()
old = '    "crates/singlefs-reference-filesystem",\n]'
assert t.count(old) == 1
open(p, 'w', encoding='utf-8').write(t.replace(old, '    "crates/singlefs-reference-filesystem",\n    "crates/attack-b19-probe",\n]'))
PY
