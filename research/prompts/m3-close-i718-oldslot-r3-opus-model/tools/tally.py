"""把探针输出（一张镜像一行）按（格去掉 -N<数> 与 -L<数>、镜像名）汇总：逐候选 I-7.18 的 G/R/NA 张数、丁乙两种读法下 I-7.19 的张数，
另核独立判法与 walk.rs 今天、丁甲两列逐张对不对得上。用法：tally.py <输出文件…>。"""
import collections, re, sys
NAMES = ["today", "jia", "yi_literal", "ding_jia_same_instance", "ding_jia", "ding_jia_gen0", "ding_yi"]
table = collections.OrderedDict()
mismatch = collections.Counter()
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8"):
        if " image=" not in line or "| I-7.18 " not in line:
            continue
        s = re.search(r"^S=(\d+)", line).group(1)
        cell = re.sub(r"-[NL]\d+(?=$|-)", "", re.search(r" cell=(\S+)", line).group(1))
        image = re.search(r" image=(\S+)", line).group(1)
        i718 = dict(re.findall(r"(\w+)=(G|R|NA)", line.split("| I-7.18 ")[1].split(" | ")[0]))
        i719_part = line.split("| I-7.19 ")[1].split(" | ")[0]
        i719 = dict(re.findall(r"(\w+)=(G|R|NA)", i719_part))
        independent = dict(re.findall(r"independent_(\w+)=(G|R)", line))
        key = (f"S={s}", cell, image)
        counter = table.setdefault(key, collections.Counter())
        counter["n"] += 1
        for name in NAMES:
            counter[("I-7.18", name, i718[name])] += 1
        counter[("I-7.19", "today", i719["today"])] += 1
        counter[("I-7.19", "ding_yi", i719["ding_yi"])] += 1
        counter[("I-7.19", "ding_yi_literal", i719["ding_yi_literal"])] += 1
        for name in ("today", "ding_jia"):
            if i718[name] in ("G", "R") and independent.get(name) != i718[name]:
                mismatch[(key, name)] += 1
def cell_text(counter, invariant, name):
    return "/".join(f"{v}{counter[(invariant, name, v)]}" for v in ("G", "R", "NA") if counter[(invariant, name, v)])
print("key | n | I-7.18 " + " ".join(NAMES) + " | I-7.19 today ding_yi ding_yi_literal")
for key, counter in table.items():
    print(" ".join(key), "| n=%d |" % counter["n"], " ".join(cell_text(counter, "I-7.18", n) for n in NAMES), "|",
          " ".join(cell_text(counter, "I-7.19", n) for n in ("today", "ding_yi", "ding_yi_literal")))
print("independent_vs_walk_mismatches", sum(mismatch.values()), dict(mismatch))
