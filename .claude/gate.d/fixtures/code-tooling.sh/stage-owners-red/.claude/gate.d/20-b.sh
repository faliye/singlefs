#!/usr/bin/env bash
# gate-stage: 样本阶段 20-b.sh（一格 b1；表里另有一行点名它没有的 b9）
# gate-cell: b1 判乙一
ROOT="${1:-.}"
case " $* " in *" --list "*) printf 'b1\t判乙一\n'; exit 0 ;; esac
cd "$ROOT" || exit 2
exit 0
