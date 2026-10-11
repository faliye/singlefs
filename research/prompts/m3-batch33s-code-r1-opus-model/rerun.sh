#!/usr/bin/env bash
# 攻方 m3-batch33s-code-r1（Opus）模型复跑：在当前目录（空目录）里从头建仓副本、编译、跑，产出 summary.txt，再与 expected-summary.txt 比。
# 用法：cd <空目录> && SINGLEFS_REPO=<singlefs 仓根> bash <本目录>/rerun.sh
# 环境变量：
#   SINGLEFS_REPO（必给）       singlefs 仓根：取 scripts/hosts/run-with-memory-cap.sh 与 capped.sh。
#   FROZEN_TREE（可选）         被判的冻结副本，默认 ~/.local/share/singlefs/work/m3-batch33s-code-r1/tree（只读，拷一份到 ./repo 再改）。
#   THREADS（可选）             线程上限，默认 7（经 capped.sh 设 CARGO_BUILD_JOBS 等）。
#   RUN_KNOWN_BAD（可选）       默认 1：在打了 ATTACK_LAZY_FOLD 补丁的副本上跑 harness 的 checker_known_bad_images（8G），量改法 F1 不翻已有坏镜像。
#   包装脚本另要的变量：无（run-with-memory-cap.sh 只读 SINGLEFS_HEAVY_TESTS，这里不跑重型测试、不设它）。
# 内存：编译 6G、跑 4G、checker_known_bad_images 8G。串行，同一时刻只起一条 cargo。中途被停（前台时限）可在同一目录再跑一次接着编。
set -euo pipefail
: "${SINGLEFS_REPO:?要给 SINGLEFS_REPO}"
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FROZEN_TREE="${FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-batch33s-code-r1/tree}"
THREADS="${THREADS:-7}"
RUN_KNOWN_BAD="${RUN_KNOWN_BAD:-1}"
WORK="$(pwd)"
CAP="$SINGLEFS_REPO/scripts/hosts/run-with-memory-cap.sh"
CAPPED="$SINGLEFS_REPO/scripts/hosts/capped.sh"
export CARGO_TARGET_DIR="$WORK/target"

if [ ! -f "$WORK/repo/.attack-ready" ]; then
    rm -rf "$WORK/repo"
    rsync -a --exclude target --exclude .git "$FROZEN_TREE/" "$WORK/repo/"
    patch --quiet "$WORK/repo/crates/singlefs-checker/src/walk.rs" < "$MODEL_DIR/walk-lazy-fold.patch"
    cp "$MODEL_DIR/attack_b33s_fold_order.rs" "$WORK/repo/crates/singlefs-harness/tests/attack_b33s_fold_order.rs"
    touch "$WORK/repo/.attack-ready"
fi
cd "$WORK/repo"
nice -n 19 bash "$CAP" 6G bash "$CAPPED" "$THREADS" cargo test --offline -p singlefs-harness --test attack_b33s_fold_order --no-run > "$WORK/build-attack.log" 2>&1
ATTACK_BIN="$(grep -o 'target/debug/deps/attack_b33s_fold_order-[0-9a-f]*' "$WORK/build-attack.log" | tail -n 1)"
ATTACK_BIN="$CARGO_TARGET_DIR/debug/deps/$(basename "$ATTACK_BIN")"
: > "$WORK/run-attack.log"
for mode in snapshot lazy; do
    if [ "$mode" = lazy ]; then
        ATTACK_LAZY_FOLD=1 nice -n 19 bash "$CAP" 4G bash "$CAPPED" "$THREADS" "$ATTACK_BIN" --test-threads 1 --nocapture >> "$WORK/run-attack.log" 2>&1
    else
        env -u ATTACK_LAZY_FOLD nice -n 19 bash "$CAP" 4G bash "$CAPPED" "$THREADS" "$ATTACK_BIN" --test-threads 1 --nocapture >> "$WORK/run-attack.log" 2>&1
    fi
done
python3 -I "$MODEL_DIR/s5_oracle_model.py" > "$WORK/run-models.log"
python3 -I "$MODEL_DIR/s2_guard_model.py" >> "$WORK/run-models.log"
: > "$WORK/run-known-bad.log"
if [ "$RUN_KNOWN_BAD" = 1 ]; then
    nice -n 19 bash "$CAP" 6G bash "$CAPPED" "$THREADS" cargo test --offline -p singlefs-harness --test checker_known_bad_images --no-run > "$WORK/build-known-bad.log" 2>&1
    KNOWN_BAD_BIN="$CARGO_TARGET_DIR/debug/deps/$(basename "$(grep -o 'target/debug/deps/checker_known_bad_images-[0-9a-f]*' "$WORK/build-known-bad.log" | tail -n 1)")"
    ATTACK_LAZY_FOLD=1 nice -n 19 bash "$CAP" 8G bash "$CAPPED" "$THREADS" "$KNOWN_BAD_BIN" > "$WORK/run-known-bad.log" 2>&1
fi
cd "$WORK"
{
    grep -hoE 'ATTACK_(HISTORY|ARM|S2|SCAN) .*' run-attack.log
    grep -hoE 'ATTACK_ORDER(_OF)? .*' run-attack.log | awk '/^ATTACK_ORDER_OF/{if(l)print l; l="ATTACK_WALK_ORDER " $2 " txgs="; next} {sub(/.*txg=/,""); l=l $0 ","} END{if(l)print l}'
    grep -hE '^S[25]_ROW ' run-models.log
    grep -hE '^test result' run-known-bad.log | sed -E 's/; finished in [0-9.]+s//; s/^/KNOWN_BAD_LAZY_FOLD /'
} | sort -u > summary.txt
bash "$MODEL_DIR/compare.sh" summary.txt "$MODEL_DIR/expected-summary.txt"
