#!/usr/bin/env bash
# gate-stage: 样本：两格，格名表、--list 与表一致、--check 写错格名退 2
# gate-cell: alpha 判甲
# gate-cell: beta 判乙
ROOT="${1:-.}"
case " $* " in
  *" --list "*) printf 'alpha\t判甲\nbeta\t判乙\n'; exit 0 ;;
  *" --check alpha "*|*" --check beta "*) ;;
  *" --check "*) echo "  ✗ 样本阶段只认 alpha、beta 两个格名"; echo "     → 怎么办：照 --list 写格名"; exit 2 ;;
esac
cd "$ROOT" || exit 2
echo "  ✓ 样本阶段 2 格"
exit 0
