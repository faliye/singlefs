#!/usr/bin/env bash
# 用法：SINGLEFS_REPO=<冻结副本根（含 crates/）> SINGLEFS_MAIN_REPO=<主仓根，只取 research/scripts/> bash rerun.sh <输出目录> [阶段…]
#   第一次调用输出目录必须为空（或不存在）；同一输出目录里可以接着调下一个阶段（分段跑：每段前台 ≤ 4 分钟，同一时刻只起一条 cargo，每条 cargo 经内存包装 6G / 8G、线程 15）。
#   阶段：static（sha256 + 变异表全表锚点）  f2diff（差分模型，不编译）  baseline（编译 + 基线 harness 用例 + core lib）
#         mut <从> <到>（变异行序号区间，追加写 mutations.txt；全表 48 行，checker 档行打 SKIPPED）  adhoc（窄版 A1–A3，追加写 adhoc.txt）
#   不给阶段 = static f2diff baseline。输出：<输出目录>/{static,f2diff,baseline,mutations,adhoc}.txt；bash compare.sh <目录甲> <目录乙> 比两次。
#   可选环境变量：SONNET_TARGET=<编译目录>（默认 <输出目录>/work/target，空目录从头编译）。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"; : "${SINGLEFS_MAIN_REPO:?要设 SINGLEFS_MAIN_REPO}"
out=${1:?要给一个输出目录}; shift || true
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
S=$SINGLEFS_MAIN_REPO/research/scripts
mkdir -p "$out"
if [ ! -d "$out/work/repo" ]; then
  [ -z "$(ls -A "$out" 2>/dev/null)" ] || { echo "输出目录第一次用必须是空的：$out" >&2; exit 2; }
  mkdir -p "$out/work/repo"
  cp -a "$SINGLEFS_REPO"/crates "$SINGLEFS_REPO"/Cargo.toml "$SINGLEFS_REPO"/Cargo.lock "$SINGLEFS_REPO"/litmus "$out/work/repo"/
fi
repo=$out/work/repo
export CARGO_TARGET_DIR=${SONNET_TARGET:-$out/work/target}
stages=("$@"); [ ${#stages[@]} -gt 0 ] || stages=(static f2diff baseline)
i=0
while [ $i -lt ${#stages[@]} ]; do
  case ${stages[$i]} in
    static)
      { printf 'R1 sha256 OK 份数\t%s\n' "$(cd "$SINGLEFS_REPO" && sha256sum -c "$here/crates-sha256.txt" 2>&1 | grep -c ': OK$' || true)"
        python3 -I "$here/static_anchor.py" "$repo" "$here/rows.txt"; } > "$out/static.txt" ;;
    f2diff)
      python3 -I "$here/f2_differential.py" > "$out/f2diff.txt" ;;
    baseline)
      cd "$repo"
      nice -n 19 bash "$S/run-with-memory-cap.sh" 6G bash "$S/capped.sh" 15 cargo test --offline --no-run -p singlefs-core --lib -p singlefs-harness $(cat "$here/test-args.txt") > "$out/work/build.log" 2>&1
      {
        nice -n 19 bash "$S/run-with-memory-cap.sh" 4G bash "$S/capped.sh" 15 cargo test --offline -p singlefs-core --lib 2>&1 | grep -E '^test result' | sed 's/^/core-lib: /'
        for t in $(sed -E 's/--test //g' "$here/test-args.txt"); do
          if [ "$t" = random_histories ]; then cap=8G; else cap=4G; fi
          printf '%s: ' "$t"
          nice -n 19 bash "$S/run-with-memory-cap.sh" $cap bash "$S/capped.sh" 15 cargo test --offline -p singlefs-harness --test "$t" 2>&1 | grep -E '^test result'
        done
      } | sed -E 's/ finished in [0-9.]+s//' > "$out/baseline.txt"
      cd - > /dev/null ;;
    mut)
      from=${stages[$((i+1))]}; to=${stages[$((i+2))]}; i=$((i+2))
      ROWS_FROM=$from ROWS_TO=$to python3 -I "$here/mutrun.py" "$repo" "$SINGLEFS_MAIN_REPO" "$here/rows.txt" >> "$out/mutations.txt" ;;
    adhoc)
      python3 -I "$here/adhoc.py" "$repo" "$SINGLEFS_MAIN_REPO" "$here/adhoc-cases.tsv" | sed -E 's/ failed=.*$//' >> "$out/adhoc.txt" ;;
    *) echo "不认的阶段：${stages[$i]}" >&2; exit 2 ;;
  esac
  i=$((i+1))
done
