#!/usr/bin/env bash
# gate-stage: 样本：带编号、不声称有样本的干净阶段（阶段只认两位数开头时，那一格只看得见它）
ROOT="${1:-.}"
cd "$ROOT" || exit 2
exit 0
