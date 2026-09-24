#!/usr/bin/env bash
# 红样本现场：带撇号的两份文件由这里现写（八进制转义），仓里的样本文件本身不出现那几个字符——
# 否则门禁 12 号扫全仓时会先把自己的红样本判红。
set -e
mkdir -p .claude/kb crates/demo/src
printf '## 候选\n\n取 K9\342\200\262（收严版），不取 K9。\n' > .claude/kb/候选.md
printf '// 变体 G5\342\200\263 的开销\nfn main() {}\n' > crates/demo/src/main.rs
