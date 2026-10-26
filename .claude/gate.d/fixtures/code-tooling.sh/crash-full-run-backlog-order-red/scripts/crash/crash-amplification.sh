#!/usr/bin/env bash
# 样本里的假崩溃放量入口：只认 --list-items，照真的那一份逐行打分项（分项名、测试目标、测试名、说明，制表符隔开）
ITEMS=(
  $'alpha-amp\talpha\talpha_amplified\t全量'
  $'gamma-amp\tgamma_amp\tgamma_amplified\t全量'
)
if [[ "${1:-}" != --list-items ]]; then
  echo "  ✗ 样本里的假放量入口只认 --list-items" >&2
  echo "  → 怎么办：它只给门禁 code-tooling 的 crash-full-run-backlog 格列 ITEMS 用" >&2
  exit 2
fi
printf '%s\n' "${ITEMS[@]}"
