#!/usr/bin/env bash
# m3-formal-code-b26-r1 攻方腿的复跑。在主仓根跑：
#   REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b26-r1/tree WORK=<草稿目录> \
#     bash research/prompts/m3-formal-code-b26-r1-opus-model/tools/rerun.sh [段 …]
# 段（不给就全跑，次序照下）：build f3 frac seglen quick r1 r3 r4a r4b r4c r5 r6 r7 r8 r9 mut mutb compare
# 单段挂钟（攻方那一趟，7 线程）：r7 约 231 秒、r8 约 200 秒、r4a 约 188 秒、r4c 约 160 秒、r4b 约 139 秒、r1/r5/r6 约 125 秒，
# 其余几秒到几十秒；r7、r8 贴着前台 240 秒的线，主 agent 复跑时放后台起。
# 每段经 research/scripts/run-with-memory-cap.sh（探针 8G，变异副本上的编译与测试 4G）与 capped.sh 7；只在 WORK 下写。
set -euo pipefail
: "${REPO:?要主仓根}" "${FROZEN:?要冻结副本的 tree 目录}" "${WORK:?要草稿目录}"
MODEL="$REPO/research/prompts/m3-formal-code-b26-r1-opus-model"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-复跑攻方腿的探针与变异副本}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-探针按参数跑的量不同，峰值表记的是别的参数那一次}"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-600}"
mkdir -p "$WORK/out"
BIN="$WORK/target/release/opus-b26-probe"
CAP=(nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh")
THREADS=(bash "$REPO/research/scripts/capped.sh" 7)

probe() { # probe <输出名> <探针参数…>：跑探针，去掉进度行与包装的提示行落进 WORK/out/<输出名>
  local name="$1"; shift
  (cd "$REPO" && "${CAP[@]}" 8G "${THREADS[@]}" "$BIN" "$@") 2>&1 \
    | grep -vE '^LAYER0_PROGRESS|^run-with-memory-cap' > "$WORK/out/$name" || true
}

step_build() {
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  rsync -a "$MODEL/probe/" "$WORK/probe/"
  cp "$WORK/tree/Cargo.lock" "$WORK/probe/Cargo.lock"
  (cd "$WORK/probe" && CARGO_TARGET_DIR="$WORK/target" "${CAP[@]}" 8G "${THREADS[@]}" cargo build --offline --release)
}
step_f3() { probe f3.txt f3; }
step_frac() {
  probe frac-384.txt frac newest-system-configuration-slot-zeros unit-area-384 24 0 80
  probe frac-256.txt frac newest-system-configuration-slot-zeros unit-area-256 24 0 80
  cat "$WORK/out/frac-384.txt" "$WORK/out/frac-256.txt" | grep '^FRAC' > "$WORK/out/frac.txt"
}
step_seglen() {
  : > "$WORK/out/seglen.txt"
  for weights in several-leaves broad orphans rollback; do
    for width in unit-area-384 4gib; do
      probe seglen-one.txt seglen "$weights" "$width" 24 0 40
      grep '^SEGLEN' "$WORK/out/seglen-one.txt" >> "$WORK/out/seglen.txt"
    done
  done
}
step_quick() {
  probe quick-4gib.txt quick broad 4gib
  python3 "$MODEL/tools/cmp_quick.py" "$WORK/out/quick-4gib.txt" > "$WORK/out/quick-4gib-cmp.txt"
}
step_r1() { probe r1-file-zeros.txt custom file unit-area-384 0 zeros 1000000 1 16; }
step_r3() {
  for ops in mount,zeros mount,pwu,zeros,pwu mount,pwu,mount,pwu mount,pwu,mount,pwu,mount; do
    probe "r3-mkfs-$(echo "$ops" | tr ',' '_').txt" custom mkfs unit-area-384 0 "$ops" 1000000 1 64
  done
}
step_r4a() { probe r4-file-p1-ow_s5_rf_1_mount.txt custom file unit-area-384 1 ow:s5,rf:1,mount 1000000 1 16; }
step_r4b() { probe r4-file-p1-unlink_mount.txt custom file unit-area-384 1 unlink,mount 1000000 1 16; }
step_r4c() { probe r4-file-p2-ow_s5_ow_s6_rbb_0_mount.txt custom file unit-area-384 2 ow:s5,ow:s6,rbb:0,mount 1000000 1 16; }
step_r5() { probe r5-file-p0-zeros_zeros.txt custom file unit-area-384 0 zeros,zeros 1000000 3 16; }
step_r6() { probe r6-file-p0-mount_mount.txt custom file unit-area-384 0 mount,mount 1000000 3 16; }
step_r7() { probe r7-file-p2-rb1-mount.txt custom file unit-area-384 2 ow:s5,ow:s6,rb:1,mount 1000000 10 16; }
step_r8() { probe r8-file-p1-unlink-inode-mount.txt custom file unit-area-384 1 unlink,inode,mount 1000000 25 16; }
step_r9() { probe r9-unmount-ow1.txt unmount broad unit-area-384 1 1000000 1 16; }
step_mut() {
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree-mut/"
  : > "$WORK/out/mutations.txt"
  python3 "$MODEL/tools/mutate_rows.py" "$WORK/tree-mut" "$WORK/target-mut" "$MODEL/tools/rows-replaced-and-mutA.tsv" "$WORK/out/mutations.txt"
  python3 "$MODEL/tools/mutate_rows.py" "$WORK/tree-mut" "$WORK/target-mut" "$MODEL/tools/rows-c664-judge.tsv" "$WORK/out/mutations.txt"
}
step_mutb() {
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree-mutb/"
  python3 "$MODEL/tools/apply_mut_b.py" "$WORK/tree-mutb"
  mkdir -p "$WORK/probe-mutb"
  rsync -a "$MODEL/probe/src/" "$WORK/probe-mutb/src/"
  sed -e 's#\.\./tree/crates#../tree-mutb/crates#' -e 's/name = "opus-b26-probe"/name = "opus-b26-probe-mutb"/' \
    "$MODEL/probe/Cargo.toml" > "$WORK/probe-mutb/Cargo.toml"
  cp "$WORK/tree/Cargo.lock" "$WORK/probe-mutb/Cargo.lock"
  (cd "$WORK/probe-mutb" && CARGO_TARGET_DIR="$WORK/target-mutb" "${CAP[@]}" 4G "${THREADS[@]}" cargo build --offline --release)
  (cd "$REPO" && "${CAP[@]}" 4G "${THREADS[@]}" "$WORK/target-mutb/release/opus-b26-probe-mutb" \
    custom mkfs unit-area-384 0 mount,pwu,mount,pwu 1000000 1 16) 2>&1 \
    | grep -vE '^LAYER0_PROGRESS|^run-with-memory-cap' > "$WORK/out/mutB1-mkfs-mount-pwu-mount-pwu.txt" || true
}
step_compare() { # 去掉耗时那几格与 S2EX 例子行之后逐份比
  local strip='s/elapsed_secs=[0-9.]*//; s/elapsed_seconds=[0-9.]*//; s/finished in [0-9.]*s//'
  for file in "$MODEL"/out/*.txt; do
    local name; name=$(basename "$file")
    [ -f "$WORK/out/$name" ] || { echo "MISSING $name"; continue; }
    # S2EX 例子行是各线程先到先记的前三个，次序随线程变，不比；计数行（S2 …=n）照比。
    if diff -q <(grep -v '^S2EX' "$file" | sed -E "$strip") <(grep -v '^S2EX' "$WORK/out/$name" | sed -E "$strip") > /dev/null; then
      echo "SAME $name"
    else
      echo "DIFF $name"
    fi
  done
}

steps=("$@")
[ ${#steps[@]} -gt 0 ] || steps=(build f3 frac seglen quick r1 r3 r4a r4b r4c r5 r6 r7 r8 r9 mut mutb compare)
for step in "${steps[@]}"; do
  echo "== $step"
  "step_$step"
done
