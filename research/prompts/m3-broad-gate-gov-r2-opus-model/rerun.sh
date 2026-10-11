#!/usr/bin/env bash
# 攻方腿 m3-broad-gate-gov-r2 的复跑：bash rerun.sh <冻结副本树根> <草稿目录>；产出写进本目录 outputs/（同名覆盖）。
# 只起样本档（cd 进样本根、根参数写同一个目录）、从驱动与 admission.py 载入的纯函数、只读的真库读取；不起驱动真跑、不起整道门禁。
set -uo pipefail
frozen="$1"; work="$2"; here="$(cd "$(dirname "$0")" && pwd)"; out="$here/outputs"
mkdir -p "$work" "$out"
base="$work/trees/base"; rm -rf "${work:?}/trees"; mkdir -p "$work/trees/h2"
rsync -a --exclude target --exclude .git "$frozen/" "$base/"
# 26 份样本在原样副本上
bash "$here/run-samples.sh" "$base" "$work/out/base" | tail -1 > "$out/samples-base.txt"
# 20 个弄坏开关
: > "$out/switches.txt"
for b in work-tree-read files-unchecked toolchain-unchecked export-range-unchecked pending-unchecked version-key-unchecked merge-conflict-unchecked source-fingerprint-unchecked global-key-unchecked marker-unguarded summary-red-passes verdict-exit-unchecked verdict-value-unchecked counts-unchecked detail-count-unchecked empty-list-passes summary-gate-totals-unchecked gate-line-optional version-unchecked tree-source-reread; do
  printf '%s\t' "$b" >> "$out/switches.txt"; CHECKER_TIER_BROAD_MUTATION_BREAK=$b bash "$here/run-samples.sh" "$base" "$work/out/brk-$b" | tail -1 >> "$out/switches.txt"
done
# H2：驱动里样本与门禁共用的函数写错 16 种，各跑 26 份样本
python3 "$here/h2-mutants.py" "$base" "$work/trees/h2" > "$out/h2-mutants.list"
: > "$out/h2-sweep.txt"
for m in $(cut -f1 "$out/h2-mutants.list"); do printf '%s\t' "$m" >> "$out/h2-sweep.txt"; bash "$here/run-samples.sh" "$work/trees/h2/$m" "$work/out/h2-$m" | tail -1 >> "$out/h2-sweep.txt"; done
# H1 第 3 条：查标记那一侧
bash "$here/exp1-marker-check.sh" "$frozen" "$work" > "$out/exp1-marker-check.txt" 2>&1
# H1 第 7 条：结局组合
mkdir -p "$work/h1-7"; awk 'NR>=344 && NR<=443' "$base/.claude/gate.d/checker-tier-broad-mutation.sh" > "$work/h1-7/gate_verdict.py"
python3 "$here/h1-7-combos.py" "$base/scripts/mutation/broad-mutation.py" "$work/h1-7/gate_verdict.py" "$work/h1-7" 400 20261003 > "$out/h1-7-combos-400.txt" 2>&1
python3 "$here/h1-7-combos.py" "$base/scripts/mutation/broad-mutation.py" "$work/h1-7/gate_verdict.py" "$work/h1-7" 5000 7 > "$out/h1-7-combos-5000.txt" 2>&1
# H1 第 5 条旁支：全局键对哪些编译配置不变
mkdir -p "$work/h1-5"; python3 "$here/h1-5-env-gap.py" "$base" "$work/h1-5" > "$out/h1-5-env-gap.txt" 2>&1
# 只读：真库此刻那一版的全局键照门禁算法现算
python3 "$here/h1-5-real-key.py" "$frozen/scripts/mutation/broad-mutation.py" "$frozen" "${SINGLEFS_BROAD_MUTATION_HOME:-$HOME/.local/share/singlefs/broad-mutation}" > "$out/h1-5-real-key.txt" 2>&1
# 排除表读工作区那一份
probe="$work/exp2"; rm -rf "${probe:?}"; mkdir -p "$probe"; cp -al "$base/." "$probe/"; git -C "$probe" init -q .; git -C "$probe" add -A
f="$probe/scripts/mutation/broad-mutation-ignored-exclusions.tsv"; cp "$f" "$f.new"; printf 'singlefs-harness::x some_case\t工作区里加的一行（没暂存）\n' >> "$f.new"; mv "$f.new" "$f"
python3 "$here/exp2-probe.py" "$probe" > "$out/exp2-probe.txt" 2>&1
echo "done"
