#!/usr/bin/env bash
# code-tooling 的 research-script-selftests 格按条复用样本的现场：git init（判定库放 git common-dir），照这一版拷一份上一版进 .unit-verdict-sample/prior/，
# 只改上一版 c 那一份自证打的那一句（恰好改中一次，改不中退 1）：这一版必须只现判 c、a 与 d 复用。
set -euo pipefail
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -a scripts "$prior/"
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' "$prior/scripts/gate/sample-pass-c.py" "c 的自证 1 条都判对" "c 的自证判对了"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
