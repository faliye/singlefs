#!/usr/bin/env bash
# m3-batch17-code-r1 攻方腿（Opus）复跑。用法：bash rerun.sh <阶段> <空工作目录 W>；`bash rerun.sh list W` 列出阶段，按列出的次序逐个跑
# （每个阶段前台 4 分钟以内，本机满载时个别阶段会到 3 分半；要一口气跑的由主 agent 放后台：`bash rerun.sh all W`）。
# 最后 `bash rerun.sh summary W` 把 W/out 汇总成判定行写到 W/summary.txt；`bash rerun.sh compare W` 与本目录 out/summary.txt 逐行比，一致才退 0。
# 只编跑 harness 档与自己的原型 crate（名字不带 layer0、不是 checker 档包的测试）；冻结树路径认 FROZEN_TREE。线程上限 15。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
export MODEL=${MODEL:-$REPO/research/prompts/m3-batch17-code-r1-opus-model}
export FROZEN_TREE=${FROZEN_TREE:-/tmp/claude-1000/m3-batch17-code-r1/tree}
STAGE=${1:?阶段名}
export ATTACK_DIR=${2:?给一个空的工作目录}
mkdir -p "$ATTACK_DIR/out" "$ATTACK_DIR/tables"
cd "$ATTACK_DIR"
S=$MODEL/scripts
TB=$ATTACK_DIR/tables
build_arm() { bash "$S/arm.sh" "$@"; ATTACK_THREADS=15 bash "$S/build.sh" "$1" > "build-$1.log" 2>&1; }
qf_arm() { local a=$1; shift; mkdir -p "$ATTACK_DIR/qf"; rsync -a --delete --exclude target --exclude .git "$FROZEN_TREE/" "$ATTACK_DIR/qf/$a/"
  for p in "$@"; do (cd "$ATTACK_DIR/qf/$a" && patch -s -p0 < "$MODEL/patches/$p"); done
  cp "$MODEL/probe/qf_attack_probe.rs" "$ATTACK_DIR/qf/$a/crates/singlefs-harness/tests/qf_attack_probe.rs"
  ATTACK_THREADS=15 bash "$S/qf.sh" "$ATTACK_DIR/qf/$a" "$ATTACK_DIR/out/qf-$a.log"; grep -h '^test result' "$ATTACK_DIR/out/qf-$a.log" || true; }
# 再挂期间读坏计划（MODE=fx 的 FX_PLANS 写法）：每块盘一种选槽法 × 读回全 0（z）/ 报读错（e），整次挂载（w）；p 是挂载交回之后还坏。
plans() { local m=$1 out=""; for f0 in - all but1 only1; do for f1 in - all but1 only1; do [ "$f0$f1" = "--" ] && continue; out="$out;w:$f0/$f1:$m"; done; done
  echo "$out;w:all/-:$m:p;w:-/all:$m:p"; }
