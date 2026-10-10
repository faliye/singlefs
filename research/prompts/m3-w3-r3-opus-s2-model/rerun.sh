#!/usr/bin/env bash
# m3-w3-r3 攻方腿第二次抽样（Opus s2）的复跑。用法：SINGLEFS_REPO=<仓根> [W3R3S2_MODEL_DIR=<模型目录>] bash rerun.sh <工作目录> <段>
# 路径只从环境变量与参数取：模型目录默认 $SINGLEFS_REPO/research/prompts/m3-w3-r3-opus-s2-model。
# 段（每段一条前台命令，各在 4 分钟以内）：
#   build        工作目录要是空的：拷模型源码、编（自己的编译目录 $W/target）
#   run-a … run-d  抽象模型 18 组（list 列出），每段至多 5 组并行，每组一个线程
#   traces       报告里引的十条逐步打印
#   recheck-r2   拷第二轮攻方与辩方的模型进 $W、编、各跑一组（攻方 main 0、辩方 defense 4），与它们自己的 expected 比
#   compare      拿 $W/out 与模型目录 expected/ 比（只比 W3R3S2 开头的行、去掉 elapsed_s、排序后比）
# 编译与跑都经 research/scripts/run-with-memory-cap.sh（编译 6G、跑 4G），线程 5（research/scripts/capped.sh）；同一时刻只起一条 cargo。
set -euo pipefail
W=${1:?工作目录}; PART=${2:?段}
R=${SINGLEFS_REPO:?设 SINGLEFS_REPO 为仓根}
M=${W3R3S2_MODEL_DIR:-$R/research/prompts/m3-w3-r3-opus-s2-model}
CAP="bash $R/research/scripts/run-with-memory-cap.sh"
THREADS="bash $R/research/scripts/capped.sh 5"
BIN=$W/target/release/w3-r3-s2-model
mkdir -p "$W/out"
model() { nice -n 19 $CAP 4G "$BIN" "$@"; }
batch() { # batch <批号> <组号…>：并行跑，退出码写 out/b<批号>-<组号>.rc，等齐后数件数、逐个读
  local b=$1; shift; local n=$# i bad=0
  rm -f "$W/out/b$b-"*.rc
  for i in "$@"; do { set +e; model run "$i" > "$W/out/run-$i.out" 2> "$W/out/run-$i.err"; echo "$?" > "$W/out/b$b-$i.rc"; } & done
  wait
  [ "$(ls "$W/out/b$b-"*.rc | wc -l)" -eq "$n" ] || { echo "批 $b 的退出码文件数不对，整批作废" >&2; exit 1; }
  for i in "$@"; do [ "$(cat "$W/out/b$b-$i.rc")" = 0 ] || { echo "组 $i 退出码 $(cat "$W/out/b$b-$i.rc")" >&2; bad=1; }; done
  return $bad
}
case "$PART" in
  build)
    [ -z "$(ls -A "$W" | grep -v '^out$')" ] || { echo "工作目录不空：$W" >&2; exit 2; }
    mkdir -p "$W/model"
    cp "$M/Cargo.toml" "$W/model/Cargo.toml"; mkdir -p "$W/model/src"; cp "$M/src/main.rs" "$W/model/src/main.rs"
    (cd "$W/model" && CARGO_TARGET_DIR=$W/target nice -n 19 $THREADS $CAP 6G cargo build --release --offline 2>>"$W/build.log")
    sha256sum "$BIN" ;;
  run-a) batch 1 0 1 2 3 4 ;;
  run-b) batch 2 5 6 7 8 9 ;;
  run-c) batch 3 10 11 12 13 14 ;;
  run-d) batch 4 15 16 17 ;;
  traces)
    {
      model trace 0 create1,crash-hide,rollback1
      model trace 0 create1,crash-hide,rollback1,crash-mid-create
      model trace 4 create1,rollback1-cur-hidden,create1
      model trace 6 create1,rollback1-cur-hidden,create1
      model trace 12 clone
      model trace 12 clone,create1,create2
      model trace 12 clone,create1,rollback1,create2
      model trace 12 create1,clone
      model trace 16 empty,create1,crash-hide-two,create1
      model trace 15 create1,crash-hide-root-and-record
    } > "$W/out/traces.out" 2> "$W/out/traces.err" ;;
  recheck-r2)
    mkdir -p "$W/recheck"
    rsync -a --exclude target "$R/research/prompts/m3-w3-r2-opus-model/timeline-model/" "$W/recheck/r2-opus-timeline/"
    rsync -a --exclude target "$R/research/prompts/m3-w3-r2-sonnet-model/timeline-model/" "$W/recheck/r2-sonnet-timeline/"
    (cd "$W/recheck/r2-opus-timeline" && CARGO_TARGET_DIR=$W/recheck/target-a nice -n 19 $THREADS $CAP 6G cargo build --release --offline 2>>"$W/build.log")
    (cd "$W/recheck/r2-sonnet-timeline" && CARGO_TARGET_DIR=$W/recheck/target-d nice -n 19 $THREADS $CAP 6G cargo build --release --offline 2>>"$W/build.log")
    nice -n 19 $CAP 4G "$W/recheck/target-a/release/w3-r2-timeline-model" main 0 > "$W/out/recheck-r2-opus-main-0.txt"
    nice -n 19 $CAP 4G "$W/recheck/target-d/release/w3r2-timeline-model" defense 4 > "$W/out/recheck-r2-sonnet-defense-4.txt"
    n2() { grep -v TRACE "$1" | sed -E 's/ elapsed_s=[0-9.]+//' | LC_ALL=C sort; }
    diff <(n2 "$W/out/recheck-r2-opus-main-0.txt") <(n2 "$R/research/prompts/m3-w3-r2-opus-model/expected/model-main-0.out") > /dev/null && echo "r2 攻方 main 0：与它的 expected/model-main-0.out 逐行相同"
    diff <(n2 "$W/out/recheck-r2-sonnet-defense-4.txt") <(n2 "$R/research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-4.out") > /dev/null && echo "r2 辩方 defense 4：与它的 expected/model-defense-4.out 逐行相同"
    grep -o 'rollback_jumps_from_abandoned_roots_only=[0-9]*' "$W/out/recheck-r2-opus-main-0.txt"
    grep 'candidate=I930/C2a-create ' "$W/out/recheck-r2-opus-main-0.txt" | grep -o 'false_positive=[0-9]*' | head -1
    python3 -I "$R/research/prompts/m3-w3-r2-sonnet-model/tools/table2.py" C4g,C2ig "$W/out/recheck-r2-sonnet-defense-4.txt" ;;
  compare)
    status=0; compared=0
    norm() { grep '^W3R3S2' "$1" | sed -E 's/ elapsed_s=[0-9.]+//' | LC_ALL=C sort; }
    for e in "$M"/expected/*.out; do
      f=$W/out/$(basename "$e")
      if [ ! -f "$f" ]; then echo "缺：$f"; status=1; continue; fi
      if diff <(norm "$e") <(norm "$f") > /dev/null; then compared=$((compared+1)); else echo "不同：$(basename "$e")"; status=1; fi
    done
    echo "compared=$compared status=$status"; exit $status ;;
  *) echo "不认的段：$PART" >&2; exit 2 ;;
esac
echo "段 $PART 跑完"
