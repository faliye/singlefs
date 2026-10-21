#!/usr/bin/env bash
# checker-tier-crash-point-replay「两趟一份库」样本的现场：样本拷进临时目录之后，
# 1. 照这一版拷一份上一版进 .unit-verdict-sample/prior/，只把库单测 tests::shared_check 的断言改回去（恰好改中一次，改不中退 1）：
#    阶段两趟开跑前各拿那一版算键、照判绿记进这一版样本的判定库（挂特性的 store_only_quick 在带特性那一趟记下，键与这一版相同）。
# 2. git init，这一版连 HEAD 一起提交（判定库放 git common-dir，单元键要 git 列文件）。
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -a Cargo.toml crates "$prior/"
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' "$prior/crates/singlefs-checker-tier/src/lib.rs" "super::one(), 1" "super::one() + 0, 1"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"