half() { echo "$1" | tr ';' '\n' | grep . | if [ "$2" = 1 ]; then head -9; else tail -n +10; fi | paste -sd';'; }
NARROW_W="192:4:3:1 192:4:10:1 192:9:3:1 192:9:10:1 216:4:3:1 216:4:10:1 216:9:3:1 216:9:10:1 248:4:3:1 248:4:10:1 248:9:3:1 248:9:10:1 280:4:3:1 280:4:10:1 280:9:3:1 280:9:10:1 320:4:3:1 320:4:10:1 320:9:3:1 320:9:10:1"
PLAIN_W="384:3:1:2 384:5:3:2 256:3:1:2 256:5:3:2 240:3:1:2 240:5:1:2"
C1="ATTACK_THREADS=1 FX_PRINT=6 RUN_CAP=256M"
make_tables() {
  : > "$TB/fxw.all"
  for s in $NARROW_W; do for m in z e; do P=$(plans $m); for q in 1 2; do
    echo "jia fxw-n${s//:/_}-$m$q MODE=fx FAMILY=list SHAPES=$s; FX_PLANS=$(half "$P" $q) AFTER_J=0 $C1" >> "$TB/fxw.all"; done; done; done
  (cd "$TB" && rm -f fxw-?? && split -l 14 -d fxw.all fxw-)
  : > "$TB/fxwp.all"
  for s in $PLAIN_W; do for m in z e; do echo "jia fxw-p${s//:/_}-$m MODE=fx FAMILY=plainlist SHAPES=$s; FX_PLANS=$(plans $m) AFTER_J=0 $C1" >> "$TB/fxwp.all"; done; done
  : > "$TB/gacmp.all"
  for a in jia ga; do for s in 192:4:3:1 192:4:10:1 192:9:3:1 192:9:10:1 216:4:3:1 216:9:3:1 248:4:3:1; do
    echo "$a gacmp-$a-${s//:/_} MODE=fx FAMILY=list SHAPES=$s; FX_PLANS=w:-/all:z;w:all/-:z;w:-/all:e;w:all/-:e;w:but1/all:z;w:all/but1:z AFTER_J=0 $C1" >> "$TB/gacmp.all"; done; done
  : > "$TB/pass.all"
  PC="MODE=fx FX_PLANS=pass FX_PASS_FILTERS=allall AFTER_J=0 $C1"
  for s in 192:4:3:1 216:4:3:1 248:4:3:1 280:4:3:1 320:4:3:1 264:6:4:0 216:7:5:4; do echo "jia pass-jia-n${s//:/_} $PC FAMILY=list SHAPES=$s;" >> "$TB/pass.all"; done
  for s in 384:3:1:2 256:5:1:2; do echo "jia pass-jia-p${s//:/_} $PC FAMILY=plainlist SHAPES=$s;" >> "$TB/pass.all"; done
  for a in rr ga zs; do echo "$a pass-$a-n264_6_4_0 $PC FAMILY=list SHAPES=264:6:4:0;" >> "$TB/pass.all"; done
  : > "$TB/cell.all"
  CC="MODE=fx FX_PLANS=P5:all/all:e;P5:all/all:z AFTER_J=0 ATTACK_THREADS=1 FX_PRINT=4 RUN_CAP=256M"
  for a in jia rr ga; do for u in 192 216 248 280 320; do for k in 4 7; do
    echo "$a cell-$a-n$u-k$k $CC FAMILY=narrow SLOTS=$u OVERWRITES=$k UNITS=3,5,10 BATCHES=1" >> "$TB/cell.all"; done; done
    echo "$a cell-$a-plain $CC FAMILY=plainlist SHAPES=384:3:1:2;384:5:3:2;256:3:1:2;256:5:3:2;240:3:1:2;240:5:1:2;" >> "$TB/cell.all"; done
  (cd "$TB" && rm -f cell-?? && split -l 11 -d cell.all cell-)
  : > "$TB/narrower.all"
  for u in 140 148 156 164 172 180; do echo "jia narrower-n$u MODE=fx FAMILY=narrow SLOTS=$u OVERWRITES=1,2 UNITS=2,3 BATCHES=1 FX_PLANS=w:-/all:z;w:all/-:z;w:-/all:e;w:all/-:e AFTER_J=0 $C1" >> "$TB/narrower.all"; done
  for u in 140 156 172; do echo "ga narrower-ga-n$u MODE=fx FAMILY=narrow SLOTS=$u OVERWRITES=1,2 UNITS=2,3 BATCHES=1 FX_PLANS=w:-/all:z;w:all/-:z;w:-/all:e;w:all/-:e AFTER_J=0 $C1" >> "$TB/narrower.all"; done
  : > "$TB/hold.all"
  for s in 4:3:1:2 4:5:3:2 384:6:1:2 384:3:3:2 256:6:3:2 256:3:1:2 240:3:3:2 240:5:1:2; do echo "jia hold-${s//:/_} MODE=fx FAMILY=plainlist SHAPES=$s; FX_PLANS=clean AFTER_J=1,3 ATTACK_THREADS=1 FX_PRINT=6 RUN_CAP=512M" >> "$TB/hold.all"; done
  : > "$TB/twice.all"
  TC="MODE=r2 TWICE=1 CUT_LIST=1,15,17,30 RCUT_PRINT=3 ATTACK_THREADS=1 RUN_CAP=512M"
  for s in 192:4:3:1 248:4:3:1 216:7:5:4 280:4:3:1 320:4:3:1; do echo "jia tw-${s//:/_} $TC FAMILY=list SHAPES=$s;" >> "$TB/twice.all"; done
  for s in 384:3:1:2 256:5:1:2 240:5:1:2; do echo "jia tw-p${s//:/_} $TC FAMILY=plainlist SHAPES=$s;" >> "$TB/twice.all"; done
  for s in 248:4:3:1 280:4:3:1 320:4:3:1; do echo "jia tw-h1-${s//:/_} $TC HIDE_DEV=1 FAMILY=list SHAPES=$s;" >> "$TB/twice.all"; done
  for s in 384:3:1:2 256:5:1:2; do echo "jia tw-h1-p${s//:/_} $TC HIDE_DEV=1 FAMILY=plainlist SHAPES=$s;" >> "$TB/twice.all"; done
  : > "$TB/ab2.all"
  for a in jia nf; do for w in 4 384 256 240; do echo "$a ab-$a-$w MODE=ab2 WIDTHS=$w OVERWRITES=2,5,8 ATTACK_THREADS=1 RUN_CAP=512M" >> "$TB/ab2.all"; done; done
}
# 被限时掐断（本机满载时）的那几件：日志里没有 FXDONE / START_FAILED / R2DONE / AB2TALLY 的，照原样再跑一遍（结果确定，重跑同一件同一个输出）。
redo() { local table=$1 batch=$2 out=$TB/redo-$(basename "$1" .all).all; : > "$out"
  while read -r arm log rest; do f=$ATTACK_DIR/out/$log.log
    if ! grep -q '^FXDONE\|^START_FAILED\|^R2DONE\|^AB2TALLY' "$f" 2>/dev/null; then echo "$arm $log $rest" >> "$out"; fi; done < "$table"
  echo "要重跑的件：$(wc -l < "$out")"; [ -s "$out" ] && bash "$S/batch.sh" "$batch" "$out" || true; }
