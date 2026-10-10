#!/usr/bin/env python3
"""把原型 crate 拷进攻方副本 crates/opus-b33-proto，并加进副本 workspace 的成员（恰好命中一次才写，否则退 3）。

用法：install_proto.py <副本根> <模型目录>
"""
import pathlib
import shutil
import sys

root = pathlib.Path(sys.argv[1])
model = pathlib.Path(sys.argv[2])
destination = root / "crates/opus-b33-proto"
if destination.exists():
    print("副本里已有 crates/opus-b33-proto", file=sys.stderr)
    sys.exit(3)
shutil.copytree(model / "src/proto", destination)
manifest = root / "Cargo.toml"
text = manifest.read_text()
old = '    "crates/singlefs-reference-filesystem",\n]'
if text.count(old) != 1:
    print("workspace 成员表的末行没恰好命中一次", file=sys.stderr)
    sys.exit(3)
manifest.write_text(text.replace(old, '    "crates/singlefs-reference-filesystem",\n    "crates/opus-b33-proto",\n]'))
print(f"installed {destination}")
