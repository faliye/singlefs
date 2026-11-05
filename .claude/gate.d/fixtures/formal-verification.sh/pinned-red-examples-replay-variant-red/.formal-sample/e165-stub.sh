#!/usr/bin/env bash
# 样本用的桩装置：e165_replay_bounded_enumeration --pinned-cases <表> 的约定——按数据行的次序每行打一行结果（就是 ⑦），末了一行汇总。
[[ "${1:-}" == --pinned-cases && -f "${2:-}" ]] || { echo "桩只认 --pinned-cases <表>" >&2; exit 2; }
grep -v '^#' "$2" | grep . | cut -f7
echo "E7RESULT name=pinned_cases rows=$(grep -v '^#' "$2" | grep -c .)"
