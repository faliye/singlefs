#!/usr/bin/env bash
# 子 agent 等长活时的缓存计时器：用 Bash 的 run_in_background 起它，默认 230 秒后退出，
# harness 的完成通知把子 agent 叫醒，那一次模型调用就把提示缓存续上了。
#
# 为什么：子 agent 的提示缓存是 5 分钟档（2026-09-18 现查会话记录：子 agent 的缓存写入全在
# `ephemeral_5m`，主会话全在 `ephemeral_1h`）。超过 5 分钟不调用模型，缓存过期、整份上下文要重写——
# 2026-09-17 实测一段执行员重写 6 次，每次 36–59 万 token。计时器由子 agent 自己起，主 agent 不参与。
# 2026-09-18 A/B 实测：起计时器的那一组 18 分钟里少花 25,807 折基础输入 token（占 5.5%）；
# 一次叫醒约 4 万、一次整份重写约 14 万，所以单段等 15 分钟以上的不起更省。
# 经过与数见 records/2026-09-16-subagent拆分提案.md 第二十一节。
#
#   cache-keepalive.sh [秒数]     # 默认 230 秒，要小于 5 分钟减去一次模型调用的时间
#   cache-keepalive.sh --selftest # 跑 1 秒的一轮，核退出码与那句下一步；CACHE_KEEPALIVE_DISABLE_HINT=1 时自检必须判红
set -uo pipefail

HINT="缓存计时器到点。看一眼你等的后台任务跑完没有：没跑完就再用 run_in_background 起一次本脚本、结束本轮接着等；跑完了就接着干，不用再起。自己已经交回或被停的，不用管。"

if [[ "${1:-}" == "--selftest" ]]; then
  output="$(bash "$0" 1 2>&1)"; rc=$?
  failures=0
  if [[ $rc -ne 0 ]]; then
    echo "  ✗ 自检：跑一轮应当退出码 0，实际 $rc"   # gate-lint:detail
    failures=1
  fi
  if [[ "$output" != *"看一眼你等的后台任务"* ]]; then
    echo "  ✗ 自检：到点那句话里没有下一步（实际输出：$output）"   # gate-lint:detail
    failures=1
  fi
  bad_seconds_rc=0
  bash "$0" abc >/dev/null 2>&1 || bad_seconds_rc=$?
  if [[ $bad_seconds_rc -eq 0 ]]; then
    echo "  ✗ 自检：秒数不是正整数时应当判红，实际退出码 0"   # gate-lint:detail
    failures=1
  fi
  if [[ $failures -ne 0 ]]; then
    echo "    → 怎么办：看本脚本的 HINT 与秒数校验；CACHE_KEEPALIVE_DISABLE_HINT 设着的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ 缓存计时器自检通过：跑一轮退出码 0、到点那句带下一步、秒数不是正整数判红（查了 3 项）"
  exit 0
fi

SECONDS_TO_WAIT="${1:-230}"
if [[ ! "$SECONDS_TO_WAIT" =~ ^[1-9][0-9]*$ ]]; then
  echo "✗ 秒数要是正整数，收到「$SECONDS_TO_WAIT」" >&2
  echo "→ 怎么办：不带参数（默认 230 秒），或给一个小于 300 的正整数。" >&2
  exit 2
fi

sleep "$SECONDS_TO_WAIT"
if [[ "${CACHE_KEEPALIVE_DISABLE_HINT:-}" == "1" ]]; then
  exit 0
fi
echo "$HINT"
