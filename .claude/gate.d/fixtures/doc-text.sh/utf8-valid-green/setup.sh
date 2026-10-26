#!/usr/bin/env bash
# 基准里一份记录是合法的；这一轮把它加一行（仍合法）、新加一份带四字节字符（U+1F600）与汉字的页（未跟踪），
# 另放一份带 NUL 与非法字节的 blob.dat、一份没有 NUL 但后缀是 .gz 的 packed.gz：后两份是二进制，不判，列在「没查的」里。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records .claude/kb
printf '# 样本调度\n\n- 起点。\n' > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
printf '%s\n' '- 这一轮加一行：汉字、标点与全角括号（甲），全合法。' >> records/2026-09-24-样本调度.md
printf '# 新页\n\n四字节字符：\xf0\x9f\x98\x80 与汉字。\n' > .claude/kb/新页.md
printf 'binary\0\xff\xfe\xc3\x28 非法字节但有 NUL\n' > blob.dat
printf '\x1f\x8b\x08\xff\xfe 压缩包的样子，没有 NUL 但是后缀 .gz\n' > packed.gz
