#!/usr/bin/env bash
# gate-stage: 样本阶段 30-c.sh（一格 c1 登记着，但不认第一个参数当项目根）
# gate-cell: c1 判丙一
case " $* " in *" --list "*) printf 'c1\t判丙一\n'; exit 0 ;; esac
exit 0
