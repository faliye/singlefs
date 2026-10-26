#!/usr/bin/env bash
# 暂存区里的记录是坏的，工作区那份已经修好：基准里一份合法的记录；这一轮往记录加一行带被拦腰切断的汉字（「中」只剩 e4 b8）并 git add，
# 然后把工作区那份改回合法的一行，不重新暂存。提交带走的是暂存区，所以要点名「暂存区」；工作区那一版合法，不点名。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records
printf '# 样本调度\n\n- 起点。\n' > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
printf '%s' '- 加一行：' >> records/2026-09-24-样本调度.md
printf '\xe4\xb8\n' >> records/2026-09-24-样本调度.md
git add records/2026-09-24-样本调度.md
printf '# 样本调度\n\n- 起点。\n- 加一行：修好了。\n' > records/2026-09-24-样本调度.md
