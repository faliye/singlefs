#!/usr/bin/env bash
# m3-w3-r3 攻方腿（Opus）的复跑。用法：SINGLEFS_REPO=<仓根> [W3R3_MODEL_DIR=<本目录>] [R2_MODEL_DIR=<第二轮攻方模型目录>] [FROZEN_TREE=<冻结树>] bash rerun.sh <工作目录> <段>
# 路径只从环境变量与参数取：W3R3_MODEL_DIR 默认 $SINGLEFS_REPO/research/prompts/m3-w3-r3-opus-model，R2_MODEL_DIR 默认 $SINGLEFS_REPO/research/prompts/m3-w3-r2-opus-model，
# FROZEN_TREE 默认 ~/.local/share/singlefs/work/m3-w3-r1/tree。编译与跑都经 research/scripts/run-with-memory-cap.sh（编译 6G、跑 4G），线程 5（research/scripts/capped.sh）。
# 段（每段一条前台命令，各在 4 分钟以内）：
#   build-model        编抽象模型（model/，不依赖 crates/）
#   model-verify       verify 组 0..4（独立重算前两轮的数）
#   model-y1           y1 组 0..1、y1x 组 0..6、y1m 组 0..2（Y1 两种取法）
#   model-clone-first  clone 组 0..7、first 组 0..1（克隆头、新实例第一个 txg）
#   build-proto        工作目录里不许已有 repo/：拷冻结树、对 m3-w3-r3-snapshot/tree-sha256.txt 核、放进第二轮攻方原型 crate（取自 $R2_MODEL_DIR/proto，原样）、编基线
#   probe-base         基线原型：层 0 全量枚举 P 那次发布（65548 个状态），四个点名状态逐步打印（与第二轮 expected/probe-AB.out 同一条命令）
#   build-proto-kept   在副本的 mount.rs 上打 proto-patch/patch-Y1-kept.diff（回退的 inode 号水位只取没被抛弃的根），再编
#   probe-kept         打过补丁的原型，同 probe-base 的命令
#   kept-shard-<i>     打过补丁的原型，B 历史（崩溃 + 下一次挂载读回全 0）层 0 分片 i/20；二十片合计约 17 分钟（长段，可选）
#   kept-merge         读二十份账本并起来（要先跑完 kept-shard-0..19）
#   compare            拿 $W/out 与 expected/ 比（模型只比 W3R3MODEL / W3R3COUNT / W3R3EXAMPLE 行；原型只比 W3R2 行、去掉 elapsed_s 与 DONE 行；排序后比）
#   compare-quick      同 compare，不比 kept-merge-of-20.out
set -euo pipefail
W=${1:?工作目录}; PART=${2:?段}
R=${SINGLEFS_REPO:?设 SINGLEFS_REPO 为仓根}
M=${W3R3_MODEL_DIR:-$R/research/prompts/m3-w3-r3-opus-model}
R2=${R2_MODEL_DIR:-$R/research/prompts/m3-w3-r2-opus-model}
FROZEN=${FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-w3-r1/tree}
CAP="bash $R/research/scripts/run-with-memory-cap.sh"
THREADS="bash $R/research/scripts/capped.sh 5"
REAL_CARGO=${REAL_CARGO:-$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo}
mkdir -p "$W/out"
MODEL=$W/target-model/release/w3r3-model
PROTO=$W/target/release/w3-r2-attack
model() { nice -n 19 $CAP 4G "$MODEL" "$@"; }
proto() { # $1 二进制 $2 分片开关（空 = 不分片） 其余：二进制参数
  local bin=$1 shard=$2; shift 2
  (cd "$W" && env W3_WIDTH=256 CARGO="$REAL_CARGO" ${shard:+SINGLEFS_LAYER0_PROGRESS_DIRECTORY=$W/l0kept SINGLEFS_LAYER0_INPUT_FINGERPRINT=w3r3kept1 SINGLEFS_LAYER0_SHARD=$shard} \
    nice -n 19 $THREADS $CAP 4G "$bin" "$@")
}
case "$PART" in
  build-model)
    rsync -a --exclude target "$M/model/" "$W/model/"
    (cd "$W/model" && CARGO_TARGET_DIR=$W/target-model nice -n 19 $THREADS $CAP 6G cargo build --release --offline 2>>"$W/build.log")
    sha256sum "$MODEL" ;;
  model-verify) for i in 0 1 2 3 4; do model verify $i > "$W/out/verify-$i.out"; done ;;
  model-y1)
    for i in 0 1; do model y1 $i > "$W/out/y1-$i.out"; done
    for i in 0 1 2 3 4 5 6; do model y1x $i > "$W/out/y1x-$i.out"; done
    for i in 0 1 2; do model y1m $i > "$W/out/y1m-$i.out"; done ;;
  model-clone-first)
    for i in 0 1 2 3 4 5 6 7; do model clone $i > "$W/out/clone-$i.out"; done
    for i in 0 1; do model first $i > "$W/out/first-$i.out"; done ;;
  build-proto)
    [ ! -e "$W/repo" ] || { echo "工作目录里已有 repo/：$W" >&2; exit 2; }
    rsync -a --exclude target --exclude .git "$FROZEN/" "$W/repo/"
    (cd "$W/repo" && sha256sum -c --quiet "$R/research/prompts/m3-w3-r3-snapshot/tree-sha256.txt" && echo "冻结树副本对上清单")
    mkdir -p "$W/repo/crates/w3-r2-attack/src"
    cp "$R2/proto/Cargo.toml" "$W/repo/crates/w3-r2-attack/Cargo.toml"
    cp "$R2/proto/src/main.rs" "$R2/proto/src/shadow.rs" "$W/repo/crates/w3-r2-attack/src/"
    python3 -I -c 'import sys
