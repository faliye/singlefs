#!/usr/bin/env bash
# gate-stage: 样本：pipefail 下用 grep -q 收尾
set -uo pipefail
if git show --stat --format= HEAD 2>/dev/null | grep -q "decisions.md"; then
  echo hit
fi
