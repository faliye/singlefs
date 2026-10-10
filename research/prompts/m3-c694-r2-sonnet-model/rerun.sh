#!/usr/bin/env bash
# m3-c694-r2 云端辩方腿（Sonnet）模型的复跑：在冻结树的拷贝上打探针补丁（probe.diff，只在 mount.rs 里加计数与候选开关，默认全关 = 今天的实现）、
# 装三条 harness 档探针（tests/）、各跑一遍，把结局行抽进 out/extracted/，再与 expected-<档>/ 比（compare.sh）。只跑 harness 档（singlefs-harness 的测试目标）。
#   SINGLEFS_REPO   主仓路径（取 research/scripts/run-with-memory-cap.sh 与 capped.sh），必填
#   FROZEN_TREE     冻结树（含 Cargo.toml、Cargo.lock、crates/、litmus/），缺省 ~/.local/share/singlefs/work/m3-c694-r2/tree
#   WORK            工作目录（第一个参数也行），缺省 ./work；自己的拷贝在 $WORK/repo、自己的编译目录在 $WORK/target
#   PROFILE         quick（缺省，前台三四分钟：384 与 256 槽各 3 段历史 × 12 步、小盘推抬 F 那一串、被抛弃根三种造法）
#                   full（quick 全部 + 384 / 256 槽各 24 段 × 24 步 + 4 GiB 宽 6 段 × 12 步；六片并行，每片一个进程，推的 15 分钟以上，交主 agent 跑）
#   THREADS         线程上限，缺省 6（full 的分片数也取它，不超过 6）
# 编译经 6G、跑经 4G 内存包装；cargo 同一时刻只起一条（full 的分片直接跑编出来的二进制，每个也经 4G 包装）。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO（主仓路径）}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FROZEN_TREE="${FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-c694-r2/tree}"
WORK="${1:-${WORK:-$PWD/work}}"
PROFILE="${PROFILE:-quick}"
THREADS="${THREADS:-6}"
MEMCAP="$SINGLEFS_REPO/research/scripts/run-with-memory-cap.sh"
CAPPED="$SINGLEFS_REPO/research/scripts/capped.sh"
mkdir -p "$WORK/out"
if [ ! -d "$WORK/repo" ]; then
    mkdir -p "$WORK/repo"
    cp -r "$FROZEN_TREE/Cargo.toml" "$FROZEN_TREE/Cargo.lock" "$FROZEN_TREE/crates" "$FROZEN_TREE/litmus" "$WORK/repo/"
    (cd "$WORK/repo" && patch -p1 --quiet < "$HERE/probe.diff")
    cp "$HERE"/tests/*.rs "$WORK/repo/crates/singlefs-harness/tests/"
fi
export CARGO_TARGET_DIR="$WORK/target"
cd "$WORK/repo"
nice -n 19 bash "$MEMCAP" 6G bash "$CAPPED" "$THREADS" cargo test --offline --release -p singlefs-harness \
    --test m3c694_k4_reach --test m3c694_k4_floor_raise --test m3c694_k4_unwitnessed_abandon --no-run > "$WORK/out/build.log" 2>&1
run() { # run <输出名> <测试目标> [环境变量…]：经 cargo test 起，单进程
    local name="$1" target="$2"; shift 2
    env "$@" nice -n 19 bash "$MEMCAP" 4G bash "$CAPPED" "$THREADS" cargo test --offline --release -p singlefs-harness \
        --test "$target" -- --ignored --nocapture --test-threads=1 > "$WORK/out/$name.raw" 2>&1
}
run reach-384-quick m3c694_k4_reach K4_WIDTH=384 K4_ARMS=none,c K4_SEEDS=3 K4_OPS=12
run reach-256-quick m3c694_k4_reach K4_WIDTH=256 K4_ARMS=none,c K4_SEEDS=3 K4_OPS=12
run floor-raise m3c694_k4_floor_raise K4_ARMS=none,c,w1,w2
run unwitnessed m3c694_k4_unwitnessed_abandon K4_ARMS=none,c,w1,w2
if [ "$PROFILE" = full ]; then
    BIN=$(ls -t "$WORK"/target/release/deps/m3c694_k4_reach-* | grep -v '\.d$' | head -1)
    shard() { # shard <批名> <宽> <每片几段> <片数> <步数> [环境变量…]
        local batch="$1" width="$2" per="$3" shards="$4" ops="$5"; shift 5
        rm -f "$WORK/out/$batch"-*.raw "$WORK/out/$batch"-*.exit
        for ((i=0;i<shards;i++)); do
            { env "$@" K4_WIDTH="$width" K4_ARMS=none,c K4_SEEDS="$per" K4_FIRST=$((i*per)) K4_OPS="$ops" \
                nice -n 19 bash "$MEMCAP" 4G "$BIN" --ignored --nocapture --test-threads=1 k4_random_histories_reach \
                > "$WORK/out/$batch-$i.raw" 2>&1; echo "$?" > "$WORK/out/$batch-$i.exit"; } &
        done
        wait
        [ "$(ls "$WORK/out/$batch"-*.exit | wc -l)" -eq "$shards" ] || { echo "$batch：退出码文件数与片数对不上，整批作废"; exit 1; }
        cat "$WORK/out/$batch"-*.exit | sort | uniq -c
    }
    S=$(( THREADS > 6 ? 6 : THREADS ))
    shard reach-384-full 384 $((24 / S)) "$S" 24 K4_X=1
    shard reach-256-full 256 $((24 / S)) "$S" 24 K4_X=1
    shard reach-4g-sample 4g 1 "$S" 12 K4_CHECK=0
fi
bash "$HERE/compare.sh" extract "$WORK/out" "$WORK/out/extracted"
echo "跑完。抽出的结局行在 $WORK/out/extracted/；与期望比：bash $HERE/compare.sh compare $WORK/out/extracted $HERE/expected-$PROFILE"
