#!/bin/bash
# 用法：verus-run.sh <crate 目录> <输出前缀> [附加 verus 参数…]  —— 直接起 verus（不经门禁脚本），参数同 scripts/gate/verus-proof-check.py 的 verus_arguments
CRATE=$1; OUT=$2; shift 2
V=${VERUS:-$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus}
cd "$CRATE" || exit 2
"$V" --no-cheating --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --num-threads "${THREADS:-31}" --output-json --rlimit 10 "$@" src/lib.rs > "$OUT.json" 2> "$OUT.err"
echo "verus rc=$?"
python3 -I - "$OUT.json" <<'PY'
import json, sys
text = open(sys.argv[1], encoding="utf-8").read().strip()
if not text.startswith("{"):
    print("no json:", text[:200]); sys.exit(0)
j = json.loads(text)
v = j.get("verification-results", {})
print("verified=%s errors=%s encountered-error=%s" % (v.get("verified"), v.get("errors"), v.get("encountered-vir-error", v.get("encountered-error"))))
PY
