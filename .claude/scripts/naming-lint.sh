#!/usr/bin/env bash
# 包装：转发到共享脚本。逻辑不写在这里，写在 .claude/singlefs-ai-sop/scripts/。
# 只多做一件事：副本不在时说清怎么办。副本被 .gitignore 挡着，新 clone 出来的仓里它不存在，
# 而 exec 一个不存在的路径只会得到 bash 的「No such file or directory」，一句出路都没有。
shared="$(dirname "${BASH_SOURCE[0]}")/../singlefs-ai-sop/scripts/naming-lint.sh"
if [[ ! -f "$shared" ]]; then
  echo "  ✗ 找不到共享脚本：$shared"
  echo "     → 怎么办： 规范副本没装（它被 .gitignore 挡着，不随仓库走）。"
  echo "                把 singlefs-ai-sop-<语言> 仓拷进 .claude/singlefs-ai-sop/，再跑它的 install.sh。"
  exit 1
fi
exec bash "$shared" "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)" "$@"
