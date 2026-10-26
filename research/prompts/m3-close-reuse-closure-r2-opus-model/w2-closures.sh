#!/usr/bin/env bash
# W2：几棵树（原树、cut、cut-mpl-import、full-mpl-import）上收四个碰得到包装的判法入口的闭包（lib_function_units.py closure --explain），
# 每行报：退出码、单元数、拒算、exec_module 那条 load 边在不在、host-dispatch.py 的 import long_run_lib 与 import preflight 两个单元在不在、
# memory_peaks_lib.py 的 import subprocess 单元在不在与它的指纹；键（key 子命令，拒算时没有键）。另跑 w2-import-census.py。
# 键取判法单元指纹（lib_verdict_library.py judging-units：去掉 NOT_JUDGING_FILES、外部程序由每条判定自己管，同判法建库时那一把）。
# 用法：w2-closures.sh <原树> <变体目录> <输出目录>
set -u
base="$1" variants="$2" out="$3"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$out"
entries=(scripts/gate/verus-proof-check.py::check_proof scripts/gate/verus-proof-check.py::check_vacuity scripts/gate/verus-proof-check.py::check_must_red_worlds scripts/mutation/broad-mutation.py::Runner.escalate_step)
for tree in full cut cut-mpl-import cut-long-run-unit full-mpl-import; do
  root="$variants/$tree"; [[ $tree == full ]] && root="$base"
  (cd "$base" && python3 -B scripts/gate/lib_function_units.py dump "$root" --language python --under scripts/hosts --under .claude/hooks --under .claude/scripts) > "$out/dump-$tree.txt" 2>&1
  index=0
  rm -f "$out"/line-$tree-*.txt
  for entry in "${entries[@]}"; do
    index=$((index + 1))
    { label="$(printf '%s' "$entry" | tr '/:.' '___')"
      (cd "$base" && python3 -B scripts/gate/lib_function_units.py closure "$root" "$entry" --explain) > "$out/closure-$tree-$label.txt" 2> "$out/closure-$tree-$label.err"
      rc=$?
      key="$(cd "$base" && python3 -B scripts/crash/lib_verdict_library.py judging-units "$root" "$entry" 2>/dev/null | awk -F'\t' '$1=="key"{print substr($2,1,16)}')"
      c="$out/closure-$tree-$label.txt"
      units=$(awk -F'\t' '$1!="via" && $1!="refused" && $1!="key"' "$c" | wc -l)
      refusals="$(awk -F'\t' '$1=="refused"{print $2" "$3}' "$c" | tr '\n' ',')"
      load=$(grep -cP '^via\tscripts/hosts/host-dispatch.py:<module-body>\tscripts/hosts/run-with-memory-cap.sh:predicted_bytes\tload$' "$c")
      long_run=$(grep -cP '^import\tlong_run_lib\tscripts/hosts/host-dispatch.py:' "$c")
      preflight=$(grep -cP '^import\tpreflight\tscripts/hosts/host-dispatch.py:' "$c")
      subprocess_unit="$(awk -F'\t' '$1=="import" && $2=="subprocess" && $3 ~ /^scripts\/hosts\/memory_peaks_lib.py:/ {print substr($4,1,12)}' "$c")"
      echo "name=closure tree=$tree entry=$entry rc=$rc units=$units key=${key:-none} load_edge_to_host_dispatch=$load import_long_run_lib=$long_run import_preflight=$preflight memory_peaks_lib_import_subprocess=${subprocess_unit:-absent} refusals=${refusals:-none}" > "$out/line-$tree-$index.txt"
    } &
  done
  wait
  count=$(ls "$out"/line-$tree-*.txt 2>/dev/null | wc -l)
  [[ $count -eq ${#entries[@]} ]] || { echo "name=batch_incomplete tree=$tree files=$count（这一批作废）"; continue; }
  cat "$out"/line-$tree-{1,2,3,4}.txt
  python3 -B "$here/w2-import-census.py" "$root" "$out/closure-$tree-scripts_gate_verus-proof-check_py__check_proof.txt" "$out/dump-$tree.txt" "$tree"
done
