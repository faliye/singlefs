#!/usr/bin/env bash
# admission: always 判的是 40、86 号今天的样子，随时可能改
# run-condition: command rsync
# defs-m2-closeout-r2 云端攻方 E2（F16）：新实验在执行员第 6 步（第 7 步写实验页之前）跑 40、86 号会不会判红。
# 在仓副本（rsync，不带 target 与 .git）里：先跑一次两道记基线，再放进一个新实验号 E999 的装置源码与产物（还没有实验页），再跑一次；最后补上实验页与索引行（第 7 步）再跑一次。
# 副本不在 git 里：40 号取不到改动范围时判全部（它文件头第 86 行那一句），所以基线与加了 E999 之后各跑一次、只比两次的差别。
# 用法（仓根下）：bash research/prompts/defs-m2-closeout-r2-opus-model/stage-demo.sh ；副本建在 ${TMPDIR:-/tmp} 下，跑完删掉。
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
copy="$(mktemp -d "${TMPDIR:-/tmp}/stage-demo.XXXXXX")"
rsync -a --exclude target --exclude .git "$root/" "$copy/"
run_both() { # run_both <标签>
  for stage in 40-results-cited.sh 86-experiment-orphans.sh; do
    output="$(cd "$copy" && bash ".claude/gate.d/$stage" "$copy" 2>&1)"; status=$?
    echo "[$1] $stage exit=$status e999_lines=$(grep -c 'e999\|E999' <<<"$output")"
    grep -m3 'e999\|E999' <<<"$output" | cut -c1-200 | sed 's/^/    /'
  done
}
run_both baseline
printf 'fn main() { println!("E7RESULT name=config experiment=e999"); }\n' > "$copy/research/e7-index-bench/src/bin/e999_probe.rs"
printf 'E7RESULT name=config experiment=e999\nE7RESULT name=done emitted=2\n' > "$copy/research/results/e999-probe-2026-09-26.out"
run_both with-e999-no-page
# 第 7 步之后：实验页与索引行点名这份产物
printf '## E999 探针 —— 已跑\n\n产物：research/results/e999-probe-2026-09-26.out\n' > "$copy/.claude/kb/experiments/999-probe.md"
printf '\n| E999 | 探针 | 已跑 | research/results/e999-probe-2026-09-26.out |\n' >> "$copy/.claude/kb/experiments.md"
run_both after-step7-page
rm -rf -- "${copy:?}"
