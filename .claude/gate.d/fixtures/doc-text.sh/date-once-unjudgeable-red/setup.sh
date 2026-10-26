#!/usr/bin/env bash
# 干跑出错不当干跑 0 处：这一轮加一份带不合法 UTF-8 字节的记录（doc-consolidate.py 读不了它、退 1）与一份干净的记录。
# date-once 点名坏字节那一份「干跑退 1，这一份没判」，判红；弄坏开关 date-once-exit-ignored 下不收退出码，读不到摘要当干跑 0 处，退 0。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records
printf '%s\n' '# 基准记录' '' '- 基准里就有的一行。' > records/2026-10-22-基准记录.md
git add -A && git commit -qm base
printf '# 坏字节样本\n\n- 这一行有两个坏字节 \377\376。\n' > records/2026-10-23-坏字节样本.md
printf '%s\n' '# 干净样本' '' '- 这一行没有日期。' > records/2026-10-23-干净样本.md
