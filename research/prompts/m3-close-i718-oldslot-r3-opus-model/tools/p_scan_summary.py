"""p 扫描（`ding_yi` 是丁乙照字面连 I-7.19 一起不判，`ding_yi_keeping_i719` 是 I-7.19 照判那一读法）：逐 (p, t) 组合列健康镜像上今天的 I-7.18/I-7.19，与写坏镜像（写 0、枚举撕法）上各候选；数「坏写者」（健康镜像上今天判红任一条的组合）在写坏镜像上被各候选整张放过（I-7.18 与 I-7.19 都不红）的张数。"""
import re, sys, collections
rows = collections.defaultdict(dict)
for line in open(sys.argv[1], encoding="utf-8"):
    m = re.search(r"cell=Z2-p-scan-p(\d+)-t(\d+) image=(\S+)", line)
    if not m:
        continue
    i718 = dict(re.findall(r"(\w+)=(G|R|NA)", line.split("| I-7.18 ")[1].split(" | ")[0]))
    i719 = dict(re.findall(r"(\w+)=(G|R|NA)", line.split("| I-7.19 ")[1].split(" | ")[0]))
    rows[(int(m.group(1)), int(m.group(2)))][m.group(3)] = (i718, i719)
names = ["today", "ding_jia", "ding_jia_gen0", "ding_yi", "ding_yi_keeping_i719"]
escaped = collections.Counter(); bad = 0
for (p, t), images in sorted(rows.items()):
    healthy = images.get("healthy_n")
    if not healthy:
        continue
    is_bad = healthy[0]["today"] == "R" or healthy[1]["today"] == "R"
    bad += is_bad
    cells = []
    for image in ("zero", "torn_enum"):
        if image not in images:
            continue
        i718, i719 = images[image]
        for name in names:
            i719_value = i719["ding_yi_literal"] if name == "ding_yi" else i719["ding_yi" if name == "ding_yi_keeping_i719" else name]
            i718_value = i718["ding_yi" if name == "ding_yi_keeping_i719" else name]
            caught = i718_value == "R" or i719_value == "R"
            if is_bad and not caught:
                escaped[(image, name)] += 1
        cells.append(f"{image}:" + ",".join(f"{n}={images[image][0][n]}/{(images[image][1]["ding_yi_literal"] if n == "ding_yi" else images[image][1][n])}" for n in names[:4]))
    print(f"p={p} t={t} healthy_today={healthy[0]['today']}/{healthy[1]['today']} bad={is_bad} " + " ".join(cells))
print("combinations", len(rows), "bad_on_the_healthy_image", bad)
for (image, name), count in sorted(escaped.items()):
    print(f"bad_writers_passed_on_the_written_bad_image image={image} candidate={name} count={count}")
