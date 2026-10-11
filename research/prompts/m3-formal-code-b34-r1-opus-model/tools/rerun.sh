#!/usr/bin/env bash
# 第 34 批代码轮第一轮攻方原型的复跑（主 agent 在自己的草稿目录跑）。
# 用法：bash tools/rerun.sh <草稿目录> [段…]；段取 build crafted r1 r2 r3 r4 rfl l41 l45 l6 l57a l57b l65a l65b l3，不给 = 全部（约 30 分钟，10 线程）。
# 每段的输出落 <草稿目录>/out/<段>.log，末行 rc=…；判结果看各段的 B34R / B34L0-DONE / B34L0T / B34X 行，与 out/*.txt 里抄的逐行比。
set -euo pipefail
W="${1:?草稿目录}"; shift
REPO="$(cd "$(dirname "$0")/../../../.." && pwd)"
MODEL="$REPO/research/prompts/m3-formal-code-b34-r1-opus-model"
FROZEN=/tmp/claude-1000/m3-formal-code-b34-r1/tree
DIFF="$REPO/research/prompts/_m3-formal-code-b34-r1-diff.md"
SEGMENTS=("$@")
[[ ${#SEGMENTS[@]} -gt 0 ]] || SEGMENTS=(build crafted r1 r2 r3 r4 rfl l41 l45 l6 l57a l57b l65a l65b l3)
mkdir -p "$W/out"
CAP=(env RUN_WITH_MEMORY_CAP_PARALLEL_REASON="m3-formal-code-b34-r1 攻方原型复跑" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="同一个二进制按参数跑大小不同的几段历史" nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 8G bash "$REPO/scripts/capped.sh" 10)
BIN="$W/target/release/b34probe"

build() {
  rm -rf "$W/tree"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$W/tree/"
  (cd "$W/tree" && sha256sum -c --quiet /tmp/claude-1000/m3-formal-code-b34-r1/crates-sha256.txt)
  # 34-1 那份 walk.rs 补丁原样从改动全文里切出来（walk.rs 那一段到扫描内核那一段之前）。
  awk '/^diff --git a\/crates\/singlefs-checker\/src\/walk.rs/{on=1} /^diff --git a\/crates\/singlefs-checker-tier\/src\/crash_judge_checker_scan.wgsl/{on=0} on' "$DIFF" > "$W/walk34-1.patch"
  local C="$W/tree/crates"
  mkdir -p "$C/b34-checker-pre34" "$C/b34-checker-fl" "$C/b34-opus-proto/src"
  cp -r "$C/singlefs-checker/src" "$C/b34-checker-pre34/"
  cp -r "$C/singlefs-checker/src" "$C/b34-checker-fl/"
  (cd "$C/b34-checker-pre34/src" && patch -s -R walk.rs < "$W/walk34-1.patch" && patch -s walk.rs < "$MODEL/patches/pre34-probe.patch")
  (cd "$C/b34-checker-fl/src" && patch -s walk.rs < "$MODEL/patches/fl.patch")
  cp "$MODEL/cargo/b34-checker-pre34.toml" "$C/b34-checker-pre34/Cargo.toml"
  cp "$MODEL/cargo/b34-checker-fl.toml" "$C/b34-checker-fl/Cargo.toml"
  cp "$MODEL/cargo/b34-opus-proto.toml" "$C/b34-opus-proto/Cargo.toml"
  cp "$MODEL"/src/*.rs "$C/b34-opus-proto/src/"
  # 换过的源文件刷新修改时间：同一个编译目录里不复用上一趟的产物。
  touch "$C"/b34-*/src/*.rs
  python3 - "$W/tree/Cargo.toml" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = '    "crates/singlefs-reference-filesystem",\n]'
assert s.count(old) == 1
open(p, 'w').write(s.replace(old, '    "crates/singlefs-reference-filesystem",\n    "crates/b34-checker-pre34",\n    "crates/b34-checker-fl",\n    "crates/b34-opus-proto",\n]'))
PY
  (cd "$W/tree" && CARGO_TARGET_DIR="$W/target" "${CAP[@]}" cargo build --release --offline -p b34-opus-proto) > "$W/out/build.log" 2>&1
  grep -c '^ *Compiling b34-opus-proto' "$W/out/build.log" > /dev/null || { echo "编译日志里没有 Compiling b34-opus-proto：这一趟不作数"; exit 1; }
}

run() { local name="$1"; shift; { "${CAP[@]}" "$@"; echo "rc=$?"; } > "$W/out/$name.log" 2>&1 || true; }

for segment in "${SEGMENTS[@]}"; do
  case "$segment" in
    build) build ;;
    crafted) run crafted "$BIN" crafted ;;
    r1) run r1 "$BIN" random 0 100 20 10 ;;
    r2) run r2 "$BIN" random 100 120 20 10 ;;
    r3) run r3 "$BIN" random 220 60 40 10 ;;
    r4) run r4 "$BIN" random 500 90 20 10 ;;
    rfl) run rfl env B34_WITH_FL=1 "$BIN" random 400 60 20 10 ;;
    l41) run l41 "$BIN" layer0 41 7 1 10 1000000 0 1 0 0 ;;
    l45) run l45 "$BIN" layer0 45 7 1 10 1000000 0 1 0 0 ;;
    l6) run l6 "$BIN" layer0 6 7 1 10 1000000 0 1 0 0 ;;
    l57a) run l57a "$BIN" layer0 57 7 1 10 1000000 0 0 0 33000 ;;
    l57b) run l57b "$BIN" layer0 57 7 1 10 1000000 0 0 33000 0 ;;
    l65a) run l65a "$BIN" layer0 65 7 1 10 1000000 0 0 0 33000 ;;
    l65b) run l65b "$BIN" layer0 65 7 1 10 1000000 0 0 33000 0 ;;
    l3) run l3 "$BIN" layer0 3 6 2 10 1000000 0 1 0 0 ;;
    *) echo "不认得的段：$segment"; exit 2 ;;
  esac
done
