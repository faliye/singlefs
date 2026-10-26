#!/usr/bin/env python3
"""把 dispatch-prompt.py 打出的骨架填成能过 --check 的提示：「预计挂钟」填 2 小时内，删「分片」「续跑」「多机」三行，别的 <填 …> 占位换成一句填充文字；
「报告：」一行骨架填了什么就留什么（这一行正是要核的）。用法：fill_skeleton.py <骨架> <输出>"""
import re, sys
out = []
for line in open(sys.argv[1], encoding="utf-8").read().split("\n"):
    if line.startswith(("分片：", "续跑：", "多机：")):
        continue
    if line.startswith("预计挂钟："):
        line = "预计挂钟：2 小时内，探针"
    if line.startswith("要动的 crates 文件："):
        line = "要动的 crates 文件：crates/singlefs-core/src/lib.rs"
    out.append(re.sub(r"<填[^>]*>", "探针用的填充文字", line))
open(sys.argv[2], "w", encoding="utf-8").write("\n".join(out))
