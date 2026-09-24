#!/usr/bin/env bash
# 样本：.claude/scripts/ 里的包装，拒绝带出路、只起一个子进程，两个 lint 都该判绿。
shared="$(dirname "${BASH_SOURCE[0]}")/../singlefs-ai-sop/scripts/check.sh"
if [[ ! -f "$shared" ]]; then
  echo "  ✗ 找不到共享脚本：$shared"
  echo "     → 怎么办：规范副本没装，把 singlefs-ai-sop-<语言> 仓拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh。"
  exit 1
fi
exec bash "$shared" "$@"
