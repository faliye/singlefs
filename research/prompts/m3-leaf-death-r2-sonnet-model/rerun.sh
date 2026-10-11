#!/usr/bin/env bash
# m3-leaf-death-r2 辩方腿（Sonnet）模型的复跑。模型 = 第一轮攻方腿（Opus）的 E177 装置拷贝（research/prompts/m3-leaf-death-r1-opus-model/）加第二轮辩方的改动：
#   - 候选二的零条目版本也配合并（M2 / M3 均分 / M3 刚好够），容器留在叶序里、合并只在有记录的片之间做；
#   - 新读法 C2Hold、C2HoldZ（候选二最强形态）、T、TS、TSZ（候选三 / 四最强形态：今天意图那次已写的整对象墓碑 + 供出规则 + 数据单元门）、FSZ（墓碑的写挪到最后一批）；
#   - r2-sweep（均分借换前提：有序删的历史）、r2-rollback（回退示例的另两个候选）、r2-indistinguishable（判别子两段历史，写路径带墓碑）。
# 不依赖 crates/ 与 scripts/experiments/Cargo.toml；仓路径只用来找 scripts/hosts/run-with-memory-cap.sh（环境变量 SINGLEFS_REPO，给仓根）。
# 每一阶段整条经内存包装起（build 6G、其余 4G），例：
#   M=research/prompts/m3-leaf-death-r2-sonnet-model; S=scripts; export SINGLEFS_REPO=$PWD
#   bash $S/run-with-memory-cap.sh 6G bash $M/rerun.sh <空的输出目录> build
#   bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh <输出目录> e177
#   for k in 0 1 2 3 4 5 6 7; do bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh <输出目录> merges-$k; done
#   bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh <输出目录> sweep rollback indist summarize
#   bash $M/compare.sh <输出目录>
# 阶段：build e177 merges-0 … merges-7 sweep rollback indist summarize。不给阶段就按次序全跑（挂钟约半小时，8 个线程；机器忙时翻倍）。
# 产物：<输出目录>/*.out（模型行）、summary/（compare.sh 比的那几份）。
set -euo pipefail
MODEL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
OUT=${1:?用法：rerun.sh <输出目录> [阶段…]}
shift
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
THREADS=${E177_THREADS:-8}
BIN="$OUT/target/release/m3-leaf-death-r2-sonnet-model"
run_model() { # 模式 输出文件 [LEAF_JOB_RANGE]
    local mode=$1 file=$2 range=${3:-}
    if [[ -n "$range" ]]; then
        LEAF_JOB_RANGE=$range E177_THREADS=$THREADS nice -n 19 "$BIN" "$mode" > "$OUT/$file"
    else
        E177_THREADS=$THREADS nice -n 19 "$BIN" "$mode" > "$OUT/$file"
    fi
    tail -n 1 "$OUT/$file"
}
stage() {
    case "$1" in
        build)
            CARGO_TARGET_DIR="$OUT/target" nice -n 19 cargo build --release --offline --manifest-path "$MODEL_DIR/Cargo.toml" ;;
        e177) run_model r2-e177 e177.out ;;
        merges-[0-7])
            local k=${1#merges-}
            run_model r2-merges "merges-$k.out" "$((k * 160)):$((k * 160 + 160))" ;;
        sweep) run_model r2-sweep sweep.out ;;
        rollback) run_model r2-rollback rollback.out ;;
        indist) run_model r2-indistinguishable indist.out ;;
        summarize)
            mkdir -p "$OUT/summary"
            awk -f "$MODEL_DIR/tools/agg.awk" "$OUT/e177.out" | sort > "$OUT/summary/agg-e177.txt"
            cat "$OUT"/merges-*.out | awk -f "$MODEL_DIR/tools/agg.awk" | sort > "$OUT/summary/agg-merges.txt"
            python3 "$MODEL_DIR/tools/counts2.py" "$OUT/e177.out" > "$OUT/summary/counts-e177.txt"
            python3 "$MODEL_DIR/tools/counts2.py" "$OUT"/merges-*.out > "$OUT/summary/counts-merges.txt"
            python3 "$MODEL_DIR/tools/window.py" "$OUT/e177.out" "$OUT"/merges-*.out > "$OUT/summary/window.txt"
            python3 "$MODEL_DIR/tools/sweep.py" "$OUT/sweep.out" > "$OUT/summary/sweep.txt"
            grep -h 'name=r2_rollback_demo' "$OUT/rollback.out" | sort > "$OUT/summary/rollback.txt"
            grep -h 'name=r2_indistinguishable' "$OUT/indist.out" | sort > "$OUT/summary/indistinguishable.txt"
            (cd "$OUT" && for f in e177.out merges-*.out sweep.out rollback.out indist.out; do
                printf '%s  %s\n' "$(grep -v 'name=job_range\|name=done\|name=config' "$f" | sed 's/^E7RESULT //' | sort | sha256sum | cut -d' ' -f1)" "$f"
            done) > "$OUT/summary/sorted-output-sha256.txt"
            ls -la "$OUT/summary" ;;
        *) echo "不认的阶段：$1" >&2; exit 2 ;;
    esac
}
if [[ $# -eq 0 ]]; then
    set -- build e177 merges-0 merges-1 merges-2 merges-3 merges-4 merges-5 merges-6 merges-7 sweep rollback indist summarize
fi
for name in "$@"; do
    echo "== 阶段 $name"
    stage "$name"
done
