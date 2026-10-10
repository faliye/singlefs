#!/usr/bin/env bash
# m3-c694-r1 云端正推腿的模型复跑：在冻结树的拷贝上打候选开关补丁（candidates.diff）、装探针用例（tests/）、只跑 harness 档，
# 把结局行抽进 out/，再与 expected/ 比（compare.sh）。
#   SINGLEFS_REPO   主仓路径（取 research/scripts/run-with-memory-cap.sh 与 capped.sh），必填
#   FROZEN_TREE     冻结树（含 Cargo.toml、crates/、litmus/），缺省 ~/.local/share/singlefs/work/m3-c694-r1/tree
#   WORK            工作目录，缺省 ./work（自己的拷贝、自己的编译目录 $WORK/target）
#   C694_KS         探针的「第一个文件之后覆盖写次数」，缺省 5（11 个拒全在 5 上）；FULL=1 取 1,2,3,5,8（60 格）
#   THREADS         线程上限，缺省 4
# 编译经 6G、跑经 4G 内存包装；同一时刻只起一条 cargo。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO（主仓路径）}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FROZEN_TREE="${FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-c694-r1/tree}"
WORK="${WORK:-$PWD/work}"
THREADS="${THREADS:-4}"
KS="${C694_KS:-5}"
[ "${FULL:-0}" = "1" ] && KS="1,2,3,5,8"
MEMCAP="$SINGLEFS_REPO/research/scripts/run-with-memory-cap.sh"
CAPPED="$SINGLEFS_REPO/research/scripts/capped.sh"
mkdir -p "$WORK/out"
if [ ! -d "$WORK/repo" ]; then
    mkdir -p "$WORK/repo"
    cp -r "$FROZEN_TREE/Cargo.toml" "$FROZEN_TREE/Cargo.lock" "$FROZEN_TREE/crates" "$FROZEN_TREE/litmus" "$WORK/repo/"
    (cd "$WORK/repo" && patch -p1 --quiet < "$HERE/candidates.diff")
    cp "$HERE"/tests/*.rs "$WORK/repo/crates/singlefs-harness/tests/"
fi
export CARGO_TARGET_DIR="$WORK/target"
cd "$WORK/repo"
nice -n 19 bash "$MEMCAP" 6G bash "$CAPPED" "$THREADS" cargo test --offline -p singlefs-harness \
    --test m3c694_switches --test m3c695_switches --no-run > "$WORK/out/build.log" 2>&1
run() { # run <输出名> <测试目标> <过滤> [环境变量…]
    local name="$1" target="$2" filter="$3"; shift 3
    env "$@" nice -n 19 bash "$MEMCAP" 4G bash "$CAPPED" "$THREADS" cargo test --offline -p singlefs-harness \
        --test "$target" -- --nocapture --test-threads=1 "$filter" > "$WORK/out/$name.raw" 2>&1
}
for switches in none w1 w4; do
    s="$switches"; [ "$s" = none ] && s=""
    : > "$WORK/out/probe-$switches.raw.all"
    for width in 384 256 240; do
        run "probe-$switches-$width" m3c694_switches m3c694_switched_probe "C694_SWITCHES=$s" "C694_WIDTHS=$width" "C694_KS=$KS"
        cat "$WORK/out/probe-$switches-$width.raw" >> "$WORK/out/probe-$switches.raw.all"
    done
done
run temporary m3c694_switches m3c694_a_temporarily_unreadable_account_under_each_candidate X=1
run journal m3c694_switches m3c694_probe_journal X=1
run c695-none m3c695_switches c695_ C695_SWITCHES=none
run c695-k3c m3c695_switches c695_ C695_SWITCHES=k3c
bash "$HERE/compare.sh" extract "$WORK/out" "$WORK/out/extracted"
echo "跑完。抽出的结局行在 $WORK/out/extracted/；与 expected/ 比：bash $HERE/compare.sh compare $WORK/out/extracted $HERE/expected"
