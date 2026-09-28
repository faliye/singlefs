#!/usr/bin/env bash
# PC-b（登记第五节 5.2）：一条臂的产物拷一份，改三处（name=segments path=transaction 的 operations= 加 1；
# 第一行 name=device_region_bytes 的 hexadecimal= 第一个十六进制字符换一个；E7INPUT 行末加一个字符），
# 拿同一个归一化脚本与原件比：该看到恰好 2 行删、2 行加，就是前两处；E7INPUT 那一处不出现。
set -euo pipefail
label="$1"; product="$2"
work=/tmp/claude-1000/e142-r19-runner/controls; mkdir -p "$work"
copy="$work/pc-b-$label.copy.out"
python3 - "$product" "$copy" <<'PY'
import re, sys
source, target = sys.argv[1], sys.argv[2]
lines = open(source, encoding='utf-8').read().split('\n')
done_operations = done_hex = done_input = False
for index, line in enumerate(lines):
    if not done_input and line.startswith('E7INPUT '):
        lines[index] = line + 'X'; done_input = True
    elif not done_operations and line.startswith('E7RESULT name=segments path=transaction '):
        lines[index] = re.sub(r' operations=(\d+) ', lambda m: f' operations={int(m.group(1)) + 1} ', line, count=1); done_operations = True
    elif not done_hex and ' name=device_region_bytes ' in (' ' + line) and 'hexadecimal=' in line:
        position = line.index('hexadecimal=') + len('hexadecimal=')
        lines[index] = line[:position] + ('1' if line[position] != '1' else '2') + line[position + 1:]; done_hex = True
assert done_operations and done_hex and done_input, (done_operations, done_hex, done_input)
open(target, 'w', encoding='utf-8').write('\n'.join(lines))
PY
normalize=/tmp/claude-1000/e142-r19-runner/diff/normalize.py
diff <(python3 "$normalize" "$product") <(python3 "$normalize" "$copy") > "$work/pc-b-$label.diff" || true
deleted=$(grep -c '^<' "$work/pc-b-$label.diff" || true); added=$(grep -c '^>' "$work/pc-b-$label.diff" || true)
input_lines=$(grep -c 'E7INPUT' "$work/pc-b-$label.diff" || true)
segments_lines=$(grep -c 'name=segments path=transaction ' "$work/pc-b-$label.diff" || true)
region_lines=$(grep -c 'name=device_region_bytes ' "$work/pc-b-$label.diff" || true)
echo "name=pc_b arm=$label copy_sha256=$(sha256sum "$copy" | cut -d' ' -f1) deleted=$deleted added=$added segments_transaction_lines=$segments_lines device_region_bytes_lines=$region_lines e7input_lines=$input_lines ok=$([ "$deleted" = 2 ] && [ "$added" = 2 ] && [ "$segments_lines" = 2 ] && [ "$region_lines" = 2 ] && [ "$input_lines" = 0 ] && echo true || echo false)"
