#!/usr/bin/env bash
# 门禁 crash-point-replay「记红没成、下一趟不复用旧绿」样本的现场：与 red-rejudged-next-pass 同一份假 cargo 与测试文件。
# 样本拷进临时目录之后，照这一版逐字拷一份上一版进 .unit-verdict-sample/prior/（连同 .claude/gate.d 下的登记表：上一版算键时带 --exclude-registered，
# 从那一版的树里读登记的崩溃枚举用例）：阶段在不带特性那一趟开跑前拿它照判绿记进库，库里就有 shared_check 同一把键的绿。
# 再 git init、这一版连 HEAD 一起提交（判定库放 git common-dir，单元键要 git 列文件）；假工具补上执行位
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -dR --preserve=timestamps,links Cargo.toml crates .claude "$prior/"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"
