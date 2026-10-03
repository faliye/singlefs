#!/usr/bin/env bash
# checker-tier-layer0-replay 「假 cargo 不可执行」样本的现场：样本拷进临时目录之后，假 cargo 去掉执行位、假 rustc 补上执行位（进库的模式与拷法不管是什么，到这里都定成这样）。
# 阶段在核假工具那一步退 2，用不到 git 仓；弄坏开关 sample-tools-unchecked 打开时，这里没有 crates/ 与 stage-inputs.tsv，阶段读登记表那一步退 1，起不到 cargo。
set -euo pipefail
chmod 644 .layer0-sample-tools/cargo
chmod 755 .layer0-sample-tools/rustc
