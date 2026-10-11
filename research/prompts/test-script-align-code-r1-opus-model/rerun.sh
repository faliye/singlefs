#!/usr/bin/env bash
# test-script-align-code-r1 攻方腿（Opus）的复跑：在一个空的工作目录里从冻结副本拷两份仓（tree 原样、tree2 改坏一处），
# 照本目录的原型源码建 proto（依赖 tree，特性 verdict-store,gpu）与 proto2（依赖 tree2），编、跑，原样输出落进 <工作目录>/runs/。
# 用法：bash rerun.sh <工作目录> [冻结副本，默认 /tmp/claude-1000/test-script-align-code-r1/tree]
# 不跑 checker 档包的任何测试；跑的都是原型二进制，经 scripts/run-with-memory-cap.sh。线程上限 8（capped.sh 8）。
set -u
model="$(cd "$(dirname "$0")" && pwd)"
repository="$(cd "$model/../../.." && pwd)"
work="${1:?要给工作目录}"; frozen="${2:-/tmp/claude-1000/test-script-align-code-r1/tree}"
cap="$repository/scripts/run-with-memory-cap.sh"; capped="$repository/scripts/capped.sh"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-600}"
mkdir -p "$work/runs"; export K2_SCRATCH="$work/runs" TMPDIR="$work/runs"
rsync -a --exclude target --exclude .git "$frozen/" "$work/tree/"
rsync -a --exclude target --exclude .git "$frozen/" "$work/tree2/"
python3 - "$work/tree2/crates/singlefs-checker-tier/src/crash_injection.rs" <<'PY'
import sys, os
p = sys.argv[1]; s = open(p, encoding="utf-8").read()
head, sep, tail = s.partition("fn draw_second_crash_points(")
old = "    for phase in WritableMountPhase::ALL {"
assert sep and old in tail
tail = tail.replace(old, "    // K2 原型的变异：二次崩溃一个都不摆（挂载途中的崩溃点整类丢掉）\n    for phase in WritableMountPhase::ALL.into_iter().take(0) {", 1)
open(p + ".tmp", "w", encoding="utf-8").write(head + sep + tail); os.replace(p + ".tmp", p)
PY
for package in proto proto2; do
  mkdir -p "$work/$package/.cargo"
  cp -r "$model/$package/src" "$work/$package/"; cp "$model/$package/Cargo.toml" "$work/$package/"
  cp "$frozen/.cargo/config.toml" "$work/$package/.cargo/"; cp "$frozen/Cargo.lock" "$work/$package/"
done
cp "$model/apply_row.py" "$model/fixA.py" "$model/variants.sh" "$model/keys_variant.sh" "$work/"
(cd "$work/proto" && nice -n 19 bash "$cap" 12G bash "$capped" 8 cargo build --release --offline --bins) > "$work/runs/build1.log" 2>&1 || { echo "proto 编不过"; exit 1; }
(cd "$work/proto2" && nice -n 19 bash "$cap" 8G bash "$capped" 8 cargo build --release --offline --bins) > "$work/runs/build2.log" 2>&1 || { echo "proto2 编不过"; exit 1; }
(cd "$work/proto" && nice -n 19 bash "$cap" 4G bash "$capped" 8 cargo run --release --offline --bin reuse_key) > "$work/runs/reuse_key.out" 2> "$work/runs/reuse_key.err"
(cd "$work/proto" && nice -n 19 bash "$cap" 4G bash "$capped" 8 cargo run --release --offline --bin stopped_line) > "$work/runs/stopped_line.out" 2> "$work/runs/stopped_line.err"
(cd "$work/proto" && nice -n 19 bash "$cap" auto ./target/release/gpu_probe) > "$work/runs/gpu_probe-normal.out" 2> "$work/runs/gpu_probe-normal.err"
(cd "$work/proto" && VK_ICD_FILENAMES=/nonexistent/nvidia_icd.json VK_DRIVER_FILES=/nonexistent/nvidia_icd.json nice -n 19 bash "$cap" auto ./target/release/gpu_probe) > "$work/runs/gpu_probe-missing.out" 2> "$work/runs/gpu_probe-missing.err"
(cd "$work/proto" && VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json nice -n 19 bash "$cap" auto ./target/release/gpu_probe) > "$work/runs/gpu_probe-lvp.out" 2> "$work/runs/gpu_probe-lvp.err"
(cd "$work/proto" && K2_SEEDS_PER_SAMPLING=6 nice -n 19 bash "$cap" auto ./target/release/repro_probe) > "$work/runs/repro_probe-6.out" 2> "$work/runs/repro_probe-6.err"
(cd "$work/proto2" && nice -n 19 bash "$cap" 4G ./target/release/ci_repro) > "$work/runs/ci_repro.out" 2> "$work/runs/ci_repro.err"
nice -n 19 bash "$cap" 8G bash "$capped" 8 bash "$work/variants.sh" "$work" "$frozen" > "$work/runs/variants.out" 2> "$work/runs/variants.err"
RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="这一趟要整份重编 checker 档与原型" nice -n 19 bash "$cap" 8G bash "$capped" 8 bash "$work/keys_variant.sh" "$work" "$frozen" > "$work/runs/keys_variant.out" 2>&1
python3 "$model/k3check.py" "$model/appended.tsv" > "$work/runs/k3check.out"
grep -h '^K2' "$work"/runs/*.out
grep -h '^VARIANT' "$work/runs/variants.out"
grep -h '^K2R1' "$work"/runs/reuse_key-*.out
grep -h '^K3KEYS' "$work"/runs/keys_probe-*.out
grep -c 'in_table=True anchor_hits=1 cols=6 test_defs=1' "$work/runs/k3check.out"
