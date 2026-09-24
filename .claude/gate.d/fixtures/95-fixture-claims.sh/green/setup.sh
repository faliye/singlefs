#!/usr/bin/env bash
# 绿样本也要建 git 仓：不是 git 仓时第 ④ 条报「未跑」，那一格就没验到。
# 两维各放一个日期落在可能区间里的对象，证明这一条在绿的一侧真的在看——
# 扫到 0 项而报绿，与判过了一模一样（show-me-test.md）。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
GIT_COMMITTER_DATE="2026-09-01T00:00:00" GIT_AUTHOR_DATE="2026-09-01T00:00:00" sh -c 'git add -A && git commit -qm base'
mkdir -p records
printf '# 样本\n' > records/2026-09-02-样本.md
printf '## D99 样本决策 —— 已定\n\n| # | 名字 | 状态 |\n|---|---|---|\n| 1 | 甲项 | **已定（2026-09-02）：取甲** |\n' \
  > .claude/gate.d/fixtures/50-good.sh/green/样本决策.md
# 下界内第 7 天：DATE_GRACE_DAYS 被改小，这一个就会越界、green 转红
printf '# 样本\n' > records/2026-08-25-样本.md
# 合法的月文件：整月与允许区间有交集（这个沙箱起点 2026-09-01，下界 2026-08-25），
# 拿 2026-08-01 当它比会误判，所以判的是「整月都在区间外」才红
mkdir -p .claude/kb/decisions-history
printf '# 样本\n' > .claude/kb/decisions-history/2026-08.md
