#!/usr/bin/env python3
"""用法：mut.py <verus-mutations.tsv> <文件行号> <源 crate 目录> <目标 crate 目录>
按必红表那一行把原文恰好替换一次（\\n 当换行），写到目标目录（目标目录先整份拷源）。"""
import shutil, sys, pathlib
tsv, line_no, src, dst = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
row = pathlib.Path(tsv).read_text().split("\n")[line_no - 1].split("\t")
name, _file, before, after, expect = row
before = before.replace("\\n", "\n"); after = after.replace("\\n", "\n")
shutil.rmtree(dst, ignore_errors=True); shutil.copytree(src, dst)
lib = pathlib.Path(dst) / "src/lib.rs"
text = lib.read_text()
hits = text.count(before)
if hits != 1:
    print(f"row={line_no} hits={hits} 不是恰好一次"); sys.exit(3)
lib.write_text(text.replace(before, after, 1))
print(f"row={line_no} hits=1 expect={expect} name={name}")
