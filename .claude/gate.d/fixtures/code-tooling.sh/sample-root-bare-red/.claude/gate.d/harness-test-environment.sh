#!/usr/bin/env bash
# gate-stage: 样本：环境检查那一道（只占名字，这一格只看它排在第几）
ROOT="${1:-.}"
cd "$ROOT" || exit 2
exit 0
