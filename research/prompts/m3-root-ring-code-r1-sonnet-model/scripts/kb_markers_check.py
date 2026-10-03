#!/usr/bin/env python3
# kb 里 format-const 标记（JOURNAL_*）与冻结副本 singlefs-format 的 pub const 逐个比；只读。
import re, glob
code = open('/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-format/src/lib.rs').read()
consts = dict((m.group(1), m.group(2).replace('_', '')) for m in re.finditer(r'pub const (\w+): u64 = ([\d_]+);', code))
for f in sorted(glob.glob('.claude/kb/**/*.md', recursive=True)):
    for n, line in enumerate(open(f), 1):
        for m in re.finditer(r'format-const: (JOURNAL_\w+) = (\d+)', line):
            name, val = m.groups()
            print(f'{f}:{n}: {name} kb={val} code={consts.get(name)} {"OK" if consts.get(name) == val else "MISMATCH"}')
