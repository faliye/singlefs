#!/usr/bin/env bash
# m3-formal-code-r3 攻方腿原型的复跑。用法：bash rerun.sh <冻结副本 tree 目录> <草稿目录> [件名…]
# 件名：spans（五种取样点的段跨步扫描，约 1 分钟）、k1（层 0 枚举域的小样，几秒）、segs（四段 rbf 历史的段表，几秒）、
#       k4t1（今天的可写挂载对系统配置槽 / 根槽一时读不出的两种时机，几秒）、inject（崩溃注入调用点原样、六件并行，约 2 分钟）
# 每条命令经 run-with-memory-cap.sh、capped.sh 6；输出写 <草稿目录>/runs/，跑完与模型目录 outputs/ 同名文件比（去掉包装行）。
set -u
frozen="$1"; D="$2"; shift 2
items=("$@"); [ ${#items[@]} -eq 0 ] && items=(spans k1 segs k4t1 inject)
here="$(cd "$(dirname "$0")/.." && pwd)"
repo="$(cd "$here/../../.." && pwd)"
W="$repo/scripts/run-with-memory-cap.sh"; C="$repo/scripts/capped.sh"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑 m3-formal-code-r3 攻方原型，逐段历史的峰值不同"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑 m3-formal-code-r3 攻方原型 r3probe 分种子区间并行"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600
mkdir -p "$D/runs"
rsync -a --exclude target --exclude .git "$frozen/" "$D/tree/"
mkdir -p "$D/tree/crates/r3probe/src"
cp "$here/prototypes/r3probe/Cargo.toml" "$D/tree/crates/r3probe/Cargo.toml"
cp "$here/prototypes/r3probe/src/main.rs" "$here/prototypes/r3probe/src/k4t1.rs" "$D/tree/crates/r3probe/src/"
grep -q '"crates/r3probe"' "$D/tree/Cargo.toml" || sed -i 's#    "crates/singlefs-reference-filesystem",#    "crates/singlefs-reference-filesystem",\n    "crates/r3probe",#' "$D/tree/Cargo.toml"
(cd "$D/tree" && CARGO_TARGET_DIR="$D/target" nice -n 19 bash "$W" 2G bash "$C" 6 cargo build --offline --release -p r3probe) > "$D/build.log" 2>&1 || { echo "build failed"; exit 1; }
P="$D/target/release/r3probe"
run() { local log="$1"; shift; nice -n 19 bash "$W" 2G bash "$C" 6 env SINGLEFS_LAYER0_THREADS=6 "$P" "$@" 2>&1 | grep -v -e '^run-with-memory-cap' -e 'elapsed_seconds' > "$D/runs/$log"; echo "$log rc=${PIPESTATUS[0]}"; }
for item in "${items[@]}"; do
  case "$item" in
    spans) run spans-broad-4g-8.log spans broad 4g 8 0 40 v; run spans-rbf-4g-20.log spans rbf 4g 20 0 20 v
           run spans-reuse-4g-20.log spans reuse 4g 20 0 20 v; run spans-broad-256-16.log spans broad 256 16 0 20 v
           run spans-rbf-256-24.log spans rbf 256 24 0 20 v ;;
    k1) run k1-broad-8.log k1 broad 4g 8 0 1 200000 20; run k1-broad-3.log k1 broad 4g 3 0 10 200000 20 ;;
    segs) for s in 0 1 2 3; do run "segs-rbf-$s.log" segs rbf 4g 20 "$s"; done ;;
    k4t1) run k4t1.log k4t1 ;;
    inject) rm -f "$D"/inject-*.rc; i=0
            for cfg in "broad 4g 8 0 20" "broad 4g 8 20 40" "broad 4g 8 40 60" "rbf 4g 20 0 10" "rbf 4g 20 10 20" "reuse 4g 20 0 10"; do
              i=$((i+1)); set -- $cfg
              { nice -n 19 bash "$W" 1G bash "$C" 1 "$P" inject "$1" "$2" "$3" "$4" "$5" 6 2>&1 | grep -v '^run-with-memory-cap' > "$D/runs/inject-$i.log"; echo "${PIPESTATUS[0]}" > "$D/inject-$i.rc"; } &
            done
            wait
            n=$(ls "$D"/inject-*.rc | wc -l); [ "$n" -eq 6 ] || { echo "inject 退出码文件 $n 个，不是 6 个：整批作废"; exit 1; }
            for i in 1 2 3 4 5 6; do echo "inject-$i.log rc=$(cat "$D/inject-$i.rc")"; done ;;
  esac
done
for f in "$D"/runs/*.log; do b=$(basename "$f"); if [ -f "$here/outputs/$b" ]; then cmp -s "$f" "$here/outputs/$b" && echo "same $b" || echo "DIFF $b"; fi; done
