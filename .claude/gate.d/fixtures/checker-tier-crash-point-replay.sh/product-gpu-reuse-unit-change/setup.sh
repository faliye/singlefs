#!/usr/bin/env bash
# checker-tier-crash-point-replay 产品 GPU 包那一格的样本现场：样本拷进临时目录之后在那里 git init（快档按条复用的判定库放 git common-dir，单元键要 git 列文件），
# 假工具补上执行位（拷的时候丢了也不碍事）。
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
# 上一版：只把 weighted_sum_of 里的权重改回去
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -a Cargo.toml crates "$prior/"
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' "$prior/crates/singlefs-background-gpu/src/lib.rs" '(index as u32 + 1)' '(index as u32 + 2)'
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"
