#!/usr/bin/env bash
# S6 样本：在当前目录（要是新的空目录）建 git 仓，假配置的值都是编的（不对应任何真机器），造 doc-text 各格的输出通道：
#   clock-times  片段取命中位置前 20 个字、后 10 个字，两头落在假值中间（左边两行、右边一行）
#   prime-marks  / session-names 同上（撇号类字符与时间戳在运行时用片段拼出来，脚本里不写整串，免得这份脚本自己被仓里的同名门禁判红）
#   date-once    文件名带假主机名、正文重复所属日期的记录
#   term-renames 说明文件放在带假仓路径的目录下，用了登记过的旧名（sweep-term 逐份打路径与处数）；按行豁免的行内锚那一路由 rerun.sh 在这棵样本的拷贝里另加一行（锚对不上时 sweep-term 把锚原样打出，并且先于逐份列出就退出）
# 用法：bash setup.sh（在新的空目录里跑）
set -e
cd "$(pwd -P)"
host="sample-second-host-77" path="/srv/sample-private/second-checkout" local_host="sample-local-host-11"
printf '%s\n' '# 样本：假的多机私有配置。所有值都是编的。' "PEER_SSH_HOST=$host" "PEER_REPOSITORY_DIRECTORY=$path" 'PEER_MEMORY_CAP=2G' > multi-host.env
printf '%s\n' "$local_host" > .private-values-hostname
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' '/.private-values-hostname' > .gitignore
mkdir -p records research/results .claude/kb scripts/kb
printf '# 样本调度\n\n## 2026-10-13\n\n- 干净的一行。\n' > records/2026-10-13-样本调度.md
printf '%s\n' '# 术语改名对照表' '' '<!-- term-renames:table -->' '' '| 旧 | 新 | 匹配 | 是什么 |' '|---|---|---|---|' '| 超级块 | 系统配置 | 整串 | 中文术语 |' > .claude/kb/term-renames.md
: > scripts/kb/sweep-term.py
printf '%s\n' 'scripts/kb/sweep-term.py  # 清扫工具自己' '.claude/term-rename-exempt  # 这张表自己' '.claude/kb/term-renames.md  # 对照表自己' > .claude/term-rename-exempt
git add -A && git commit -qm base
iso_date="2026-10-13"; iso_time="10:31:05"; ts="${iso_date}""T""${iso_time}"
prime="$(python3 -c 'print(chr(0x2032), end="")')"
printf '%s\n' "- 第二台仓路径 $path 上 $ts 收尾。" >> records/2026-10-13-样本调度.md
printf '%s\n' "- 对照：第二台 $host 上 ${iso_date}""T""11:02:00 开跑。" >> records/2026-10-13-样本调度.md
printf '%s\n' "- 第二台仓路径 $path 之后五个字再出现 K9${prime} 这个角标。" >> records/2026-10-13-样本调度.md
printf '%s\n' "- 另一行：主机 $host 紧挨着前面 G5${prime}${prime} 角标。" >> records/2026-10-13-样本调度.md
session="singlefs""-ab12cd"
printf '%s\n' "- 会话 $session 之后紧跟 $host 这个第二台。" >> records/2026-10-13-样本调度.md
printf '%s\n' "- 会话 $session 之后紧跟 $path 这个路径。" >> records/2026-10-13-样本调度.md
printf '%s\n' '# 第二台对拍' '' '## 2026-10-13' '' '- 对拍开跑（2026-10-13），第二台照常。' > "records/2026-10-13-$host-对拍.md"
d="research/results/from-peer${path}"
mkdir -p "$d"
printf '%s\n' '第二台的说明：超级块那一节照旧。' > "$d/notes.md"