p=sys.argv[1]; s=open(p).read(); old="    \"crates/singlefs-reference-filesystem\",\n]"
assert s.count(old)==1, "工作区成员表的写法变了"
open(p,"w").write(s.replace(old,"    \"crates/singlefs-reference-filesystem\",\n    \"crates/w3-r2-attack\",\n]"))' "$W/repo/Cargo.toml"
    (cd "$W/repo" && CARGO_TARGET_DIR=$W/target nice -n 19 $THREADS $CAP 6G cargo build --release --offline -p w3-r2-attack 2>>"$W/build.log")
    mkdir -p "$W/bin"; cp "$PROTO" "$W/bin/base"; sha256sum "$W/bin/base" ;;
  probe-base) W3_ONLY_VERBOSE=1 proto "$W/bin/base" "" run - 2 'A:c11;B:c1,1' x 524287,262143,65535,1 > "$W/out/probe-base.out" 2>&1 ;;
  build-proto-kept)
    (cd "$W/repo" && patch -p1 --forward < "$M/proto-patch/patch-Y1-kept.diff")
    (cd "$W/repo" && CARGO_TARGET_DIR=$W/target nice -n 19 $THREADS $CAP 6G cargo build --release --offline -p w3-r2-attack 2>>"$W/build.log")
    mkdir -p "$W/bin"; cp "$PROTO" "$W/bin/kept"; sha256sum "$W/bin/kept" ;;
  probe-kept) W3_ONLY_VERBOSE=1 proto "$W/bin/kept" "" run - 2 'A:c11;B:c1,1' x 524287,262143,65535,1 > "$W/out/probe-kept.out" 2>&1 ;;
  kept-shard-*) i=${PART#kept-shard-}; mkdir -p "$W/l0kept"; proto "$W/bin/kept" "$i/20" run - 2 'B:c1' x > "$W/out/kept-shard-$i-of-20.out" 2>&1; grep '^W3R2DONE' "$W/out/kept-shard-$i-of-20.out" ;;
  kept-merge) proto "$W/bin/kept" "merge/20" run - 2 'B:c1' x > "$W/out/kept-merge-of-20.out" 2>&1 ;;
  compare|compare-quick)
    status=0; compared=0
    for e in "$M"/expected/*.out; do
      b=$(basename "$e"); f=$W/out/$b
      if [ "$PART" = compare-quick ] && [ "$b" = kept-merge-of-20.out ]; then continue; fi
      if [ ! -f "$f" ]; then echo "缺：$f"; status=1; continue; fi
      norm() { grep -E '^(W3R3(MODEL|COUNT|EXAMPLE)|W3R2)' "$1" | grep -v '^W3R2\(DONE\|SHARDDONE\)' | sed -E 's/ elapsed_s=[0-9.]+//' | LC_ALL=C sort; }
      if diff <(norm "$e") <(norm "$f") > /dev/null; then compared=$((compared+1)); else echo "不同：$b"; status=1; fi
    done
    echo "compared=$compared status=$status"; exit $status ;;
  *) echo "不认的段：$PART" >&2; exit 2 ;;
esac
echo "段 $PART 跑完"
