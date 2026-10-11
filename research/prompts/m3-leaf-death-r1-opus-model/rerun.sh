#!/usr/bin/env bash
# m3-leaf-death-r1 攻方腿（Opus）模型的复跑。仓路径从 SINGLEFS_REPO 取（K3 那两阶段拷它的 crates/ 改副本；给冻结树或仓根都行）。
# 每一阶段整条经内存包装与线程上限起（编译那两阶段 6G、其余 4G），例：
#   M=research/prompts/m3-leaf-death-r1-opus-model; S=scripts
#   bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 7 bash $M/rerun.sh <空的输出目录> build
#   bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 7 bash $M/rerun.sh <输出目录> e177
# 阶段（按这个次序各跑一次就是全量）：build e177 merges-0 … merges-7 rename-0 rename-1 rc indistinguishable rollback
#   k3build k3run summarize。不给阶段就按次序全跑（整条约 25 分钟，7 个线程）。
# 产物：<输出目录>/*.out（模型行）、k3.out（副本上的 harness 行）、summary/（compare.sh 比的那几份）。
set -euo pipefail
MODEL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
OUT=${1:?用法：rerun.sh <输出目录> [阶段…]}
shift
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
THREADS=${E177_THREADS:-7}
BIN="$OUT/target/release/m3-leaf-death-r1-opus-model"
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
        e177) run_model leaf-death-e177 e177.out ;;
        merges-[0-7])
            local k=${1#merges-}
            run_model leaf-death-merges "merges-$k.out" "$((k * 80)):$((k * 80 + 80))" ;;
        rename-[01])
            local k=${1#rename-}
            run_model leaf-death-rename "rename-$k.out" "$((k * 48)):$((k * 48 + 48))" ;;
        rc) run_model rename-collision rc.out ;;
        indistinguishable) run_model indistinguishable indistinguishable.out ;;
        rollback) run_model rollback-demo rollback.out ;;
        k3build)
            local repo=${SINGLEFS_REPO:?K3 阶段要 SINGLEFS_REPO}
            rm -rf "$OUT/repo"
            rsync -a --exclude target --exclude .git "$repo/" "$OUT/repo/"
            python3 "$MODEL_DIR/k3-copy/patch_checker.py" "$OUT/repo/crates/singlefs-checker/src/walk.rs"
            cp "$MODEL_DIR/k3-copy/opus_k3_borrowed_leaf_images.rs" "$OUT/repo/crates/singlefs-harness/tests/"
            for target in opus_k3_borrowed_leaf_images checker_known_bad_images pool_checker_judges_the_deletion_state_of_every_walked_version; do
                (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test "$target" --no-run)
            done ;;
        k3run)
            : > "$OUT/k3.out"
            for n1 in 0 1; do
                if [[ $n1 == 1 ]]; then export OPUS_K3_I94_N1=1; else unset OPUS_K3_I94_N1; fi
                (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test opus_k3_borrowed_leaf_images -- --nocapture) \
                    | grep '^OPUSK3' >> "$OUT/k3.out"
                for t in the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target \
                         a_leaf_holding_the_same_inode_number_twice_reddens_only_the_container_number_invariant; do
                    (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test checker_known_bad_images -- --exact "$t") \
                        | grep '^test ' | sed "s/^/KNOWNBAD n1_rule=$n1 /" >> "$OUT/k3.out"
                done
            done
            unset OPUS_K3_I94_N1
            for t in two_tombstones_of_one_object_on_the_lineage_redden_only_the_tombstone_record_invariant \
                     a_tombstone_that_does_not_cover_the_whole_object_reddens_only_the_tombstone_record_invariant; do
                (cd "$OUT/repo" && CARGO_TARGET_DIR="$OUT/repo-target" nice -n 19 cargo test --offline -p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version -- --exact "$t") \
                    | grep '^test ' | sed 's/^/I920 /' >> "$OUT/k3.out"
            done
            cat "$OUT/k3.out" ;;
        summarize)
            mkdir -p "$OUT/summary"
            awk -f "$MODEL_DIR/tools/agg.awk" "$OUT/e177.out" | sort > "$OUT/summary/agg-e177.txt"
            cat "$OUT"/merges-*.out | awk -f "$MODEL_DIR/tools/agg.awk" | sort > "$OUT/summary/agg-merges.txt"
            cat "$OUT"/rename-*.out | awk -f "$MODEL_DIR/tools/agg.awk" | sort > "$OUT/summary/agg-rename.txt"
            awk -f "$MODEL_DIR/tools/counts.awk" "$OUT/e177.out" | sort > "$OUT/summary/counts-e177.txt"
            cat "$OUT"/merges-*.out | awk -f "$MODEL_DIR/tools/counts.awk" | sort > "$OUT/summary/counts-merges.txt"
            cat "$OUT"/rename-*.out | awk -f "$MODEL_DIR/tools/counts.awk" | sort > "$OUT/summary/counts-rename.txt"
            grep -h 'name=leaf_cell\|name=leaf_counts' "$OUT/rc.out" | sort > "$OUT/summary/rc.txt"
            grep -h 'name=indistinguishable' "$OUT/indistinguishable.out" | sort > "$OUT/summary/indistinguishable.txt"
            grep -h 'name=rollback_demo' "$OUT/rollback.out" | sort > "$OUT/summary/rollback.txt"
            sort "$OUT/k3.out" > "$OUT/summary/k3.txt"
            (cd "$OUT" && for f in e177.out merges-*.out rename-*.out rc.out indistinguishable.out rollback.out; do
                printf '%s  %s\n' "$(grep -v 'name=job_range\|name=done\|name=config' "$f" | sed 's/^E7RESULT //' | sort | sha256sum | cut -d' ' -f1)" "$f"
            done) > "$OUT/summary/sorted-output-sha256.txt"
            ls -la "$OUT/summary" ;;
        *) echo "不认的阶段：$1" >&2; exit 2 ;;
    esac
}
if [[ $# -eq 0 ]]; then
    set -- build e177 merges-0 merges-1 merges-2 merges-3 merges-4 merges-5 merges-6 merges-7 rename-0 rename-1 rc indistinguishable rollback k3build k3run summarize
fi
for name in "$@"; do
    echo "== 阶段 $name"
    stage "$name"
done
