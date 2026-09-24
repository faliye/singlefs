#!/usr/bin/env bash
# 红样本要判的第三条是「样本目录是空壳」：目录在，里面一个 red / green 都没有。
# 这个空目录只能在这里建——git 不跟踪空目录，放在样本的文件树里带不进 `gate.sh --staged`
# 的临时 worktree，那一格就判不出来（stage-selftest.sh 先把样本拷进临时目录再跑这个脚本）。
set -euo pipefail
mkdir -p .claude/gate.d/fixtures/51-hollow.sh

# 第 ④ 条（日期不许是编的）的坏样本也在这里现造，不摆进仓里——摆进来会被这一条在真仓跑时自己扫到（C441）。
# 先做一个真实日期的提交：project_start_date 取不到值时下界为空，红样本就只证了上界，
# 而这次踩的坑在下界（2026-01-01 比第一个提交早大半年）。
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
GIT_COMMITTER_DATE="2026-09-01T00:00:00" GIT_AUTHOR_DATE="2026-09-01T00:00:00" sh -c 'git add -A && git commit -qm base'
# 文件名那一维
mkdir -p records
printf '# 样本\n' > records/2026-01-01-样本.md
# 正文那一维：挂在已有的孤儿目录下，不新建目录——新建会把第 ① 条的红（声称了而目录不存在）消掉
printf '## D99 样本决策 —— 已定\n\n| # | 名字 | 状态 |\n|---|---|---|\n| 1 | 甲项 | **已定（2026-01-02）：取甲** |\n' \
  > .claude/gate.d/fixtures/99-orphan.sh/red/样本决策.md
# 边界那两支也要有红样本，否则砍掉上界、或者把 DATE_GRACE_DAYS 放大，红绿两格照样「判得对」
# （2026-09-23 变异实测：M1 砍掉「晚于今天」整支、M9 把 7 改成 200，两格都没被抓到）。
printf '# 样本\n' > records/2099-01-01-样本.md            # 上界：晚于最晚时区的今天
printf '# 样本\n' > records/2026-08-24-样本.md            # 下界外第 8 天（这个沙箱的起点是 2026-09-01）
# 按月命名的文件也要有红样本：它不含「日」，YYYY-MM-DD 那把尺抓不到
mkdir -p .claude/kb/decisions-history
printf '# 样本\n' > .claude/kb/decisions-history/2026-01.md
