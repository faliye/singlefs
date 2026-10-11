#!/usr/bin/env bash
# 第 35 批代码轮第一轮攻方原型的复跑（主 agent 在自己的草稿目录跑）。
# 用法：bash tools/rerun.sh <草稿目录> [段…]；段取 build crafted r1 r2 r3 l43 l18 l45 s256 s1536，不给 = 全部（10 线程约 22 分钟）。
# 每段的输出落 <草稿目录>/out/<段>.log，末行 rc=…；判结果看各段的 B35R / B35L0-DONE / B35L0T / B35X / B35F / B35E / B35S 行，与 out/*.txt 里留的逐行比
# （B35R / B35L0T 的计数行逐字比；B35H 行的 seconds= 与 B35L0-DONE 的 seconds= 是挂钟，不比）。
set -euo pipefail
W="${1:?草稿目录}"; shift
REPO="$(cd "$(dirname "$0")/../../../.." && pwd)"
MODEL="$REPO/research/prompts/m3-formal-code-b35-r1-opus-model"
DIFF="$REPO/research/prompts/_m3-formal-code-b35-r1-diff.md"
SEGMENTS=("$@")
[[ ${#SEGMENTS[@]} -gt 0 ]] || SEGMENTS=(build crafted r1 r2 r3 l43 l18 l45 s256 s1536)
mkdir -p "$W/out"
CAP=(env RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-formal-code-b35-r1 攻方原型复跑" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="同一个二进制按参数跑大小不同的几段历史" nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 8G bash "$REPO/scripts/hosts/capped.sh" 10)
BIN="$W/target/release/b35probe"

build() {
  bash "$MODEL/tools/setup.sh" "$W" "$MODEL" "$DIFF"
  (cd "$W/tree" && CARGO_TARGET_DIR="$W/target" "${CAP[@]}" cargo build --release --offline -p b35-opus-proto) > "$W/out/build.log" 2>&1
  grep -q '^ *Compiling b35-opus-proto' "$W/out/build.log" || { echo "编译日志里没有 Compiling b35-opus-proto：这一趟不作数"; exit 1; }
}

run() { local name="$1"; shift; { "${CAP[@]}" "$@"; echo "rc=$?"; } > "$W/out/$name.log" 2>&1 || true; }

for segment in "${SEGMENTS[@]}"; do
  case "$segment" in
    build) build ;;
    crafted) run crafted env B35_DUMP=1 "$BIN" crafted 10 ;;
    r1) run r1 "$BIN" random 0 120 20 10 ;;
    r2) run r2 "$BIN" random 120 120 20 10 ;;
    r3) run r3 "$BIN" random 240 50 40 10 ;;
    l43) run l43 "$BIN" layer0 43 7 1 10 1000000 0 ;;
    l18) run l18 "$BIN" layer0 18 7 1 10 1000000 0 ;;
    l45) run l45 "$BIN" layer0 45 7 1 10 1000000 0 ;;
    s256) run smallring-256 "$BIN" smallring 256 10 80 ;;
    s1536) run smallring-1536 "$BIN" smallring 1536 10 500 ;;
    *) echo "不认得的段：$segment"; exit 2 ;;
  esac
done
