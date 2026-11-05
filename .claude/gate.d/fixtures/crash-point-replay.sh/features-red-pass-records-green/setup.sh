#!/usr/bin/env bash
# 门禁 crash-point-replay「一趟里有红也照键记绿」样本（feature-gated-tests 格）的现场：与 quick-red-rejudged-next-pass 同一份假 cargo，挂特性的 store_only.rs 里一红一绿，没有上一版（.unit-verdict-sample/prior/）。
# 样本拷进临时目录之后 git init，这一版连 HEAD 一起提交（判定库放 git common-dir，单元键要 git 列文件）；假工具补上执行位
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
git init -q .
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"
