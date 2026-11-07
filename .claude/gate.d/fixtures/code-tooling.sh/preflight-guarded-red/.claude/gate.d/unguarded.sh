#!/usr/bin/env bash
# gate-stage: 样本：source 了 preflight.sh，preflight 那一行（第 6 行）之后没有守卫；树里有替身 preflight.sh，跑起来照常，只有守卫那一样不对
# gate-cell: alpha 判甲
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-.}"
case " $* " in
  *" --list "*) printf 'alpha\t判甲\n'; exit 0 ;;
  *" --check alpha "*) ;;
  *" --check "*) echo "  ✗ 样本阶段只认 alpha 一个格名"; echo "     → 怎么办：照 --list 写格名"; exit 2 ;;
esac
cd "$ROOT" || exit 2
echo "  ✓ 样本阶段 1 格"
exit 0
