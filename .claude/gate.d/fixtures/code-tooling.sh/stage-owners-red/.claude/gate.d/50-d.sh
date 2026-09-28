#!/usr/bin/env bash
# gate-stage: 样本阶段 50-d.sh（有格名表，--list 却退 1：列不出格）
# gate-cell: d1 判丁一
ROOT="${1:-.}"
case " $* " in *" --list "*) echo "  样本：--list 坏了"; exit 1 ;; esac
cd "$ROOT" || exit 2
exit 0
