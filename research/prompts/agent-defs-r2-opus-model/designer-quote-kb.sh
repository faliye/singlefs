#!/usr/bin/env bash
# agent-defs-r2 云端攻方腿模型三：experiment-designer「登记的固定节名」第二节要求「research/scripts/quote-kb.py 整段抄，脚本回读一致」，
# 写范围「只写 research/prompts/e<号>-preregistration.md（由 claim-experiment.sh 排他新建，之后 >> 追加或定点替换）」，输入里没有草稿目录。
# quote-kb.py 只有「写一个出口文件」这一种产出；出口写成登记本身时，claim-experiment.sh 写的文件头（占号时刻）与第一节被整份盖掉，脚本照样退出码 0。
# 只在临时目录里造一份登记与一份条款文件；仓里的 quote-kb.py 只读调用。
#   TMPDIR=<草稿目录> bash designer-quote-kb.sh
set -uo pipefail
REPO="${REPO:-/home/fy5090/code/singlefs}"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK:?}"' EXIT
cd "$WORK" || exit 2
mkdir -p research/prompts kb
# 与 claim-experiment.sh 第 57 行同形的文件头
( set -C; printf '# E%s 跑前登记：%s\n\n写于 %s JST，装置写之前。\n' 999 模型 "2026-09-17 16:30" > research/prompts/e999-preregistration.md )
printf '\n## 一、问题\n\n主 agent 给的问题逐字。\n' >> research/prompts/e999-preregistration.md
printf '#### 已定项 7：样本条款\n\n条款正文第一行。\n条款正文第二行。\n' > kb/sample.md
lines_before="$(wc -l < research/prompts/e999-preregistration.md)"
header_before="$(grep -c '装置写之前' research/prompts/e999-preregistration.md)"
python3 "$REPO/research/scripts/quote-kb.py" research/prompts/e999-preregistration.md 'kb/sample.md:1-4' >quote.log 2>&1; rc=$?
echo "name=direct_to_prereg quote_kb_rc=$rc lines_before=$lines_before lines_after=$(wc -l < research/prompts/e999-preregistration.md) header_before=$header_before header_after=$(grep -c '装置写之前' research/prompts/e999-preregistration.md) section_one_after=$(grep -c '^## 一、问题' research/prompts/e999-preregistration.md) log=$(tail -1 quote.log | tr -s ' ' | tr ' ' '_')"
echo "name=done cases=1"
