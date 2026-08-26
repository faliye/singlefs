#!/usr/bin/env bash
# 包装：转发到共享脚本。逻辑不写在这里，写在 .claude/singlefs-ai-sop/scripts/。
exec bash "$(dirname "${BASH_SOURCE[0]}")/../singlefs-ai-sop/scripts/lkmm.sh" "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)" "$@"
