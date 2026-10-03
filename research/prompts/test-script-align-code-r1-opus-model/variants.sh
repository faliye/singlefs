#!/usr/bin/env bash
# 在草稿副本 tree 上逐个换上一种改动（变异表第 3、4 行那两条复用对照的变异；原型改法 A），重编原型、重跑 reuse_key，
# 每种之前先把副本换回冻结副本的样子。参数：<草稿目录> <冻结副本>
set -u
here="$1"; frozen="$2"
export K2_SCRATCH="$here/runs" TMPDIR="$here/runs"
restore() { rsync -rlD --checksum --delete --exclude target --exclude .git "$frozen/" "$here/tree/"; }  # 不留原 mtime：换回的文件按现在的时刻落盘，cargo 才会重编
for variant in mutation-key-without-write-table mutation-block-key-without-judge-version fix-a-versions-in-the-plan-hash; do
  restore
  case "$variant" in
    mutation-key-without-write-table) python3 "$here/apply_row.py" "$here/tree" "崩溃放量 复用键不罩写表" ;;
    mutation-block-key-without-judge-version) python3 "$here/apply_row.py" "$here/tree" "判定存储 判定块的键不接判法版本" ;;
    fix-a-versions-in-the-plan-hash)
      python3 "$here/fixA.py" "$here/tree/crates/singlefs-checker-tier/src/crash_amplification.rs" &&
        mv "$here/tree/crates/singlefs-checker-tier/src/crash_amplification.rs.tmp" "$here/tree/crates/singlefs-checker-tier/src/crash_amplification.rs" ;;
  esac
  (cd "$here/proto" && cargo build --release --offline --bin reuse_key) > "$here/runs/build-$variant.log" 2>&1 || { echo "VARIANT $variant build_failed"; continue; }
  (cd "$here/proto" && cargo run --release --offline --bin reuse_key) > "$here/runs/reuse_key-$variant.out" 2> "$here/runs/reuse_key-$variant.err"
  echo "VARIANT $variant rc=$?"
done
restore
