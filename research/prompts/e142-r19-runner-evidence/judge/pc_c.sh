#!/usr/bin/env bash
# PC-c（登记第五节 5.2）：合成的「新」= 臂 N18 产物的拷贝，第一行 name=done 之前、step 最大的那两次 device_region_bytes 写里
# 设备 1 那一次的十六进制，区域内偏移 6 那个字节异或 0x01；跑 old-new（合成, N18）与 old-new（N18, N18）。
set -euo pipefail
arm_n18="$1"; bin=research/target/release/e142-region-diff-independent
work=/tmp/claude-1000/e142-r19-runner/controls; mkdir -p "$work"
synthetic="$work/pc-c-synthetic-new.out"
python3 - "$arm_n18" "$synthetic" <<'PY'
import re, sys
source, target = sys.argv[1], sys.argv[2]
lines = open(source, encoding='utf-8').read().split('\n')
first_done = next(index for index, line in enumerate(lines) if re.search(r'(^|\s)name=done(\s|$)', line))
candidates = [(int(re.search(r' step=(\d+)', line).group(1)), index) for index, line in enumerate(lines[:first_done]) if re.search(r'(^|\s)name=device_region_bytes\s', line)]
top_steps = sorted({step for step, _ in candidates})[-2:]
picked = [index for step, index in candidates if step in top_steps and ' device=1 ' in lines[index]]
assert len(picked) == 1, picked
index = picked[0]
line = lines[index]
start = line.index('hexadecimal=') + len('hexadecimal=')
hex_text = line[start:].split(' ')[0]
byte = int(hex_text[12:14], 16) ^ 0x01
new_hex = hex_text[:12] + f'{byte:02x}' + hex_text[14:]
lines[index] = line[:start] + new_hex + line[start + len(hex_text):]
print(f'picked_line={index + 1} steps={top_steps} region_offset=6 old_byte={hex_text[12:14]} new_byte={byte:02x}')
open(target, 'w', encoding='utf-8').write('\n'.join(lines))
PY
echo "synthetic_sha256=$(sha256sum "$synthetic" | cut -d' ' -f1)"
"$bin" old-new "$synthetic" "$arm_n18" | grep 'name=old_new_independent_summary'
"$bin" old-new "$arm_n18" "$arm_n18" | grep 'name=old_new_independent_summary'
