#!/usr/bin/env bash
# m3-batch10z-no-code-r1 攻方 K4：两种遮蔽写法各打在 k4copy 上、cargo check 一次（只编不跑），并照搬扫源码比一次。用法：bash k4_check.sh <草稿目录> <模型目录>
set -u
W=$1; MD=$2
for v in wgpu std; do
  cp "$W/gpu_unit_checks.orig.rs" "$W/k4copy/crates/singlefs-checker-tier/src/gpu_unit_checks.rs"
  python3 "$MD/k4_shadow.py" "$W/k4copy" "$v" > /dev/null || exit 3
  echo "variant=$v mod_lines=$(grep -c "mod $v {" "$W/k4copy/crates/singlefs-checker-tier/src/gpu_unit_checks.rs")"
  ( cd "$W/k4copy" && CARGO_TARGET_DIR="$W/target-k4" nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 8 cargo check --offline -p singlefs-checker-tier --features gpu --lib > "$W/k4-check-$v.log" 2>&1; echo "check $v rc=$?" )
  grep -E 'Checking singlefs-checker-tier|^warning|^error|Finished' "$W/k4-check-$v.log" | head -5
  python3 "$MD/scan_port.py" "$W/k4copy/crates/singlefs-checker-tier/src/gpu_unit_checks.rs"
done
# 阴性对照：遮蔽模块里写一个不存在的变体，cargo check 必须拒（说明遮蔽模块真被类型检查过）；扫源码的阴性对照：方法体最前面加 if true { return; }
python3 - "$W/k4copy/crates/singlefs-checker-tier/src/gpu_unit_checks.rs" <<'PY'
import sys
p = sys.argv[1]; s = open(p).read()
old = "                    Ok(Ok(::wgpu::PollStatus::QueueEmpty))\n"
assert s.count(old) == 1
open(p, 'w').write(s.replace(old, "                    Ok(Ok(::wgpu::PollStatus::NoSuchVariant))\n"))
PY
( cd "$W/k4copy" && CARGO_TARGET_DIR="$W/target-k4" nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 8 cargo check --offline -p singlefs-checker-tier --features gpu --lib > "$W/k4-check-negative.log" 2>&1; echo "check negative rc=$?" )
grep -E '^error\[' "$W/k4-check-negative.log" | head -1
python3 - "$W/gpu_unit_checks.orig.rs" "$W/scan-negative.rs" <<'PY'
import sys
s = open(sys.argv[1]).read()
old = "            fn take_in_the_devices_found_idle(&self) {\n"
assert s.count(old) == 1
open(sys.argv[2], 'w').write(s.replace(old, old + "                if true { return; }\n"))
PY
python3 "$MD/scan_port.py" "$W/gpu_unit_checks.orig.rs"
python3 "$MD/scan_port.py" "$W/scan-negative.rs"
