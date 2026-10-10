#!/usr/bin/env bash
# m3-batch10-code-r1 攻方腿（Opus）复跑。用法：bash rerun.sh <阶段> <空工作目录 W>；`bash rerun.sh list W` 列出阶段，按列出的次序逐个跑。
# 原型取第 17 批攻方模型 research/prompts/m3-batch17-code-r1-opus-model/ 的 probe/main.rs 与 scripts/，在拷贝上打这里 patches/ 的两刀
# （multistage.py：一次挂载里几段读坏用「+」连起来；i79dump.py：FX_I79=1 时打出前三个 I-7.9 判红状态的 checker 原文），
# 另加一个只跑历史、切段、数掉电接缝的小原型 seamprobe/（名字不带 layer0）。只编跑 harness 档与自己的原型 crate。线程上限 15。
# 冻结副本认 FROZEN_TREE（默认 /tmp/claude-1000/m3-batch10-code-r1/tree），10c 之前的树认 PRE_TREE。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
HERE=$REPO/research/prompts/m3-batch10-code-r1-opus-model
export MODEL=$REPO/research/prompts/m3-batch17-code-r1-opus-model
FROZEN_TREE=${FROZEN_TREE:-/tmp/claude-1000/m3-batch10-code-r1/tree}
PRE_TREE=${PRE_TREE:-/tmp/claude-1000/-home-fy5090-code-singlefs/e594a7a0-d003-4a02-9455-a9dc350ed330/scratchpad/m3b17-today/tree}
STAGE=${1:?阶段名}
export ATTACK_DIR=${2:?给一个空的工作目录}
mkdir -p "$ATTACK_DIR/out" "$ATTACK_DIR/tables"
cd "$ATTACK_DIR"
S=$MODEL/scripts
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿原型批里几件并行，件数不超过线程上限"
arm() { local name=$1 tree=$2 a=$ATTACK_DIR/$1
  rsync -a --delete --exclude target --exclude .git "$tree/" "$a/"
  (cd "$a" && patch -s -p0 < "$MODEL/patches/workspace-member.patch")
  mkdir -p "$a/crates/attack-b17-probe/src" "$a/crates/attack-b10-seam/src"
  cp "$MODEL/probe/Cargo.toml" "$a/crates/attack-b17-probe/Cargo.toml"
  cp "$MODEL/probe/main.rs" "$a/crates/attack-b17-probe/src/main.rs.orig"
  python3 "$HERE/patches/multistage.py" "$a/crates/attack-b17-probe/src/main.rs.orig"
  python3 "$HERE/patches/i79dump.py" "$a/crates/attack-b17-probe/src/main.rs.orig.new"
  python3 "$HERE/patches/finalchk.py" "$a/crates/attack-b17-probe/src/main.rs.orig.new.new"
  mv "$a/crates/attack-b17-probe/src/main.rs.orig.new.new.new" "$a/crates/attack-b17-probe/src/main.rs"
  rm -f "$a/crates/attack-b17-probe/src/main.rs.orig" "$a/crates/attack-b17-probe/src/main.rs.orig.new" "$a/crates/attack-b17-probe/src/main.rs.orig.new.new"
  cp "$HERE/seamprobe/Cargo.toml" "$a/crates/attack-b10-seam/Cargo.toml"
  cp "$HERE/seamprobe/src/main.rs" "$a/crates/attack-b10-seam/src/main.rs"
  mkdir -p "$a/crates/attack-b10-seam/src/bin"; cp "$HERE/seamprobe/src/bin/phases.rs" "$a/crates/attack-b10-seam/src/bin/phases.rs"
  python3 - "$a/Cargo.toml" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
s=s.replace('"crates/attack-b17-probe",','"crates/attack-b17-probe",\n    "crates/attack-b10-seam",',1)
open(p+'.tmp','w').write(s)
PY
  mv "$a/Cargo.toml.tmp" "$a/Cargo.toml"; }
