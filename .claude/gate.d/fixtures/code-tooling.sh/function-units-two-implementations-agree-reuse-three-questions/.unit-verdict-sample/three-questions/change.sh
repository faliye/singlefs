#!/usr/bin/env bash
# 复用三问里改登记入口那一条的输入：stage-inputs.tsv 去掉一个登记入口 heaviest（恰好改中一次，改不中退 1）；dump 那一条的输入不动
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' .claude/gate.d/stage-inputs.tsv " crates/fsample/src/lib.rs::heaviest" ""
