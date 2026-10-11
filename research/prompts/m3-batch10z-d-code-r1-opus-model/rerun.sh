#!/usr/bin/env bash
# 复跑：bash rerun.sh <新工作目录> [规格名 …]（规格名是 scripts/spec-<名>.tsv 的 <名>，缺省 a1 a2 a3 a4 a5 a6 x x2 全跑）。
# 环境变量：SINGLEFS_FROZEN=冻结副本根（缺省 ~/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree），SINGLEFS_TOOLS=仓根（取内存包装与线程上限脚本），
# THREADS=并行件数（缺省 6）。每件经 run-with-memory-cap.sh 4G 与 capped.sh 1 跑；末尾按规格汇总，与 out/ 里的汇总逐行比。
set -u
work=$1; shift
specs=("$@"); [[ ${#specs[@]} -gt 0 ]] || specs=(a1 a2 a3 a4 a5 a6 x x2)
here=$(cd "$(dirname "$0")" && pwd)
frozen=${SINGLEFS_FROZEN:-$HOME/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree}
tools=${SINGLEFS_TOOLS:?要仓根}
threads=${THREADS:-6}
mkdir -p "$work/runs"
rsync -a --exclude target --exclude .git "$frozen/" "$work/repo/"
(cd "$work/repo" && patch -p0 < "$here/src/opus10zd-arms.patch")
mkdir -p "$work/repo/crates/opus10zd-probe"
cp -r "$here/src/opus10zd-probe/." "$work/repo/crates/opus10zd-probe/"
export CARGO_TARGET_DIR=$work/target
for t in opus10zd_h2_borrowed opus10zd_abandoned_slot; do
  (cd "$work/repo" && nice -n 19 bash "$tools/scripts/hosts/run-with-memory-cap.sh" 12G bash "$tools/scripts/hosts/capped.sh" "$threads" \
    cargo test --release --offline -p opus10zd-probe --test "$t" --no-run) > "$work/build-$t.log" 2>&1 || { echo "编译 $t 失败，见 $work/build-$t.log"; exit 1; }
done
bin_of() { ls -t "$work/target/release/deps/" | grep "^$1-[0-9a-f]*\$" | head -1; }
h2bin=$(bin_of opus10zd_h2_borrowed); xbin=$(bin_of opus10zd_abandoned_slot)
for spec in "${specs[@]}"; do
  n=0
  while IFS=$'\t' read -r name envs; do
    envs=${envs//OPUS_BIN=opus10zd_abandoned_slot-445dd4894c7ea5ed/}
    bin=$h2bin; [[ $name == x* || $name == z* ]] && bin=$xbin
    { env $envs nice -n 19 bash "$tools/scripts/hosts/run-with-memory-cap.sh" 4G bash "$tools/scripts/hosts/capped.sh" 1 \
        "$work/target/release/deps/$bin" --nocapture opus_sweep > "$work/runs/$name.log" 2>&1; echo "$?" > "$work/runs/$name.rc"; } &
    n=$((n+1))
    if (( n % threads == 0 )); then wait; fi
  done < "$here/scripts/spec-$spec.tsv"
  wait
done
for spec in "${specs[@]}"; do
  case $spec in
    x) python3 "$here/scripts/aggx.py" "$work/runs/x1-" > "$work/summary-x1.txt"; python3 "$here/scripts/aggx.py" "$work/runs/z1-" > "$work/summary-z1.txt" ;;
    x2) python3 "$here/scripts/aggx.py" "$work/runs/x2-" > "$work/summary-x2.txt" ;;
    *) python3 "$here/scripts/agg.py" "$work/runs/$spec-" x > "$work/summary-$spec.txt" ;;
  esac
done
for f in "$work"/summary-*.txt; do
  b=$(basename "$f")
  if cmp -s "$f" "$here/out/$b"; then echo "$b 全同"; else echo "$b 不同"; fi
done
echo "退出码不为 0 的件：$(grep -lv '^0$' "$work"/runs/*.rc 2>/dev/null | wc -l)"
