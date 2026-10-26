#!/usr/bin/env bash
# kb 目录登记位 .claude/kb-directories 写进一个不是 UTF-8 的字节：scripts/kb/quote-kb.py 的 kb_candidate_roots 读它时抛 UnicodeDecodeError，
# 门禁里调它的那段 python 起不来、退非 0。declared-counts 要判红、说登记位读不出，不许报成「一个带不变量表行的文件都没扫到」
# （改前不看退出码，候选目录当空的）；欠账表那一半照数（样本里的 checks-owed.md 一条欠着、一条还清）。字节在这里现写，不把坏文件放进仓。
set -e
mkdir -p .claude
printf '.claude/kb  # 样本\n\377\n' > .claude/kb-directories