L0() { local arm=$1 log=$2; shift 2; env "$@" MODE=fx SCREEN=1 ATTACK_THREADS=14 RUN_CAP=6G bash "$S/run.sh" "$arm" "out/$log.log"; grep -h '^FXENUM ' "out/$log.log" | sed 's/ elapsed=.*//' || true; }
run_stage() {
  case "$1" in
    build-jia) build_arm jia ;;
    build-ga|build-rr|build-zs|build-nf|build-diag) a=${1#build-}; build_arm "$a" "$a.patch" ;;
    tables) make_tables; wc -l "$TB"/*.all ;;
    diag) env MODE=fx FAMILY=list "SHAPES=264:6:4:0;" "FX_PLANS=w:-/-:z;P5:all/all:e" CUT_LIST=15 FX_DIAG=1 SCREEN=0 ATTACK_THREADS=1 RUN_CAP=256M bash "$S/run.sh" diag out/diag.log ;;
    fxw-0[0-5]) bash "$S/batch.sh" "1${1#fxw-}" "$TB/$1" ;;
    fxw-plain) bash "$S/batch.sh" 20 "$TB/fxwp.all" ;;
    redo-fxw) redo "$TB/fxw.all" 19 ;;
    redo-cell) redo "$TB/cell.all" 39 ;;
    gacmp) bash "$S/batch.sh" 21 "$TB/gacmp.all" ;;
    pass) bash "$S/batch.sh" 22 "$TB/pass.all" ;;
    cell-0[0-2]) bash "$S/batch.sh" "3${1#cell-}" "$TB/$1" ;;
    hold) bash "$S/batch.sh" 40 "$TB/hold.all" ;;
    narrower) make_tables > /dev/null; bash "$S/batch.sh" 43 "$TB/narrower.all" ;;
    twice) bash "$S/batch.sh" 41 "$TB/twice.all" ;;
    ab2) bash "$S/batch.sh" 42 "$TB/ab2.all" ;;
    l0-1) L0 jia l0-jia-264-n15-rebuildpass-e FAMILY=list "SHAPES=264:6:4:0;" "FX_PLANS=P5:all/all:e" CUT_LIST=15 "FX_RENUM=15@r120-144:all/all:e" ;;
    l0-2) L0 jia l0-jia-264-n15-but1all-z FAMILY=list "SHAPES=264:6:4:0;" "FX_PLANS=w:but1/all:z" CUT_LIST=15 "FX_RENUM=15@w:but1/all:z" ;;
    l0-3) L0 zs l0-zs-264-n15-but1all-z FAMILY=list "SHAPES=264:6:4:0;" "FX_PLANS=w:but1/all:z" CUT_LIST=15 "FX_RENUM=15@w:but1/all:z" ;;
    l0-4) L0 jia l0-jia-192-n17-rebuildpass-e FAMILY=list "SHAPES=192:4:3:1;" "FX_PLANS=P5:all/all:e" CUT_LIST=17 "FX_RENUM=17@r120-144:all/all:e" ;;
    l0-5) L0 jia l0-jia-192k9-n8-allbut1-z FAMILY=list "SHAPES=192:9:3:1;" "FX_PLANS=w:all/but1:z" CUT_LIST=8 "FX_RENUM=8@w:all/but1:z" ;;
    l0-6) L0 jia l0-jia-192k9-n8-but1all-z FAMILY=list "SHAPES=192:9:3:1;" "FX_PLANS=w:but1/all:z" CUT_LIST=8 "FX_RENUM=8@w:but1/all:z" ;;
    l0-7) L0 jia l0-jia-248-n17-clean FAMILY=list "SHAPES=248:4:3:1;" FX_PLANS=clean CUT_LIST=17 "FX_RENUM=17@w:-/-:z" ;;
    l0-8) L0 jia l0-jia-p256-n10-clean FAMILY=plainlist "SHAPES=256:5:1:2;" FX_PLANS=clean CUT_LIST=10 "FX_RENUM=10@w:-/-:z" ;;
    rr-cell) env MODE=fx FAMILY=list "SHAPES=192:4:3:1;264:6:4:0;" "FX_PLANS=P5:all/all:e;P5:all/all:z" "CUT_LIST=15,17" ATTACK_THREADS=1 RUN_CAP=256M bash "$S/run.sh" rr out/rr-rebuildpass.log; grep -h '^FXREFUSED' out/rr-rebuildpass.log | sed 's/.*writes_before_err=/writes_before_err=/' | sort | uniq -c ;;
    qf-qfj) qf_arm qfj ;;
    qf-qfm|qf-qfu|qf-qfr) a=${1#qf-}; qf_arm "$a" "$a.patch" ;;
    summary) bash "$S/summary.sh" "$ATTACK_DIR" > "$ATTACK_DIR/summary.txt"; wc -l "$ATTACK_DIR/summary.txt" ;;
    compare) bash "$S/summary.sh" "$ATTACK_DIR" > "$ATTACK_DIR/summary.txt"
      if diff "$MODEL/out/summary.txt" "$ATTACK_DIR/summary.txt" > "$ATTACK_DIR/summary.diff"; then echo "compare: 判定行与 out/summary.txt 逐行相同（$(wc -l < "$ATTACK_DIR/summary.txt") 行）"; else echo "compare: 有不同，见 $ATTACK_DIR/summary.diff"; exit 1; fi ;;
    *) echo "不认得的阶段 $1"; exit 2 ;;
  esac
}
STAGES="build-jia build-ga build-rr build-zs build-nf build-diag tables diag fxw-00 fxw-01 fxw-02 fxw-03 fxw-04 fxw-05 redo-fxw fxw-plain gacmp pass cell-00 cell-01 cell-02 redo-cell
hold narrower twice ab2 rr-cell l0-1 l0-2 l0-3 l0-4 l0-5 l0-6 l0-7 l0-8 qf-qfj qf-qfm qf-qfu qf-qfr summary"
case "$STAGE" in
  list) echo $STAGES ;;
  all) for s in $STAGES; do echo "== $s"; run_stage "$s"; done ;;
  *) run_stage "$STAGE" ;;
esac
