#!/usr/bin/env bash
# kb 目录登记位 .claude/kb-directories 写进一个不是 UTF-8 的字节：scripts/kb/quote-kb.py 的 kb_candidate_roots 读它时抛 UnicodeDecodeError，
# 门禁里调它的那段 python 起不来、退非 0。audit-contradictions 要判红、说登记位读不出，不许当成 kb 里一份不变量表都没有往下判
# （改前不看退出码，引了不变量的行全按 kb 别处去找）。records/ 下放一份总审核记录，这一格才有对象可判。字节在这里现写，不把坏文件放进仓。
set -e
mkdir -p .claude records
printf '.claude/kb  # 样本\n\377\n' > .claude/kb-directories
printf '# 样本总审核\n\n判别力样本，不是真的总审核记录。\n' > records/2026-01-01-总审核.md
