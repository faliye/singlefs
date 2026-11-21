#!/usr/bin/env bash
# code-round-checklist 格的样本现场：一份标题日期就是第九格截止日（2026-11-19）、不在只判八格名单里的代码轮正文八格齐，缺第九格「放的位置」。
set -euo pipefail
mkdir -p research/prompts
{
  printf '# c999-code-r1：样本代码轮第一轮正文（2026-11-19）\n\n'
  for label in 走不到的理由二次查实 类型与名字 不兼容旧代码与旧格式 不新建判定库 所有任务多机可用 测试优先多线程 "带 --item 的命令与项" 新崩溃流多罩了什么; do
    printf '### 检查单：%s\n\n成立：样本里的答案与证据，crates/demo/src/lib.rs:1。\n\n' "$label"
  done
} > research/prompts/_c999-code-r1-body.md
