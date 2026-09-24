#!/usr/bin/env bash
# T10 辩方复核：复现 r1 判决报的「72 个顶格 pub const 里 15 个读不出整数」这个数。
# 用法：bash t10-none-count.sh（从仓库根目录跑）
set -euo pipefail
python3 - <<'PYEOF'
import sys, importlib.util
spec = importlib.util.spec_from_file_location("fc", ".claude/gate.d/lib-format-const.py")
fc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fc)
text = open("crates/singlefs-format/src/lib.rs", encoding="utf-8").read()
decls = fc.read_rust_consts(text, value_reading="integer_literal", only_top_level_public=True)
none_count = sum(1 for d in decls if d.value is None)
print("total:", len(decls), "none:", none_count)
PYEOF
