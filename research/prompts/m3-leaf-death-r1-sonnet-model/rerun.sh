#!/usr/bin/env bash
# m3-leaf-death-r1 正推腿的副本装置：把入库的 E177 装置拷一份，加五层补丁（候选 retire / zero / all_suppressed / shrink_range / retire_shrink，G-紧字面读法 tight_literal，合并政策 absorb / absorb_borrow），
# 跑 16 次 segment-one，压成一张摘要，与 expected/summary.txt 逐行比。
# 副本上的数不进 kb；要引，先在入库装置上重做（.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」）。
#
# 用法（工作目录必须是空的或上一次本脚本建的）：
#   SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录> build      # 拷装置、核原文件 sha256、打补丁、编译（经内存包装，上限 6G）
#   SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录> run        # 跑 16 次（经内存包装，上限 4G），已有完整产物的那一次跳过；一次前台最多约 3 次，可反复调用
#   bash rerun.sh <工作目录> compare                          # 摘要与 expected/summary.txt 逐行比（不依赖 SINGLEFS_REPO）
#   SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录> all        # build、run、compare 依次
# 环境变量：E177_THREADS（默认 8，只影响挂钟）。同一时刻只起一条 cargo；编译目录在 <工作目录>/build 里。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
work="${1:?用法：rerun.sh <工作目录> build|run|compare|all}"
stage="${2:?用法：rerun.sh <工作目录> build|run|compare|all}"
original_sha="9061c07b72a3c74693ddcacb885b0b4a9ebc4e4dc2c7082cfee320b3112848f5"
bin_name="e177-per-tree-resurrection-without-tombstones"
bin_file="e7-index-bench/src/bin/e177_per_tree_resurrection_without_tombstones.rs"
runs=(
  "baseline none" "baseline absorb" "baseline absorb_borrow"
  "retire none" "retire absorb" "retire absorb_borrow"
  "zero none" "zero absorb" "zero absorb_borrow"
  "all_suppressed none" "all_suppressed absorb" "all_suppressed absorb_borrow"
  "shrink_range none"
  "retire_shrink none" "retire_shrink absorb_borrow"
  "tight_literal none"
)

need_repo() { : "${SINGLEFS_REPO:?要设 SINGLEFS_REPO=<仓根>}"; }

do_build() {
  need_repo
  mkdir -p "$work/build" "$work/runs"
  if [[ ! -d "$work/build/e7-index-bench" ]]; then
    cp -r "$SINGLEFS_REPO/research/e7-index-bench" "$work/build/e7-index-bench"
    cp "$SINGLEFS_REPO/research/Cargo.toml" "$SINGLEFS_REPO/research/Cargo.lock" "$work/build/"
  fi
  actual_sha="$(sha256sum "$SINGLEFS_REPO/research/$bin_file" | cut -d' ' -f1)"
  if [[ "$actual_sha" != "$original_sha" ]]; then
    echo "入库的 E177 装置 sha256 是 $actual_sha，不是冻结树那一份 $original_sha：补丁的旧串可能对不上，停" >&2
    exit 3
  fi
  cp "$SINGLEFS_REPO/research/$bin_file" "$work/original.rs"
  python3 "$here/apply_patch.py" "$work/original.rs" "$work/patched1.rs"
  python3 "$here/apply_patch2.py" "$work/patched1.rs" "$work/patched2.rs"
  python3 "$here/apply_patch3.py" "$work/patched2.rs" "$work/patched3.rs"
  python3 "$here/apply_patch4.py" "$work/patched3.rs" "$work/patched4.rs"
  python3 "$here/apply_patch5.py" "$work/patched4.rs" "$work/patched5.rs"
  cp "$work/patched5.rs" "$work/build/$bin_file"
  (cd "$work/build" && bash "$SINGLEFS_REPO/research/scripts/run-with-memory-cap.sh" 6G nice -n 19 cargo build --release --offline --bin "$bin_name")
  echo "build 完成：$work/build/target/release/$bin_name"
}

do_run() {
  need_repo
  local binary="$work/build/target/release/$bin_name"
  [[ -x "$binary" ]] || { echo "没有编译好的二进制，先跑 build" >&2; exit 4; }
  mkdir -p "$work/runs"
  for entry in "${runs[@]}"; do
    read -r candidate merge <<<"$entry"
    out="$work/runs/${candidate}_${merge}.out"
    if [[ -f "$out" ]] && tail -n 1 "$out" | grep -q 'name=done'; then
      echo "跳过 ${candidate}_${merge}（已有完整产物）"
      continue
    fi
    rm -f "$out"
    tight=""; run_candidate="$candidate"
    if [[ "$candidate" == "tight_literal" ]]; then tight="literal"; run_candidate="baseline"; fi
    E177_CANDIDATE="$run_candidate" E177_TIGHT="$tight" E177_MERGE="$merge" E177_THREADS="${E177_THREADS:-8}" \
      bash "$SINGLEFS_REPO/research/scripts/run-with-memory-cap.sh" 4G nice -n 19 "$binary" segment-one \
      > "$out.partial" 2> "$work/runs/${candidate}_${merge}.err"
    mv "$out.partial" "$out"
    echo "跑完 ${candidate}_${merge}：$(wc -l < "$out") 行"
  done
  # 基线跑出来的产物应当与入库的 E177 第一段产物逐行相同（只多一行 name=candidate_run、emitted 数多 1）。
  original_out="$SINGLEFS_REPO/research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out"
  if [[ -f "$work/runs/baseline_none.out" && -f "$original_out" ]]; then
    if diff <(grep -v 'name=candidate_run\|name=done' "$work/runs/baseline_none.out") <(grep -v 'name=done' "$original_out") >/dev/null; then
      echo "基线复现入库产物：逐行相同（除 name=candidate_run 与 name=done）"
    else
      echo "基线与入库产物不同" >&2
      exit 5
    fi
  fi
}

do_compare() {
  for entry in "${runs[@]}"; do
    read -r candidate merge <<<"$entry"
    [[ -f "$work/runs/${candidate}_${merge}.out" ]] || { echo "缺 ${candidate}_${merge}.out，先跑 run" >&2; exit 6; }
  done
  python3 "$here/summary.py" --compare "$work/runs" "$here/expected/summary.txt"
}

case "$stage" in
  build) do_build ;;
  run) do_run ;;
  compare) do_compare ;;
  all) do_build; do_run; do_compare ;;
  *) echo "阶段只有 build|run|compare|all" >&2; exit 2 ;;
esac
