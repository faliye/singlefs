#!/usr/bin/env bash
# 用法：run_mut.sh <变异表行号> <包> <测试目标> <过滤> [额外 libtest 参数…]
# 把冻结副本变异表那一行施加到自己的拷贝，跑一遍，还原；日志在 logs/row-<行号>.log
set -uo pipefail
W="$HOME/.local/share/singlefs/work/m3-batch58-code-r1-sonnet"
row="$1"; package="$2"; target="$3"; filter="$4"; shift 4
mkdir -p "$W/logs"
log="$W/logs/row-$row.log"
python3 "$W/mutate_row.py" apply "$row" > "$log.apply" 2>&1 || { cat "$log.apply"; exit 3; }
export CARGO_TARGET_DIR="$W/target"
cargo test --offline -p "$package" --test "$target" --manifest-path "$W/repo/Cargo.toml" -- "$filter" "$@" > "$log" 2>&1
code=$?
python3 "$W/mutate_row.py" restore "$row" >> "$log.apply" 2>&1
echo "row=$row cargo_exit=$code"
grep -E '^test .*(FAILED|ok)$|test result' "$log" | sed 's/^/  /'
