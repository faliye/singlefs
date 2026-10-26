#!/usr/bin/env bash
# 一份 16 MiB + 100 字节的未跟踪页，第 16 MiB - 1 字节处起是一个三字节汉字（「中」e4 b8 ad）：它跨在第一块与第二块之间（块是 16 MiB），整份合法，
# 判法要把块尾被切开的序列带到下一块再判，不能当成坏字节。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '# 起点\n' > base.md
git add -A && git commit -qm base
python3 - <<'PY'
chunk = 16 * 1024 * 1024
open("跨块页.md", "wb").write(b"a" * (chunk - 1) + "中".encode() + b"b" * 100 + b"\n")
PY
