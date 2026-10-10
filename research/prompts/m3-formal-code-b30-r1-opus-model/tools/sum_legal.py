#!/usr/bin/env python3
"""汇总 b30_opus_e1_i54_legal_histories 打出的 B30GROUP 行：按组合计图像数、I-2.4 违例、I-5.4 各结局、新式部分没判与几类操作做成次数。用法：sum_legal.py <日志>…"""
import collections, re, sys
total = collections.defaultdict(collections.Counter)
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8"):
        if not line.startswith("B30GROUP "):
            continue
        name = line.split()[1]
        for key in ("images", "i24_violated", "i54_new_partial", "unlink_applied", "rollback_applied", "mount_applied", "seeds"):
            total[name][key] += int(re.search(rf"\b{key}=(\d+)", line).group(1))
        for label, count in re.findall(r'"(\w+)": (\d+)', line.split("i54=")[1].split("}")[0]):
            total[name]["i54_" + label] += int(count)
grand = collections.Counter()
for name, counter in total.items():
    print(name, dict(counter))
    grand.update(counter)
print("ALL", dict(grand))
