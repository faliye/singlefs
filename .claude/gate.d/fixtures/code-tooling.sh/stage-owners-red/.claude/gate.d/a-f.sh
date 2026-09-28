#!/usr/bin/env bash
# gate-stage: 样本：字母开头、文件头没有格名表的阶段（不起它的 --list，报列不出格）
ROOT="${1:-.}"
cd "$ROOT" || exit 2
exit 0
