#!/usr/bin/env bash
# 红样本现场：一份带 fn run_cell 与 struct CellRun 的 rs，一份已经改好名却还登记在排除表里的 rs，一条指向不存在文件的排除项；
# 源文件由这里现写，仓里的样本目录本身不带 .rs（门禁 13 号扫的是 crates/ 与 research/e7-index-bench/src/，本来也扫不到这里，这样更稳）。
set -e
mkdir -p .claude crates/demo/src
printf 'run  # 没说跑的是什么\ncell  # 没说是哪一格\ncheck  # 没说查什么\nget  # 没说取的是什么\n' > .claude/naming-vague-words
printf 'crates/demo/src/renamed.rs  # 样本：已经改好名还登记着\ncrates/demo/src/gone.rs  # 样本：指向不存在的文件\n' > .claude/naming-vague-exclude
printf 'struct CellRun;\nfn run_cell() {}\nfn enumerate_stream_domain() {}\nfn main() { run_cell(); enumerate_stream_domain(); }\n' > crates/demo/src/main.rs
printf 'fn judge_the_segment_two_items() {}\n' > crates/demo/src/renamed.rs
git init -q . && git add -A
