"""从原型的 c834_state 行按那一判记下的凭据推一个派生候选的结局（同一（状态，次序）上放过那一支的结局取任一判放过的候选的结局，拒那一支取 always_refuse 的）：
mapping_root_only = 只认 K6 的映射根槽那一句（第一轮正推的变异 k6_only_mapping_root）。逐（历史/读故障）数：放过而这次挂载新写掉 R 单元的状态（R 的单元此前一份不少）、拒的状态、放过而没写掉的状态。
用法：python3 derive.py <输出文件…>"""
import collections, re, sys
counts = collections.Counter()
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8"):
        if not line.startswith("name=c834_state") or "creds=not_reached" in line:
            continue
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        creds = fields["creds"]
        if not creds.startswith("once;"):
            counts[(fields["history"] + "/" + fields["faults"], "judge_reached_more_than_once")] += 1
            continue
        mapping = "mapping_root_newer:true" in creds
        cells = {cell.split(":", 2)[0]: cell.split(":", 2)[1:] for cell in fields["cands"].split(";")}
        if mapping:
            passing = [outcome for verdict, outcome in cells.values() if verdict == "pass"]
            outcome = passing[0]
            new = re.search(r"new=(\d+)", outcome).group(1)
            kind = "pass_overwrote" if new != "0" else "pass_kept"
        else:
            kind = "refuse"
        intact = fields["r_units_overwritten_in_state"] == "0"
        counts[(fields["history"] + "/" + fields["faults"], f"mapping_root_only:{kind}:r_units_intact_before={intact}:order={fields['order']}")] += 1
for key, count in sorted(counts.items()):
    print(f"name=c834_derived history={key[0]} {key[1]} states={count}")
