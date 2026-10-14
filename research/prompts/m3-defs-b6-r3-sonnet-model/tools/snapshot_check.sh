#!/usr/bin/env bash
# Y6：三份冻结副本对各自快照 sha256 的核对，以及零轮之前那一版对各冻结副本的差异文件数。用法：snapshot_check.sh <仓根> <工作根>
set -u
repo="${1:?仓根}"; work="${2:?工作根}"
for r in r1 r2 r3; do
  list="$repo/research/prompts/m3-defs-b6-$r-snapshot/frozen-copy-sha256.txt"
  res=$(cd "$work/m3-defs-b6-$r/repo" && sha256sum -c "$list" 2>&1 | awk '{print $NF}' | sort | uniq -c | tr -s ' ' | tr '\n' ';')
  echo "name=snapshot round=$r lines=$(wc -l < "$list") result=$res"
  n=$(diff -rq "$work/m3-defs-b3-r2/repo" "$work/m3-defs-b6-$r/repo" 2>&1 | grep -v 'multi-host.env' | grep -vc __pycache__)
  nf=$(diff -rq "$work/m3-defs-b3-r2/repo" "$work/m3-defs-b6-$r/repo" 2>&1 | grep -v 'multi-host.env' | grep -v __pycache__ | grep -vc '/fixtures/\|fixtures/')
  echo "name=diff_vs_b3 round=$r all_entries=$n non_fixture_entries=$nf"
done
for r in b3-r2 b6-r1 b6-r2 b6-r3; do
  d="m3-defs-$r"; [[ $r == b3-r2 ]] && d=m3-defs-b3-r2
  f="$work/$d/repo/.claude/heavy-thread-exempt"
  echo "name=exempt_table frozen=$r total_lines=$(wc -l < "$f") data_rows=$(grep -vc '^\s*#\|^\s*$' "$f")"
done
python3 -I - "$work" <<'PY'
import sys
w = sys.argv[1]
def keys(r):
    return {l.rstrip("\n").split("  #")[0] for l in open(f"{w}/{r}/repo/.claude/heavy-thread-exempt", encoding="utf-8") if l.strip() and not l.lstrip().startswith("#")}
r1, r2 = keys("m3-defs-b6-r1"), keys("m3-defs-b6-r2")
print(f"name=exempt_keys r2_not_in_r1={len(r2 - r1)} r1_not_in_r2={len(r1 - r2)}")
PY
