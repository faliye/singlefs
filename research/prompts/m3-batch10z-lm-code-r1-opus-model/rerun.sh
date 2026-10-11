#!/usr/bin/env bash
# m3-batch10z-lm-code-r1 攻方复跑。工作目录仓根；WORK 给一个空的草稿目录（不在仓里）。线程上限 15，每件内存 4G（编译 16G）。
#   WORK=~/.local/share/singlefs/work/<自己的目录> bash research/prompts/m3-batch10z-lm-code-r1-opus-model/rerun.sh
# 做的事：冻结副本 rsync 到 $WORK/copy，打 copy-vs-frozen.diff（mount.rs 两个只看 OPUS_LM_ARM 的臂 narrow / nodbl、allocator.rs 两个只读探针、
# gpu_memory_ledger.rs 测试模块里两条攻方用例、新探针 opus_lm_probe.rs），release 编，按批并行跑，最后汇总、与 runs-summary/ 逐份比。
set -euo pipefail
ROOT=$(pwd)
MD=$ROOT/research/prompts/m3-batch10z-lm-code-r1-opus-model
FROZEN=${FROZEN:-$HOME/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree}
WORK=${WORK:?给一个草稿目录}
CAP="$ROOT/scripts/run-with-memory-cap.sh"
CAPPED="$ROOT/scripts/capped.sh"
mkdir -p "$WORK/runs" "$WORK/summary" "$WORK/scan"
rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/copy/"
mkdir -p "$WORK/copy/.cargo" && cp "$ROOT/.cargo/config.toml" "$WORK/copy/.cargo/config.toml"
patch -d "$WORK/copy" -p1 < "$MD/copy-vs-frozen.diff"
need() { local n; n=$(grep -c -- "$2" "$WORK/copy/$1" || true); [[ "$n" -ge 1 ]] || { echo "补丁没打上：$1 里没有 $2"; exit 3; }; }
need crates/singlefs-core/src/mount.rs OPUS_LM_ARM
need crates/singlefs-core/src/allocator.rs probe_isolated_and_occupied
need crates/singlefs-harness/src/gpu_memory_ledger.rs opus_lm_random_ledger_model
need crates/singlefs-harness/tests/opus_lm_probe.rs S2_TRACE
cd "$WORK/copy"
CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$CAPPED" 15 cargo build --offline --release -p singlefs-harness --test opus_lm_probe
CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$CAPPED" 15 cargo test --offline --release -p singlefs-harness --lib --no-run
cd "$WORK"
B=$(ls -t "$WORK"/target/release/deps/opus_lm_probe-* | grep -v '\.d$' | head -1)
L=$(ls -t "$WORK"/target/release/deps/singlefs_harness-* | grep -v '\.d$' | head -1)
COMMON="S2_ALPHABET=onsFb S2_SEED=31 S2_SHORTEST=20 S2_LONGEST=34 S2_PREROOT=1"
F4="S2_FAULTS=racct;racct,rlin;rall;racct,oldroot"
P24=oooooooooooooooooooooooo
# 批 b3：200 / 384 槽，tri 与 nodbl，R = b1 b3 n（12 件）
rm -f runs/b3-*.rc; i=0
for sl in 200 384; do for arm in tri nodbl; do for r in b1 b3 n; do i=$((i+1))
  { env OPUS_LM_ARM=$arm S2_SLOTS=$sl S2_PREFIXES=oooo S2_RS=$r "$F4" $COMMON S2_COUNT=4 S2_LABEL=b3 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 1 "$B" --nocapture s2_sweep > runs/b3-$i.log 2>&1; echo "$?" > runs/b3-$i.rc; } &
