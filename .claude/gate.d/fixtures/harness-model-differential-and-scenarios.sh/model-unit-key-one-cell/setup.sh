#!/usr/bin/env bash
# harness-model-differential-and-scenarios 按格复用样本的现场：样本拷进临时目录之后，照这一版拷一份上一版进 .unit-verdict-sample/prior/，
# 只改回复用取样点那一格调的 reuse_steps（恰好改中一次，改不中退 1）；git init（判定库放 git common-dir），假工具补上执行位。
set -euo pipefail
chmod +x .model-differential-sample-tools/cargo .model-differential-sample-tools/rustc
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -a Cargo.toml crates "$prior/"
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' "$prior/crates/singlefs-harness/tests/random_histories.rs" "fn reuse_steps() -> u32 { 1183 }" "fn reuse_steps() -> u32 { 1180 + 3 }"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
