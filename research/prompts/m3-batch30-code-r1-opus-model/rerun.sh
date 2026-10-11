#!/usr/bin/env bash
# m3-batch30-code-r1 攻方腿（opus）复跑脚本。
#
# 这条腿攻 K2 的 M1（F-W）、M2（F-IT）、CPU/GPU 判得不一样。结论是「没打中」：
# 用 harness 档的 check_pool_image，在 F-W 坏镜像上比第 30 批（b30，含 F-W）与
# 第 29 批基线（b29，把 F-W 的 walk.rs 改动 patch -R 退回）两份代码的违例集合，
# 坐实 b30 在 F-W 覆盖的那一形上严格更对、没引入漏判 / 误红。GPU、F-IT 只读码，不跑。
#
# 用法：
#   SINGLEFS_REPO=<冻结副本仓根> [SINGLEFS_TOOLS=<脚本所在仓根>] bash rerun.sh <空工作目录> all
#     跑完往 <空工作目录>/summary.txt 写排序过的汇总（OPUSPROBE 各行 + 新坏镜像用例的 test result）。
#   bash rerun.sh <summary-a> compare <summary-b>
#     比两份 summary（各自先排序再 diff），不依赖 SINGLEFS_REPO；一致打印 SUMMARY_MATCHES，不一致打印 diff 并退 1。
#
# 变量：
#   SINGLEFS_REPO  冻结副本仓根（含 crates/ 与 _m3-batch30-code-r1-diff.md 用到的那几份），必给（all 模式）。
#   SINGLEFS_TOOLS 脚本（run-with-memory-cap.sh、capped.sh）所在仓根；默认 = SINGLEFS_REPO。
#   内存上限：编译 6G、跑 4G（显式写死，不用 auto）。线程：capped.sh 7。同一时刻只起一条 cargo。
set -euo pipefail

MODE="${2:-all}"

if [[ "$MODE" == "compare" ]]; then
    A="$1"; B="$3"
    if diff <(sort "$A") <(sort "$B") > /tmp/.m3b30-opus-cmp.$$ 2>&1; then
        echo "SUMMARY_MATCHES $(wc -l < "$A") 行"
        rm -f /tmp/.m3b30-opus-cmp.$$
    else
        echo "SUMMARY_DIFFERS:"
        cat /tmp/.m3b30-opus-cmp.$$
        rm -f /tmp/.m3b30-opus-cmp.$$
        exit 1
    fi
    exit 0
fi

WORK="$1"
: "${SINGLEFS_REPO:?给冻结副本仓根}"
TOOLS="${SINGLEFS_TOOLS:-$SINGLEFS_REPO}"
HERE="$(cd "$(dirname "$0")" && pwd)"
CAP="$TOOLS/scripts/hosts/run-with-memory-cap.sh"
CAPPED="$TOOLS/scripts/hosts/capped.sh"
HARNESS="crates/singlefs-harness/tests/checker_known_bad_images.rs"
WALK="crates/singlefs-checker/src/walk.rs"

mkdir -p "$WORK"
# b30 与 b29 两份副本（b29 = 退回 F-W 的 walk.rs 改动）。
rsync -a --exclude target --exclude .git "$SINGLEFS_REPO/" "$WORK/b30/"
rsync -a --exclude target --exclude .git "$SINGLEFS_REPO/" "$WORK/b29/"
# F-W 的 walk.rs 改动原样（_m3-batch30-code-r1-diff.md 第 6..150 行那一段 diff），退回得到 b29 基线。
awk 'NR>=6 && NR<=150' "$HERE/walk-fw.patch.src" > "$WORK/walk-fw.patch" 2>/dev/null || cp "$HERE/walk-fw.patch" "$WORK/walk-fw.patch"
( cd "$WORK/b29" && patch -R -p1 < "$WORK/walk-fw.patch" )

# 两份都把探针测试追加进 harness 用例。
for V in b30 b29; do
    cat "$HERE/probe-append.rs" >> "$WORK/$V/$HARNESS"
done

: > "$WORK/summary.txt"
for V in b30 b29; do
    ( cd "$WORK/$V" && CARGO_TARGET_DIR="$WORK/target-$V" \
        bash "$CAP" 6G bash "$CAPPED" 7 cargo test -p singlefs-harness --test checker_known_bad_images --no-run ) >/dev/null 2>&1
    ( cd "$WORK/$V" && CARGO_TARGET_DIR="$WORK/target-$V" \
        bash "$CAP" 4G bash "$CAPPED" 7 cargo test -p singlefs-harness --test checker_known_bad_images -- \
        opus_probe_fw_full_verdicts beside_a_copy_verifying_as_a_node child_pointer_birth_tree_is_one_off_over_a_leaf whose_second_copy_verifies_as_a_node --nocapture ) \
        2>&1 | grep -E 'OPUSPROBE|test result' | sed "s/^/$V /" >> "$WORK/summary.txt" || true
done
sort -o "$WORK/summary.txt" "$WORK/summary.txt"
echo "summary.txt:"
cat "$WORK/summary.txt"