done; done; done; wait
[[ $(ls runs/b3-*.rc | wc -l) -eq 12 ]] || { echo "b3 件数对不上"; exit 4; }
# 批 b4：回退目标深（前缀 24 次覆写），768 / 1536 槽（8 件）
rm -f runs/b4-*.rc; i=0
for sl in 1536 768; do for arm in tri nodbl; do for r in B23 B24; do i=$((i+1))
  { env OPUS_LM_ARM=$arm S2_SLOTS=$sl S2_PREFIXES=$P24 S2_RS=$r "$F4" $COMMON S2_COUNT=4 S2_LABEL=b4 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 1 "$B" --nocapture s2_sweep > runs/b4-$i.log 2>&1; echo "$?" > runs/b4-$i.rc; } &
done; done; done; wait
[[ $(ls runs/b4-*.rc | wc -l) -eq 8 ]] || { echo "b4 件数对不上"; exit 4; }
# 批 b5：R 的种类 × 打包单元故障（14 件）
rm -f runs/b5-*.rc; i=0
F7="S2_FAULTS=racct;racct,rnonlin;racct,rpk0;racct,rpk1;racct,rpk2;racct,rlin;rall"
for arm in tri nodbl; do for spec in "oooo|b1" "oooo|b3" "oooo|n" "oooo|s" "oooo|o" "oooon|d"; do IFS='|' read -r pre r <<<"$spec"; i=$((i+1))
  { env OPUS_LM_ARM=$arm S2_SLOTS=384 S2_PREFIXES=$pre S2_RS=$r "$F7" S2_ALPHABET=onsFb S2_SEED=37 S2_SHORTEST=20 S2_LONGEST=34 S2_PREROOT=1 S2_COUNT=2 S2_LABEL=b5 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 1 "$B" --nocapture s2_sweep > runs/b5-$i.log 2>&1; echo "$?" > runs/b5-$i.rc; } &
done; done
for spec in "oooo|b1" "oooon|d"; do IFS='|' read -r pre r <<<"$spec"; i=$((i+1))
  { env OPUS_LM_ARM=tri S2_SLOTS=1536 S2_PREFIXES=$pre S2_RS=$r "$F7" S2_ALPHABET=onsFb S2_SEED=37 S2_SHORTEST=20 S2_LONGEST=34 S2_PREROOT=1 S2_COUNT=2 S2_LABEL=b5 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 1 "$B" --nocapture s2_sweep > runs/b5-$i.log 2>&1; echo "$?" > runs/b5-$i.rc; } &
done; wait
[[ $(ls runs/b5-*.rc | wc -l) -eq 14 ]] || { echo "b5 件数对不上"; exit 4; }
# 批 b9 / b10：释放过一大批之后的宽护与卡死（9 件）
rm -f runs/b9-*.rc runs/b10-*.rc; i=0
for arm in tri narrow nodbl; do for spec in "768|SSSSSSSSSSSSb6" "1536|SSSSSSSSSSSSSSSSSSSSSSb8"; do IFS='|' read -r sl pre <<<"$spec"; i=$((i+1))
  { env S2_TRACE=1 OPUS_LM_ARM=$arm S2_SLOTS=$sl S2_PREFIXES=$pre S2_RS=n "S2_FAULTS=rall;racct" S2_ALPHABET=osF S2_COUNT=1 S2_SEED=47 S2_SHORTEST=50 S2_LONGEST=50 S2_PREROOT=1 S2_LABEL=b9 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 1 "$B" --nocapture s2_sweep > runs/b9-$i.log 2>&1; echo "$?" > runs/b9-$i.rc; } &
done; done
j=0; for arm in tri narrow nodbl; do j=$((j+1))
  { env OPUS_LM_ARM=$arm S2_SLOTS=1536 S2_PREFIXES=SSSSSSSSSSSSSSSSSSSSSSb8 "S2_RS=B3;B10" "S2_FAULTS=racct;racct,rlin" S2_ALPHABET=osF S2_COUNT=1 S2_SEED=47 S2_SHORTEST=50 S2_LONGEST=50 S2_PREROOT=1 S2_LABEL=b10 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 1 "$B" --nocapture s2_sweep > runs/b10-$j.log 2>&1; echo "$?" > runs/b10-$j.rc; } &