build() { ATTACK_THREADS=15 bash "$S/build.sh" "$1" > "build-$1.log" 2>&1; tail -1 "build-$1.log"; }
build_seam() { (cd "$ATTACK_DIR/jia" && CARGO_TARGET_DIR=$ATTACK_DIR/target-jia nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" 4G \
  bash "$REPO/research/scripts/capped.sh" 15 cargo build --release --offline -p attack-b10-seam --bins) > build-seam.log 2>&1; tail -1 build-seam.log; }
sum() { bash "$S/fxsum2.sh" "$@"; }
batches() { local base=$1 table=$2 n=0; split -l 15 -d "$table" "$table-"; for t in "$table"-0*; do n=$((n + 1)); bash "$S/batch.sh" "$base$n" "$t"; done; }
L0() { local arm=$1 log=$2; shift 2; env "$@" FX_I79=1 MODE=fx SCREEN=1 ATTACK_THREADS=14 RUN_CAP=6G bash "$S/run.sh" "$arm" "out/$log.log"
  grep -h '^FXPLAN\|^FXREFUSED\|^FXENUM \|^FXENUMV\|^FXI79' "out/$log.log" | sed 's/ elapsed=.*//' > "out/$log.txt" || true; cat "out/$log.txt"; }
MS_PLANS() { local P="" m f; for m in e z; do for f in -/all -/but1 -/only1 all/- all/all all/but1 all/only1 but1/- but1/all but1/but1 but1/only1 only1/- only1/all only1/but1 only1/only1; do
  P="$P;P5:all/all:$m+P6:$f:$m"; done; done; echo "${P#;}"; }
run_stage() {
  case "$1" in
    arm-jia) arm jia "$FROZEN_TREE" ;;
    arm-pre) arm pre "$PRE_TREE" ;;
    build-jia) build jia; build_seam ;;
    build-pre) build pre ;;
    screen-192k9) env MODE=fx FAMILY=list "SHAPES=192:9:3:1;" "FX_PLANS=w:-/-:z;w:all/but1:z;w:but1/all:z;w:all/but1:e;w:but1/all:e" CUT_LIST=8 AFTER_J=0 ATTACK_THREADS=1 FX_PRINT=6 RUN_CAP=512M \
      bash "$S/run.sh" jia out/k2-192k9-n8-screen.log; grep -h '^FXPLAN\|^FXREFUSED\|^FXTALLY' out/k2-192k9-n8-screen.log | cut -c1-260 ;;
    fxw) bash "$MODEL/rerun.sh" tables "$ATTACK_DIR" > /dev/null; cat tables/fxw.all tables/fxwp.all > tables/fxall; batches 9 tables/fxall ;;
    pass-jia) grep '^jia ' tables/pass.all > tables/pass-jia; bash "$S/batch.sh" 60 tables/pass-jia ;;
    k2ms) P=$(MS_PLANS); : > tables/k2ms.all
      for s in 192:4:3:1 264:6:4:0 216:7:5:4 192:9:3:1 248:9:10:1; do echo "jia k2ms-n${s//:/_} MODE=fx FAMILY=list SHAPES=$s; FX_PLANS=$P AFTER_J=0,3 ATTACK_THREADS=1 FX_PRINT=4 RUN_CAP=512M" >> tables/k2ms.all; done
      for s in 384:3:1:2 256:5:1:2; do echo "jia k2ms-p${s//:/_} MODE=fx FAMILY=plainlist SHAPES=$s; FX_PLANS=$P AFTER_J=0,3 ATTACK_THREADS=1 FX_PRINT=4 RUN_CAP=512M" >> tables/k2ms.all; done
      for s in 192:4:3:1 264:6:4:0 216:7:5:4; do echo "jia k1pass3-n${s//:/_} MODE=fx FAMILY=list SHAPES=$s; FX_PLANS=pass AFTER_J=0 ATTACK_THREADS=1 FX_PRINT=4 RUN_CAP=512M" >> tables/k2ms.all; done
      for s in 384:3:1:2 256:5:1:2; do echo "jia k1pass3-p${s//:/_} MODE=fx FAMILY=plainlist SHAPES=$s; FX_PLANS=pass AFTER_J=0 ATTACK_THREADS=1 FX_PRINT=4 RUN_CAP=512M" >> tables/k2ms.all; done
      bash "$S/batch.sh" 70 tables/k2ms.all ;;
    seam) SEEDS=120 STEPS=30 nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" 170M bash "$REPO/research/scripts/capped.sh" 1 \
      "$ATTACK_DIR/target-jia/release/attack-b10-seam" > out/seam-s120.log 2>&1; tail -1 out/seam-s120.log ;;
    fin) sed 's/^jia fxw-/jia fin-/; s/$/ FX_FINALCHK=1 SCREEN=0/' tables/fxall > tables/finall; batches 8 tables/finall
      cat out/fin-*.log | grep '^FXFINAL' > out/fin-frozen.txt; grep -c 'I-7.9' out/fin-frozen.txt ;;
    fin-pre) env FX_FINALCHK=1 MODE=fx FAMILY=list "SHAPES=192:9:3:1;" "FX_PLANS=w:all/but1:z;w:-/only1:z;w:but1/only1:z" CUT_LIST=8 AFTER_J=0 SCREEN=0 ATTACK_THREADS=1 RUN_CAP=512M \
      bash "$S/run.sh" pre out/fin-pre-192k9-n8.log; grep '^FXFINAL' out/fin-pre-192k9-n8.log ;;
    k1j) P=""; for m in z e; do for f in only1/- but1/- all/- -/only1 -/but1 -/all only1/only1; do P="$P;w:$f:$m:p"; done; done; P=${P#;}; : > tables/k1j.all
      for s in 192:4:3:1 264:6:4:0 216:7:5:4 248:9:10:1 320:9:3:1; do echo "jia k1j-n${s//:/_} MODE=fx FAMILY=list SHAPES=$s; FX_PLANS=$P AFTER_J=1,3,6,13 FX_FINALCHK=1 SCREEN=0 ATTACK_THREADS=1 FX_PRINT=4 RUN_CAP=512M" >> tables/k1j.all; done
      for s in 384:3:1:2 256:5:1:2 240:5:1:2; do echo "jia k1j-p${s//:/_} MODE=fx FAMILY=plainlist SHAPES=$s; FX_PLANS=$P AFTER_J=1,3,6,13 FX_FINALCHK=1 SCREEN=0 ATTACK_THREADS=1 FX_PRINT=4 RUN_CAP=512M" >> tables/k1j.all; done
      bash "$S/batch.sh" 50 tables/k1j.all ;;
    phases) SEEDS=24 PER=3 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型跑崩溃注入库函数" nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" 4G bash "$REPO/research/scripts/capped.sh" 14 \
      "$ATTACK_DIR/target-jia/release/phases" > out/phases-s24.log 2>&1; grep '^PHASES' out/phases-s24.log ;;
    k3-pre) L0 pre k3-pre-l0-5 FAMILY=list "SHAPES=192:9:3:1;" "FX_PLANS=w:all/but1:z" CUT_LIST=8 "FX_RENUM=8@w:all/but1:z" ;;
    k3-frozen) L0 jia k3-frozen-320-n20-only1 FAMILY=list "SHAPES=320:9:3:1;" "FX_PLANS=w:-/only1:z" CUT_LIST=20 "FX_RENUM=20@w:-/only1:z" ;;
    summary) { for f in $(awk '{print $2}' tables/fxw.all); do echo "out/$f.log"; done | xargs bash "$S/fxsum2.sh" | sed 's/^/[fxw-narrow] /'
        for f in $(awk '{print $2}' tables/fxwp.all); do echo "out/$f.log"; done | xargs bash "$S/fxsum2.sh" | sed 's/^/[fxw-plain] /'
        for l in $(awk '{print $2}' tables/pass-jia tables/k2ms.all); do sum "out/$l.log" | sed "s/^/[$l] /"; done
        grep -h '^SEAMTOTAL' out/seam-s120.log; } > summary.txt; wc -l summary.txt ;;
    compare) diff "$HERE/out/summary.txt" summary.txt && echo "compare: 与 out/summary.txt 逐行相同（$(wc -l < summary.txt) 行）" ;;
    *) echo "不认得的阶段 $1"; exit 2 ;;
  esac
}
STAGES="arm-jia arm-pre build-jia build-pre screen-192k9 fxw pass-jia k2ms seam fin fin-pre k1j phases k3-pre summary"
case "$STAGE" in
  list) echo $STAGES ;;
  *) run_stage "$STAGE" ;;
esac
