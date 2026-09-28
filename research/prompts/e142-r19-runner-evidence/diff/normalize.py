#!/usr/bin/env python3
"""E142 第十九次跑 R37 的归一化：删 `E7INPUT ` 起头的行；每行 `hexadecimal=<十六进制>` 换成
`hexadecimal_sha256=<那段十六进制文本按 ASCII 算的 sha256>`；其余逐字不动、次序不动。
用法：normalize.py <产物> [--model-side-only]
--model-side-only：Q142.56 的取法——只留第一行 name=done 之前的行，并剔掉读导出的六种行。"""
import hashlib, re, sys

HEX = re.compile(r'(?<![A-Za-z_])hexadecimal=([0-9a-fA-F]*)')
EXPORT_READING = ('impl_bytes_equal', 'impl_bytes_unmatched', 'impl_bytes_equal_summary', 'window_segments_summary', 'done')

def name_of(line):
    match = re.search(r'(?:^|\s)name=(\S+)', line)
    return match.group(1) if match else None

def normalize(path, model_side_only):
    out = []
    with open(path, encoding='utf-8') as handle:
        for raw in handle:
            line = raw.rstrip('\n')
            if line.startswith('E7INPUT '):
                continue
            name = name_of(line)
            if model_side_only:
                if name == 'done':
                    break
                if name in EXPORT_READING:
                    continue
                if name == 'window_segments' and ' side=crates' in line:
                    continue
            line = HEX.sub(lambda m: 'hexadecimal_sha256=' + hashlib.sha256(m.group(1).encode('ascii')).hexdigest(), line)
            out.append(line)
    return out

if __name__ == '__main__':
    model_only = '--model-side-only' in sys.argv[2:]
    sys.stdout.write(''.join(line + '\n' for line in normalize(sys.argv[1], model_only)))
