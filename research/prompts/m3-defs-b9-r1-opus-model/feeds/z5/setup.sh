#!/usr/bin/env bash
# Z5 样本：在当前目录建 git 仓（假配置的值都是编的），提交一版干净记录与改名表，再造这一次改动：
#   records/2026-10-13-样本调度.md 加一行：从第二台进度日志抄来的一句，第二台仓路径紧挨着一个时间戳（clock-times 红，片段截在值的中间）；
#     再加一行对照：第二台主机名整个落在片段里
#   records/2026-10-13-demo-peer-host-42-对拍.md：文件名里有第二台主机名（编的），正文一行重复标题日期（date-once 红）
#   research/results/from-peer/srv/demo-private/peer-checkout/notes.md：从第二台 tar 过来的说明，路径里带第二台仓路径，正文用了登记过的旧名（term-renames 红）
# 用法：bash setup.sh（在一个新的空目录里跑，multi-host.env 与 .private-values-hostname 从本脚本旁边拷）
set -e
here="$(cd "$(dirname "$0")" && pwd)"
cp "$here/multi-host.env" "$here/.private-values-hostname" .
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' '/.private-values-hostname' > .gitignore
mkdir -p records research/results .claude/kb scripts/kb
printf '# 样本调度\n\n## 2026-10-13\n\n- 干净的一行。\n' > records/2026-10-13-样本调度.md
printf '%s\n' '# 术语改名对照表' '' '<!-- term-renames:table -->' '' '| 旧 | 新 | 匹配 | 是什么 |' '|---|---|---|---|' '| 超级块 | 系统配置 | 整串 | 中文术语 |' > .claude/kb/term-renames.md
: > scripts/kb/sweep-term.py
printf '%s\n' 'scripts/kb/sweep-term.py  # 清扫工具自己' '.claude/term-rename-exempt  # 这张表自己' '.claude/kb/term-renames.md  # 对照表自己' > .claude/term-rename-exempt
git add -A && git commit -qm base
printf '%s\n' '- 第二台进度日志抄来的一句：跑在 /srv/demo-private/peer-checkout 上 2026-10-13T10:31:05 收尾。' >> records/2026-10-13-样本调度.md
printf '%s\n' '- 对照：第二台 demo-peer-host-42 上 2026-10-13T11:02:00 开跑。' >> records/2026-10-13-样本调度.md
printf '%s\n' '# 第二台对拍' '' '## 2026-10-13' '' '- 对拍开跑（2026-10-13），第二台照常。' > records/2026-10-13-demo-peer-host-42-对拍.md
d=research/results/from-peer/srv/demo-private/peer-checkout
mkdir -p "$d"
printf '%s\n' '第二台的说明：超级块那一节照旧。' > "$d/notes.md"
