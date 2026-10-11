#!/usr/bin/env bash
# 重跑 test-script-align-code-r3 正推腿的全部探针（只读冻结副本，不改它；不跑 checker 档测试二进制）。
#   用法：bash rerun.sh [冻结副本根] [输出目录]
#   冻结副本根默认 /tmp/claude-1000/test-script-align-code-r3/tree；输出目录默认 /tmp/claude-1000/test-script-align-code-r3-sonnet/rerun-out
# 做五件事：
#   1 probe/src/main.rs：数值环境变量各入口对「写错、写 0、空串、前后空白、非 UTF-8」的结局（只调库的公开函数）
#   2 probe/src/bin/closure.rs 对冻结副本：每条登记的崩溃枚举用例的判法闭包与 crash_identity.rs 里「history.rs 只许由测试文件直接引」那条断言的取数
#   3 同上，对一份把 crash_amplification.rs:77 改经 history 的副本：断言会红的用例数（证那一条依赖判法有牙）
#   4 同上，对一份把 record_checker_judges_absence_by_the_persisted_set.rs:1117 改引新模块的副本：闭包少哪几份文件
#   5 silent/gen.sh：从 E161/E165/E168 三份装置原样抽线程数解析函数，量非 UTF-8 / 前后空白 / 0 / 空串
set -euo pipefail
TREE=${1:-/tmp/claude-1000/test-script-align-code-r3/tree}
OUT=${2:-/tmp/claude-1000/test-script-align-code-r3-sonnet/rerun-out}
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../.." && pwd)
mkdir -p "$OUT"
export CARGO=$(command -v cargo)
# 内存包装：探针峰值只有几十 MB，峰值表里有记录之后 8G 会被拒，写明理由再起
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="探针内存峰值只有几十兆，取保守的八吉上限"
capped() { nice -n 19 bash "$ROOT/scripts/run-with-memory-cap.sh" 8G bash "$ROOT/scripts/capped.sh" 8 "$@"; }
export CARGO_TARGET_DIR="$OUT/target"
nice -n 19 bash "$ROOT/scripts/capped.sh" 8 cargo build --offline --manifest-path "$HERE/probe/Cargo.toml" 2>&1 | tail -2
PROBE=$OUT/target/debug/r3-l2-probe
CLOSURE=$OUT/target/debug/closure
capped "$PROBE" > "$OUT/probe-l2.out" 2>&1
# 拷两份副本（只拷 crates/、Cargo.lock、登记表）
copy_tree() { local d=$1; rm -rf "$d"; mkdir -p "$d/.claude/gate.d"; cp -r "$TREE/crates" "$d/crates"; cp "$TREE/Cargo.lock" "$d/"; cp "$TREE/.claude/gate.d/stage-inputs.tsv" "$d/.claude/gate.d/"; }
replace_once() { python3 - "$1" "$2" "$3" <<'PY'
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
assert text.count(old) == 1, f"旧串在 {path} 里不是恰好一次：{text.count(old)}"
open(path, "w", encoding="utf-8").write(text.replace(old, new))
PY
}
copy_tree "$OUT/mut-tree"
replace_once "$OUT/mut-tree/crates/singlefs-checker-tier/src/crash_amplification.rs" \
  '            process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),' \
  '            singlefs_harness::history::process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),'
copy_tree "$OUT/mut2-tree"
replace_once "$OUT/mut2-tree/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs" \
  'singlefs_harness::history::process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE)' \
  'singlefs_harness::numeric_environment_variable::process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE)'
T=record_checker_judges_absence_by_the_persisted_set
R3_LIST=$T capped "$CLOSURE" "$TREE"            > "$OUT/closure-base.out" 2>&1
capped "$CLOSURE" "$OUT/mut-tree"               > "$OUT/closure-mut-crash-amplification-via-history.out" 2>&1
R3_LIST=$T capped "$CLOSURE" "$OUT/mut2-tree"   > "$OUT/closure-mut2-record-checker-direct.out" 2>&1
diff <(grep '^F ' "$OUT/closure-base.out" | sort) <(grep '^F ' "$OUT/closure-mut2-record-checker-direct.out" | sort) > "$OUT/closure-diff-record-checker.txt" || true
bash "$HERE/silent/gen.sh" "$TREE" "$OUT/silent.rs"
rustc --edition 2021 -o "$OUT/silent" "$OUT/silent.rs"
capped "$OUT/silent" > "$OUT/silent.out" 2>&1
rm -rf "$OUT/mut-tree" "$OUT/mut2-tree"
echo "done: $OUT"; ls "$OUT"
