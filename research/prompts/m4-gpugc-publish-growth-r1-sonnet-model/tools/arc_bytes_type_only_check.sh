#!/usr/bin/env bash
# 只改类型的编译检查（Q2 甲档）：在副本里把 PublishedUnit.bytes 从 Vec<u8> 换成 std::sync::Arc<[u8]>，别的一个字不改，
# 只编核心层库，列出编译器报的错按文件分的条数。报错条数 = 「只改类型」不够、还要逐处改的地方的下限（核心层库这一包；
# 下游包要核心层改通之后才编得到，这一步量不到）。
# 用法（整条交 task-server 跑，工作目录随意）：
#   task-server/task submit -- bash <这份脚本> --src <仓根或冻结副本> --work <新的草稿目录>
set -euo pipefail
SRC="" WORK=""
while (($#)); do
  case "$1" in
    --src) SRC="$2"; shift 2 ;;
    --work) WORK="$2"; shift 2 ;;
    *) echo "认不出参数：$1" >&2; exit 2 ;;
  esac
done
[[ -d "$SRC/crates/singlefs-core" ]] || { echo "--src 不像仓根或冻结副本" >&2; exit 2; }
[[ -n "$WORK" ]] || { echo "缺 --work" >&2; exit 2; }
if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then echo "--work 已存在且非空" >&2; exit 2; fi
mkdir -p "$WORK/tree/.claude"
rsync -a --exclude target --exclude .git --exclude crates/singlefs-harness --exclude crates/singlefs-checker-tier \
  "$SRC/Cargo.toml" "$SRC/Cargo.lock" "$SRC/.cargo" "$SRC/crates" "$SRC/prune-server" "$WORK/tree/"
cp "$SRC/.claude/heavy-thread-default" "$WORK/tree/.claude/heavy-thread-default"
python3 -I - "$WORK/tree/crates/singlefs-core/src/transaction.rs" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = "    pub identity: TransactionUnit,\n    pub bytes: Vec<u8>,\n}\n\n/// 一次发布写出的一条 journal 记录与它的 4096 字节。"
new = "    pub identity: TransactionUnit,\n    pub bytes: std::sync::Arc<[u8]>,\n}\n\n/// 一次发布写出的一条 journal 记录与它的 4096 字节。"
assert text.count(old) == 1, "PublishedUnit 的定义在副本里恰好命中一次"
open(path, "w", encoding="utf-8").write(text.replace(old, new))
print("name=type_change_applied file=crates/singlefs-core/src/transaction.rs")
PY
cd "$WORK/tree"
status=0
cargo check -p singlefs-core --lib --message-format=short > "$WORK/check-core-lib.out" 2>&1 || status=$?
echo "name=cargo_check_exit code=$status"
echo "name=error_lines count=$(grep -c ': error' "$WORK/check-core-lib.out" || true)"
grep ': error' "$WORK/check-core-lib.out" | sed -E 's/:[0-9]+:[0-9]+:.*//' | sort | uniq -c | sort -rn || true
exit 0
