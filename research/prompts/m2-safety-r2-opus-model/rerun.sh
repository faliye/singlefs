#!/usr/bin/env bash
# m2-safety-r2 云端攻方腿（Opus）模型复跑。
# 把冻结副本拷一份，打上 core-arms.patch（A1–A4 与攻方腿自己的零轮臂，只在副本里），放进 opus_s4_attack.rs，逐段跑，输出写进 <输出目录>。
#   bash rerun.sh [冻结副本的 crates 目录] [工作目录] [输出目录]
# 默认：/tmp/claude-1000/m2-safety-r2/tree/crates、/tmp/claude-1000/m2-safety-r2-opus/rerun、<工作目录>/out。
# 编译与跑一律经 research/scripts/run-with-memory-cap.sh（10G）与 research/scripts/capped.sh（8 线程）。约 6 分钟（8 线程）。
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../.." && pwd)"
FROZEN="${1:-/tmp/claude-1000/m2-safety-r2/tree/crates}"
WORK="${2:-/tmp/claude-1000/m2-safety-r2-opus/rerun}"
OUT="${3:-$WORK/out}"
CAP=10G
THREADS=8
mkdir -p "$WORK/tree" "$OUT"
# 冻结副本里要打补丁的三份文件，补丁是对着这三份做的（sha256 不对就停）。
( cd "$FROZEN/singlefs-core/src" && sha256sum -c - ) <<'SUMS'
529da256fbb2499094ee97b9ff6ca2cc62423ccdf9f42edaff0fd1d4621d8cbd  admission.rs
dc5d00838f0f4e039fea62d7a0ca19bbc428eaa5c404ca12db5cc8b237744b5a  transaction.rs
c9f7a2b01614fde366f3e3643d5d95244da3299672fd4c9c77a830299d1c2cf3  mount.rs
SUMS
rsync -a --delete --exclude target --exclude .git "$FROZEN" "$WORK/tree/"
cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$WORK/tree/"
( cd "$WORK/tree" && patch -p2 --forward < "$HERE/core-arms.patch" )
cp "$HERE/opus_s4_attack.rs" "$WORK/tree/crates/singlefs-harness/tests/opus_s4_attack.rs"
export CARGO_TARGET_DIR="$WORK/target"
run() {  # run <日志名> <用例名前缀> [环境变量…]
  local log="$1" filter="$2"; shift 2
  ( cd "$WORK/tree" && env "$@" nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" "$CAP" \
      bash "$REPO/research/scripts/capped.sh" "$THREADS" \
      cargo test --release -p singlefs-harness --test opus_s4_attack "$filter" -- --nocapture ) > "$OUT/$log" 2>&1
  grep -E '^test result' "$OUT/$log"
}
run e1.log e1_ OPUS_ARMS=T,A1,A2,ND,A4,A1+A4,A2+A4,ND+A4
run e2.log e2_ OPUS_ARMS=T,A1,A2,A4,A3ge,A3gt,A3d,A2+A3ge,A2+A3gt,A2+A3d,A2+A3gt+A4,A2+A3ge+A4
run e3.log e3_ OPUS_ARMS=T,A1,A2,ND,A4,A3ge,A3gt,A2+A3ge,A2+A3gt,A2+A4,A2+A3gt+A4,A2+A3ge+A4
run e3-zero-round.log e3_ OPUS_ARMS=A1x,ND+A1n,ND+A1n+A4,ND+A1n+A3gt,ND+A1n+A3gt+A4
run e4.log e4_ OPUS_ARMS=T,A1,A2,ND,A4,A3ge,A3gt,A2+A3ge,A2+A3gt,A2+A4,A2+A3gt+A4,A2+A3ge+A4
run e4-zero-round.log e4_ OPUS_ARMS=A1x,ND+A1n,ND+A1n+A3gt,ND+A1n+A3gt+A4
run e5.log e5_ OPUS_ARMS=T,A1,A2,ND,A4,A3ge,A3gt,A2+A3ge,A2+A3gt,A2+A4,A2+A3gt+A4,A2+A3ge+A4
run e6.log e6_
run e7.log e7_
python3 "$HERE/summarize_e2.py" "$OUT/e2.log" > "$OUT/e2-summary.txt"
python3 "$HERE/summarize_e3.py" "$OUT/e3.log" > "$OUT/e3-summary.txt"
python3 "$HERE/summarize_e3.py" "$OUT/e3-zero-round.log" > "$OUT/e3-zero-round-summary.txt"
python3 "$HERE/summarize_e4.py" "$OUT/e4.log" > "$OUT/e4-summary.txt"
python3 "$HERE/summarize_e4.py" "$OUT/e4-zero-round.log" > "$OUT/e4-zero-round-summary.txt"
echo "输出在 $OUT"
