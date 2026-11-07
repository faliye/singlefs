#!/usr/bin/env bash
# 复用三问里改一组的输入：变异表 second 那一行的替换文换一个（恰好改中一次，改不中退 1）；只 crates/tiny/src/second.rs 那一组的行原文变了
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' crates/mutations.tsv "{ value * 3 }" "{ value * 4 }"
