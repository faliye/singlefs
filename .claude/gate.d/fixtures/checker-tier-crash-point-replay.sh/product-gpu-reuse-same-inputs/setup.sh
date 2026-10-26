#!/usr/bin/env bash
# checker-tier-crash-point-replay 产品 GPU 包那一格的样本现场：样本拷进临时目录之后在那里 git init（快档按条复用的判定库放 git common-dir，单元键要 git 列文件），
# 假工具补上执行位（拷的时候丢了也不碍事）。
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
# 上一版：与这一版逐字相同（先拷一份，替换文与原文相同）
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -a Cargo.toml crates "$prior/"
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' "$prior/crates/singlefs-background-gpu/src/lib.rs" 'pub fn spare_function() -> u32 {
    7
}' 'pub fn spare_function() -> u32 {
    7
}'
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"
