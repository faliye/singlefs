#!/usr/bin/env bash
# 只喂 JSON 给 heavy-test-guard.sh 看退出码，不执行任何被判的命令。在仓根跑：bash research/prompts/defs-m3-formal-gate-r1-sonnet-model/rerun.sh
set -uo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
scratch="${SCRATCH:-/tmp/claude-1000/defs-m3-formal-gate-r1-sonnet}"
mkdir -p "$scratch"
nice -n 19 python3 "$root/research/prompts/defs-m3-formal-gate-r1-sonnet-model/probe.py" "$root" "$scratch/hook-detections.jsonl"
# D5 机械那一半：上下文指代与自称词，只读
nice -n 19 python3 "$root/research/prompts/defs-m3-formal-gate-r1-sonnet-model/d5-scan.py" "$root/research/prompts/_defs-m3-formal-gate-r1-diff.md"
