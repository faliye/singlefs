#!/usr/bin/env bash
# gate-stage: 文档里的链接与「第 N 节」指向到不到得了
#
# 两类失效都是静默的：
#   ① 相对路径少一层多一层 —— 点开是空的；更坏的是它**恰好指到另一个同名文件**，
#      那时连「打不开」这个信号都没有。
#   ② 「见 X.md 第三节」 —— 目标文档增删一节，这个序号就指向别的东西了，
#      而没有一个字看起来别扭（`.claude/singlefs-ai-sop/rules/kb-discipline.md`
#      第 5 条「编号只能做索引，不能做称呼」的同一个病，落在小节序号上）。
#
#   python3 .claude/gate.d/lib-link-targets.py
set -uo pipefail
LIB="$(cd "$(dirname "$0")" && pwd)/lib-link-targets.py"
cd "${1:-$(dirname "$0")/../..}" || exit 2
exec python3 "$LIB"