done; wait
[[ $(ls runs/b9-*.rc runs/b10-*.rc | wc -l) -eq 9 ]] || { echo "b9/b10 件数对不上"; exit 4; }
# 显存账：随机模型与并发压测
env OPUS_LM_SEEDS=5000 OPUS_LM_STEPS=400 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 1 "$L" --nocapture --exact gpu_memory_ledger::tests::opus_lm_random_ledger_model > runs/ledger-model.log 2>&1
env OPUS_LM_THREADS=14 OPUS_LM_ROUNDS=5000 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 14 "$L" --nocapture --exact gpu_memory_ledger::tests::opus_lm_early_lost_mark_racing_the_restart > runs/ledger-stress.log 2>&1
# 扫源码用例的逐字搬运小程序：原样、变异表那一行、两种骗过的写法
nice -n 19 rustc --edition 2021 -O -o scan/scan_probe "$MD/src/scan_probe.rs"
python3 "$MD/src/variants.py" "$FROZEN/crates/singlefs-checker-tier/src" scan/variants
for f in scan/variants/*.rs; do n=$(basename "$f" .rs); nice -n 19 bash "$CAP" 1G scan/scan_probe "${n%%-*}" "$f" || true; done > summary/scan-variants.raw
# 汇总、对照
python3 "$MD/src/pair.py" runs/b3-*.log runs/b4-*.log > summary/pair-tri-nodbl-b3-b4.txt
python3 "$MD/src/agg.py" runs/b5-*.log > summary/agg-r-kinds-b5.txt
grep -h 'OPUS_LM_LEDGER\|OPUS_LM_UNDER' runs/ledger-model.log > summary/ledger-random-model.txt
grep -h 'OPUS_LM_STRESS' runs/ledger-stress.log > summary/ledger-early-mark-stress.txt
for s in pair-tri-nodbl-b3-b4 agg-r-kinds-b5 ledger-random-model; do
  if diff -q "summary/$s.txt" "$MD/runs-summary/$s.txt" > /dev/null; then echo "同：$s"; else echo "不同：$s"; fi
done
grep -h '^S2 ' runs/b9-*.log runs/b10-*.log | grep -o 'arm=[^ ]*\|slots=[^ ]*\| r=[^ ]*\|faults=[^ ]*\|iso_mount=[^ ]*\|refused_steps=[^ ]*\|r_left_at=[^ ]*\|iso_end=[^ ]*\|hit=[^ ]*' | paste -d' ' - - - - - - - - -
cat summary/ledger-early-mark-stress.txt summary/scan-variants.raw
# 批 b11：同一形挂载那一刻的分配器计数（已分配、空闲、三种挡位都没置的、隔离、两项都扣的），三臂各一段
rm -f runs/b11-*.rc; i=0
for arm in tri nodbl narrow; do i=$((i+1))
  { env OPUS_LM_ARM=$arm S2_SLOTS=1536 S2_PREFIXES=SSSSSSSSSSSSSSSSSSSSSSb8 S2_RS=n S2_FAULTS=rall S2_ALPHABET=osF S2_COUNT=1 S2_SEED=47 S2_SHORTEST=3 S2_LONGEST=3 S2_LABEL=b11 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 1 "$B" --nocapture s2_sweep > runs/b11-$i.log 2>&1; echo "$?" > runs/b11-$i.rc; } &
done; wait
[[ $(ls runs/b11-*.rc | wc -l) -eq 3 ]] || { echo "b11 件数对不上"; exit 4; }
grep -h 'S2 label' runs/b11-*.log | sed 's/.*S2 label/S2 label/' | tr ' ' '\n' | grep -E '^(arm|alloc_mount|free_mount|unblocked_mount|dbl_mount|iso_mount|refused_steps)=' | paste -d' ' - - - - - - -
