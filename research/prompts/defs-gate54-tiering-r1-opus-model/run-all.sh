#!/usr/bin/env bash
# 复跑：bash research/prompts/defs-gate54-tiering-r1-opus-model/run-all.sh
# 合成仓都建在 /tmp/claude-1000/defs54-attack/ 下；主仓只读（cp 与 git show HEAD:…），不写主仓的 git common-dir。
# 输出落进本目录的 outputs/（每次覆盖）。不编译 Rust：cargo 是 fake-bin/cargo。
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p /tmp/claude-1000/defs54-attack "$here/outputs"
for scenario in s1-worktree-vs-staged s2-trigger-gaps s3-marker-slot s4-marker-provenance s5-transient-edit s6-fix-arms; do
  nice -n 19 bash "$here/$scenario.sh" > "$here/outputs/$scenario.out" 2>&1
  echo "$scenario：$(wc -l < "$here/outputs/$scenario.out") 行 → outputs/$scenario.out"
done
