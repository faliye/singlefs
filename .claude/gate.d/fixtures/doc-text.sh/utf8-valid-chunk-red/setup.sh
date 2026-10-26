#!/usr/bin/env bash
# 一份未跟踪页：开头 5 行 "x"（10 字节），接 16 MiB + 990 个 "a"，再接 0xc3 0x28（0xc3 后面本该是续字节、来的是 0x28），再接 "tail\n"。
# 坏字节从偏移 10 + 16 MiB + 990 起，在第二块里；行号是第 6 行（前面 5 个换行）。偏移要加上前一块的长度才对得上。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '# 起点\n' > base.md
git add -A && git commit -qm base
python3 - <<'PY'
chunk = 16 * 1024 * 1024
open("跨块坏页.md", "wb").write(b"x\n" * 5 + b"a" * (chunk + 990) + b"\xc3\x28" + b"tail\n")
PY
