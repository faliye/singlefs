#!/usr/bin/env bash
# 复跑（m2-closeout-code-r3 云端攻方腿）：在快照 refs/sop/m2-closeout-code-r3-snapshot 的副本上加原型包 opus-r3-proto 跑，不动主工作区。
# 用法（在仓根下）：bash research/prompts/m2-closeout-code-r3-opus-model/rerun.sh <草稿目录>
# 线程上限 8、内存上限 8G（research/scripts/replay.sh 的缺省）。建四份副本：snapshot（原样）、fixB / fixC（原型 B、C 两个改法）、mut736（变异表第 736 行那条变异）。
# 1. checks.sh：快照自带的几份 harness 档用例、X6 字段逐格与盘体对调、X1 写行之后掉电加一槽坏、第 736 行变异下的新旧两个靶
# 2. X6 旧快照 / 一槽坏 / 内容核对：snapshot、fixB、fixC 各跑一遍
# 3. X5 ① 批 2、批 3（batch_x5.sh；共约 45 分钟），analyze_x5.py 分格
set -euo pipefail
S=${1:?草稿目录}
R=$(git rev-parse --show-toplevel)
M=$R/research/prompts/m2-closeout-code-r3-opus-model
CAP="bash $R/research/scripts/run-with-memory-cap.sh 8G bash $R/research/scripts/capped.sh"
mkdir -p "$S/logs"
for arm in snapshot fixB fixC mut736; do
  mkdir -p "$S/$arm"
  git -C "$R" archive refs/sop/m2-closeout-code-r3-snapshot crates litmus Cargo.toml Cargo.lock .cargo | tar -x -C "$S/$arm"
  mkdir -p "$S/$arm/crates/opus-r3-proto/src" "$S/$arm/crates/opus-r3-proto/tests"
  cp "$M/Cargo.toml" "$S/$arm/crates/opus-r3-proto/Cargo.toml"
  cp "$M/lib.rs" "$S/$arm/crates/opus-r3-proto/src/lib.rs"
  cp "$M"/opus_r3_*.rs "$S/$arm/crates/opus-r3-proto/tests/"
  python3 - "$S/$arm/Cargo.toml" <<'PY'
import sys
p = sys.argv[1]
t = open(p).read()
anchor = '    "crates/singlefs-checker-tier",\n]'
assert t.count(anchor) == 1
open(p, 'w').write(t.replace(anchor, '    "crates/singlefs-checker-tier",\n    "crates/opus-r3-proto",\n]'))
PY
done
( cd "$S/fixB" && patch -p1 < "$M/fixB.patch" )
( cd "$S/fixC" && patch -p1 < "$M/fixC.patch" )
( cd "$S/mut736" && patch -p1 < "$M/mut736.patch" )
bash "$M/checks.sh" "$S" "$R" > "$S/logs/checks.out" 2>&1
X6=opus_r3_x6_stale_snapshot_mid_publish_and_one_slot_fault
for arm in snapshot fixB fixC; do
  ( cd "$S/$arm" && nice -n 19 $CAP 3 cargo test --offline --release -p opus-r3-proto --test $X6 -- --nocapture --test-threads 3 > "$S/logs/x6-$arm.log" 2>&1 )
done
bash "$M/batch_x5.sh" 2 "$S/snapshot" "$S/logs" "$R"
bash "$M/batch_x5.sh" 3 "$S/snapshot" "$S/logs" "$R"
python3 "$M/analyze_x5.py" "$S"/logs/x5[a-g]*.log | grep -v '^REDLINE'
