"""e171-conclusion-r3 攻方：从入库产物造几份「骗判官」的产物（草稿，不入库）。
short   少跑了行：只留登记的 12 行与只靠 O1 的第 32 行、只靠 O2 的第 8 行的逐行结果行（别的行像没跑一样一行不留）。
o3red   永远答红的 O3：每行批 1 的 O3甲 改成 red。
o3in    永远答在的 O3：Q2 的 o3_only_strict 改成 0（行上 O3 本来就 0 红，不用改）。
用法：python3 make_products.py <入库产物> <输出目录>
"""
import os
import re
import sys

source, out = sys.argv[1], sys.argv[2]
lines = open(source, encoding="utf-8").read().splitlines(keepends=True)
keep = {15, 23, 27, 34, 42, 92, 112, 242, 269, 270, 271, 277, 32, 8}
per_row = re.compile(r"E7RESULT name=(row|row_touch|row_matrix|row_s5|row_o3|trajectory|substitution) row=(\d+) ")


def write(name, rows):
    with open(os.path.join(out, name), "w", encoding="utf-8") as handle:
        handle.writelines(rows)


short = [line for line in lines if not (per_row.match(line) and int(per_row.match(line).group(2)) not in keep)]
write("short.out", short)
write("o3red.out", [re.sub(r"(name=row_matrix row=\d+ batch=1 .*?)O3甲=quiet", r"\1O3甲=red", line) for line in lines])
write("o3in.out", [re.sub(r"o3_only_strict=\d+ ", "o3_only_strict=0 ", line) if "name=q2_family" in line else line for line in lines])
print("made short.out o3red.out o3in.out")
