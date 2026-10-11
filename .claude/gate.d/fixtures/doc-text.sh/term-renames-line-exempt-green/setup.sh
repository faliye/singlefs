#!/usr/bin/env bash
# term-renames 那一格的绿样本现场：git 仓里一张带「正则」行的登记表、一份带按行豁免的排除表、一份 git 忽略的本机配置。
# 「正则」那一行只打 D13 那一层的写法：记录里树层、外层那一行不该打中；用户原话那一行按行豁免留旧名；git 忽略的 multi-host.env 里有旧名也不该读。
# sweep-term 正则去掉断言（SWEEP_TERM_BREAK=regex-assertions-dropped）或扫了 git 忽略的文件（ignored-files-walked），这份样本就判红；
# 按行豁免当成整份豁免（line-exemption-covers-the-file）在这里看不出，由 term-renames-line-exempt-red 判。
# 旧名由几段拼进变量，这份 setup.sh 与样本目录里都不出现旧名的字面：仓里跑这一道时 term-renames 那一格扫得到样本目录。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
layer="层"
old_layer_one="${layer} 1"
old_chinese="超""级块"
lookbehind='(?<![每第内外里叶二])(?<![0-9] )'
mkdir -p .claude/kb records
printf '%s\n' '# 判别力样本的豁免表' '.claude/kb/term-renames.md  # 表自己写着旧名' '.claude/term-rename-exempt  # 这张表自己写着旧名' \
  "records/quote.md:「用户原话「${old_chinese}要改名」」  # 用户原话，换了就成假话" > .claude/term-rename-exempt
printf '%s\n' '# 判别力样本的改名登记表' '' '<!-- term-renames:table -->' '' \
  '| 旧 | 新 | 匹配 | 是什么 |' '|---|---|---|---|' \
  "| ${old_chinese} | 系统配置 | 整串 | 中文术语 |" \
  "| \`${lookbehind}${old_layer_one}(?![0-9])\` | 中等负载崩溃点剪枝穷举测试 | 正则 | D13 那一层 |" \
  '' '## 历史版本' '' '### 2026-10-10' '' '- 首版。' > .claude/kb/term-renames.md
printf '%s\n' "用户原话「${old_chinese}要改名」，原样留着" '树表每层 134 棵，第 1 层 2 个；外层 16.33 ms，内部层 110 个孩子' \
  '中等负载崩溃点剪枝穷举测试与系统配置照新名写' > records/quote.md
printf '%s\n' 'multi-host.env' > .gitignore
printf '%s\n' "PEER_REPOSITORY_DIRECTORY=/home/peer/${old_chinese}" "# ${old_layer_one} 常规" > multi-host.env
git add -A && git commit -qm base
