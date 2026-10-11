#!/bin/bash
# m3-batch10o-code-r1 攻方（opus）原型复跑。原型从 m3-batch10i-code-r1 攻方原型（research/prompts/m3-batch10i-code-r1-opus-model/main.rs）改来，
# agg.py 原样拷自那一份；port.py 在冻结副本的拷贝上加计数与 T2「回退不拒」臂（OPUS_T2_ARM=push），opus-o1-arms.patch 是它打出来的 diff。
# 用法：bash rerun.sh setup <空的草稿目录> [冻结副本根]   拷冻结副本、加计数与臂、放原型与两份 harness 用例、release 编原型
#       bash rerun.sh k1 <草稿目录>     K1 那几组（日志进 <草稿目录>/out/）
#       bash rerun.sh t2 <草稿目录>     T2 两臂那几组
#       bash rerun.sh hole <草稿目录>   带洞的环（K1-d）
#       bash rerun.sh k4 <草稿目录>     跨进程显存账五条（harness 档）
#       bash rerun.sh hb <草稿目录>     H-b 镜像上 I-5.1 的原文（harness 档）
#       bash rerun.sh summary <草稿目录>  每份日志一行汇总（summary.py）
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=/home/fy5090/code/singlefs
CAP="bash $REPO/scripts/run-with-memory-cap.sh"
THR="bash $REPO/scripts/capped.sh 15"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="复跑攻方原型，与别的会话并行" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型各子命令起点不同"
case ${1:-} in
  setup)
    W=${2:?草稿目录}; FROZEN=${3:-$HOME/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2}
    mkdir -p "$W"; [ -z "$(ls -A "$W")" ] || { echo "草稿目录不空：$W" >&2; exit 2; }
    mkdir -p "$W/tree" "$W/out"
    rsync -a --exclude target --exclude .git "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/crates" "$W/tree/"
    python3 "$HERE/port.py" "$W/tree"
    python3 - "$W/tree/Cargo.toml" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read()
s=s.replace('    "crates/singlefs-reference-filesystem",\n','    "crates/singlefs-reference-filesystem",\n    "crates/opus-o1-attack",\n',1); open(p,'w').write(s)
PY
    mkdir -p "$W/tree/crates/opus-o1-attack/src"
    cp "$HERE/Cargo.toml" "$W/tree/crates/opus-o1-attack/Cargo.toml"; cp "$HERE/main.rs" "$W/tree/crates/opus-o1-attack/src/main.rs"
    cp "$HERE/opus_o1_gpu_memory_ledger_attack.rs" "$W/tree/crates/singlefs-harness/tests/"
    { echo; cat "$HERE/hb-test-append.rs"; } >> "$W/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs"
    cp "$HERE/run-stage.sh" "$HERE/summary.py" "$HERE/k2sum.sh" "$HERE/agg.py" "$W/"
    (cd "$W/tree" && nice -n 19 $THR cargo build --offline --release -p opus-o1-attack > "$W/out/build.log" 2>&1)
    echo "setup 完：$W";;
  k1)
    W=${2:?}; cd "$W"
    for s in scan1-prod-u8-s6 scan1-prod-u8-s9 scan1-prod-u8-s12 scan1-prod-u8-s15 live-prod-ab384 live-prod-mat-384-30-8 live-prod-narrow-264-6-4 live-prod-narrow-384-20-2 base-prod; do bash run-stage.sh "$W" $s; done
    for sp in ab-384-1-0 ab-384-3-0 ab-384-3-5; do SPS=$sp bash run-stage.sh "$W" ab384-owu; done;;
  t2)
    W=${2:?}; cd "$W"
    for s in k2-refuse-s3 k2-push-s3 k2-refuse-s9 k2-push-s9; do bash run-stage.sh "$W" $s; done
    for arm in refuse push; do SPS=ab-384-3-5 bash run-stage.sh "$W" k2-$arm-s15; SPS=mat-384-30-8 bash run-stage.sh "$W" k2-$arm-s15; PICK=1 SPS=ab-384-3-5 bash run-stage.sh "$W" k2-$arm-s9; done
    for sp in ab-384-1-0 ab-384-3-0; do for arm in refuse push; do for st in 3 15; do S1CLASS=A SPS=$sp bash run-stage.sh "$W" k2-$arm-s$st; done; done; done
    export OPUS_AFTER_ROLLBACK_STEPS=6 OPUS_AFTER_ROLLBACK_UNMOUNT=1
    for cls in A V; do for arm in refuse push; do S1CLASS=$cls SPS=ab-384-3-5 bash run-stage.sh "$W" k2-$arm-s9; mv out/k2-$arm-s9-c$cls-ab-384-3-5.log out/k2after-$arm-s9-c$cls-ab-384-3-5.log; done; done
    S1CLASS=V SPS=mat-384-30-8 bash run-stage.sh "$W" k2-push-s9; mv out/k2-push-s9-cV-mat-384-30-8.log out/k2after-push-s9-cV-mat-384-30-8.log;;
  hole)
    W=${2:?}; cd "$W"
    env OPUS_STEPS=20 OPUS_WORKERS=14 nice -n 19 $CAP 3G $THR "$W/tree/target/release/opus-o1-attack" s1grid hole-1-20 > out/hole-1-20.log
    for sp in hole-1-20 hole-2-20 young-hole; do env OPUS_STEPS=20 OPUS_FORCE_CEILING_RAISE_EVERY=2 OPUS_WORKERS=14 nice -n 19 $CAP 3G $THR "$W/tree/target/release/opus-o1-attack" s1grid $sp > out/holeF-$sp.log; done;;
  k4)
    W=${2:?}; cd "$W/tree" && nice -n 19 $CAP 2G $THR cargo test --offline -p singlefs-harness --test opus_o1_gpu_memory_ledger_attack -- --nocapture --test-threads=1 > "$W/out/k4-run.log" 2>&1; grep -E 'OPUSO1-K4|test result' "$W/out/k4-run.log";;
  hb)
    W=${2:?}; cd "$W/tree" && nice -n 19 $CAP 4G $THR cargo test --offline -p singlefs-harness --test checker_known_bad_images opus_o1_print_the_i51_text_on_the_h_b_image -- --nocapture > "$W/out/hb.log" 2>&1; grep -E 'OPUSO1-HB|test result' "$W/out/hb.log";;
  summary)
    W=${2:?}; cd "$W/out" && python3 "$W/summary.py" *.log 2>/dev/null | grep SUMMARY;;
  *) echo "用法见文件头" >&2; exit 2;;
esac
