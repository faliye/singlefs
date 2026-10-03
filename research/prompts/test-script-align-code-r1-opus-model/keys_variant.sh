#!/usr/bin/env bash
# 原样与换上变异表第 3 行（复用键不罩写表）各跑一次 keys_probe。参数：<草稿目录> <冻结副本>
set -u
here="$1"; frozen="$2"; export TMPDIR="$here/runs"
restore() { rsync -rlD --checksum --delete --exclude target --exclude .git "$frozen/" "$here/tree/"; }  # 不留原 mtime：换回的文件按现在的时刻落盘，cargo 才会重编
restore
(cd "$here/proto" && cargo run --release --offline --bin keys_probe) > "$here/runs/keys_probe-as-merged.out" 2> "$here/runs/keys_probe-as-merged.err"; echo "KEYS as-merged rc=$?"
python3 "$here/apply_row.py" "$here/tree" "崩溃放量 复用键不罩写表"
(cd "$here/proto" && cargo run --release --offline --bin keys_probe) > "$here/runs/keys_probe-mutation-key-without-write-table.out" 2> "$here/runs/keys_probe-mutation-key-without-write-table.err"; echo "KEYS mutation rc=$?"
restore
(cd "$here/proto" && cargo run --release --offline --bin reuse_key) > "$here/runs/reuse_key-recheck.out" 2> "$here/runs/reuse_key-recheck.err"; echo "RECHECK reuse_key rc=$?"
