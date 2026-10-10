#!/usr/bin/env bash
# 第 36 批代码轮第一轮攻方原型的复跑（主 agent 在自己的草稿目录跑，不在模型目录里跑）。
# 用法：bash tools/rerun.sh <草稿目录> [段…]；段取 setup build crafted crafted2 l57 l66 l14 l12 l36a l36b l36c r1 r2 r3 naga mut-a mut-b，不给 = 全部。
# 冻结副本默认 ~/.local/share/singlefs/work/m3-formal-code-b36-r1（B36_FROZEN 可换），setup 先按它的清单核 sha256。
# 每段的输出落 <草稿目录>/out/<段>.log（与模型 out/ 里留的同名文件比）；判结果看 B36X / B36XT / B36XT2 / B36L0-DONE / B36L0T / B36R / B36G / B36M 行。
# B36R-DONE、B36L0-DONE 的 seconds= 与 B36H 行的 seconds= 是挂钟，不比；B36R / B36L0T 的计数行逐字比；FIRST 行里的原文是各线程最先遇到的那一张，线程次序会换，只比键。
# 每条 cargo 与每次跑编出来的二进制都经 research/scripts/run-with-memory-cap.sh；同一时刻只起一条。线程 16（B36_THREADS 可压小）。
set -euo pipefail
W="${1:?草稿目录}"; shift
REPO="$(cd "$(dirname "$0")/../../../.." && pwd)"
MODEL="$REPO/research/prompts/m3-formal-code-b36-r1-opus-model"
DIFF="$REPO/research/prompts/_m3-formal-code-b36-r1-diff.md"
T="${B36_THREADS:-16}"
SEGMENTS=("$@")
[[ ${#SEGMENTS[@]} -gt 0 ]] || SEGMENTS=(setup build crafted crafted2 l57 l66 l14 l12 l36a l36b l36c r1 r2 r3 naga mut-a mut-b)
mkdir -p "$W/out"
CAP() { local cap="$1"; shift; env RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="m3-formal-code-b36-r1 攻方原型复跑：同一个二进制按参数跑大小不同的几段" nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" "$cap" bash "$REPO/research/scripts/capped.sh" "$T" "$@"; }
BIN="$W/target/release/b36probe"
run() { local name="$1"; shift; { CAP 3G "$@"; echo "rc=$?"; } > "$W/out/$name.log" 2>&1 || true; }

for segment in "${SEGMENTS[@]}"; do
  case "$segment" in
    setup) bash "$MODEL/tools/setup.sh" "$W" "$MODEL" "$DIFF" ;;
    build)
      (cd "$W/tree" && CARGO_TARGET_DIR="$W/target" CAP 8G cargo build --release --offline -p b36-opus-proto -p b36-opus-naga) > "$W/out/build.log" 2>&1
      for p in b36-opus-proto b36-opus-naga b36-checker-probe b36-checker-pre36; do
        grep -q "^ *Compiling $p " "$W/out/build.log" || { echo "编译日志里没有 Compiling $p：这一趟不作数"; exit 1; }
      done ;;
    crafted) run crafted "$BIN" crafted ;;
    crafted2) run crafted2 "$BIN" crafted2 ;;
    l57) run l57 "$BIN" layer0 57 7 1 "$T" 1000000 0 ;;
    l66) run l66 "$BIN" layer0 66 7 1 "$T" 1000000 0 ;;
    l14) run l14 "$BIN" layer0 14 7 1 "$T" 1000000 0 ;;
    l12) run l12 "$BIN" layer0 12 7 1 "$T" 1000000 0 ;;
    l36a) run l36a "$BIN" layer0 36 7 1 "$T" 1000000 0 0 22000 ;;
    l36b) run l36b "$BIN" layer0 36 7 1 "$T" 1000000 0 22000 44000 ;;
    l36c) run l36c "$BIN" layer0 36 7 1 "$T" 1000000 0 44000 0 ;;
    r1) run r1 "$BIN" random 0 112 20 "$T" ;;
    r2) run r2 "$BIN" random 112 112 20 "$T" ;;
    r3) run r3 "$BIN" random 400 48 40 "$T" ;;
    naga) { CAP 3G "$W/target/release/b36naga" "$W/wgsl-pre36-2" "$W/tree/crates/mutations.tsv"; echo "rc=$?"; } > "$W/out/naga.log" 2>&1 || true ;;
    mut-a|mut-b)
      # 副本上跑四行 CPU 变异（harness 档用例）：拷一份装好的 tree 当副本，各用自己的编译目录
      [[ -d "$W/mut" ]] || rsync -a "$W/tree/" "$W/mut/"
      rows=(963 2927); [[ "$segment" == mut-b ]] && rows=(3020 3021)
      bash "$MODEL/tools/mutrows.sh" "$W" "${rows[@]}" > "$W/out/mutrows-${segment#mut-}.txt" 2>&1 || true ;;
    *) echo "不认得的段：$segment"; exit 2 ;;
  esac
done
