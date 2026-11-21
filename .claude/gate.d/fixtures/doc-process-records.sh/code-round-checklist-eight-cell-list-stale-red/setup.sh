#!/usr/bin/env bash
# code-round-checklist 格的样本现场：只判八格的名单（样本根里的 .claude/gate.d/code-round-checklist-eight-cells.tsv）第一行指的 _c996-code-r1-body.md 不在；
# 另有一份标题日期晚于第九格截止日的代码轮正文九格齐（这一格有对象可判）。日期在跑样本时现算（取今天往后 30 天）。
set -euo pipefail
sample_date="$(TZ=Asia/Tokyo date -d '+30 days' +%F)"
mkdir -p research/prompts
{
  printf '# c998-code-r1：样本代码轮第一轮正文（%s）\n\n' "$sample_date"
  for label in 走不到的理由二次查实 类型与名字 不兼容旧代码与旧格式 不新建判定库 所有任务多机可用 测试优先多线程 "带 --item 的命令与项" 新崩溃流多罩了什么 放的位置; do
    printf '### 检查单：%s\n\n成立：样本里的答案与证据，crates/demo/src/lib.rs:1。\n\n' "$label"
  done
} > research/prompts/_c998-code-r1-body.md
