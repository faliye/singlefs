"""换几何取样的汇总：每个（R 的种类，前缀）一行，写 M1（入库 urJ 那一形、写行根之前全落）之后 R 的树表、映射根四个落点上是什么、
R 写出的单元被盖掉几份。用法：python3 survey-summary.py <survey 输出目录>"""
import glob, os, re, sys

for path in sorted(glob.glob(os.path.join(sys.argv[1], "*.txt"))):
    name = os.path.basename(path)[:-4]
    text = open(path, encoding="utf-8").read()
    history = re.search(r"name=c834_history (.*)", text).group(1)
    if "refused=" in history:
        print(f"name=c834_geometry case={name} {history}")
        continue
    slots = {}
    covered = 0
    for line in text.splitlines():
        if not line.startswith("name=c834_r_unit"):
            continue
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        if fields["r_wrote"] != fields["after_m1"]:
            covered += 1
        if fields["role"] != "unit":
            slots[fields["role"]] = f"{fields['slot']}:{fields['after_m1']}"
    state_count = re.search(r"state_count=(\d+)", history).group(1)
    print(f"name=c834_geometry case={name} r_copies_covered_by_m1={covered} tt0={slots['tt0']} tt1={slots['tt1']} mr0={slots['mr0']} mr1={slots['mr1']} state_count={state_count}")
