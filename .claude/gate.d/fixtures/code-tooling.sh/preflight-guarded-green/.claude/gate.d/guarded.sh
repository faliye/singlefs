#!/usr/bin/env bash
# gate-stage: 样本：source 了 preflight.sh，preflight 那一行之后紧跟守卫（找不到 preflight 时打 ✗ 与出路、退 2）；树里有替身 preflight.sh
# gate-cell: alpha 判甲
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ guarded.sh：source .claude/scripts/preflight.sh 没成，preflight 没有定义、参数已被清空，拒绝往下跑"
  echo "     → 怎么办：样本树里补上 .claude/scripts/preflight.sh"
  exit 2
fi
ROOT="${1:-.}"
case " $* " in
  *" --list "*) printf 'alpha\t判甲\n'; exit 0 ;;
  *" --check alpha "*) ;;
  *" --check "*) echo "  ✗ 样本阶段只认 alpha 一个格名"; echo "     → 怎么办：照 --list 写格名"; exit 2 ;;
esac
cd "$ROOT" || exit 2
echo "  ✓ 样本阶段 1 格"
exit 0
