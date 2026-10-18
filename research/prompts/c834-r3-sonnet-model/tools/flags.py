"""辩方腿原型每个状态那一行的判决字段里，不论跑的是哪个候选，都记着映射根凭据 mapping_newer_index_node 与指针槽凭据 pointer_fixed
（`research/prompts/c834-r2-sonnet-model/patch/apply_candidates.py` 的 `c834_candidate_says_refuse`）。这里用跑过 mapping_root_only 的逐状态输出做两件事：
(一) 逐状态数两个凭据相同与不同（只数走到那一判的状态，不同的按（前缀, 两个凭据取值）分组）；
(二) 指针槽读法的判（pointer_fixed 为真放过、为假拒）按（前缀, 盘序）数，和第二轮 aggregate-all.txt 里 pointer_slots_fixed 的同一格逐格比——
    只比头尾加中间一起覆盖了整个序号范围的前缀（乙两个前缀：clean/y2、tree_table_and_record/y2），别的前缀只覆盖了头尾两段，不比。
用法：python3 flags.py <第二轮 aggregate-all.txt> <这一轮辩方腿逐状态输出…>
打出：name=r3_flags …（一）与（二）各若干行；第二轮的数对不上时逐格列出。退出码 0 全对上，1 有对不上。"""
import collections, re, sys

reference, *paths = sys.argv[1:]
pairs, verdicts, covered = collections.Counter(), collections.Counter(), collections.defaultdict(set)
counts = {}
for path in paths:
    for line in open(path, encoding="utf-8"):
        if line.startswith("name=c834_history"):
            fields = dict(item.split("=", 1) for item in line.split() if "=" in item)
            counts[fields["prefix"]] = int(fields["state_count"])
        if not line.startswith("name=c834_state") or "candidate=mapping_root_only" not in line:
            continue
        fields = dict(re.findall(r"(\w+)=(\S+|\[[^\]]*\])", line))
        prefix = fields["prefix"] + "/" + fields.get("m2_faults", "y2")
        covered[prefix].add((int(fields["ordinal"]), fields["order"]))
        flags = dict(re.findall(r"(mapping_newer_index_node|pointer_fixed)=(true|false)", line))
        if not flags:
            continue
        pairs[(prefix, flags["mapping_newer_index_node"], flags["pointer_fixed"])] += 1
        verdicts[(prefix, fields["order"], "pass" if flags["pointer_fixed"] == "true" else "refuse")] += 1
status = 0
for (prefix, mapping, pointer), rows in sorted(pairs.items()):
    print(f"name=r3_flags pair prefix={prefix} mapping_newer_index_node={mapping} pointer_fixed={pointer} rows={rows}")
differ = sum(rows for (prefix, mapping, pointer), rows in pairs.items() if mapping != pointer)
print(f"name=r3_flags pair_total rows={sum(pairs.values())} differ={differ}")
reference_verdicts = collections.Counter()
for line in open(reference, encoding="utf-8"):
    if line.startswith("name=c834_cell") and "candidate=pointer_slots_fixed" in line:
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        if fields["judge"] in ("pass", "refuse"):
            reference_verdicts[(fields["prefix"], fields["order"], fields["judge"])] += int(fields["states"])
for prefix in sorted(covered):
    full = len(covered[prefix]) == 2 * counts.get(prefix.split("/")[0], -1)
    if not full:
        print(f"name=r3_flags pointer_vs_reference prefix={prefix} skipped=only_head_and_tail rows_covered={len(covered[prefix])}")
        continue
    for order in ("forward", "reversed"):
        for judge in ("pass", "refuse"):
            mine, ref = verdicts.get((prefix, order, judge), 0), reference_verdicts.get((prefix, order, judge), 0)
            verdict = "same" if mine == ref else "differs"
            if mine != ref:
                status = 1
            print(f"name=r3_flags pointer_vs_reference prefix={prefix} order={order} judge={judge} mine={mine} reference={ref} {verdict}")
sys.exit(status)
