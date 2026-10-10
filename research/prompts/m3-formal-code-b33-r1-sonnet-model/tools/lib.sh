#!/usr/bin/env bash
# 共用：环境与 cargo 跑法。用法：source lib.sh；需要 REPO（主工作区，取脚本）、WORK（草稿目录：tree/ 是冻结副本的拷贝，target/ 是自己的编译目录，log/ 放日志）。
set -u
: "${REPO:?REPO 是仓根}" "${WORK:?WORK 是草稿目录}"
MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export TREE="$WORK/tree"
export CARGO_TARGET_DIR="$WORK/target"   # 每份副本用自己的编译目录，不与别的副本共用
mkdir -p "$WORK/log"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-三方腿复跑 m3-formal-code-b33-r1 在草稿副本里跑 harness 档，别的会话也有 cargo 在跑}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-按派发给的上限，副本里的命令峰值表里没有或偏小}"
CAP="${CAP:-10}"          # 线程上限
MEM="${MEM:-8G}"          # 内存上限

# cg <日志名> <cargo 参数…>：在 TREE 里经内存包装与线程上限跑 cargo，日志写 $WORK/log/<日志名>.log，打汇总行与 Compiling 的包名
cg() {
    local name="$1"; shift
    local log="$WORK/log/$name.log"
    ( cd "$TREE" && nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" "$MEM" bash "$REPO/research/scripts/capped.sh" "$CAP" cargo "$@" ) > "$log" 2>&1
    local rc=$?
    echo "== $name exit=$rc"
    grep -E '^test result|^error(\[|:)|panicked at' "$log" | head -"${HEAD:-8}"
    echo "   compiled: $(grep -E 'Compiling singlefs' "$log" | sed -E 's/ \(.*//; s/^ +//' | sort | uniq -c | tr '\n' ';')"
    return $rc
}

# mutrun <日志名> <行号|规格文件> <cargo 参数…>：施加一行变异（表里的行号，或 variants/*.spec 规格文件），跑 cargo，撤回，核撤回之后文件与冻结副本逐字节同
mutrun() {
    local name="$1" what="$2"; shift 2
    if [[ "$what" =~ ^[0-9]+$ ]]; then python3 "$MODEL_DIR/tools/mut.py" apply "$TREE" "$what" || return 3
    else python3 "$MODEL_DIR/tools/mut.py" apply-spec "$TREE" "$what" || return 3; fi
    local rc=0
    cg "$name" "$@" || rc=$?
    python3 "$MODEL_DIR/tools/mut.py" restore "$TREE"
    ( cd "$TREE" && find crates -type f -name '*.rs' -o -type f -name '*.wgsl' -o -name mutations.tsv -path 'crates/*' | sort | xargs sha256sum ) > "$WORK/log/$name.after-restore.sha" 2>/dev/null
    echo "   cargo exit=$rc"
    return 0
}
