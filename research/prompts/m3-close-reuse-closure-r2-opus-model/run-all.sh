#!/usr/bin/env bash
# 攻方腿（第二轮）全部装置一趟跑完：<树> 是被判脚本在里面、sha256 对得上 judged-sha256.txt 的那棵树，<输出目录> 放原样输出；
# 末尾把可比的结论行（name=…，进程号、排队标签这类随跑变的字换成占位）写进 <输出目录>/summary.txt。几组互不相干的装置并行跑（各写各的文件），用不带参数的 wait 等齐。
# 用法：run-all.sh <树> <输出目录>
set -u
tree="$1" out="$2"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$out"
(cd "$tree" && sha256sum -c "$here/judged-sha256.txt") > "$out/judged-sha256.check" 2>&1 || { cat "$out/judged-sha256.check"; echo "被判脚本的 sha256 对不上" >&2; exit 3; }
python3 -B "$here/make-variants.py" "$tree" "$out/variants" > "$out/variants.txt" || exit 4
v="$out/variants"
export ATTACK_OUT="$out/worlds"
rm -f "$out"/part-*.rc
{ bash "$here/w1-wrapper-hits.sh" "$tree" "$out/hits" > "$out/part-hits.txt" 2>&1
  for spec in "peer43-refused|$tree|6|none" "lookup-off-refused|$tree|6|none|RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off" \
              "hr-int-cap-refused|$v/hr-int-cap|6|none" "hr-int-cap-refused-lookup-off|$v/hr-int-cap|6|none|RUN_WITH_MEMORY_CAP_HOST_LOOKUP=off" \
              "item1-literal-refused|$v/item1-literal|6|none" "item1-catchall-refused|$v/item1-catchall|6|none" \
              "hr-int-cap-admitted|$v/hr-int-cap|30|3" "item1-literal-admitted|$v/item1-literal|30|3" "item1-catchall-admitted|$v/item1-catchall|30|3"; do
    IFS='|' read -r name root wait_seconds release extra <<< "$spec"
    bash "$here/w1-queue-world.sh" "$root" "$name" "$wait_seconds" "$release" ${extra:+"$extra"} > "$out/world-line-$name.txt" 2>&1 &
  done
  ATTACK_PEER_AVAILABLE_GIB=41 bash "$here/w1-queue-world.sh" "$tree" peer41-refused 6 none > "$out/world-line-peer41-refused.txt" 2>&1 &
  wait
  python3 -B "$here/w1-strip-judges.py" "$tree" "$out/hits" "$ATTACK_OUT/world-peer43-refused" "$ATTACK_OUT/world-peer41-refused" \
    "$ATTACK_OUT/world-lookup-off-refused" "$ATTACK_OUT/world-hr-int-cap-admitted" > "$out/part-strip.txt" 2>&1
  echo 0 > "$out/part-w1.rc"; } &
{ bash "$here/w2-closures.sh" "$tree" "$v" "$out/w2" > "$out/part-w2-closures.txt" 2>&1
  for name in cut cut-mpl-import cut-long-run-unit; do bash "$here/w2-wrapper-start.sh" "$v/$name" "$name" "$out/starts"; done > "$out/part-w2-starts.txt" 2>&1
  (cd "$tree" && python3 -B scripts/gate/lib_function_units.py dump "$v/cut" --language python --under scripts --under .claude/gate.d --under .claude/hooks --under .claude/scripts --under .claude/singlefs-ai-sop/scripts) > "$out/w2/dump-cut-all.txt" 2>&1
  for entry in scripts_gate_verus-proof-check_py__check_proof scripts_mutation_broad-mutation_py__Runner_escalate_step; do
    python3 -B "$here/w2-module-census.py" "$out/w2/closure-cut-$entry.txt" "$out/w2/dump-cut-all.txt" "$entry"
  done > "$out/part-w2-census.txt" 2>&1
  bash "$here/w2-module-level-fix.sh" "$v" "$out/w2ml" > "$out/part-w2-module-level-fix.txt" 2>&1
  echo 0 > "$out/part-w2.rc"; } &
{ bash "$here/w2-entries.sh" "$tree" "$v/cut" "$here/w2-entries.txt" "$out/w2-entries" > "$out/part-w2-entries.txt" 2>&1; echo 0 > "$out/part-entries.rc"; } &
{ python3 -B "$here/w3-needed-files.py" "$tree" "$out/w3" > "$out/part-w3-needed.txt" 2>&1
  bash "$here/w3-sync-stale.sh" "$tree" "$out/w3" > "$out/part-w3-stale.txt" 2>&1
  bash "$here/w4-declaration-fingerprints.sh" "$tree" "$out/w4" 2>/dev/null | grep '^name=' > "$out/part-w4-declaration.txt"
  for name in cut cut-hr-int-cap; do
    echo "name=judging_key tree=$name entry=check_proof key=$(cd "$tree" && python3 -B scripts/crash/lib_verdict_library.py judging-units "$v/$name" scripts/gate/verus-proof-check.py::check_proof 2>/dev/null | awk -F'\t' '$1=="key"{print substr($2,1,16)}')"
  done > "$out/part-w4-keys.txt" 2>&1
  echo 0 > "$out/part-w3w4.rc"; } &
wait
count=$(ls "$out"/part-*.rc 2>/dev/null | wc -l)
[[ $count -eq 4 ]] || { echo "name=run_all_incomplete parts=$count（四组里有没跑完的，这一趟作废）" >&2; exit 5; }
C="$out/w2/closure-cut-scripts_gate_verus-proof-check_py__check_proof.txt"
{ echo "name=judged_sha256 status=ok"
  cat "$out/part-hits.txt"
  for name in peer43-refused peer41-refused lookup-off-refused hr-int-cap-refused hr-int-cap-refused-lookup-off item1-literal-refused item1-catchall-refused hr-int-cap-admitted item1-literal-admitted item1-catchall-admitted; do
    cat "$out/world-line-$name.txt"
  done
  grep -E '^(name=|  queue_refusal_where=|  problem_has_peer_numbers=)' "$out/part-strip.txt" | python3 -c 'import sys; [print(line.rstrip("\n")[:100]) for line in sys.stdin]'
  grep -E '^name=' "$out/part-w2-closures.txt"
  cat "$out/part-w2-starts.txt" "$out/part-w2-census.txt" "$out/part-w2-module-level-fix.txt"
  grep -E '^name=' "$out/part-w2-entries.txt" | sed -E 's/^name=entry index=[0-9]+ /name=entry /'
  cat "$out/part-w3-needed.txt" "$out/part-w3-stale.txt" "$out/part-w4-declaration.txt" "$out/part-w4-keys.txt"
  echo "name=not_judging_files_units_in_cut_closure entry=check_proof $(awk -F'\t' '$1!="via" && $1!="refused" && ($3 ~ /^scripts\/crash\/lib_verdict_library.py:/ || $3 ~ /^scripts\/gate\/lib_function_units.py:/){split($3,a,":"); print a[1]}' "$C" | sort | uniq -c | awk '{printf "%s=%s ", $2, $1}')"
} | sed -E 's/moved-from-queue-[0-9]+-[0-9]+/moved-from-queue-<进程号>/g; s/包装进程 [0-9]+/包装进程 <进程号>/g' > "$out/summary.txt"
cat "$out/summary.txt"
