#!/usr/bin/env bash
# gate-stage: 样本：先落到变量再判
set -uo pipefail
# ⚠️ 注释里写 `... | grep -q ...` 不算违规，检查只看非注释行。
out=$(git show --stat --format= HEAD 2>/dev/null || true)
if grep -q "decisions.md" <<<"$out"; then
  echo hit
fi
