#!/usr/bin/env bash
# 样本用的照档打的桩装置：e165_replay_bounded_enumeration --pinned-cases <表> [档参数…]——按数据行的次序每行打一行结果（⑦ 去掉 counters 里的 j6qn，
# 命令行带 --profile r2 时再加回去：像装置 r2 档多判 J6Qn），末了一行汇总。不带档参数时一律按 r1 打，r2 档那几行就与 ⑦ 不同。
[[ "${1:-}" == --pinned-cases && -f "${2:-}" ]] || { echo "桩只认 --pinned-cases <表> [档参数…]" >&2; exit 2; }
if [[ " ${*:3} " == *" --profile r2 "* ]]; then extra=,j6qn; else extra=; fi
grep -v '^#' "$2" | grep . | cut -f7 | sed "s/counters=states,j6qn,/counters=states,/; s/counters=states,/counters=states$extra,/"
echo "E7RESULT name=pinned_cases rows=$(grep -v '^#' "$2" | grep -c .)"
