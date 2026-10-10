#!/usr/bin/env bash
# m3-w3-r2 攻方腿（Opus）的复跑。用法：SINGLEFS_REPO=<仓根> [W3R2_MODEL_DIR=<模型目录>] [FROZEN_TREE=<冻结树>] bash rerun.sh <工作目录> <段>
# 路径只从环境变量与参数取：模型目录默认 $SINGLEFS_REPO/research/prompts/m3-w3-r2-opus-model，冻结树默认 ~/.local/share/singlefs/work/m3-w3-r1/tree。
# 段（每段一条前台命令，负载低时各在 4 分钟以内）：
#   build-proto   工作目录要是空的：拷冻结树、对 m3-w3-r2-snapshot/tree-sha256.txt 核、放进原型 crate、编（仓副本，不动 crates/）
#   build-model   编抽象时间线模型
#   model-main-a / model-main-b / model-rest   抽象模型（main 0 1 / main 2 3 / mutation 0..11 与 ring6 0 1）
#   probe         原型：层 0 全量枚举 P 那次建 inode 的发布（65548 个状态），只对四个点名的状态逐步打印（A、B 两种历史）
#   shard-<i>     原型：层 0 自己的分片 i/20（i = 0..19），A、B 两种历史各跑一遍；二十片合计约 35 分钟（长段，可选）
#   merge         读二十份账本并起来（要先跑完 shard-0..19）
#   compare       拿 $W/out 与模型目录 expected/ 比（去掉 elapsed_s 字段、只比 W3R2 开头的行、排序后比）；compare-quick 不比 merge 那一份
# 编译与跑都经 research/scripts/run-with-memory-cap.sh（编译 6G、跑 4G），线程 6（research/scripts/capped.sh）。
set -euo pipefail
W=${1:?工作目录}; PART=${2:?段}
R=${SINGLEFS_REPO:?设 SINGLEFS_REPO 为仓根}
M=${W3R2_MODEL_DIR:-$R/research/prompts/m3-w3-r2-opus-model}
FROZEN=${FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-w3-r1/tree}
CAP="bash $R/research/scripts/run-with-memory-cap.sh"
THREADS="bash $R/research/scripts/capped.sh 6"
REAL_CARGO=${REAL_CARGO:-$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo}
mkdir -p "$W/out"
BIN=$W/target/release/w3-r2-attack
MODEL=$W/target-model/release/w3-r2-timeline-model
proto() { # $1 分片开关（空 = 不分片） 其余：二进制参数
  local shard=$1; shift
  (cd "$W" && env W3_WIDTH=256 CARGO="$REAL_CARGO" ${shard:+SINGLEFS_LAYER0_PROGRESS_DIRECTORY=$W/l0progress SINGLEFS_LAYER0_INPUT_FINGERPRINT=w3r2ab1 SINGLEFS_LAYER0_SHARD=$shard} \
    nice -n 19 $THREADS $CAP 4G "$BIN" "$@")
}
model() { nice -n 19 $CAP 4G "$MODEL" "$@"; }
case "$PART" in
  build-proto)
    [ -z "$(ls -A "$W" | grep -v '^out$')" ] || { echo "工作目录不空：$W" >&2; exit 2; }
    rsync -a --exclude target --exclude .git "$FROZEN/" "$W/repo/"
    (cd "$W/repo" && sha256sum -c --quiet "$R/research/prompts/m3-w3-r2-snapshot/tree-sha256.txt" && echo "冻结树副本对上清单")
    mkdir -p "$W/repo/crates/w3-r2-attack/src"
    cp "$M/proto/Cargo.toml" "$W/repo/crates/w3-r2-attack/Cargo.toml"
    cp "$M/proto/src/main.rs" "$M/proto/src/shadow.rs" "$W/repo/crates/w3-r2-attack/src/"
    python3 -I -c 'import sys
p=sys.argv[1]; s=open(p).read(); old="    \"crates/singlefs-reference-filesystem\",\n]"
assert s.count(old)==1, "工作区成员表的写法变了"
open(p,"w").write(s.replace(old,"    \"crates/singlefs-reference-filesystem\",\n    \"crates/w3-r2-attack\",\n]"))' "$W/repo/Cargo.toml"
    (cd "$W/repo" && CARGO_TARGET_DIR=$W/target nice -n 19 $THREADS $CAP 6G cargo build --release --offline -p w3-r2-attack 2>>"$W/build.log")
    sha256sum "$BIN" ;;
  build-model)
    rsync -a --exclude target "$M/timeline-model/" "$W/timeline-model/"
    (cd "$W/timeline-model" && CARGO_TARGET_DIR=$W/target-model nice -n 19 $THREADS $CAP 6G cargo build --release --offline 2>>"$W/build.log")
    sha256sum "$MODEL" ;;
  model-main-a) for i in 0 1; do model main $i > "$W/out/model-main-$i.out"; done ;;
  model-main-b) for i in 2 3; do model main $i > "$W/out/model-main-$i.out"; done ;;
  model-rest)
    for i in $(seq 0 11); do model mutation $i > "$W/out/model-mutation-$i.out"; done
    for i in 0 1; do model ring6 $i > "$W/out/model-ring6-$i.out"; done ;;
  probe) W3_ONLY_VERBOSE=1 proto "" run - 2 'A:c11;B:c1,1' x 524287,262143,65535,1 > "$W/out/probe-AB.out" 2>&1 ;;
  shard-*) i=${PART#shard-}; proto "$i/20" run - 2 'A:c11;B:c1' x > "$W/out/ab-shard-$i-of-20.out" 2>&1; grep '^W3R2DONE' "$W/out/ab-shard-$i-of-20.out" ;;
  merge) proto "merge/20" run - 2 'A:c11;B:c1' x > "$W/out/ab-merge-of-20.out" 2>&1 ;;
  compare|compare-quick)
    status=0; compared=0
    for e in "$M"/expected/*.out; do
      f=$W/out/$(basename "$e")
      if [ "$PART" = compare-quick ] && [ "$(basename "$e")" = ab-merge-of-20.out ]; then continue; fi
      if [ ! -f "$f" ]; then echo "缺：$f"; status=1; continue; fi
      norm() { grep '^W3R2' "$1" | grep -v '^W3R2\(DONE\|SHARDDONE\)' | sed -E 's/ elapsed_s=[0-9.]+//' | LC_ALL=C sort; }
      if diff <(norm "$e") <(norm "$f") > /dev/null; then compared=$((compared+1)); else echo "不同：$(basename "$e")"; status=1; fi
    done
    echo "compared=$compared status=$status"; exit $status ;;
  *) echo "不认的段：$PART" >&2; exit 2 ;;
esac
echo "段 $PART 跑完"
