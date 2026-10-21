"""把一份 run 产物的 name=sig 行按列合并计数。用法：python3 summarize.py <产物> [列名 …]
默认列：phase choice verdict read_only writable raw_red regen_rerun sysconf_guard[new] sysconf_guard[old] unit_scan[new] unit_scan[old]。
choice 只留「哪个池」（new/old/other_old/Err:…），unit_scan 只留「实例代号 > 0 的有没有」（hi>0 / hi=0）。"""
import collections
import re
import sys

columns = sys.argv[2:] or ["phase", "choice", "verdict", "read_only", "writable", "raw_red", "regen_rerun",
                           "sysconf_guard[new]", "sysconf_guard[old]", "unit_scan[new]", "unit_scan[old]"]


def field(line, name):
    if name == "raw_red":
        return re.search(r"raw_red=(\[[^\]]*\])", line).group(1).replace(" ", "")
    m = re.search(r"(?:^| )" + re.escape(name) + r"=(\S+)", line)
    value = m.group(1) if m else "-"
    if name == "choice":
        value = value.split("@")[0] if not value.startswith("Err:") else value
    if name.startswith("unit_scan["):
        value = "hi>0" if value != "-" and int(value.split("/")[0]) > 0 else "hi=0"
    if name.startswith("sysconf_guard["):
        value = value.split("/")[0]
    return value


agg = collections.Counter()
total = 0
for line in open(sys.argv[1]):
    if not line.startswith("name=sig"):
        continue
    count = int(re.search(r"count=(\d+)", line).group(1))
    total += count
    agg[tuple(field(line, c) for c in columns)] += count
print("# " + " ".join(["count"] + columns) + f"  (total {total})")
for key, value in sorted(agg.items(), key=lambda kv: (kv[0][0], -kv[1])):
    print(value, *key)
