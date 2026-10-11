#!/usr/bin/env bash
# 复跑 m3-delete-checker-r1 攻方腿的原型（副本专用，不进 crates/）。
# 用法（工作目录 = 当前目录，空目录里起；整条经内存包装与线程上限）：
#   bash scripts/run-with-memory-cap.sh 5G bash scripts/capped.sh 10 bash <本目录>/rerun.sh <阶段>
# 阶段：prepare（拷源树、打补丁、编）| legal <批大小> | random | mut <批大小> | extra | summary | all（依次全做，约 6 分钟）
# 源树默认冻结副本 /tmp/claude-1000/m3-delete-checker-r1/tree，M3DC_SOURCE 可改；prepare 先按快照的 sha256 核源树。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
work="$PWD"
source_tree="${M3DC_SOURCE:-/tmp/claude-1000/m3-delete-checker-r1/tree}"
export CARGO_TARGET_DIR="$work/target"
mutations="zombie orphan noqueue nlink1 stale tag4 notag4row allocbump qwidth16 loggedops keepq"

run_test() { # $1 = 输出文件，其余环境变量由调用方给
  (cd "$work/tree" && nice -n 19 cargo test --offline --release -p singlefs-harness --test m3dc_candidates -- --nocapture) > "$1" 2>&1
}

stage="${1:-}"
case "$stage" in
  prepare)
    (cd "$source_tree" && sha256sum -c "$repo/research/prompts/m3-delete-checker-r1-snapshot/crates-sha256.txt")
    mkdir -p "$work/out"
    rsync -a --exclude target --exclude .git "$source_tree/" "$work/tree/"
    (cd "$work/tree" && patch -p2 --quiet < "$here/copy.patch")
    cmp "$here/m3dc_candidates.rs" "$work/tree/crates/singlefs-harness/tests/m3dc_candidates.rs"
    (cd "$work/tree" && nice -n 19 cargo test --offline --release -p singlefs-harness --test m3dc_candidates --no-run)
    ;;
  legal)
    M3DC_BATCH="$2" M3DC_PLAN=legal M3DC_CRASH=0 M3DC_THREADS=10 M3DC_LINES=1 run_test "$work/out/legal2-b$2.log"
    grep M3DC_SUMMARY "$work/out/legal2-b$2.log"
    ;;
  random)
    M3DC_PLAN=random M3DC_SEED_BASE=1000 M3DC_SEEDS=600 M3DC_CRASH=0 M3DC_THREADS=10 M3DC_LINES=1 \
      run_test "$work/out/random-1000-600.log"
    grep M3DC_SUMMARY "$work/out/random-1000-600.log"
    ;;
  mut)
    for m in $mutations; do
      M3DC_BATCH="$2" M3DC_MUT="$m" M3DC_PLAN=mut M3DC_CRASH=0 M3DC_THREADS=4 M3DC_LINES=1 \
        run_test "$work/out/mut-$m-b$2.log"
      grep M3DC_SUMMARY "$work/out/mut-$m-b$2.log"
    done
    ;;
  extra)
    # 缺标签 4 那一行的镜像上冷启动与挂载（改坏 notag4row，合法计划第 1..3 段：冷启动、挂载、建 inode 再挂载）
    M3DC_MUT=notag4row M3DC_BATCH=2 M3DC_PLAN=legal M3DC_SKIP=1 M3DC_LIMIT=3 M3DC_CRASH=0 M3DC_THREADS=3 M3DC_LINES=1 \
      run_test "$work/out/extra-notag4row-mount.log"
    grep " ENDING " "$work/out/extra-notag4row-mount.log" | cut -c1-210
    ;;
  summary)
    python3 "$here/summarize.py" "$work/out" > "$work/out/summary.txt"
    diff "$here/expected-summary.txt" "$work/out/summary.txt" && echo "M3DC_RERUN summary 与 expected-summary.txt 逐字相同"
    sha256sum "$work/out/summary.txt"
    ;;
  all)
    bash "$here/rerun.sh" prepare
    for b in 144 1 2 3 5; do bash "$here/rerun.sh" legal "$b"; done
    bash "$here/rerun.sh" random
    for b in 144 1 2 3; do bash "$here/rerun.sh" mut "$b"; done
    bash "$here/rerun.sh" extra
    bash "$here/rerun.sh" summary
    ;;
  *)
    echo "用法：rerun.sh prepare | legal <批> | random | mut <批> | summary | all" >&2
    exit 2
    ;;
esac
