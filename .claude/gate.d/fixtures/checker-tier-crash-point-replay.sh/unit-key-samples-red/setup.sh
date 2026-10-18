#!/usr/bin/env bash
# checker-tier-crash-point-replay 按条复用样本的现场（对照样本 S17、N19、F1、H2、H3）：样本拷进临时目录之后，
# 1. 照这一版拷一份上一版进 .unit-verdict-sample/prior/，逐处改回去：观察者 observe（S17）、test_u 的一个字（N19）、中间那一态的函数 middle_state（F1）、h2_value（H2）；
#    每一处恰好改中一次，改不中就退 1（样本坏了，不往下走）。阶段开跑前拿那一版算键、照判绿记进这一版样本的判定库。
# 2. git init，这一版（含 H2 那一处）连 HEAD 一起提交、refs/sop/staged-green 也挪到 HEAD，再做一次空提交（H3）：键不看 HEAD 与提交，H2 照判、H3 照复用。
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -a Cargo.toml crates "$prior/"
revert() { # <文件> <这一版的串> <上一版的串>：恰好命中一次才改
  python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' "$prior/$1" "$2" "$3"
}
revert crates/singlefs-checker-tier/tests/setting.rs "state + 1" "1 + state"
revert crates/singlefs-checker-tier/tests/setting.rs '"observed"' '"observes"'
revert crates/singlefs-checker-tier/tests/states.rs "value * 2" "value * 3"
revert crates/singlefs-checker-tier/tests/history.rs $'    22\n}' $'    21\n}'
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版（含 H2 那一处）"
git update-ref refs/sop/staged-green HEAD
git -c user.name=sample -c user.email=sample@example.invalid commit -q --allow-empty -m "样本：H3 空提交"
