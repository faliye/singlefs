#!/usr/bin/env bash
# 样本用的桩装置：e165_replay_bounded_enumeration --pinned-cases <表> 的约定——按数据行的次序每行打一行结果（就是 ⑦），末了一行汇总。
# 这一份把 ordinal=10 那一行（钉表第 21 行）打错：只点名别的两行时判绿，点名那一行时判红。
[[ "${1:-}" == --pinned-cases && -f "${2:-}" ]] || { echo "桩只认 --pinned-cases <表>" >&2; exit 2; }
grep -v '^#' "$2" | grep . | cut -f7 | sed '/ordinal=10 / s/returned:/returned:WRONG:/'
echo "E7RESULT name=pinned_cases rows=$(grep -v '^#' "$2" | grep -c .)"
