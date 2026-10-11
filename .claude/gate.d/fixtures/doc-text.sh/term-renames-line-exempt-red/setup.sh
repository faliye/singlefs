#!/usr/bin/env bash
# term-renames 那一格的红样本现场：与 term-renames-line-exempt-green 同一张登记表、同一份排除表与 git 忽略的 multi-host.env，
# 只是按行豁免的那份记录里另有一行没豁免、还用着旧名：只报那一行（1 份文件、1 处），判红。
# sweep-term 把按行豁免当成整份豁免（SWEEP_TERM_BREAK=line-exemption-covers-the-file）就判绿；正则去掉断言或扫了 git 忽略的文件，报的处数就不是 1。
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
  "这一句不是原话，${old_chinese}还没换" > records/quote.md
printf '%s\n' 'multi-host.env' > .gitignore
printf '%s\n' "PEER_REPOSITORY_DIRECTORY=/home/peer/${old_chinese}" "# ${old_layer_one} 常规" > multi-host.env
git add -A && git commit -qm base
