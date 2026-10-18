"""把一份汇总压成一格一行（盘序 forward 与 reversed 结局相同时只数 forward，两序不同的另行标出）。
用法：python3 cells.py opus|sonnet <候选> <汇总文件>
每行：<历史或前缀/读故障> state_count=N complete=True|False | <判>:<结局> 状态数 …
opus 汇总的候选名是 pointer_per_kind、k6 等；sonnet 汇总的候选名是 pointer_slots_fixed、mapping_root_only 等。"""
import collections, re, sys

kind, candidate, path = sys.argv[1:]
coverage, cells = {}, collections.defaultdict(lambda: collections.defaultdict(int))
for line in open(path, encoding="utf-8"):
    fields = dict(re.findall(r"(\w+)=(\S+)", line))
    if line.startswith("name=c834_coverage"):
        if kind == "opus":
            key = fields["history"] + "/" + fields["faults"]
        else:
            if fields["candidate"] != candidate:
                continue
            key = fields["prefix"]
        coverage[key] = (fields["state_count"], fields["complete"])
    elif line.startswith("name=c834_cell") and fields["candidate"] == candidate:
        key = fields["history"] if kind == "opus" else fields["prefix"]
        label = f"{fields['judge']}:{fields['outcome']}"
        cells[key][(fields["order"], label)] += int(fields["states"])
for key in sorted(cells):
    count, complete = coverage.get(key, ("?", "?"))
    forward = {label: n for (order, label), n in cells[key].items() if order == "forward"}
    reversed_ = {label: n for (order, label), n in cells[key].items() if order == "reversed"}
    parts = " ; ".join(f"{label} {n}" for label, n in sorted(forward.items()))
    note = "" if forward == reversed_ else "  （两种盘序不同，reversed：" + " ; ".join(f"{label} {n}" for label, n in sorted(reversed_.items())) + "）"
    print(f"{key} state_count={count} complete={complete} | {parts}{note}")
