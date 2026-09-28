#!/usr/bin/env bash
# m2-safety-r3 云端攻方腿（Opus）模型复跑。
# 把与开工快照逐文件相同的一份 crates 拷进工作目录，打上 core-arms.patch（式子两处改动与三条零轮变体的开关，只在副本里），
# 放进 opus_r3_p3_attack.rs，逐段跑，输出写进 <输出目录>，再出各段汇总。
#   bash rerun.sh [与快照相同的仓根（含 crates/、Cargo.toml、Cargo.lock）] [工作目录] [输出目录]
# 默认：/tmp/claude-1000/safety-r3-frozen、/tmp/claude-1000/m2-safety-r3-opus-rerun、<工作目录>/out。
# ⚠️ 冻结副本 /tmp/claude-1000/safety-r3-frozen 在 2026-09-26 被改过四个文件，下面的 sha256 核对会拒它；
#    给一份与快照相同的仓根（这条腿交回时留着的 /tmp/claude-1000/m2-safety-r3-opus/tree 倒回补丁之后就是，见报告「没做什么」）。
# 编译与跑一律经 research/scripts/run-with-memory-cap.sh（10G）与 research/scripts/capped.sh（6 线程）。
# 挂钟（6 线程，这条腿量的）：E1 2.6 分、E3 0.7 分、E4 0.8 分、E6 4.8 分、E7 13.1 分、E8 10.7 分、E10 7.3 分、E2 13.0 分、E5 43.9 分、E7/E1 的 P3r 3 分。
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../.." && pwd)"
SRC="${1:-/tmp/claude-1000/safety-r3-frozen}"
WORK="${2:-/tmp/claude-1000/m2-safety-r3-opus-rerun}"
OUT="${3:-$WORK/out}"
CAP=10G
THREADS=6
( cd "$SRC" && sha256sum -c --quiet "$REPO/research/prompts/m2-safety-r3-snapshot/crates-sha256sums.txt" )
mkdir -p "$WORK/tree" "$OUT"
rsync -a --delete --exclude target --exclude .git "$SRC/crates" "$SRC/Cargo.toml" "$SRC/Cargo.lock" "$WORK/tree/"
( cd "$WORK/tree" && patch -p1 --forward < "$HERE/core-arms.patch" )
cp "$HERE/opus_r3_p3_attack.rs" "$WORK/tree/crates/singlefs-harness/tests/opus_r3_p3_attack.rs"
export CARGO_TARGET_DIR="$WORK/target"
run() {  # run <日志名> <用例名前缀> [环境变量…]
  local log="$1" filter="$2"; shift 2
  ( cd "$WORK/tree" && env "$@" nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" "$CAP" \
      bash "$REPO/research/scripts/capped.sh" "$THREADS" \
      cargo test --release -p singlefs-harness --test opus_r3_p3_attack "$filter" -- --nocapture ) > "$OUT/$log" 2>&1
  grep -E '^test result' "$OUT/$log"
}
ALL=T,P1,P2,P3,P3pl
run e1.log e1_ OPUS_ARMS=$ALL
run e3.log e3_ OPUS_ARMS=$ALL
run e4.log e4_
run e6.log e6_ OPUS_ARMS=T,P1,P2,P3
run e7.log e7_ OPUS_ARMS=$ALL
run e8.log e8_ OPUS_ARMS=$ALL
run e10.log e10_ OPUS_ARMS=$ALL
run e2.log e2_ OPUS_ARMS=$ALL
run e9.log e9_ "OPUS_E9=384:P3:a-ow150:40:53;384:P1:a-ow150:40:53"
run e5-count.log e5_ OPUS_E5_COUNT_ONLY=1 OPUS_ARMS=P1,P3,P3pl OPUS_SLOTS=240,384,768
run e5-count-raise-only.log e5_ OPUS_E5_COUNT_ONLY=1 OPUS_E5_SKIP_DELETE=1 OPUS_ARMS=P1,P3
run e5.log e5_ OPUS_ARMS=P1,P3 OPUS_SLOTS=240,256,320,384,448,512 \
  "OPUS_E5_PICK=256:P1:a-ow150;320:P1:c-grow6-truncate-x10;384:P1:a-ow150;448:P1:c-grow6-truncate-x10;512:P1:a-ow150;240:P3:a-ow150;240:P3:c-grow6-truncate-x10;256:P3:a-ow150;320:P3:c-grow6-truncate-x10;384:P3:c-grow6-truncate-x10;448:P3:c-grow6-truncate-x10;512:P3:c-grow6-truncate-x10"
run e7-p3r.log e7_ OPUS_ARMS=P3,P3r OPUS_SLOTS=256,320,384,448
run e1-p3r.log e1_ OPUS_ARMS=P3r
python3 "$HERE/summarize_e1.py" "$OUT/e1.log" > "$OUT/e1-summary.txt"
python3 "$HERE/summarize_e1.py" "$OUT/e1-p3r.log" > "$OUT/e1-p3r-summary.txt"
python3 "$HERE/summarize_e2.py" "$OUT/e2.log" > "$OUT/e2-summary.txt"
python3 "$HERE/summarize_e3.py" "$OUT/e3.log" > "$OUT/e3-summary.txt"
python3 "$HERE/summarize_e4.py" "$OUT/e4.log" > "$OUT/e4-summary.txt"
python3 "$HERE/summarize_e6.py" "$OUT/e6.log" > "$OUT/e6-summary.txt"
python3 "$HERE/summarize_e7.py" "$OUT/e7.log" > "$OUT/e7-summary.txt"
python3 "$HERE/summarize_e7.py" "$OUT/e7-p3r.log" > "$OUT/e7-p3r-summary.txt"
python3 "$HERE/summarize_e8.py" "$OUT/e8.log" > "$OUT/e8-summary.txt"
python3 "$HERE/summarize_e10.py" "$OUT/e10.log" > "$OUT/e10-summary.txt"
echo "输出在 $OUT"
