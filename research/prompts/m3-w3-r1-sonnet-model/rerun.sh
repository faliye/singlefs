#!/usr/bin/env bash
# admission: always 每次调都现判（拷冻结树、施补丁、编、跑、比），上一次的结论不替这一次作保
# run-condition: none 只要 cargo 和冻结树在
# W3 正推腿的模型：在 E175 装置（research/results/e175-.../device-source.rs）上加一段「候选历史」的另算，
# 在冻结树的两份拷贝上跑（V1 臂 = 施加 patch-V1.diff，P 臂 = 原样），比每个候选对 E175 每个发号事件的判红。
#
# 用法（每一步可单独调，前一步没做就拒；跑的步骤可重复调到打出 DONE）：
#   SINGLEFS_REPO=<冻结树根，含 Cargo.toml Cargo.lock .cargo crates> bash rerun.sh prepare
#   bash rerun.sh build
#   bash rerun.sh run <步名>      步名：d、g-long-v1、f2-v1、f1-broad-v1、f1-reuse-v1、f1-deleting-v1、f1-newest-v1、f3a-v1、g-long-p、f3a-p
#   bash rerun.sh compare         （不读 SINGLEFS_REPO）
# 环境变量：W3_WORK（工作目录，默认 ~/.local/share/singlefs/work/m3-w3-r1-sonnet-rerun；prepare 要它不存在或为空）、
#           W3_BUDGET（一趟领新历史的时限，秒，默认 200）、W3_THREADS（默认 6）。
# 内存：编译 6G、跑 4G，都经 scripts/run-with-memory-cap.sh；同一时刻只起一条 cargo；用自己的编译目录。
set -euo pipefail
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_OF_THIS_SCRIPT=$(cd "$HERE/../../.." && pwd)
E175=$REPO_OF_THIS_SCRIPT/research/results/e175-product-path-inode-watermark-regression-2026-10-08
WORK=${W3_WORK:-$HOME/.local/share/singlefs/work/m3-w3-r1-sonnet-rerun}
THREADS=${W3_THREADS:-6}
BUDGET=${W3_BUDGET:-200}
CODE_BASELINE=w3-sonnet-model
CAPPED=$REPO_OF_THIS_SCRIPT/scripts/capped.sh
CAP=$REPO_OF_THIS_SCRIPT/scripts/run-with-memory-cap.sh
TEST=e175_product_path_inode_watermark_regression

step=${1:-}
case "$step" in
prepare)
    : "${SINGLEFS_REPO:?SINGLEFS_REPO 没设：冻结树根（含 Cargo.toml Cargo.lock .cargo crates）}"
    if [ -e "$WORK" ] && [ -n "$(ls -A "$WORK" 2>/dev/null)" ]; then echo "工作目录 $WORK 不空，拒" >&2; exit 2; fi
    mkdir -p "$WORK/out"
    for arm in V1 P; do
        mkdir -p "$WORK/repo-$arm"
        cp -r "$SINGLEFS_REPO/Cargo.toml" "$SINGLEFS_REPO/Cargo.lock" "$SINGLEFS_REPO/.cargo" "$SINGLEFS_REPO/crates" "$WORK/repo-$arm/"
    done
    # 冻结树里这五份与 E175 冻结副本逐字节相同（baseline-frozen-crates.sha 里同名那行）
    for f in crates/singlefs-core/src/mount.rs crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/recovery.rs crates/singlefs-core/src/instance_table.rs crates/singlefs-checker/src/walk.rs; do
        want=$(grep " $f\$" "$E175/baseline-frozen-crates.sha" | cut -d' ' -f1)
        have=$(sha256sum "$WORK/repo-P/$f" | cut -d' ' -f1)
        [ "$want" = "$have" ] || { echo "$f 与 E175 冻结副本不同" >&2; exit 3; }
    done
    patch "$WORK/repo-V1/crates/singlefs-core/src/mount.rs" < "$E175/patch-V1.diff" > /dev/null
    cp "$E175/device-source.rs" "$WORK/device-source-w3.rs"
    patch "$WORK/device-source-w3.rs" < "$HERE/w3-device.diff" > /dev/null
    for arm in V1 P; do cp "$WORK/device-source-w3.rs" "$WORK/repo-$arm/crates/singlefs-harness/tests/$TEST.rs"; done
    echo "prepare 做完：$WORK"
    ;;
build)
    for arm in V1 P; do
        ( cd "$WORK/repo-$arm" && CARGO_TARGET_DIR="$WORK/target-$arm" nice -n 19 bash "$CAPPED" "$THREADS" bash "$CAP" 6G cargo test --release --offline -p singlefs-harness --test "$TEST" --no-run ) > "$WORK/build-$arm.log" 2>&1
        grep "Executable tests/$TEST.rs" "$WORK/build-$arm.log" | sed -e 's/.*(\(.*\))/\1/' > "$WORK/bin-$arm.path"
        echo "build $arm: $(cat "$WORK/bin-$arm.path")"
    done
    ;;
run)
    name=${2:?步名}
    case "$name" in
        d)               list="V1:D:d-V1.out: P:D:d-P.out:" ;;
        g-long-v1)       list="V1:G-long:g-long-V1.out:" ;;
        f2-v1)           list="V1:F2:f2-V1.out:" ;;
        f1-broad-v1)     list="V1:F1-broad:f1-broad-V1.out:" ;;
        f1-reuse-v1)     list="V1:F1-reuse-after-raising-the-floor:f1-reuse-V1.out:" ;;
        f1-deleting-v1)  list="V1:F1-deleting-files-of-several-lower-leaves:f1-deleting-V1.out:" ;;
        f1-newest-v1)    list="V1:F1-newest-system-configuration-slot-reading-zeros:f1-newest-V1.out:" ;;
        f3a-v1)          list="V1:F3a:f3a-V1.out:" ;;
        g-long-p)        list="P:G-long:g-long-P.out:" ;;
        f3a-p)           list="P:F3a:f3a-P.out:1500" ;;
        *) echo "不认的步名 $name" >&2; exit 2 ;;
    esac
    for item in $list; do
        IFS=: read -r arm family file limit <<< "$item"
        bin=$(cat "$WORK/bin-$arm.path")
        limit_env=()
        if [ -n "$limit" ]; then limit_env=(E175_INDEX_LIMIT="$limit"); fi
        env E175_ARM="$arm" E175_FAMILY="$family" E175_RESULT_FILE="$WORK/out/$file" E175_CODE_BASELINE="$CODE_BASELINE" \
            E175_WORKER_THREADS="$THREADS" E175_TIME_BUDGET_SECONDS="$BUDGET" "${limit_env[@]}" \
            nice -n 19 bash "$CAPPED" "$THREADS" bash "$CAP" 4G "$bin" --ignored --exact run_one_family_on_one_arm_into_the_result_file \
            > "$WORK/out/$file.log" 2>&1 || { echo "$name $file 退出码 $?（250–254 是内存包装自己的结局，输出不算数）" >&2; tail -3 "$WORK/out/$file.log" >&2; exit 4; }
        remaining=$(grep '^E7TIMING' "$WORK/out/$file" | tail -1 | sed -e 's/.*remaining=\([0-9]*\).*/\1/')
        if [ "$remaining" = "0" ]; then echo "$file DONE"; else echo "$file remaining=$remaining（再调一次同一条命令）"; fi
    done
    ;;
compare)
    python3 -I "$HERE/compare.py" "${W3_OUT:-$WORK/out}" "$HERE/expected"
    ;;
*)
    sed -n 4,12p "${BASH_SOURCE[0]}" >&2
    exit 2
    ;;
esac
