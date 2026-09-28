#!/usr/bin/env bash
# admission: always 复核这一轮云端正推腿的一次性观测（E161 装置在当前 HEAD 上跑 feasibility 模式，不改代码）
# run-condition: command cargo
# 复跑做法（与本轮跑法相同）：
#   1. 在仓根 `git archive HEAD | tar -x -C <某目录>`（不改主仓）
#   2. 把主仓 `.claude/singlefs-ai-sop/` 拷进那份副本（它被 .gitignore 忽略、不随 git archive 走）
#   3. cd 进副本，`CARGO_TARGET_DIR=<副本外的编译目录> cargo build --release --bin e161_crash_state_dedup_and_time_split -p singlefs-checker-tier`（经 run-with-memory-cap.sh 8G、capped.sh 4）
#   4. 跑 `<编译目录>/release/e161_crash_state_dedup_and_time_split feasibility`（同样经内存与线程包装），E161_THREADS=4
# 本轮实测：跑到 `LAYER0_PARALLEL_START states=16777260 ...` 那一行说明 first_small domain 在今天的段结构下已经不是
# 2026-09-27 产物里的 37 个状态、而是整条第一条流的 16777260 个状态（11 段、24 写段在 segment=7，不是当时的 26 写段），
# 与 `research/prompts/m3-prune-gpu-r1-facts-k4.md` 第一节「现工作区复现不出：流的形状变了」一致；
# 这一条不整段展开（资源约束「只跑小域或按步长取样，不整段展开」），跑到这一行就用 proc.py 逐层停掉。
echo "见本文件头注释；e161-rerun-head.out 是这一次跑出来的原样输出（被中途停掉，最后一行是 LAYER0_PARALLEL_START）"
