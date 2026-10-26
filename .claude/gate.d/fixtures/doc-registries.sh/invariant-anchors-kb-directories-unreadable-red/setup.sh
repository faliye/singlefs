#!/usr/bin/env bash
# kb 目录登记位 .claude/kb-directories 写进一个不是 UTF-8 的字节：scripts/kb/quote-kb.py 的 kb_candidate_roots 读它时抛 UnicodeDecodeError，
# 门禁里调它的那段 python 起不来、退非 0。invariant-anchors 要判红、说登记位读不出，不许当成候选目录是空的、退 77（改前不看退出码）。
# 这一格只在 git 仓里判，所以先建一个小仓。字节在这里现写，不把坏文件放进仓。
set -e
mkdir -p .claude
printf '.claude/kb  # 样本\n\377\n' > .claude/kb-directories
git init -q .
