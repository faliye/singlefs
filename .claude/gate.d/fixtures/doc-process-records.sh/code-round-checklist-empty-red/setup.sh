#!/usr/bin/env bash
# code-round-checklist 格的样本现场：一份代码轮正文八格的标题都在，「测试优先多线程」底下紧接下一个标题、一个字没写；日期在跑样本时现算（取今天往后 30 天）。
set -euo pipefail
sample_date="$(TZ=Asia/Tokyo date -d '+30 days' +%F)"
mkdir -p research/prompts
{
  printf '# c992-code-r1：样本代码轮第一轮正文（%s）\n\n' "$sample_date"
  for label in 走不到的理由二次查实 类型与名字 不兼容旧代码与旧格式 不新建判定库 所有任务多机可用 测试优先多线程 "带 --item 的命令与项" 新崩溃流多罩了什么; do
    if [[ "$label" == 测试优先多线程 ]]; then
      printf '### 检查单：%s\n\n' "$label"
    else
      printf '### 检查单：%s\n\n成立：样本里的答案与证据，crates/demo/src/lib.rs:1。\n\n' "$label"
    fi
  done
} > research/prompts/_c992-code-r1-body.md
