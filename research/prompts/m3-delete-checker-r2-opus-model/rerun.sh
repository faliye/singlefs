#!/usr/bin/env bash
# 删除线 checker 设计轮第二轮攻方腿（Opus）原型复跑：冻结树拷一份到工作目录，打两份补丁（workspace 加原型 crate；walk.rs 加只读的
# attack_stash，把删除态扫描那份版本清单上每版的事实留给原型，不改任何判定），编原型，逐步跑，出摘要与 expected-summary.txt 逐行比。
#   FROZEN_TREE=<冻结树，缺省 /tmp/claude-1000/m3-delete-checker-r2/tree> SINGLEFS_REPO=<仓根，缺省 /home/fy5090/code/singlefs> \
#     bash rerun.sh <空的工作目录> <步骤>
# 步骤：prepare build sweep-f1 sweep-f2 sweep-f3 sweep-fc-a sweep-fc-b sweep-r-a sweep-r-b press s4a knob t7-a t7-y xleaf tomb
#       crash-count show analyze summary；all 依次跑全部（实测见报告，单步不超过约 3 分钟）。
# 编译与跑编出来的代码一律经 research/scripts/run-with-memory-cap.sh（编译 6G、跑 4G），线程经 research/scripts/capped.sh 6；
# 原型线程数 M3DCR2_THREADS（缺省 6）。只跑 harness 档：不起 checker 档的测试目标与 bin，只把 singlefs-checker-tier 当库链进来。
set -euo pipefail
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="${SINGLEFS_REPO:-/home/fy5090/code/singlefs}"
frozen="${FROZEN_TREE:-/tmp/claude-1000/m3-delete-checker-r2/tree}"
work="${1:?工作目录}"
step="${2:?步骤}"
threads="${M3DCR2_THREADS:-6}"
cap="$repo/research/scripts/run-with-memory-cap.sh"
capped="$repo/research/scripts/capped.sh"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-三方攻方腿原型复跑，与同机别的活互不依赖}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示定编原型 6G、跑扫描 4G，峰值表只记了别的一次}"
binary="$work/target/release/m3dcr2-opus-proto"
run() {
  local output="$1"
  shift
  (cd "$work" && M3DCR2_THREADS="${THREADS_OVERRIDE:-$threads}" nice -n 19 bash "$capped" 6 bash "$cap" 4G "$binary" "$@") > "$work/out/$output.out" 2>&1
  grep -h -E '^(SWEEP|ELAPSED)' "$work/out/$output.out" || true
}
R20="$(python3 -c "print(','.join(['C1']*20))")"
case "$step" in
  prepare)
    [[ -z "$(ls -A "$work" 2>/dev/null)" ]] || { echo "工作目录要是空的：$work" >&2; exit 2; }
    (cd "$frozen" && sha256sum --quiet -c "$repo/research/prompts/m3-delete-checker-r2-snapshot/crates-sha256.txt")
    mkdir -p "$work/out"
    rsync -a --exclude target --exclude .git "$frozen/" "$work/tree/"
    (cd "$work/tree" && patch -s -p1 < "$model/workspace.patch" && patch -s -p1 < "$model/walk-attack-stash.patch")
    mkdir -p "$work/tree/crates/m3dcr2-opus-proto"
    cp -r "$model/proto/Cargo.toml" "$model/proto/src" "$work/tree/crates/m3dcr2-opus-proto/"
    echo "prepared $work"
    ;;
  build)
    (cd "$work/tree" && CARGO_TARGET_DIR="$work/target" nice -n 19 bash "$capped" 6 bash "$cap" 6G cargo build --offline --release -p m3dcr2-opus-proto) > "$work/out/build.log" 2>&1
    tail -n 1 "$work/out/build.log"
    ;;
  sweep-f1) run sweep-f1 sweep f1 ;;
  sweep-f2) run sweep-f2 sweep f2 ;;
  sweep-f3) run sweep-f3 sweep f3 ;;
  sweep-fc-a) run sweep-fc-a sweep fc 0 450 ;;
  sweep-fc-b) run sweep-fc-b sweep fc 450 100000 ;;
  sweep-r-a) M3DCR2_SEED_BASE=1 M3DCR2_SEEDS=150 run sweep-r-a sweep r ;;
  sweep-r-b) M3DCR2_SEED_BASE=151 M3DCR2_SEEDS=150 run sweep-r-b sweep r ;;
  press) M3DCR2_FOLLOWS=0,1,30 run bad-press bad press C3 C2,U3 C2,U3,R1 C2,U3,R2 C2,U3,R3 C3,R2 C3,R3 C3,R4 C2,U2,U3 C1,X,C1 C2,U3,X "C2,U3,$R20" "C3,$R20" ;;
  s4a) run bad-s4a bad s4a ;;
  knob) run bad-knob bad knob ;;
  t7-a) run bad-t7-a bad t7 C2,U3,R2 ;;
  t7-y) run bad-t7-y bad t7 C2,U3,Y4 ;;
  xleaf) run bad-xleaf bad xleaf 240 ;;
  tomb) run bad-tomb bad tomb ;;
  crash-count) run crash-count-c1 crash count c1; run crash-count-c2 crash count c2 ;;
  show)
    for history in U1,R3 U1,R2,R1 C2,C1,X,C1 U1,Y6,R2 C2,U1,R2,U2,U3,C1 C31800; do
      THREADS_OVERRIDE=1 run "show-${history//,/_}" show "$history" 144
    done
    ;;
  analyze)
    python3 "$model/aggregate.py" "$work/out" > "$work/out/analysis-sweeps.out"
    python3 "$model/press_aggregate.py" "$work/out/bad-press.out" > "$work/out/analysis-press.out"
    grep -c . "$work/out/analysis-sweeps.out" "$work/out/analysis-press.out"
    ;;
  summary)
    python3 "$model/summarize.py" "$work/out" > "$work/summary.txt"
    if diff -q "$model/expected-summary.txt" "$work/summary.txt" > /dev/null; then
      echo "SUMMARY_MATCHES lines=$(wc -l < "$work/summary.txt")"
    else
      diff "$model/expected-summary.txt" "$work/summary.txt" | head -n 20
      echo "SUMMARY_DIFFERS" >&2
      exit 1
    fi
    ;;
  all)
    for one in prepare build sweep-f1 sweep-f2 sweep-f3 sweep-fc-a sweep-fc-b sweep-r-a sweep-r-b press s4a knob t7-a t7-y xleaf tomb crash-count show analyze summary; do
      bash "$0" "$work" "$one"
    done
    ;;
  *) echo "不认得的步骤：$step" >&2; exit 2 ;;
esac
