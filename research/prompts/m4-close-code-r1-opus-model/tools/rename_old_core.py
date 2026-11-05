#!/usr/bin/env python3
"""把 HEAD 那一份 singlefs-core 的清单改名成 singlefs-core-old（links 一起改），两个路径依赖指到冻结副本那一份（格式与 replay-prefix 的执行代码两份相同）。
用法：python3 rename_old_core.py <HEAD 那一份的 crates/singlefs-core/Cargo.toml>"""
import sys

path = sys.argv[1]
text = open(path, encoding='utf-8').read()
for old, new in [('name = "singlefs-core"', 'name = "singlefs-core-old"'), ('links = "singlefs-core"', 'links = "singlefs-core-old"'),
                 ('{ path = "../singlefs-format" }', '{ path = "../../../new/crates/singlefs-format" }'),
                 ('{ path = "../singlefs-replay-prefix" }', '{ path = "../../../new/crates/singlefs-replay-prefix" }')]:
    if text.count(old) != 1:
        sys.exit(f"清单里 {old!r} 不是恰好一处")
    text = text.replace(old, new)
open(path, 'w', encoding='utf-8').write(text)
