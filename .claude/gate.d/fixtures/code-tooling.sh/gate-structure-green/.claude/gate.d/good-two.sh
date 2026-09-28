#!/usr/bin/env bash
# gate-stage: 样本：一格，格名表、--list 与表一致、--check 写错格名退 2
# gate-cell: gamma 判丙
ROOT="${1:-.}"
case " $* " in
  *" --list "*) printf 'gamma\t判丙\n'; exit 0 ;;
  *" --check gamma "*) ;;
  *" --check "*) echo "  ✗ 样本阶段只认 gamma 一个格名"; echo "     → 怎么办：照 --list 写格名"; exit 2 ;;
esac
cd "$ROOT" || exit 2
echo "  ✓ 样本阶段 1 格"
exit 0
