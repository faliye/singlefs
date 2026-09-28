#!/usr/bin/env bash
# gate-stage: 样本：格名表两格，--list 只打一格
# gate-cell: alpha 判甲
# gate-cell: beta 判乙
ROOT="${1:-.}"
case " $* " in
  *" --list "*) printf 'alpha\t判甲\n'; exit 0 ;;
  *" --check "*) echo "  ✗ 样本阶段只认 alpha、beta 两个格名"; echo "     → 怎么办：照 --list 写格名"; exit 2 ;;
esac
cd "$ROOT" || exit 2
exit 0
