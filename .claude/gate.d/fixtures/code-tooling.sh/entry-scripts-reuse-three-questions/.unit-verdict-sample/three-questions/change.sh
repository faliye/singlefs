#!/usr/bin/env bash
# 复用三问里改一份的输入：sample-beta.sh 打那一行之前加一行代码（恰好改中一次，改不中退 1），字段照旧带齐；只 beta 那一行的源码变了
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' scripts/gate/sample-beta.sh 'echo "SAMPLE_BETA' $'beta_item="${1:-all}"\necho "SAMPLE_BETA'
